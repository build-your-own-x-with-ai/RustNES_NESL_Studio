//! Built-in constants and NES API functions.

/// Controller button masks as returned by `joypad()`.
pub const CONSTANTS: &[(&str, u32)] = &[
    ("BTN_A", 0x80),
    ("BTN_B", 0x40),
    ("BTN_SELECT", 0x20),
    ("BTN_START", 0x10),
    ("BTN_UP", 0x08),
    ("BTN_DOWN", 0x04),
    ("BTN_LEFT", 0x02),
    ("BTN_RIGHT", 0x01),
];

pub fn constant(name: &str) -> Option<u32> {
    CONSTANTS.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

use crate::ast::{Expr, Pos, Type};
use crate::codegen::{Codegen, R};
use crate::runtime::Emitter;

pub struct Signature {
    pub params: &'static [Type],
    pub ret: Option<Type>,
}

pub fn signature(_name: &str) -> Option<Signature> {
    None
}

pub fn is_builtin_fn(name: &str) -> bool {
    signature(name).is_some()
}

/// Runtime routines backing the built-in functions.
pub fn emit_runtime(_e: &mut Emitter) {}

impl Codegen {
    pub(crate) fn builtin_call(&mut self, name: &str, _args: &[Expr], pos: Pos) -> R<Option<Type>> {
        Err(crate::Diagnostic::new(pos, format!("unknown function `{name}`")))
    }
}
