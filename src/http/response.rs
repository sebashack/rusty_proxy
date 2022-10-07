use anyhow::{Error, Result};
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

use crate::http::headers;

/*Response = Status-line
 *      *(( general-header
 *      | response-header
 *      | entity-header ) CRLF)
 *      CRLF
 *      [ message body ]
 *
 * Status-Line = HTTP-Version SP Status-Code SP Reason-Phrase CRLF   */

#[derive(Debug, Clone)]
pub enum Code {
    Code100,
    Code101,
    Code200,
    Code201,
    Code202,
    Code203,
    Code204,
    Code205,
    Code206,

    Code300,
    Code301,
    Code302,
    Code303,
    Code304,
    Code305,
    Code307,

    Code400,
    Code401,
    Code402,
    Code403,
    Code404,
    Code405,
    Code406,

    Code407,
    Code408,
    Code409,
    Code410,
    Code411,
    Code412,
    Code413,
    Code414,
    Code415,
    Code416,
    Code417,

    Code500,
    Code501,
    Code502,
    Code503,
    Code504,
    Code505,
}

impl Code {
    fn to_buffer(&self) -> &[u8] {
        match &self {
            Code::Code100 => "100".as_bytes(),
            Code::Code101 => "101".as_bytes(),
            Code::Code200 => "200".as_bytes(),
            Code::Code201 => "201".as_bytes(),
            Code::Code202 => "202".as_bytes(),
            Code::Code203 => "203".as_bytes(),
            Code::Code204 => "204".as_bytes(),
            Code::Code205 => "205".as_bytes(),
            Code::Code206 => "206".as_bytes(),

            Code::Code300 => "300".as_bytes(),
            Code::Code301 => "301".as_bytes(),
            Code::Code302 => "302".as_bytes(),
            Code::Code303 => "303".as_bytes(),
            Code::Code304 => "304".as_bytes(),
            Code::Code305 => "305".as_bytes(),
            Code::Code307 => "307".as_bytes(),

            Code::Code400 => "400".as_bytes(),
            Code::Code401 => "401".as_bytes(),
            Code::Code402 => "402".as_bytes(),
            Code::Code403 => "403".as_bytes(),
            Code::Code404 => "404".as_bytes(),
            Code::Code405 => "405".as_bytes(),
            Code::Code406 => "406".as_bytes(),

            Code::Code407 => "407".as_bytes(),
            Code::Code408 => "408".as_bytes(),
            Code::Code409 => "409".as_bytes(),
            Code::Code410 => "410".as_bytes(),
            Code::Code411 => "411".as_bytes(),
            Code::Code412 => "412".as_bytes(),
            Code::Code413 => "413".as_bytes(),
            Code::Code414 => "414".as_bytes(),
            Code::Code415 => "415".as_bytes(),
            Code::Code416 => "416".as_bytes(),
            Code::Code417 => "417".as_bytes(),

            Code::Code500 => "500".as_bytes(),
            Code::Code501 => "501".as_bytes(),
            Code::Code502 => "502".as_bytes(),
            Code::Code503 => "503".as_bytes(),
            Code::Code504 => "504".as_bytes(),
            Code::Code505 => "505".as_bytes(),
        }
    }
}

type Headers = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Response {
    pub header: ResponseHeader,
    pub body: Vec<u8>,
}

impl Response {
    pub fn from_tcp_stream(stream: &mut TcpStream) -> Result<Self> {
        let (header, body) = Self::split_res(stream)?;
        Ok(Response { header, body })
    }

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = self.header.to_buffer();
        let mut body = self.body.clone();
        buffer.append(&mut body);
        buffer
    }

    fn split_res(mut stream: &TcpStream) -> Result<(ResponseHeader, Vec<u8>)> {
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
                    return Err(Error::msg(format!("Error while reading response")));
                }
                None => {}
            }
        }

        let header_str = String::from_utf8(header_buff)?;
        let header = headers::parse_response_header(header_str.as_str())?;

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
pub struct ResponseHeader {
    pub status: StatusLine,
    pub headers: Headers,
}
impl ResponseHeader {
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
        let mut buffer = self.status.to_buffer();
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
pub struct StatusLine {
    pub version: String,
    pub code: Code,
    pub reason: String,
}

impl StatusLine {
    fn to_buffer(&self) -> Vec<u8> {
        let version = self.version.as_bytes();
        let code = self.code.to_buffer();
        let reason = self.reason.as_bytes();
        let sp = [' ' as u8];
        let crlf = [0x0D, 0x0A];
        let line = [version, &sp, code, &sp, reason, &crlf].concat();

        line.to_vec()
    }
}
