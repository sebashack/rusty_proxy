use anyhow::{Context, Error, Result};
use std::collections::HashMap;

type Headers = HashMap<String, String>;

pub fn parse_headers(input: &str) -> Result<Headers> {
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

pub fn parse_version(input: &str) -> Result<&str> {
    if input == "HTTP/1.1" {
        Ok("1.1")
    } else {
        Err(Error::msg(format!("Unsupported version: {:?}", input)))
    }
}
