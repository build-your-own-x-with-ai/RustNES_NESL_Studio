//! Browser editor: a minimal std-only HTTP/1.1 server around `nesl::compile`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use crate::json::{builtins_json, check_json, compile_json, examples_json};

pub const MAX_BODY: usize = 1 << 20;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

const INDEX_HTML: &str = include_str!("../web/index.html");
const STUDIO_JS: &str = include_str!("../web/studio.js");

fn bad(status: u16, message: &str) -> HttpError {
    HttpError { status, message: message.into() }
}

/// Reads one HTTP/1.1 request (headers + `Content-Length` body).
pub fn parse_request(r: &mut impl Read) -> Result<Request, HttpError> {
    let mut reader = BufReader::new(r);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|_| bad(400, "bad request line"))?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or_else(|| bad(400, "missing method"))?.to_string();
    let target = parts.next().ok_or_else(|| bad(400, "missing path"))?;
    let path = target.split('?').next().unwrap_or("/").to_string();
    let mut content_length = 0usize;
    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(|_| bad(400, "bad header"))?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().map_err(|_| bad(400, "bad Content-Length"))?;
            }
        }
    }
    if content_length > MAX_BODY {
        return Err(bad(413, "request body too large"));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).map_err(|_| bad(400, "truncated body"))?;
    Ok(Request { method, path, body })
}

fn json(body: String) -> Response {
    Response { status: 200, content_type: "application/json; charset=utf-8", body: body.into_bytes() }
}

/// Routes a request (pure function, no I/O).
pub fn handle(req: &Request) -> Response {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => {
            Response { status: 200, content_type: "text/html; charset=utf-8", body: INDEX_HTML.as_bytes().to_vec() }
        }
        ("GET", "/api/examples") => json(examples_json()),
        ("POST", "/api/compile") => json(compile_json(&String::from_utf8_lossy(&req.body))),
        ("POST", "/api/check") => json(check_json(&String::from_utf8_lossy(&req.body))),
        ("GET", "/studio.js") => {
            Response { status: 200, content_type: "text/javascript; charset=utf-8", body: STUDIO_JS.as_bytes().to_vec() }
        }
        ("GET", "/api/builtins") => json(builtins_json()),
        _ => Response { status: 404, content_type: "text/plain; charset=utf-8", body: b"not found".to_vec() },
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Internal Server Error",
    }
}

fn write_response(stream: &mut TcpStream, r: &Response) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        r.status,
        reason(r.status),
        r.content_type,
        r.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&r.body)?;
    stream.flush()
}

pub fn handle_connection(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let response = match parse_request(&mut stream) {
        Ok(req) => handle(&req),
        Err(e) => Response { status: e.status, content_type: "text/plain; charset=utf-8", body: e.message.into_bytes() },
    };
    let _ = write_response(&mut stream, &response);
}

/// Serves the editor on 127.0.0.1:`port` until the process is stopped.
pub fn serve(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("NESL editor running at http://127.0.0.1:{port}/  (Ctrl+C to stop)");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || handle_connection(s));
            }
            Err(e) => eprintln!("connection failed: {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
