use log::{error, info};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time;

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

            // TODO: Traverse cache_dir and delete expired files.

            thread::sleep(one_min);
        });

        CacheCleaner { thread }
    }
}
