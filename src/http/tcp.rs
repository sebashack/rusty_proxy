use anyhow::{Context, Result};
use log::warn;
use std::net::{TcpListener, TcpStream};

pub fn mk_tcp_listener(addr: &str, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{:?}", addr, port);
    return TcpListener::bind(addr.clone())
        .context(format!("Failed to bind TcpListener to {}", addr));
}

pub fn listen_connections(listener: &TcpListener, handler: &dyn Fn(TcpStream) -> ()) {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => handler(stream),
            Err(err) => warn!("{:?}", err),
        }
    }
}
