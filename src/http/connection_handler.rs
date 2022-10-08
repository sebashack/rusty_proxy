use log::error;
use std::net::TcpStream;

use crate::concurrent::addr_queue::AddrQueue;
use crate::http::{request::Request, response::Response};

pub fn http_handler(client_stream: TcpStream, addr_queue: AddrQueue) {
    match Request::read(&client_stream) {
        Ok(mut req) => {
            if let Ok(lock) = addr_queue.poller.lock() {
                if let Ok((addr, port)) = lock.recv() {
                    drop(lock);
                    let addr_ = addr.clone();
                    addr_queue.pusher.send((addr, port)).unwrap();

                    let host = format!("{addr_}:{port}");
                    if let Ok(service_stream) = TcpStream::connect(host.clone()) {
                        req.write(&service_stream, host);

                        match Response::read(&service_stream) {
                            Ok(mut res) => res.write(&client_stream),
                            Err(_) => error!("Failed to parse server response"),
                        };
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
