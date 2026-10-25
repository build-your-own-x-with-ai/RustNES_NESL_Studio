//! JSON payloads shared by the local HTTP server (`server`) and the browser
//! build (`nesc-wasm`).
//!
//! This module is deliberately free of I/O so that it compiles unchanged for
//! `wasm32-unknown-unknown`; both backends answer with byte-identical JSON.

use nesl::Diagnostic;

/// The programs shipped in `examples/`, embedded at compile time.
pub const EXAMPLES: &[(&str, &str)] = &[
    ("hello", include_str!("../../../examples/hello.nesl")),
    ("sprite", include_str!("../../../examples/sprite.nesl")),
    ("game", include_str!("../../../examples/game.nesl")),
    ("shooter", include_str!("../../../examples/shooter.nesl")),
    ("maze", include_str!("../../../examples/maze.nesl")),
    ("runner", include_str!("../../../examples/runner.nesl")),
    ("quest", include_str!("../../../examples/quest.nesl")),
    ("split", include_str!("../../../examples/split.nesl")),
    ("mario", include_str!("../../../output/decompiled/super_mario_bros.nesl")),
];

/// `{"ok":true,"rom":"..","prg_used":N,"asm":".."}` or `{"ok":false,"errors":[..]}`.
pub fn compile_json(src: &str) -> String {
    match nesl::compile(src) {
        Ok(out) => format!(
            "{{\"ok\":true,\"rom\":{},\"prg_used\":{},\"asm\":{}}}",
            json_str(&base64(&out.rom)),
            out.prg_used,
            json_str(&out.listing)
        ),
        Err(errs) => format!("{{\"ok\":false,\"errors\":{}}}", diagnostics(&errs)),
    }
}

/// `{"ok":true,"errors":[]}` or `{"ok":false,"errors":[..]}` — a compile with no ROM.
pub fn check_json(src: &str) -> String {
    match nesl::compile(src) {
        Ok(_) => "{\"ok\":true,\"errors\":[]}".to_string(),
        Err(errs) => format!("{{\"ok\":false,\"errors\":{}}}", diagnostics(&errs)),
    }
}

/// `[{"name":..,"source":..}, ..]` for the example picker.
pub fn examples_json() -> String {
    let items: Vec<String> = EXAMPLES
        .iter()
        .map(|(name, src)| format!("{{\"name\":{},\"source\":{}}}", json_str(name), json_str(src)))
        .collect();
    format!("[{}]", items.join(","))
}

/// Everything the editor needs for completion and hover, straight from the compiler.
pub fn builtins_json() -> String {
    let functions: Vec<String> = nesl::builtins::DOCS
        .iter()
        .map(|(n, sig, doc)| format!("{{\"name\":{},\"sig\":{},\"doc\":{}}}", json_str(n), json_str(sig), json_str(doc)))
        .collect();
    let constants: Vec<String> = nesl::builtins::CONSTANTS.iter().map(|(n, _)| json_str(n)).collect();
    // reserved keywords, contextual declarations and type names
    let extra = ["map", "metasprite", "palette_zone", "music", "sound", "true", "false", "u8", "i8", "u16"];
    let keywords: Vec<String> = nesl::lexer::KEYWORDS.iter().chain(extra.iter()).map(|k| json_str(k)).collect();
    format!(
        "{{\"functions\":[{}],\"constants\":[{}],\"keywords\":[{}]}}",
        functions.join(","),
        constants.join(","),
        keywords.join(",")
    )
}

/// `[{"line":N,"col":N,"message":".."}, ..]`
pub fn diagnostics(errs: &[Diagnostic]) -> String {
    let items: Vec<String> = errs
        .iter()
        .map(|d| format!("{{\"line\":{},\"col\":{},\"message\":{}}}", d.line, d.col, json_str(&d.message)))
        .collect();
    format!("[{}]", items.join(","))
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

/// Standard base64, used to move ROM bytes through JSON.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_reports_errors() {
        let json = compile_json("fn main() { nonsense( }");
        assert!(json.starts_with("{\"ok\":false"), "{json}");
    }

    #[test]
    fn examples_and_builtins_are_valid_json_lists() {
        let ex = examples_json();
        assert!(ex.starts_with('[') && ex.contains("\"hello\""));
        let bi = builtins_json();
        assert!(bi.contains("\"functions\":[") && bi.contains("\"keywords\":["));
    }

    #[test]
    fn round_trips_base64() {
        assert_eq!(base64(b"hello"), "aGVsbG8=");
        assert_eq!(base64(b""), "");
    }
}
