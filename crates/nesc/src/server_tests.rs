use super::*;
use std::io::{Read, Write};

fn req(method: &str, path: &str, body: &str) -> Request {
    Request { method: method.into(), path: path.into(), body: body.as_bytes().to_vec() }
}

#[test]
fn parses_http_requests_with_bodies() {
    let raw = b"POST /api/compile HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\n\r\nhello";
    let r = parse_request(&mut &raw[..]).unwrap();
    assert_eq!((r.method.as_str(), r.path.as_str(), r.body.as_slice()), ("POST", "/api/compile", &b"hello"[..]));
    let raw = b"GET /?x=1 HTTP/1.1\r\n\r\n";
    assert_eq!(parse_request(&mut &raw[..]).unwrap().path, "/");
}

#[test]
fn rejects_oversized_bodies() {
    let raw = format!("POST /api/compile HTTP/1.1\r\nContent-Length: {}\r\n\r\n", MAX_BODY + 1);
    assert_eq!(parse_request(&mut raw.as_bytes()).unwrap_err().status, 413);
}

#[test]
fn json_and_base64_encoding() {
    assert_eq!(json_str("a\"b\\c\n\u{1}é"), "\"a\\\"b\\\\c\\n\\u0001é\"");
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"f"), "Zg==");
    assert_eq!(base64(b"fo"), "Zm8=");
    assert_eq!(base64(b"foobar"), "Zm9vYmFy");
}

#[test]
fn index_serves_the_editor() {
    let r = handle(&req("GET", "/", ""));
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("text/html"));
    assert!(String::from_utf8_lossy(&r.body).contains("NESL"));
}

#[test]
fn compile_success_returns_rom_and_listing() {
    let r = handle(&req("POST", "/api/compile", "fn main() {}"));
    let body = String::from_utf8(r.body).unwrap();
    assert_eq!(r.status, 200);
    assert!(body.starts_with("{\"ok\":true"), "{body}");
    let rom = nesl::compile("fn main() {}").unwrap().rom;
    assert!(body.contains(&json_str(&base64(&rom))));
    assert!(body.contains("\"asm\":"));
    assert!(body.contains("\"prg_used\":"));
}

#[test]
fn compile_failure_returns_positioned_errors() {
    let r = handle(&req("POST", "/api/compile", "fn main() {\n  x = 1;\n}"));
    let body = String::from_utf8(r.body).unwrap();
    assert_eq!(r.status, 200);
    assert!(body.contains("\"ok\":false"), "{body}");
    assert!(body.contains("{\"line\":2,\"col\":3,\"message\":\"unknown variable `x`\"}"), "{body}");
}

#[test]
fn examples_endpoint_lists_bundled_examples() {
    let r = handle(&req("GET", "/api/examples", ""));
    let body = String::from_utf8(r.body).unwrap();
    assert!(r.content_type.starts_with("application/json"));
    for name in ["hello", "sprite", "game", "shooter", "maze", "runner", "quest", "split"] {
        assert!(body.contains(&format!("\"name\":\"{name}\"")), "{body}");
    }
    for (_, src) in EXAMPLES {
        assert!(nesl::compile(src).is_ok());
    }
}

#[test]
fn unknown_routes_are_404() {
    assert_eq!(handle(&req("GET", "/nope", "")).status, 404);
    assert_eq!(handle(&req("DELETE", "/api/compile", "")).status, 404);
}

#[test]
fn serves_over_tcp() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream);
    });
    let mut s = std::net::TcpStream::connect(addr).unwrap();
    s.write_all(b"POST /api/compile HTTP/1.1\r\nContent-Length: 12\r\n\r\nfn main() {}").unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    assert!(out.starts_with("HTTP/1.1 200 OK\r\n"), "{out}");
    assert!(out.contains("\"ok\":true"));
}

#[test]
fn serves_the_studio_script() {
    let r = handle(&req("GET", "/studio.js", ""));
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("text/javascript"));
    assert!(String::from_utf8_lossy(&r.body).contains("Studio"));
}

#[test]
fn builtins_endpoint_lists_documented_functions_constants_and_keywords() {
    let r = handle(&req("GET", "/api/builtins", ""));
    let body = String::from_utf8(r.body).unwrap();
    assert!(r.content_type.starts_with("application/json"));
    assert!(body.contains("{\"name\":\"sprite\",\"sig\":\"sprite(i, x, y, tile, attr)\",\"doc\":"), "{body}");
    assert_eq!(body.matches("\"sig\":").count(), nesl::builtins::DOCS.len());
    for c in ["\"BTN_A\"", "\"SFX_JUMP\"", "\"fn\"", "\"map\"", "\"palette_zone\"", "\"u16\""] {
        assert!(body.contains(c), "missing {c}");
    }
}

#[test]
fn check_endpoint_reports_diagnostics_without_a_rom() {
    let ok = String::from_utf8(handle(&req("POST", "/api/check", "fn main() {}")).body).unwrap();
    assert_eq!(ok, "{\"ok\":true,\"errors\":[]}");
    let bad = String::from_utf8(handle(&req("POST", "/api/check", "fn main() {\n  q = 1;\n}")).body).unwrap();
    assert!(bad.contains("\"ok\":false") && bad.contains("{\"line\":2,\"col\":3,"), "{bad}");
    assert!(!bad.contains("rom"));
}
