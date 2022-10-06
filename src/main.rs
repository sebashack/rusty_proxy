use rusty_proxy::concurrent::pool::ThreadPool;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    env_logger::init();
    //let result =
    //    parse_request("GET / HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nUser-Agent: curl/7.68.0\r\n\r\n")
    //        .unwrap();
    //println!("{:?}", result);

    let pool = ThreadPool::new(5);
    let listener = mk_tcp_listener("127.0.0.1", 7878).unwrap();
    listen_connections(&listener, &pool);
}
