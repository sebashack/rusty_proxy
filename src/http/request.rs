use crate::http::headers;
use anyhow::{Error, Result};
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
}

impl Method {
    fn to_buffer(&self) -> &[u8] {
        match &self {
            Method::Options => "OPTIONS".as_bytes(),
            Method::Get => "GET".as_bytes(),
            Method::Head => "HEAD".as_bytes(),
            Method::Post => "POST".as_bytes(),
            Method::Put => "PUT".as_bytes(),
            Method::Delete => "DELETE".as_bytes(),
            Method::Trace => "TRACE".as_bytes(),
            Method::Connect => "CONNECT".as_bytes(),
        }
    }
}

type Headers = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Request {
    pub header: RequestHeader,
    pub body: Vec<u8>,
}

impl Request {
    pub fn from_tcp_stream(stream: &mut TcpStream) -> Result<Self> {
        let (header, body) = Self::split_req(stream)?;
        Ok(Request { header, body })
    }

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = self.header.to_buffer();
        let mut body = self.body.clone();
        buffer.append(&mut body);
        buffer
    }

    fn split_req(mut stream: &TcpStream) -> Result<(RequestHeader, Vec<u8>)> {
        let buff = BufReader::new(&mut stream);
        let mut header_buff: Vec<u8> = Vec::new();
        let mut body: Vec<u8> = Vec::new();
        let mut crlfs = 0;
        let mut it = buff.bytes();

        while crlfs != 2 {
            match it.next() {
                Some(Ok(byte)) => {
                    header_buff.push(byte);
                    match byte {
                        0x0D => {} //do nothing
                        0x0A => {
                            crlfs += 1;
                        }
                        _ => {
                            crlfs = 0;
                        }
                    }
                }
                Some(Err(_)) => {
                    return Err(Error::msg(format!("Error while reading request")));
                }
                None => {}
            }
        }

        let header_str = String::from_utf8(header_buff)?;
        let header = headers::parse_request_header(header_str.as_str())?;

        if let Some(len) = header.get_content_length() {
            let mut read_bytes = 0;

            while read_bytes < len {
                match it.next() {
                    Some(Ok(byte)) => {
                        body.push(byte);
                        read_bytes += 1;
                    }
                    Some(Err(_)) => {
                        return Err(Error::msg(format!("Error while reading request")));
                    }
                    None => {}
                }
            }
        }

        return Ok((header, body));
    }
}

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub metadata: RequestLine,
    pub headers: Headers,
}

impl RequestHeader {
    pub fn get_content_length(&self) -> Option<usize> {
        if let Some(h) = self.headers.get("content-length") {
            h.parse().ok()
        } else {
            None
        }
    }

    pub fn insert_header(&mut self, k: String, v: String) {
        self.headers.insert(k, v);
    }

    pub fn remove_header(&mut self, k: String) {
        self.headers.remove(&k);
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = self.metadata.to_buffer();
        for (key, value) in self.headers.iter() {
            let mut element = format!("{key}:{value}\r\n").as_bytes().to_vec();
            buffer.append(&mut element);
        }

        buffer.push(0x0D);
        buffer.push(0x0A);

        buffer
    }
}

#[derive(Debug, Clone)]
pub struct RequestLine {
    pub method: Method,
    pub uri: String,
    pub version: String,
}

impl RequestLine {
    fn to_buffer(&self) -> Vec<u8> {
        let method = self.method.to_buffer();
        let uri = self.uri.as_bytes();
        let version = self.version.as_bytes();
        let sp = [' ' as u8];
        let crlf = [0x0D, 0x0A];
        let line = [method, &sp, uri, &sp, version, &crlf].concat();

        line.to_vec()
    }
}
