//! Recursive-descent parser producing the AST.

use crate::ast::*;
use crate::Diagnostic;

pub fn parse(_src: &str) -> Result<Program, Diagnostic> {
    Err(Diagnostic::new(Pos::default(), "not implemented"))
}
