// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

fn generate_response_body(code: u16, len: u16, body: &str) -> String {
    let header = format!(
        "HTTP/1.1 {} {}\r\n",
        code,
        if code == 200 { "OK" } else { "Not Found" }
    );
    let content_header = format!(
        "Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
        len
    );
    return format!("{}{}{}", header, content_header, body);
}

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

    println!("got {} {}", command, arg);

    match *command {
        "GET" => {
            let response = if *arg == "/" {
                "HTTP/1.1 200 OK\r\n\r\n".to_string()
            } else if arg.starts_with("/echo/") && arg.len() > 6 {
                let input_str = arg.split("/").last().unwrap();
                generate_response_body(200, input_str.len() as u16, input_str)
            } else {
                "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
            };
            Ok(response)
        }
        s => Err(format!("unknown command {}", s)),
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let response = parse_request(&stream);

    match response {
        Ok(_s) => {
            println!("{}", _s);
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
