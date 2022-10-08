use rusty_proxy::concurrent::addr_queue::AddrQueue;
use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    env_logger::init();
    let pool = ThreadPool::new(5);
    let addrs = vec![("127.0.0.1".to_string(), 3000)];
    let addr_queue = AddrQueue::new(addrs);
    let listener = mk_tcp_listener("127.0.0.1", 7878).unwrap();
    listen_connections(&listener, &pool, &addr_queue);
}
