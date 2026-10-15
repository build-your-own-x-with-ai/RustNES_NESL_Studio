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

pub fn lex(_src: &str) -> Result<Vec<Token>, Diagnostic> {
    Err(Diagnostic::new(Pos::default(), "not implemented"))
}
