use anyhow::{Context, Error, Result};
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use url::Url;

#[derive(Debug)]
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

type Headers = HashMap<String, String>;

#[derive(Debug)]
pub struct Request {
    pub header: RequestHeader,
    pub body: Vec<u8>,
}

impl Request {
    pub fn from_tcp_stream(stream: &mut TcpStream) -> Result<Self> {
        let (header, body) = split_req(stream)?;
        Ok(Request { header, body })
    }
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: Method,
    pub path: String,
    pub version: String,
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
    let header = parse_request_header(header_str.as_str())?;

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

fn parse_request_header(input: &str) -> Result<RequestHeader> {
    let (s, rest) = input
        .split_once("\r\n")
        .context(format!("Invalid request headers: {}", input))?;
    let metadata = parse_request_line(format!("{s}\r\n").as_str())?;
    let headers = parse_headers(rest)?;

    Ok(RequestHeader { headers, metadata })
}

fn parse_headers(input: &str) -> Result<Headers> {
    let mut crlfs = 0;
    let mut headers = HashMap::new();
    for s in input.split("\r\n") {
        if s == "" {
            crlfs += 1;
        } else {
            let (key, val) = parse_header(s)?;
            headers.insert(key, val.to_string());
        }
    }

    if crlfs != 2 {
        return Err(Error::msg(format!("Invalid end of headers {}", input)));
    } else {
        return Ok(headers);
    }
}

fn parse_header(input: &str) -> Result<(String, &str)> {
    let (key, val) = input
        .split_once(':')
        .context(format!("Invalid request-line: {}", input))?;
    Ok((key.to_lowercase(), val.trim()))
}

fn parse_request_line(input: &str) -> Result<RequestLine> {
    let (s, rest) = input
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let method = parse_method(s)?;
    let (s, rest) = rest
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let path = parse_uri(s)?;
    let (s, _) = rest
        .split_once("\r\n")
        .context(format!("Invalid request-line: {}", input))?;
    let version = parse_version(s)?;

    Ok(RequestLine {
        method: method,
        path: path.to_string(),
        version: version.to_string(),
    })
}

fn parse_version(input: &str) -> Result<&str> {
    if input == "HTTP/1.1" {
        Ok("1.1")
    } else {
        Err(Error::msg(format!("Unsupported version: {:?}", input)))
    }
}

fn parse_method(input: &str) -> Result<Method> {
    match input {
        "OPTIONS" => Ok(Method::Options),
        "GET" => Ok(Method::Get),
        "HEAD" => Ok(Method::Head),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "DELETE" => Ok(Method::Delete),
        "TRACE" => Ok(Method::Trace),
        "CONNECT" => Ok(Method::Connect),
        _ => Err(Error::msg(format!("Invalid method: {:?}", input))),
    }
}

fn parse_uri<'a>(input: &'a str) -> Result<&str> {
    let prefix = if input.starts_with("/") {
        "http://host"
    } else {
        ""
    };

    if let Ok(_) = Url::parse(format!("{}{}", prefix, input).as_str()) {
        Ok(input)
    } else {
        Err(Error::msg(format!("Invalid request-uri: {:?}", input)))
    }
}
