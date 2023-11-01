// Uncomment this block to pass the first stage
use std::{net::{TcpListener, TcpStream}, io::{BufWriter, Write}};

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let res = "HTTP/1.1 200 OK\r\n\r\n";
    BufWriter::new(&stream).write_all(res.as_bytes()).unwrap();
    stream.flush()
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                match handle_connection(_stream) {
                    Ok(_r) => println!("sent a response"),
                    Err(e) => println!("error happened: {}", e)
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
