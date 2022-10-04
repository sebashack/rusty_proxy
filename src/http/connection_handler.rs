use anyhow::{Context, Result};
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

pub fn http_handler(mut stream: TcpStream) {
    let buff = BufReader::new(&mut stream);
    let http_request: Vec<_> = buff
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
