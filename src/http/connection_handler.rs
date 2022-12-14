use anyhow::{Error, Result};
use log::{error, info, warn};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::time;

use crate::cache::io::{mk_file_path, CacheFile};
use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::http::{
    request::{Method, Request},
    response::Response,
};
use crate::opts::Service;
use std::path::PathBuf;

pub fn http_handler(
    client_stream: TcpStream,
    cache_dir: PathBuf,
    cache_ttl: u64,
    cache_sender: Sender<CacheFile>,
    addr_queue: CCFifoQueue<Service>,
    failure_delay: u64,
    failure_retries: u16,
) {
    match Request::read(&client_stream) {
        Ok(mut req) => {
            req.header.pretty_log();
            if let Ok(lock) = addr_queue.poller.lock() {
                if let Ok(service) = lock.recv() {
                    drop(lock);
                    addr_queue.pusher.send(service.clone()).unwrap();

                    let is_get_req = req.header.metadata.method == Method::Get;
                    let file_path = mk_file_path(&cache_dir, req.header.metadata.uri.clone());
                    let proxy_pass = || {
                        proxy_pass(
                            service,
                            &mut req,
                            &client_stream,
                            &cache_dir,
                            cache_sender,
                            cache_ttl,
                            is_get_req,
                            failure_delay,
                            failure_retries,
                        )
                    };

                    match (is_get_req, file_path.as_path().is_file()) {
                        (true, true) => {
                            if let Ok(metadata) = CacheFile::read_header(&file_path) {
                                if !metadata.is_expired() {
                                    if let Ok(cache_file) = CacheFile::read(file_path, metadata) {
                                        info!("Retrieving resource from cache");
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
    service: Service,
    req: &mut Request,
    client_stream: &TcpStream,
    cache_dir: &PathBuf,
    cache_sender: Sender<CacheFile>,
    cache_ttl: u64,
    is_get_req: bool,
    failure_delay: u64,
    failure_retries: u16,
) {
    info!("Proxy passing");
    let host = format!("{}:{}", service.addr, service.port);
    let service_stream = connect_to_service(service, failure_delay, failure_retries);
    match service_stream {
        Ok(service_stream) => {
            req.write(&service_stream, host);
            match Response::read(&service_stream) {
                Ok(mut res) => {
                    res.header.pretty_log();
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

                    res.write(&client_stream);
                }
                Err(_) => {
                    error!("Failed to parse server response");
                    Response::response500().write(&client_stream);
                }
            };
        }
        Err(err) => {
            error!("{}", err);
            Response::response500().write(&client_stream);
        }
    }
}

#[inline(always)]
fn connect_to_service(service: Service, delay_millis: u64, retries: u16) -> Result<TcpStream> {
    let host = format!("{}:{}", service.addr, service.port);
    if let Ok(service_stream) = TcpStream::connect(host.clone()) {
        return Ok(service_stream);
    } else {
        if retries < 1 {
            return Err(Error::msg(format!(
                "Failed to establish connection with service"
            )));
        } else {
            let dur = time::Duration::from_millis(delay_millis);
            std::thread::sleep(dur);
            warn!("Connection with server try {}", retries);
            connect_to_service(service, delay_millis, retries - 1)
        }
    }
}
