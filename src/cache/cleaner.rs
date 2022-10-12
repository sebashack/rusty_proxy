use log::{error, info};
use std::path::{self, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::{fs, time};

use super::io::{delete_cache_file, CacheFile};

#[allow(dead_code)]
pub struct CacheCleaner {
    thread: JoinHandle<()>,
}

static SLEEP_TIME: u64 = 60; // secs

impl CacheCleaner {
    pub fn run(cache_dir: PathBuf) -> Self {
        let one_min = time::Duration::from_secs(SLEEP_TIME);
        let thread = thread::spawn(move || loop {
            info!("Cleaning cache ...");

            Self::traverse_files(cache_dir.clone());

            thread::sleep(one_min);
        });

        CacheCleaner { thread }
    }

    fn traverse_files(path: PathBuf) {
        for entry in fs::read_dir(path.as_path()) {
            for dir_entry in entry {
                if let Ok(dir_entry) = dir_entry {
                    let dir_entry_path = dir_entry.path();
                    if dir_entry_path.is_file() {
                        if let Ok(metadata) = CacheFile::read_header(&dir_entry_path) {
                            if metadata.is_expired() {
                                delete_cache_file(dir_entry_path)
                                    .expect("Failed to delete expired cache file");
                            }
                        }
                    } else {
                        Self::traverse_files(dir_entry_path);
                    }
                }
            }
        }
    }
}
