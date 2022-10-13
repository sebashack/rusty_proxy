use anyhow::{Context, Result};
use log::warn;
use std::net::TcpListener;
use std::sync::mpsc::Sender;

use crate::cache::io::CacheFile;
use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::concurrent::pool::ThreadPool;
use crate::http::connection_handler::http_handler;
use crate::opts::Service;
use std::path::PathBuf;

pub fn mk_tcp_listener(addr: String, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{:?}", addr, port);
    return TcpListener::bind(addr.clone())
        .context(format!("Failed to bind TcpListener to {}", addr));
}

pub fn listen_connections(
    listener: &TcpListener,
    pool: &ThreadPool,
    cache_dir: PathBuf,
    cache_ttl: u64,
    cache_sender: &Sender<CacheFile>,
    addr_queue: &CCFifoQueue<Service>,
    failure_delay: u64,
    failure_retries: u16,
) {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let addr_q = addr_queue.clone();
                let sender = cache_sender.clone();
                let cache = cache_dir.clone();
                pool.execute(move || {
                    http_handler(
                        stream,
                        cache,
                        cache_ttl,
                        sender,
                        addr_q,
                        failure_delay,
                        failure_retries,
                    )
                });
            }
            Err(err) => warn!("{:?}", err),
        }
    }
}
