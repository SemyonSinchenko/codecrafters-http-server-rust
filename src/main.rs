// Uncomment this block to pass the first stage
use std::{
    io::{BufWriter, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    println!("inside a function handle_connection");
    let res = "HTTP/1.1 200 OK\r\n\r\n";
    BufWriter::new(&stream).write_all(res.as_bytes()).unwrap();
    stream.flush()
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_connection(_stream).expect("should sent a response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
