// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    thread,
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
    let request_reader = BufReader::new(stream);
    let mut lines = Vec::<String>::new();

    for l in request_reader.lines() {
        match l {
            Ok(_s) if _s.is_empty() => break,
            Ok(_s) => lines.push(_s),
            Err(_e) => return Err("failed to parse request".to_string()),
        }
    }

    println!("got request of {} lines", lines.len());

    if lines.len() == 0 {
        return Err("empty string was passed".to_string());
    }

    let words: Vec<&str> = lines.get(0).unwrap().split_ascii_whitespace().collect();
    let (command, arg) = (words.get(0).unwrap_or(&""), words.get(1).unwrap_or(&""));

    println!("got {} {}", command, arg);

    match *command {
        "GET" => {
            let response = if *arg == "/" {
                "HTTP/1.1 200 OK\r\n\r\n".to_string()
            } else if arg.starts_with("/echo/") && arg.len() > 6 {
                let input_str = &arg[6..];
                generate_response_body(200, input_str.len() as u16, input_str)
            } else if *arg == "/user-agent" {
                if lines.len() < 3 {
                    return Err("wrong request".to_string());
                } else {
                    let user_agent = lines.get(2).unwrap();
                    if !user_agent.starts_with("User-Agent: ") {
                        return Err("wrong request, bad header".to_string());
                    } else {
                        generate_response_body(
                            200,
                            (user_agent.len() - 12) as u16,
                            user_agent[11..].trim(),
                        )
                    }
                }
            } else {
                "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
            };
            Ok(response)
        }
        s => Err(format!("unknown command {}", s)),
    }
}

fn handle_connection(stream: std::io::Result<TcpStream>) -> Result<(), String> {
    match stream {
        Ok(mut _stream) => {
            println!("accepted new connection");
            let response = parse_request(&_stream);

            match response {
                Ok(_s) => {
                    println!("{}", _s);
                    BufWriter::new(&_stream).write_all(_s.as_bytes()).unwrap();
                    let _ = _stream.flush();
                    Ok(())
                }
                Err(_e) => {
                    println!("error happened! {}", _e);
                    Ok(())
                }
            }
        }
        Err(_e) => {
            println!("error happened! {}", _e);
            Ok(())
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        thread::spawn(move || {
            let _ = handle_connection(stream);
        });
    }
}
