use std::path::Path;

use rusty_proxy::concurrent::ccfifo_queue::CCFifoQueue;
use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    env_logger::init();
    let cache_dir = Path::new("/home/sebastian/university/networking/rusty_proxy/proxy_cache");
    let pool = ThreadPool::new(5);
    let addrs = vec![("127.0.0.1".to_string(), 3000)];
    let ccfifo_queue = CCFifoQueue::new(addrs);
    let listener = mk_tcp_listener("127.0.0.1".to_string(), 7878).unwrap();
    listen_connections(&listener, &pool, &ccfifo_queue);
}
