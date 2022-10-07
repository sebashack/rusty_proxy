use log::error;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::net::TcpStream;

use crate::concurrent::addr_queue::AddrQueue;
use crate::http::{request::Request, response::Response};

pub fn http_handler(mut client_stream: TcpStream, addr_queue: AddrQueue) {
    let mut req = Request::from_tcp_stream(&mut client_stream).unwrap();

    // TODO: Remove this
    println!("{:?}", req);

    if let Ok(lock) = addr_queue.poller.lock() {
        if let Ok((addr, port)) = lock.recv() {
            drop(lock);
            let addr_ = addr.clone();
            addr_queue.pusher.send((addr, port)).unwrap();

            let addr_port = format!("{addr_}:{port}");
            if let Ok(mut service_stream) = TcpStream::connect(addr_port.clone()) {
                req.header.remove_header("transfer-encoding".to_string());
                req.header.remove_header("accept-encoding".to_string());
                req.header.remove_header("content-encoding".to_string());
                req.header.insert_header("host".to_string(), addr_port);

                let mut writer = BufWriter::new(&mut service_stream);
                writer.write_all(req.to_buffer().as_slice());
                writer.flush().unwrap();

                // TODO: Parse response and reply to client.

                let response = "HTTP/1.1 200 OK\r\n\r\n";
                client_stream.write_all(response.as_bytes()).unwrap();
            }
        } else {
            error!("http_handler: Failed to poll addr");
        }
    } else {
        error!("http_handler: Failed to get lock");
    }
}
