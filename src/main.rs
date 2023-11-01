// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

fn parse_request(stream: &TcpStream) -> Result<String, String> {
    let mut request_reader = BufReader::new(stream);
    let mut request: String = Default::default();

    if request_reader.read_line(&mut request).is_err() {
        return Err("error parsing request".to_string());
    }

    if request.is_empty() {
        return Err("empty string was passed".to_string());
    }

    let words: Vec<&str> = request.split_ascii_whitespace().collect();

    let (command, arg) = (words.get(0).unwrap_or(&""), words.get(1).unwrap_or(&""));

    match *command {
        "GET" => {
            let response = if arg.starts_with("/") {
                "HTTP/1.1 200 OK\r\n\r\n"
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n"
            };
            Ok(response.to_string())
        }
        s => Err(format!("unknown command {}", s)),
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let response = parse_request(&stream);

    match response {
        Ok(_s) => {
            BufWriter::new(&stream).write_all(_s.as_bytes()).unwrap();
            let _ = stream.flush();
            Ok(())
        }
        Err(e) => Err(e),
    }
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
