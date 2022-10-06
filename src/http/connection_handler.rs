use log::error;
use std::io::prelude::*;
use std::net::TcpStream;

use crate::concurrent::addr_queue::AddrQueue;
use crate::http::request::Request;

pub fn http_handler(mut stream: TcpStream, addr_queue: AddrQueue) {
    let req = Request::from_tcp_stream(&mut stream).unwrap();

    if let Ok(lock) = addr_queue.poller.lock() {
        if let Ok((addr, port)) = lock.recv() {
            drop(lock);
            addr_queue.pusher.send((addr, port));
        } else {
            error!("http_handler: Failed to poll addr");
        }
    } else {
        error!("http_handler: Failed to get lock");
    }

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
