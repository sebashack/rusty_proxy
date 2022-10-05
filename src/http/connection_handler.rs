use std::io::prelude::*;
use std::net::TcpStream;

use crate::http::request::Request;

pub fn http_handler(mut stream: TcpStream) {
    let req = Request::from_tcp_stream(&mut stream).unwrap();

    println!("{:?}", req);
    println!("{:?}", String::from_utf8(req.body));

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
