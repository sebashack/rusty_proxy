use anyhow::{Context, Result};
use log::warn;
use std::net::TcpListener;
use std::sync::mpsc::Sender;

use crate::cache::io::CacheFile;
use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::concurrent::pool::ThreadPool;
use crate::http::connection_handler::http_handler;
use std::path::PathBuf;

type Addr = String;
type Port = u16;

pub fn mk_tcp_listener(addr: Addr, port: Port) -> Result<TcpListener> {
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
    addr_queue: &CCFifoQueue<(Addr, Port)>,
) {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let addr_q = addr_queue.clone();
                let sender = cache_sender.clone();
                let cache = cache_dir.clone();
                pool.execute(move || http_handler(stream, cache, cache_ttl, sender, addr_q));
            }
            Err(err) => warn!("{:?}", err),
        }
    }
}
