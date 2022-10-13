use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Opts {
    pub port: u16,
    pub addr: String,
    pub cache_dir: String,
    pub cache_ttl_mins: u16,
    pub workers: u16,
    pub failure_delay: u64,
    pub failure_retries: u16,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub addr: String,
    pub port: u16,
}

pub fn read_opts_file(path: &str) -> Opts {
    let path = Path::new(path);
    let read_err = format!("Could not read file in '{:?}'", path);
    let contents = fs::read_to_string(path).expect(&read_err);

    parse_opts(contents.as_str())
}

// Helpers
fn parse_opts(input: &str) -> Opts {
    serde_yaml::from_str(input).unwrap()
}
