use log::{error, info};
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use crate::cache::io::CacheFile;

#[allow(dead_code)]
pub struct CacheWriter {
    thread: JoinHandle<()>,
}

impl CacheWriter {
    pub fn run(cache_receiver: Receiver<CacheFile>) -> Self {
        let thread = thread::spawn(move || loop {
            if let Ok(cache_file) = cache_receiver.recv() {
                if cache_file.path.as_path().is_file() {
                    info!("File already exists. Not writing");
                } else {
                    if let Err(error) = cache_file.write() {
                        error!("{error}");
                    }
                }
            } else {
                error!("Failed to receive cache file");
            }
        });

        CacheWriter { thread }
    }
}
