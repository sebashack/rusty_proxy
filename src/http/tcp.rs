use crate::concurrent::pool::ThreadPool;
use crate::http::connection_handler::http_handler;
use anyhow::{Context, Result};
use log::warn;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

pub fn mk_tcp_listener(addr: &str, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{:?}", addr, port);
    return TcpListener::bind(addr.clone())
        .context(format!("Failed to bind TcpListener to {}", addr));
}

pub fn listen_connections(listener: &TcpListener, pool: &ThreadPool) {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                pool.execute(|| {
                    println!("Before execution");
                    http_handler(stream);
                    thread::sleep(Duration::from_secs(7));
                    println!("Ended execution");
                });
            }
            Err(err) => warn!("{:?}", err),
        }
    }
}
