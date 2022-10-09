use anyhow::{Context, Result};
use log::warn;
use std::net::TcpListener;

use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::concurrent::pool::ThreadPool;
use crate::http::connection_handler::http_handler;

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
    addr_queue: &CCFifoQueue<(Addr, Port)>,
) {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let addrs = addr_queue.clone();
                pool.execute(|| http_handler(stream, addrs));
            }
            Err(err) => warn!("{:?}", err),
        }
    }
}
