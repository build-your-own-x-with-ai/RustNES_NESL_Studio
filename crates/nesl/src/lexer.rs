//! Tokenizer for NESL source text.

use crate::ast::Pos;
use crate::Diagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    Int(u32),
    Str(Vec<u8>),
    Ident(String),
    Kw(&'static str),
    Sym(&'static str),
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

pub const KEYWORDS: &[&str] = &[
    "fn", "let", "var", "const", "tile", "if", "else", "while", "loop", "for", "in", "break",
    "continue", "return", "as", "struct", "match",
];

// Longest symbols first so that matching is greedy.
const SYMBOLS: &[&str] = &[
    "<<=", ">>=", "..=", "->", "=>", "..", "==", "!=", "<=", ">=", "<<", ">>", "&&", "||", "+=", "-=", "*=",
    "/=", "%=", "&=", "|=", "^=", "(", ")", "{", "}", "[", "]", ",", ";", ":", ".", "+", "-", "*",
    "/", "%", "&", "|", "^", "~", "!", "=", "<", ">",
];

struct Lexer<'a> {
    src: &'a [u8],
    i: usize,
    line: u32,
    col: u32,
}

impl Lexer<'_> {
    fn pos(&self) -> Pos {
        Pos { line: self.line, col: self.col }
    }

    fn peek(&self, k: usize) -> u8 {
        self.src.get(self.i + k).copied().unwrap_or(0)
    }

    fn bump(&mut self) -> u8 {
        let c = self.peek(0);
        self.i += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else if c & 0xC0 != 0x80 {
            // count columns in characters, not UTF-8 continuation bytes
            self.col += 1;
        }
        c
    }

    fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            match (self.peek(0), self.peek(1)) {
                (b' ' | b'\t' | b'\r' | b'\n', _) => {
                    self.bump();
                }
                (b'/', b'/') => {
                    while self.i < self.src.len() && self.peek(0) != b'\n' {
                        self.bump();
                    }
                }
                (b'/', b'*') => {
                    let start = self.pos();
                    self.bump();
                    self.bump();
                    while !(self.peek(0) == b'*' && self.peek(1) == b'/') {
                        if self.i >= self.src.len() {
                            return Err(Diagnostic::new(start, "unterminated block comment"));
                        }
                        self.bump();
                    }
                    self.bump();
                    self.bump();
                }
                _ => return Ok(()),
            }
        }
    }

    fn escape(&mut self, start: Pos) -> Result<u8, Diagnostic> {
        let c = self.bump();
        Ok(match c {
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'0' => 0,
            b'\\' => b'\\',
            b'"' => b'"',
            b'\'' => b'\'',
            b'x' => {
                let h = [self.bump(), self.bump()];
                let s = std::str::from_utf8(&h).unwrap_or("");
                u8::from_str_radix(s, 16)
                    .map_err(|_| Diagnostic::new(start, "invalid \\x escape (expected 2 hex digits)"))?
            }
            _ => return Err(Diagnostic::new(start, format!("unknown escape `\\{}`", c as char))),
        })
    }

    fn number(&mut self, start: Pos) -> Result<Tok, Diagnostic> {
        let (radix, skip) = match (self.peek(0), self.peek(1)) {
            (b'0', b'x' | b'X') => (16, 2),
            (b'0', b'b' | b'B') => (2, 2),
            _ => (10, 0),
        };
        for _ in 0..skip {
            self.bump();
        }
        let mut digits = String::new();
        while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == b'_' {
            let c = self.bump();
            if c != b'_' {
                digits.push(c as char);
            }
        }
        if digits.is_empty() {
            return Err(Diagnostic::new(start, "malformed number"));
        }
        match u64::from_str_radix(&digits, radix) {
            Ok(v) if v <= u32::MAX as u64 => Ok(Tok::Int(v as u32)),
            Ok(_) => Err(Diagnostic::new(start, "number too large")),
            Err(_) => {
                if digits.chars().all(|c| c.is_digit(radix)) {
                    Err(Diagnostic::new(start, "number too large"))
                } else {
                    Err(Diagnostic::new(start, format!("invalid digit in number `{digits}`")))
                }
            }
        }
    }

    fn next(&mut self) -> Result<Token, Diagnostic> {
        self.skip_trivia()?;
        let pos = self.pos();
        let c = self.peek(0);
        if self.i >= self.src.len() {
            return Ok(Token { tok: Tok::Eof, pos });
        }
        let tok = if c.is_ascii_digit() {
            self.number(pos)?
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let start = self.i;
            while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == b'_' {
                self.bump();
            }
            let word = std::str::from_utf8(&self.src[start..self.i]).unwrap();
            match word {
                "true" => Tok::Int(1),
                "false" => Tok::Int(0),
                _ => match KEYWORDS.iter().find(|k| **k == word) {
                    Some(k) => Tok::Kw(k),
                    None => Tok::Ident(word.to_string()),
                },
            }
        } else if c == b'"' {
            self.bump();
            let mut bytes = Vec::new();
            loop {
                if self.i >= self.src.len() || self.peek(0) == b'\n' {
                    return Err(Diagnostic::new(pos, "unterminated string literal"));
                }
                match self.peek(0) {
                    b'"' => {
                        self.bump();
                        break;
                    }
                    b'\\' => {
                        let at = self.pos();
                        self.bump();
                        bytes.push(self.escape(at)?);
                    }
                    _ => bytes.push(self.bump()),
                }
            }
            Tok::Str(bytes)
        } else if c == b'\'' {
            self.bump();
            let v = match self.peek(0) {
                b'\\' => {
                    let at = self.pos();
                    self.bump();
                    self.escape(at)?
                }
                b'\'' | b'\n' => return Err(Diagnostic::new(pos, "empty character literal")),
                _ => self.bump(),
            };
            if self.bump() != b'\'' {
                return Err(Diagnostic::new(pos, "unterminated character literal"));
            }
            Tok::Int(v as u32)
        } else {
            let rest = &self.src[self.i..];
            match SYMBOLS.iter().find(|s| rest.starts_with(s.as_bytes())) {
                Some(s) => {
                    for _ in 0..s.len() {
                        self.bump();
                    }
                    Tok::Sym(s)
                }
                None => {
                    let ch = std::str::from_utf8(rest)
                        .ok()
                        .and_then(|s| s.chars().next())
                        .unwrap_or('?');
                    return Err(Diagnostic::new(pos, format!("unexpected character `{ch}`")));
                }
            }
        };
        Ok(Token { tok, pos })
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer { src: src.as_bytes(), i: 0, line: 1, col: 1 };
    let mut out = Vec::new();
    loop {
        let t = lx.next()?;
        let eof = t.tok == Tok::Eof;
        out.push(t);
        if eof {
            return Ok(out);
        }
    }
}
