use std::path::Path;

use std::sync::mpsc;

use rusty_proxy::cache::cleaner::edit_crontab;
use rusty_proxy::cache::writer::CacheWriter;
use rusty_proxy::concurrent::ccfifo_queue::CCFifoQueue;
use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    env_logger::init();
    let cache_dir = Path::new("/home/sebastian/university/networking/rusty_proxy/proxy_cache");
    let pool = ThreadPool::new(5);
    let addrs = vec![("127.0.0.1".to_string(), 3000)];
    let cache_ttl = 180;
    let addr_queue = CCFifoQueue::new(addrs);
    let (cache_sender, cache_receiver) = mpsc::channel();
    let listener = mk_tcp_listener("127.0.0.1".to_string(), 7878).unwrap();

    CacheWriter::run(cache_receiver);
    listen_connections(
        &listener,
        &pool,
        cache_dir.to_path_buf(),
        cache_ttl,
        &cache_sender,
        &addr_queue,
    );
    //let cmd = Path::new("/home/sebastian/university/networking/cron-playground/task.sh");
    //edit_crontab("sebastian", cmd.to_path_buf());
}
