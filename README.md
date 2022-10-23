# Rusty Reverse Proxy Server

## Introduction

This programming project consists of the implementation of a reverse proxy sever with load balancing. A reverse proxy
server (RPS) is an entry-point where all requests coming from web clients are forwarded to several web services [1]. On the
other hand, load balancing is a feature that allows a RPS to efficiently distribute incoming network requests across those
services [4]. The name of our RPS is `Rusty-Proxy` and in its first version it has some limitations:

- It accepts HTTP/1.1 requests only.
- It does not support special encodings such as `GZIP` and `Chunked transfer encoding`.
- It works with the round robin balancing policy and it's not possible to configure other strategies.

The Rust programming language was chosen for the implementation of this project. Rust is a compiled systems programming
language that provides fine control over memory management and shares the principle of zero-cost abstraction [3]. Additionally,
Rust introduces the concepts of ownership, moves, and borrows together with a flexible static type system that helps the
developer explicitly state the lifetime for each resource, which makes garbage collection unnecessary and establishes the
foundations for robust concurrent programs [2] such as a RPS.


## Implementation

The implementation consists of three major modules, namely, `concurrent`, `http`, and `cache`.

The `concurrent` module defines all of the concurrency tools to manage incoming network requests. First, a concurrent FIFO
queue is defined in `concurrent/ccfifo_queue.rs` which is basically a wrapper around a multiple-producer-single-consumer
(MPSC) channel [5]. Secondly, `concurrent/pool.rs` defines all of the data types and utilities for working with a `ThreadPool`.
A `ThreadPool` is basically a list of `Worker` threads that are constantly listening to available `Jobs` via a MPSC channel.
A Job is just a pointer to a dynamically defined closure [6]:

```
type Job = Box<dyn FnOnce() + Send + 'static>;
```

The `http` module defines all of the utilities for handling socket connections, reading and writing to TCP streams, and
parsing HTTP requests and responses. `http/tcp.rs` defines utilities for binding the RPS to a specific address and port
so that it starts listening to TCP connections. For each incoming connection a `Job` is created and dispatched to the
`ThreadPool` channel. If there is any `Worker` available, the `Job` is executed. The specific `Job` that is dispatched to the
`ThreadPool` is an `http_handler` defined in `http/connection_handler.rs`. Here, the main logic of the proxy server is contained,
that is, the handler parses the incoming client request and checks whether the resource is in the cache. If the resource
is in the cache, it will be read directly from disk, otherwise the request will be proxy-passed to one of the servers available
in the server queue. The server queue is a FIFO queue of the form:

```
CCFifoQueue<Service>
```

where `Service` is defined as follows:


```
pub struct Service {
    pub addr: String,
    pub port: u16,
}
```

that is, the server queue allows each `Worker` to pop the service available on its head and push it to its tail. Thus
the services are round-robined and once the `Worker` accesses the service's address and port, it is capable of proxy-passing
the incoming request. When the service is done processing the request, it will reply to the corresponding `Worker` so it
decides whether or not the response should be cached and replies to the client. There are some criteria to determine if
a server response is cacheable:

- Its status code must be one of 20X.
- It must be a static resource, namely, its content-type must be one of `application/octet-stream`, `text/css`, `text/javascript`,
  `image/apng`, `image/avif`, `image/gif`, `image/jpeg`, `image/png`, `image/svg+xml`, `image/webp`, `image/bmp`, `image/x-icon`,
  `image/tiff`, `audio/webm`, `audio/mpeg`, `audio/ogg`, `audio/x-wav`, `audio/mp4`, `application/ogg`, and `application/pdf`.
  The reason is that other content types such as HTML and JSON are subject to dynamic changes depending on the client that
  sends the request, for example, an application might reply with different JSON payloads or HTML pages depending on the user
  account that sends the request.
- It must be a response to an HTTP GET request, which is the one specific for requesting resources.
- It must be a response whose body is not longer than 30MB. The rationale of this restriction is to prevent filling the
  available disk space with huge assets. On the other hand, this RPS supports neither compression nor chunked encodings
  which dwarfs the benefits of caching large assets.

A basic failure mechanism has been implemented in case that one of the proxied services is unavailable, that is, if
a request is proxied to a service that is temporarily unavailable, and the connection fails, the `Worker` thread will
retry the request `n` times with a specific `delay`. The maximum number of attempts and the delay are configurable parameters.

The `cache` module defines all of the utilities reading, writing and handling cache files. Whenever a `Worker` thread
determines that a given service response is cacheable, it will send a `CacheFile` to the `CacheWriter`:


```
pub struct CacheWriter {
    thread: JoinHandle<()>,
}

impl CacheWriter {
    pub fn run(cache_receiver: Receiver<CacheFile>) -> Self {
        ...
    }
}
```

The `CacheWriter` is basically a thread that is waiting for `CacheFile` requests on a queue. Whenever there is a new
file on the queue dispatched by a `Worker` thread, the `CacheWriter` will store that file in the cache directory via
the IO utilities provided by `cache/io.rs` submodule:


```
pub struct CacheFile {
    pub metadata: FileMetadata,
    pub path: PathBuf,
    pub content_data: Vec<u8>,
}

impl CacheFile {
    ...
    pub fn read_header(path: &PathBuf) -> Result<FileMetadata> {...}
    pub fn read(path: PathBuf, metadata: FileMetadata) -> Result<CacheFile> {...}
    pub fn write(&self) -> Result<()> {...}
}
```

Notice that `CacheWriter` requires a `FileMetadata` type with the following definition:

```
pub struct FileMetadata {
    timestamp: Duration, // System time when resource was stored.
    ttl_secs: Duration, // Time span in which the resource is valid.
    pub content_type: Option<String>, // Content type of the resource.
    pub content_length: u64, // Content length of the resource.
}
```

This metadata is appended to the header of each resource file on writing and removed on reading.

Cache files are not stored indefinitely on disk, but there is a cleaner thread defined in `cache/cleaner.rs` whose purpose
is to periodically traverse the cache directory and delete the expired cached files. This cleaner thread parses the
`FileMetadada` header from the file, and given the timestamp and ttl, it determines whether the file should be removed or not.



## Overall architecture

The following diagrams provides a global picture of the implementation discussed above:


![architectue](https://raw.githubusercontent.com/sebashack/rusty_proxy/main/rusty_proxy_arch.png)

## References

- [1] https://www.cloudflare.com/learning/cdn/glossary/reverse-proxy
- [2] Programming Rust, First Edition, O'Reilly; Jim Blandy, Jason Orendorff;
- [3] [The Rust Programming Language](https://doc.rust-lang.org/book/); Steve Klabnik, Carol Nichols;
- [4] https://www.nginx.com/resources/glossary/load-balancing
- [5] https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html
- [6] https://doc.rust-lang.org/rust-by-example/fn/closures.html


## Install dependencies

Execute:

```
./ first-time-install.sh
```

## Build binary

Execute:

```
make build
```

The binary's name is `rusty_proxy` and will be located at `<project-root>/dist`.


## Execute binary

```
./dist/rusty_proxy /path/to/config.yaml
```

There is an example configuration file at the root of this project named `example_config.yaml`.
