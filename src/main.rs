use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    env_logger::init();
    let pool = ThreadPool::new(50);
    let listener = mk_tcp_listener("127.0.0.1", 7878).unwrap();
    listen_connections(&listener, &pool);
}
