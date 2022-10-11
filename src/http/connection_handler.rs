use log::{error, warn};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use crate::cache::io::{mk_file_path, CacheFile};
use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::http::{
    request::{Method, Request},
    response::Response,
};
use std::path::PathBuf;

type Addr = String;
type Port = u16;

pub fn http_handler(
    client_stream: TcpStream,
    cache_dir: PathBuf,
    cache_ttl: u64,
    cache_sender: Sender<CacheFile>,
    addr_queue: CCFifoQueue<(Addr, Port)>,
) {
    match Request::read(&client_stream) {
        Ok(mut req) => {
            if let Ok(lock) = addr_queue.poller.lock() {
                if let Ok((addr, port)) = lock.recv() {
                    drop(lock);
                    let addr_ = addr.clone();
                    addr_queue.pusher.send((addr, port)).unwrap();

                    let is_get_req = req.header.metadata.method == Method::Get;
                    let file_path = mk_file_path(&cache_dir, req.header.metadata.uri.clone());
                    let proxy_pass = || {
                        proxy_pass(
                            addr_,
                            port,
                            &mut req,
                            &client_stream,
                            &cache_dir,
                            cache_sender,
                            cache_ttl,
                            is_get_req,
                        )
                    };

                    match (is_get_req, file_path.as_path().is_file()) {
                        (true, true) => {
                            if let Ok(metadata) = CacheFile::read_header(&file_path) {
                                if !metadata.is_expired() {
                                    if let Ok(cache_file) = CacheFile::read(file_path, metadata) {
                                        let mut res = Response::from_cache_file(cache_file);
                                        res.write(&client_stream)
                                    } else {
                                        proxy_pass();
                                    }
                                } else {
                                    proxy_pass();
                                }
                            } else {
                                warn!("Failed to read cache file metadata");
                                proxy_pass();
                            }
                        }
                        _ => proxy_pass(),
                    }
                } else {
                    error!("http_handler: Failed to poll addr");
                }
            } else {
                error!("http_handler: Failed to get lock");
            }
        }
        Err(_) => Response::response400().write(&client_stream),
    }
}

#[inline(always)]
fn proxy_pass(
    addr: String,
    port: u16,
    req: &mut Request,
    client_stream: &TcpStream,
    cache_dir: &PathBuf,
    cache_sender: Sender<CacheFile>,
    cache_ttl: u64,
    is_get_req: bool,
) {
    let host = format!("{addr}:{port}");
    if let Ok(service_stream) = TcpStream::connect(host.clone()) {
        req.write(&service_stream, host);

        match Response::read(&service_stream) {
            Ok(mut res) => {
                if is_get_req && res.is_cacheable() {
                    if let Ok(cache_file) = CacheFile::new(
                        cache_ttl,
                        res.body.len() as u64,
                        mk_file_path(cache_dir, req.header.metadata.uri.clone()),
                        res.body.clone(),
                        res.get_content_type(),
                    ) {
                        if let Err(_) = cache_sender.send(cache_file) {
                            error!("Failed to queue cache file");
                        }
                    } else {
                        error!("Failed to cache resource file");
                    }
                }

                res.write(&client_stream)
            }
            Err(_) => error!("Failed to parse server response"),
        };
    }
}
