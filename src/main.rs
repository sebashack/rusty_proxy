use rusty_proxy::http::connection_handler::http_handler;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    let listener = mk_tcp_listener("127.0.0.1", 7878).unwrap();
    listen_connections(&listener, &http_handler);
}
