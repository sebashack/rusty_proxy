use rusty_proxy::http::connection_handler::http_handler;
//use rusty_proxy::http::request::parse_request;
use rusty_proxy::http::tcp::{listen_connections, mk_tcp_listener};

fn main() {
    //let result =
    //    parse_request("GET / HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nUser-Agent: curl/7.68.0\r\n\r\n")
    //        .unwrap();
    //println!("{:?}", result);

    let listener = mk_tcp_listener("127.0.0.1", 7878).unwrap();
    listen_connections(&listener, &http_handler);
}
