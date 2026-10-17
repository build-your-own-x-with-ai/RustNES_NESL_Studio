//! Browser editor: a minimal std-only HTTP/1.1 server around `nesl::compile`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub const MAX_BODY: usize = 1 << 20;

pub const EXAMPLES: &[(&str, &str)] = &[
    ("hello", include_str!("../../../examples/hello.nesl")),
    ("sprite", include_str!("../../../examples/sprite.nesl")),
    ("game", include_str!("../../../examples/game.nesl")),
    ("shooter", include_str!("../../../examples/shooter.nesl")),
    ("maze", include_str!("../../../examples/maze.nesl")),
];

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
        ("GET", "/api/examples") => {
            let items: Vec<String> = EXAMPLES
                .iter()
                .map(|(name, src)| format!("{{\"name\":{},\"source\":{}}}", json_str(name), json_str(src)))
                .collect();
            json(format!("[{}]", items.join(",")))
        }
        ("POST", "/api/compile") => {
            let src = String::from_utf8_lossy(&req.body);
            json(match nesl::compile(&src) {
                Ok(out) => format!(
                    "{{\"ok\":true,\"rom\":{},\"prg_used\":{},\"asm\":{}}}",
                    json_str(&base64(&out.rom)),
                    out.prg_used,
                    json_str(&out.listing)
                ),
                Err(errs) => {
                    let items: Vec<String> = errs
                        .iter()
                        .map(|d| format!("{{\"line\":{},\"col\":{},\"message\":{}}}", d.line, d.col, json_str(&d.message)))
                        .collect();
                    format!("{{\"ok\":false,\"errors\":[{}]}}", items.join(","))
                }
            })
        }
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

/// JSON string literal with escaping.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn base64(data: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
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
