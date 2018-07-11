use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

extern crate server;
use server::WorkerPool;

fn main() {
    let num_workers: usize = 15;
    let max_reqs: usize = 50;
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let workers = WorkerPool::new(num_workers);

    for stream in listener.incoming().take(max_reqs) {
        let stream = stream.unwrap();
        workers.run(|| handler(stream));
    }

    println!("Received {} requests, which is the maximum. Closing all connections and stopping the server!", max_reqs);
}

fn handler(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let root = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, html) = if buffer.starts_with(root) {
        ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let mut filename = File::open(html).unwrap();
    let mut body = String::new();
    filename.read_to_string(&mut body).unwrap();

    let response = format!("{}{}", status_line, body);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
