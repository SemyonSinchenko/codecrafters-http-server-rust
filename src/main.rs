use std::{
    collections::HashMap,
    env, fs,
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};

#[derive(Debug, PartialEq, Eq)]
enum Command {
    GET,
    POST,
}

#[derive(Debug, PartialEq, Eq)]
struct Request {
    command: Command,
    route: String,
    client: String,
    body: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: u16,
    content_type: Option<String>,
    content_len: Option<u16>,
    body: Option<String>,
}

impl Response {
    pub fn to_pure_string(&self) -> Result<String, String> {
        let status_line = if self.status == 404 {
            "HTTP/1.1 404 NOT FOUND\r\n"
        } else if self.status == 200 {
            "HTTP/1.1 200 OK\r\n"
        } else if self.status == 201 {
            "HTTP/1.1 201 OK\r\n"
        } else {
            return Err(format!("unknown status {}", self.status));
        };
        let header_lines = if self.status == 404 {
            "".to_string()
        } else {
            match self {
                Response {
                    status: _,
                    content_len: Some(ct),
                    content_type: Some(cl),
                    body: _,
                } => format!("Content-Type: {}\r\nContent-Length: {}\r\n", cl, ct),
                _ => "".to_string(),
            }
        };
        let body = match &self.body {
            Some(s) => format!("\r\n{}\r\n", s),
            None => "\r\n".to_string(),
        };

        Ok(format!("{}{}{}", status_line, header_lines, body))
    }
}

impl Request {
    pub fn parse_request(request: Vec<&str>) -> Result<Self, String> {
        if request
            .clone()
            .into_iter()
            .filter(|s| !s.is_empty())
            .count()
            == 1
        {
            let line: Vec<&str> = request.get(0).unwrap().split_ascii_whitespace().collect();
            if (*line.get(0).unwrap() == "GET") && (*line.get(1).unwrap() == "/") {
                return Ok(Request {
                    command: Command::GET,
                    route: "/".to_string(),
                    client: "".to_string(),
                    body: None,
                });
            }
        }

        let headers: HashMap<String, String> = {
            request[1..]
                .into_iter()
                .take_while(|s| !s.is_empty())
                .map(|s| s.split(":").collect())
                .fold(
                    HashMap::<String, String>::new(),
                    |mut acc: HashMap<String, String>, val: Vec<&str>| {
                        acc.insert(
                            val.get(0).unwrap().trim().to_string(),
                            val.get(1).unwrap().trim().to_string(),
                        );
                        acc
                    },
                )
        };

        println!("Headers:");
        for (k, v) in headers.iter() {
            println!("{}: {}", k, v)
        }

        let first_line: Vec<&str> = request.get(0).unwrap().split_ascii_whitespace().collect();
        if first_line.len() < 3 {
            return Err(format!("god bad line {}", request.get(0).unwrap()));
        };
        let command = match *first_line.get(0).unwrap() {
            "GET" => Command::GET,
            "POST" => Command::POST,
            _ => return Err(format!("unknown command {}", first_line.get(0).unwrap())),
        };
        let route = first_line.get(1).unwrap();

        let client = match headers.get("User-Agent") {
            Some(agent) => agent,
            None => return Err("bad client type".to_string()),
        };

        let content_length = match headers.get("Content-Length") {
            Some(s) => match s.parse::<u16>() {
                Ok(v) => v,
                _ => 0,
            },
            None => 0,
        };

        let body = if content_length > 0 {
            match request.get(1 + headers.len()..) {
                Some(body_lines) => {
                    println!("Length of body: {}", body_lines.len());
                    Some(body_lines.join(""))
                }
                None => None,
            }
        } else {
            None
        };

        println!("body: {}", body.is_some());

        Ok(Request {
            command,
            route: route.to_string(),
            client: client.to_string(),
            body,
        })
    }
}

fn generate_response(mut stream: &TcpStream, directory: PathBuf) -> Result<String, String> {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(n) => {
            let req_string = String::from_utf8_lossy(&buffer[..n]);
            let request = match Request::parse_request(req_string.split("\r\n").collect()) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };

            match request.command {
                Command::GET => {
                    if request.route == "/" {
                        Response {
                            status: 200,
                            content_len: None,
                            content_type: None,
                            body: None,
                        }
                        .to_pure_string()
                    } else if request.route.starts_with("/echo/") {
                        if request.route.len() > 6 {
                            let body = &request.route[6..];
                            Response {
                                status: 200,
                                content_len: Some(body.len() as u16),
                                content_type: Some("text/plain".to_string()),
                                body: Some(body.to_string()),
                            }
                            .to_pure_string()
                        } else {
                            return Err("bad request".to_string());
                        }
                    } else if request.route.starts_with("/files/") {
                        if request.route.len() > 7 {
                            let path = directory.join(&request.route[7..]);
                            if path.exists() {
                                let content = fs::read(path);
                                match content {
                                    Ok(s) => {
                                        let text = String::from_utf8(s).unwrap();

                                        Response {
                                            status: 200,
                                            content_len: Some(text.len() as u16),
                                            content_type: Some(
                                                "application/octet-stream".to_string(),
                                            ),
                                            body: Some(text),
                                        }
                                        .to_pure_string()
                                    }
                                    Err(_e) => return Err("error reading file".to_string()),
                                }
                            } else {
                                Response {
                                    status: 404,
                                    content_len: None,
                                    content_type: None,
                                    body: None,
                                }
                                .to_pure_string()
                            }
                        } else {
                            return Err("empty path was passed".to_string());
                        }
                    } else if request.route == "/user-agent" {
                        let body = request.client;
                        Response {
                            status: 200,
                            content_len: Some(body.len() as u16),
                            content_type: Some("text/plain".to_string()),
                            body: Some(body),
                        }
                        .to_pure_string()
                    } else {
                        Response {
                            status: 404,
                            content_len: None,
                            content_type: None,
                            body: None,
                        }
                        .to_pure_string()
                    }
                }
                Command::POST => {
                    if request.route.starts_with("/files/") {
                        if request.route.len() > 7 {
                            let path: PathBuf = directory.join(&request.route[7..]);

                            match request.body {
                                None => Err("empty body for post".to_string()),
                                Some(b) => {
                                    println!("Body: {}", b);
                                    fs::write(path, b).unwrap();
                                    Response {
                                        status: 201,
                                        content_len: None,
                                        content_type: None,
                                        body: None,
                                    }
                                    .to_pure_string()
                                }
                            }
                        } else {
                            Response {
                                status: 404,
                                content_len: None,
                                content_type: None,
                                body: None,
                            }
                            .to_pure_string()
                        }
                    } else {
                        Err("bad route for POST".to_string())
                    }
                }
            }
        }
        Err(_) => Err("error reading stream".to_string()),
    }
}

fn handle_connection(stream: std::io::Result<TcpStream>, directory: PathBuf) -> Result<(), String> {
    match stream {
        Ok(mut _stream) => {
            println!("accepted new connection");
            let response = generate_response(&_stream, directory);

            match response {
                Ok(_s) => {
                    println!("Response:\n{}", _s);
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
    let mut directory = PathBuf::from(".");
    let mut args = env::args().into_iter();

    while let Some(arg) = args.next() {
        if arg == "--directory" {
            directory = PathBuf::from(args.next().unwrap());
            break;
        }
    }
    println!("working directory: {}", directory.display());

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        let dir = directory.clone();
        thread::spawn(move || handle_connection(stream, dir));
    }
}
