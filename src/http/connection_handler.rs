use log::error;
use std::net::TcpStream;

use crate::concurrent::ccfifo_queue::CCFifoQueue;
use crate::http::{request::Request, response::Response};

type Addr = String;
type Port = u16;

pub fn http_handler(client_stream: TcpStream, addr_queue: CCFifoQueue<(Addr, Port)>) {
    match Request::read(&client_stream) {
        Ok(mut req) => {
            if let Ok(lock) = addr_queue.poller.lock() {
                if let Ok((addr, port)) = lock.recv() {
                    drop(lock);
                    let addr_ = addr.clone();
                    addr_queue.pusher.send((addr, port)).unwrap();

                    let host = format!("{addr_}:{port}");
                    if let Ok(service_stream) = TcpStream::connect(host.clone()) {
                        // TODO: If resource with valid content-type is present:
                        // If resource url is found in cache, then return cached file.
                        // Else, send request to service.
                        req.write(&service_stream, host);

                        match Response::read(&service_stream) {
                            Ok(mut res) => {
                                // TODO: Cache resource if body with content-type:
                                // html
                                // images (jpg, png, etc...)
                                // javascript
                                // octet stream
                                // audio
                                // Before caching, validate MAXIMUM file size.
                                // Before caching, staus code must be one of 2XX.
                                res.write(&client_stream)
                            }
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
