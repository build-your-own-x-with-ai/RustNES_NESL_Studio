//! Browser editor: a minimal std-only HTTP/1.1 server around `nesl::compile`.

use std::io::Read;
use std::net::TcpStream;

pub const MAX_BODY: usize = 1 << 20;

pub const EXAMPLES: &[(&str, &str)] = &[
    ("hello", include_str!("../../../examples/hello.nesl")),
    ("sprite", include_str!("../../../examples/sprite.nesl")),
    ("game", include_str!("../../../examples/game.nesl")),
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

pub fn parse_request(_r: &mut impl Read) -> Result<Request, HttpError> {
    Err(HttpError { status: 500, message: "todo".into() })
}

pub fn handle(_req: &Request) -> Response {
    Response { status: 500, content_type: "text/plain", body: vec![] }
}

pub fn handle_connection(_stream: TcpStream) {}

pub fn json_str(_s: &str) -> String {
    String::new()
}

pub fn base64(_data: &[u8]) -> String {
    String::new()
}

pub fn serve(_port: u16) -> std::io::Result<()> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "not implemented yet"))
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
