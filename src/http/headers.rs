use anyhow::{Context, Error, Result};
use std::collections::HashMap;

pub type Headers = HashMap<String, String>;

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
    if input == "HTTP/1.1" || input == "HTTP-1.1" {
        Ok("HTTP/1.1")
    } else {
        Err(Error::msg(format!("Unsupported version: {:?}", input)))
    }
}

pub fn is_cacheable_content_type(headers: &Headers) -> bool {
    if let Some(ct) = headers.get("content-type") {
        return cacheable_types().contains(&ct.as_str());
    } else {
        return false;
    }
}

#[inline(always)]
fn cacheable_types<'a>() -> Vec<&'a str> {
    vec![
        "application/octet-stream",
        "text/css",
        "text/javascript",
        "image/apng",
        "image/avif",
        "image/gif",
        "image/jpeg",
        "image/png",
        "image/svg+xml",
        "image/webp",
        "image/bmp",
        "image/x-icon",
        "image/tiff",
        "audio/webm",
        "audio/mpeg",
        "audio/ogg",
        "audio/x-wav",
        "audio/mp4",
        "application/ogg",
        "application/pdf",
    ]
}
