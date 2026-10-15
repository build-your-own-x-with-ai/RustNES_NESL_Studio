//! 6502 assembler: an in-memory instruction IR plus a two-pass assembler with
//! automatic branch relaxation and zero-page addressing selection.

use std::collections::BTreeMap;
use std::fmt::Write as _;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV, CMP, CPX,
    CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA,
    PLP, ROL, ROR, RTI, RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

/// A 16-bit value: a number or a label plus offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Val {
    Num(u16),
    Label(String, i32),
}

/// An immediate 8-bit value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Imm {
    Num(u8),
    Lo(String),
    Hi(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Imp,
    Acc,
    Imm(Imm),
    Abs(Val),
    AbsX(Val),
    AbsY(Val),
    Ind(Val),
    IndX(u8),
    IndY(u8),
}

impl Operand {
    pub fn imm(v: u8) -> Self { Operand::Imm(Imm::Num(v)) }
    pub fn imm_lo(l: &str) -> Self { Operand::Imm(Imm::Lo(l.into())) }
    pub fn imm_hi(l: &str) -> Self { Operand::Imm(Imm::Hi(l.into())) }
    pub fn abs(a: u16) -> Self { Operand::Abs(Val::Num(a)) }
    pub fn abs_x(a: u16) -> Self { Operand::AbsX(Val::Num(a)) }
    pub fn abs_y(a: u16) -> Self { Operand::AbsY(Val::Num(a)) }
    pub fn label(l: &str) -> Self { Operand::Abs(Val::Label(l.into(), 0)) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Label(String),
    Ins(Op, Operand),
    /// Conditional branch to a label; relaxed to `B!cc +3 / JMP label` when out of range.
    Branch(Op, String),
    Bytes(Vec<u8>),
    /// Little-endian 16-bit words.
    Words(Vec<Val>),
    Comment(String),
}

impl Item {
    pub fn ins(op: Op, operand: Operand) -> Self { Item::Ins(op, operand) }
}

#[derive(Clone, Debug)]
pub struct Assembled {
    pub origin: u16,
    pub code: Vec<u8>,
    pub symbols: BTreeMap<String, u16>,
    pub listing: String,
}

pub fn assemble(_items: &[Item], _origin: u16) -> Result<Assembled, String> {
    Err("not implemented".into())
}

#[cfg(test)]
#[path = "asm_tests.rs"]
mod tests;
