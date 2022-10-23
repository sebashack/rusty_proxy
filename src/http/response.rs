use anyhow::{Context, Error, Result};
use log::{info, warn};
use mt_logger::{mt_log, Level};
use std::collections::HashMap;
use std::io::{prelude::*, BufReader, BufWriter};
use std::net::TcpStream;

use crate::cache::io::CacheFile;
use crate::http::headers::{self, Headers};

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

#[derive(Debug, Clone)]
pub struct Response {
    pub header: ResponseHeader,
    pub body: Vec<u8>,
}

static MAX_CACHE_SIZE_MB: u32 = 30;
impl Response {
    pub fn from_cache_file(file: CacheFile) -> Self {
        let status = StatusLine {
            version: "HTTP/1.1".to_string(),
            code: Code::Code200,
            reason: "OK".to_string(),
        };
        let mut header = ResponseHeader::new(status);

        if let Some(content_type) = file.metadata.content_type {
            header.insert_header("content-type".to_string(), content_type);
        }
        header.insert_header(
            "content-length".to_string(),
            file.metadata.content_length.to_string(),
        );

        Response {
            header,
            body: file.content_data,
        }
    }

    pub fn response400() -> Self {
        let status = StatusLine {
            version: "HTTP/1.1".to_string(),
            code: Code::Code400,
            reason: "Malformed request".to_string(),
        };
        let header = ResponseHeader::new(status);

        Response {
            header,
            body: Vec::new(),
        }
    }

    pub fn response500() -> Self {
        let status = StatusLine {
            version: "HTTP/1.1".to_string(),
            code: Code::Code500,
            reason: "Internal server error".to_string(),
        };
        let header = ResponseHeader::new(status);

        Response {
            header,
            body: Vec::new(),
        }
    }

    pub fn get_content_type(&self) -> Option<String> {
        self.header
            .headers
            .get(&"content-type".to_string())
            .map(|s| s.clone())
    }

    pub fn read(stream: &TcpStream) -> Result<Self> {
        let (header, body) = split_res(stream)?;
        Ok(Response { header, body })
    }

    pub fn write(&mut self, stream: &TcpStream) {
        self.header
            .insert_header("server".to_string(), "rusty-proxy".to_string());

        let mut writer = BufWriter::new(stream);
        let data = self.to_buffer();
        let size = data.len();
        let buff_size = if size < 2048 { size } else { size / 1024 };

        for chunk in data.chunks(buff_size) {
            let mut pos = 0;
            while pos < chunk.len() {
                if let Ok(bytes_written) = writer.write(&chunk[pos..]) {
                    pos += bytes_written;
                    if let Err(_) = writer.flush() {
                        warn!("Failed to flush response buffer");
                        return;
                    }
                } else {
                    warn!("Failed to write response");
                    return;
                }
            }
        }
    }

    pub fn is_cacheable(&self) -> bool {
        let is_valid_status_code = match self.header.status.code {
            Code::Code200
            | Code::Code201
            | Code::Code202
            | Code::Code203
            | Code::Code204
            | Code::Code205
            | Code::Code206 => true,
            _ => false,
        };

        self.body_size_mb() <= MAX_CACHE_SIZE_MB
            && is_valid_status_code
            && headers::is_cacheable_content_type(&self.header.headers)
    }

    pub fn body_size_mb(&self) -> u32 {
        (self.body.len() / 1048576) as u32
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = self.header.to_buffer();
        let mut body = self.body.clone();
        buffer.append(&mut body);
        buffer
    }
}

#[derive(Debug, Clone)]
pub struct ResponseHeader {
    pub status: StatusLine,
    pub headers: Headers,
}

impl ResponseHeader {
    pub fn new(status: StatusLine) -> Self {
        let mut headers = HashMap::new();
        headers.insert("server".to_string(), "rusty-proxy".to_string());

        ResponseHeader { status, headers }
    }

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

    pub fn pretty_log(&self) {
        let res_line_str = String::from_utf8(self.to_buffer()).map(|s| s.replace("\r\n", " "));
        let mut headers_str = String::new();

        self.headers.iter().for_each(|(k, v)| {
            let entry = format!("{k}:{v};");
            headers_str.push_str(entry.as_str());
        });

        if let Ok(mut res_line_str) = res_line_str {
            res_line_str.push_str("~~  ");
            res_line_str.push_str(headers_str.as_str());
            info!("{}", res_line_str);
            mt_log!(Level::Info, "{}", res_line_str);
        }
    }

    pub fn to_buffer(&self) -> Vec<u8> {
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
    let header = parse_response_header(header_str.as_str())?;

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

pub fn parse_response_header(input: &str) -> Result<ResponseHeader> {
    let (s, rest) = input
        .split_once("\r\n")
        .context(format!("Invalid request headers: {}", input))?;
    let status = parse_status_line(format!("{s}\r\n").as_str())?;
    let headers = headers::parse_headers(rest)?;

    Ok(ResponseHeader { status, headers })
}

fn parse_status_line(input: &str) -> Result<StatusLine> {
    let (s, rest) = input
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let version = headers::parse_version(s)?;
    let (s, rest) = rest
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let code = parse_code(s)?;
    let (s, _) = rest
        .split_once("\r\n")
        .context(format!("Invalid request-line: {}", input))?;
    let reason = s;

    Ok(StatusLine {
        version: version.to_string(),
        code: code,
        reason: reason.to_string(),
    })
}

fn parse_code(input: &str) -> Result<Code> {
    match input {
        "100" => Ok(Code::Code100),
        "101" => Ok(Code::Code101),
        "200" => Ok(Code::Code200),
        "201" => Ok(Code::Code201),
        "202" => Ok(Code::Code202),
        "203" => Ok(Code::Code203),
        "204" => Ok(Code::Code204),
        "205" => Ok(Code::Code205),
        "206" => Ok(Code::Code206),
        "300" => Ok(Code::Code300),
        "301" => Ok(Code::Code301),
        "302" => Ok(Code::Code302),
        "303" => Ok(Code::Code303),
        "304" => Ok(Code::Code304),
        "305" => Ok(Code::Code305),
        "307" => Ok(Code::Code307),
        "401" => Ok(Code::Code401),
        "402" => Ok(Code::Code402),
        "403" => Ok(Code::Code403),
        "404" => Ok(Code::Code404),
        "405" => Ok(Code::Code405),
        "406" => Ok(Code::Code406),

        "407" => Ok(Code::Code407),
        "408" => Ok(Code::Code408),
        "409" => Ok(Code::Code409),
        "410" => Ok(Code::Code410),
        "411" => Ok(Code::Code411),
        "412" => Ok(Code::Code412),
        "413" => Ok(Code::Code413),
        "414" => Ok(Code::Code414),
        "415" => Ok(Code::Code415),
        "416" => Ok(Code::Code416),
        "417" => Ok(Code::Code417),
        "500" => Ok(Code::Code417),
        "501" => Ok(Code::Code417),
        "502" => Ok(Code::Code417),
        "503" => Ok(Code::Code417),
        "504" => Ok(Code::Code417),
        "505" => Ok(Code::Code417),
        fail_code => match fail_code.chars().nth(0).unwrap() {
            '1' => Ok(Code::Code100),
            '2' => Ok(Code::Code200),
            '3' => Ok(Code::Code300),
            '4' => Ok(Code::Code400),
            '5' => Ok(Code::Code500),
            _ => Err(Error::msg(format!("Invalid method: {:?}", input))),
        },
    }
}
