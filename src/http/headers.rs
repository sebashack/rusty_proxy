use crate::http::request::{Method, RequestHeader, RequestLine};
use crate::http::response::{Code, ResponseHeader, StatusLine};
use anyhow::{Context, Error, Result};
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use url::Url;

type Headers = HashMap<String, String>;

pub fn parse_request_header(input: &str) -> Result<RequestHeader> {
    let (s, rest) = input
        .split_once("\r\n")
        .context(format!("Invalid request headers: {}", input))?;
    let metadata = parse_request_line(format!("{s}\r\n").as_str())?;
    let headers = parse_headers(rest)?;

    Ok(RequestHeader { headers, metadata })
}

pub fn parse_response_header(input: &str) -> Result<ResponseHeader> {
    let (s, rest) = input
        .split_once("\r\n")
        .context(format!("Invalid request headers: {}", input))?;
    let status = parse_status_line(format!("{s}\r\n").as_str())?;
    let headers = parse_headers(rest)?;

    Ok(ResponseHeader { status, headers })
}

fn parse_status_line(input: &str) -> Result<StatusLine> {
    let (s, rest) = input
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let version = parse_version(s)?;
    let (s, rest) = rest
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let code = parse_code(s)?;
    let (s, _) = rest
        .split_once("\r\n")
        .context(format!("Invalid request-line: {}", input))?;
    let reason = parse_reason(s)?;

    Ok(StatusLine {
        version: version.to_string(),
        code: code,
        reason: reason.to_string(),
    })
}

fn parse_request_line(input: &str) -> Result<RequestLine> {
    let (s, rest) = input
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let method = parse_method(s)?;
    let (s, rest) = rest
        .split_once(' ')
        .context(format!("Invalid request-line: {}", input))?;
    let uri = parse_uri(s)?;
    let (s, _) = rest
        .split_once("\r\n")
        .context(format!("Invalid request-line: {}", input))?;
    let version = parse_version(s)?;

    Ok(RequestLine {
        method: method,
        uri: uri.to_string(),
        version: version.to_string(),
    })
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

//Parse status line
fn parse_version(input: &str) -> Result<&str> {
    if input == "HTTP/1.1" {
        Ok("1.1")
    } else {
        Err(Error::msg(format!("Unsupported version: {:?}", input)))
    }
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

fn parse_reason<'a>(input: &'a str) -> Result<&str> {
    //parse text?
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

//Parse request line
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
