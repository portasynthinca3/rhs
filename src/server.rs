use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::io::Write;
use std::fs;

use crate::log;
use crate::config::Config;

fn read_line(stream: &mut TcpStream, buf: &mut String) -> Result<usize, std::io::Error> {
    buf.clear();
    let mut char_buf = [0u8];
    loop {
        stream.read(&mut char_buf)?;
        let character = char_buf[0] as char;

        if character == '\n' {
            return Ok(buf.len());
        }
        if character != '\r' {
            buf.push(character);
        }
    }
}

enum HttpReadState {
    ReadInitial,
    ReadHeaders,
}

enum HttpRequest {
    Get { path: String, headers: HashMap<String, String> },
    Unsupported,
}

struct HttpResponse {
    status: String,
    headers: HashMap<String, String>,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, std::io::Error> {
    // for storing request data
    let mut method = String::new();
    let mut path = String::new();
    let mut headers = HashMap::<String, String>::new();

    // read request line by line
    let mut buffer = String::new();
    let mut state = HttpReadState::ReadInitial;
    loop {
        read_line(stream, &mut buffer)?;

        match state {
            HttpReadState::ReadInitial => {
                let mut parts = buffer.split_whitespace();
                method = parts.next().unwrap().to_string();
                path = parts.next().unwrap().to_string();
                state = HttpReadState::ReadHeaders;
            },
            HttpReadState::ReadHeaders => {
                if buffer.is_empty() {
                    break;
                }
                let mut parts = buffer.split_whitespace();
                let key = parts.next().unwrap().to_string();
                let val = parts.next().unwrap().to_string();
                headers.insert(key, val);
            }
        }
    }

    if method == "GET" {
        return Ok(HttpRequest::Get{
            path,
            headers,
        });
    } else {
        return Ok(HttpRequest::Unsupported);
    }
}

fn write_response(stream: &mut TcpStream, response: HttpResponse) -> Result<(), std::io::Error> {
    let status_line = format!("HTTP/1.1 {}\r\n", response.status);
    stream.write(status_line.as_bytes())?;

    for (key, val) in response.headers {
        let header_line = format!("{}: {}\r\n", key, val);
        stream.write(header_line.as_bytes())?;
    }

    stream.write(b"\r\n")?;
    stream.write(response.body.as_bytes())?;

    Ok(())
}

fn read_file(path: &str, trying_index: bool) -> Result<(&str, String), std::io::Error> {
    let read_result = match trying_index {
        false => fs::read_to_string(path),
        true  => fs::read_to_string(&format!("{}/index.html", path)),
    };

    match read_result {
        Ok(contents) => {
            return Ok(("200 OK", contents))
        },
        Err(_) => if trying_index {
            // if we're already trying index.html and it doesn't exist, it's a 404
            return Ok(("404 Not Found", "404 Not Found\nrhs/0.1".to_string()));
        } else {
            // else, try index.html
            return read_file(path, true);
        }
    };
}

fn handle_connection(config: &Config, mut stream: TcpStream) -> Result<(), std::io::Error> {
    log::info(&format!("got connection from {}", stream.peer_addr().unwrap()));
    let request = read_request(&mut stream)?;

    let mut headers = HashMap::<String, String>::new();
    headers.insert("Server".to_string(), "rhs/0.1".to_string());

    match request {
        HttpRequest::Unsupported => {
            write_response(&mut stream, HttpResponse {
                status: "501 Not Implemented".to_string(),
                headers: headers,
                body: "501 Not Implemented\nrhs/0.1".to_string(),
            })?;
            return Ok(());
        },
        HttpRequest::Get{ path, headers } => {
            let file_path = config.dir.clone() + &path;
            let (status, body) = read_file(&file_path, false)?;
            write_response(&mut stream, HttpResponse {
                status: status.to_string(),
                body: body,
                headers
            })?;
            return Ok(());
        }
    }
}

pub fn serve(config: Config) {
    log::done(&format!("serving `{}` on port `{}`", config.dir, config.port));

    // bind to localhost:<port>
    let address = format!("127.0.0.1:{}", config.port);
    let address = address.as_str();
    let listener = match TcpListener::bind(address) {
        Ok(l) => l,
        Err(_) => {
            log::error(&format!("could not bind to {}", address));
            return;
        }
    };

    // iterate over incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(&config, stream).unwrap(),
            Err(_) => log::error(&"could not accept connection".to_string())
        };
    }
}