use anyhow::{Context, Error, Result};
use std::collections::HashMap;
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

//type Body
type Headers = HashMap<String, String>;

#[derive(Debug)]
pub struct RequestHeader {
    pub metadata: RequestLine,
    pub headers: Headers,
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: Method,
    pub path: String,
    pub version: String,
}

pub fn parse_request_header(input: &str) -> Result<RequestHeader> {
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
    Ok((key.to_lowercase(), val))
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
        Err(Error::msg(format!("Invalid version: {:?}", input)))
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
    if let Ok(_) = Url::parse(format!("http://host/{}", input).as_str()) {
        Ok(input)
    } else {
        Err(Error::msg(format!("Invalid request-uri: {:?}", input)))
    }
}
