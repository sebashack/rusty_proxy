use std::path::Path;

use mt_logger::{mt_new, Level, OutputStream};
use std::process::exit;
use std::sync::mpsc;

use rusty_proxy::cache::cleaner::CacheCleaner;
use rusty_proxy::cache::writer::CacheWriter;
use rusty_proxy::concurrent::ccfifo_queue::CCFifoQueue;
use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};
use rusty_proxy::opts::read_opts_file;

fn main() {
    env_logger::init();
    mt_new!(None, Level::Info, OutputStream::File);

    let maybe_path = std::env::args().nth(1);
    match maybe_path {
        Some(path) => {
            let opts = read_opts_file(path.as_str());

            if opts.workers < 1 {
                println!("Property 'workers' must be > 0");
                exit(1);
            }

            if opts.cache_ttl_mins < 1 {
                println!("Property 'cache_ttl_mins' must be > 0");
                exit(1);
            }

            let cache_dir = Path::new(opts.cache_dir.as_str());
            let cache_ttl_secs = (opts.cache_ttl_mins * 60) as u64;
            let pool = ThreadPool::new(opts.workers as usize);
            let addr_queue = CCFifoQueue::new(opts.services);
            let (cache_sender, cache_receiver) = mpsc::channel();

            println!("Listening on {}:{}", opts.addr, opts.port);
            let listener = mk_tcp_listener(opts.addr, opts.port).unwrap();

            CacheWriter::run(cache_receiver);
            CacheCleaner::run(cache_dir.clone().to_path_buf());

            listen_connections(
                &listener,
                &pool,
                cache_dir.to_path_buf(),
                cache_ttl_secs,
                &cache_sender,
                &addr_queue,
                opts.failure_delay,
                opts.failure_retries,
            );
        }
        None => println!(
            "Path to process file not provided. Usage: `rusty_proxy /path/to/process.yaml`"
        ),
    }
}
