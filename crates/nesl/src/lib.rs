//! NESL compiler library: compiles NESL source code into NES (iNES) ROM images.

pub mod asm;
pub mod ast;
pub mod lexer;
pub mod parser;

use std::fmt;

/// A compile error with a 1-based source position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn new(pos: ast::Pos, message: impl Into<String>) -> Self {
        Diagnostic { line: pos.line, col: pos.col, message: message.into() }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: error: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests;
