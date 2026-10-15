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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode { Imp, Acc, Imm, Zp, ZpX, ZpY, Abs, AbsX, AbsY, Ind, IndX, IndY, Rel }

fn opcode(op: Op, mode: Mode) -> Option<u8> {
    use Mode::*;
    use Op::*;
    // (imm, zp, zpx, abs, absx, absy, indx, indy) for the regular ALU group
    let alu = |base: u8| -> Option<u8> {
        Some(match mode {
            Imm => base + 0x08, Zp => base + 0x04, ZpX => base + 0x14, Abs => base + 0x0C,
            AbsX => base + 0x1C, AbsY => base + 0x18, IndX => base, IndY => base + 0x10,
            _ => return None,
        })
    };
    // shifts / inc / dec: (acc, zp, zpx, abs, absx)
    let rmw = |base: u8, acc: bool| -> Option<u8> {
        Some(match mode {
            Acc if acc => base + 0x0A, Zp => base + 0x06, ZpX => base + 0x16,
            Abs => base + 0x0E, AbsX => base + 0x1E,
            _ => return None,
        })
    };
    let only = |m: Mode, code: u8| if mode == m { Some(code) } else { None };
    match op {
        ORA => alu(0x01), AND => alu(0x21), EOR => alu(0x41), ADC => alu(0x61),
        LDA => alu(0xA1), CMP => alu(0xC1), SBC => alu(0xE1),
        STA => if mode == Imm { None } else { alu(0x81) },
        ASL => rmw(0x00, true), ROL => rmw(0x20, true), LSR => rmw(0x40, true), ROR => rmw(0x60, true),
        DEC => rmw(0xC0, false), INC => rmw(0xE0, false),
        BPL => only(Rel, 0x10), BMI => only(Rel, 0x30), BVC => only(Rel, 0x50), BVS => only(Rel, 0x70),
        BCC => only(Rel, 0x90), BCS => only(Rel, 0xB0), BNE => only(Rel, 0xD0), BEQ => only(Rel, 0xF0),
        BIT => match mode { Zp => Some(0x24), Abs => Some(0x2C), _ => None },
        JMP => match mode { Abs => Some(0x4C), Ind => Some(0x6C), _ => None },
        JSR => only(Abs, 0x20),
        CPX => match mode { Imm => Some(0xE0), Zp => Some(0xE4), Abs => Some(0xEC), _ => None },
        CPY => match mode { Imm => Some(0xC0), Zp => Some(0xC4), Abs => Some(0xCC), _ => None },
        LDX => match mode {
            Imm => Some(0xA2), Zp => Some(0xA6), ZpY => Some(0xB6), Abs => Some(0xAE), AbsY => Some(0xBE),
            _ => None,
        },
        LDY => match mode {
            Imm => Some(0xA0), Zp => Some(0xA4), ZpX => Some(0xB4), Abs => Some(0xAC), AbsX => Some(0xBC),
            _ => None,
        },
        STX => match mode { Zp => Some(0x86), ZpY => Some(0x96), Abs => Some(0x8E), _ => None },
        STY => match mode { Zp => Some(0x84), ZpX => Some(0x94), Abs => Some(0x8C), _ => None },
        _ => {
            let code = match op {
                BRK => 0x00, PHP => 0x08, CLC => 0x18, PLP => 0x28, SEC => 0x38, RTI => 0x40,
                PHA => 0x48, CLI => 0x58, RTS => 0x60, PLA => 0x68, SEI => 0x78, DEY => 0x88,
                TXA => 0x8A, TYA => 0x98, TXS => 0x9A, TAY => 0xA8, TAX => 0xAA, CLV => 0xB8,
                TSX => 0xBA, INY => 0xC8, DEX => 0xCA, CLD => 0xD8, INX => 0xE8, NOP => 0xEA,
                SED => 0xF8,
                _ => unreachable!(),
            };
            only(Imp, code)
        }
    }
}

fn invert(op: Op) -> Op {
    match op {
        Op::BEQ => Op::BNE, Op::BNE => Op::BEQ, Op::BCC => Op::BCS, Op::BCS => Op::BCC,
        Op::BPL => Op::BMI, Op::BMI => Op::BPL, Op::BVC => Op::BVS, Op::BVS => Op::BVC,
        other => other,
    }
}

/// Picks the concrete addressing mode (zero page when the address is a small number).
fn select_mode(op: Op, operand: &Operand) -> Result<(Mode, u8), String> {
    let small = |v: &Val| matches!(v, Val::Num(n) if *n < 0x100);
    let candidates: &[Mode] = match operand {
        Operand::Imp => &[Mode::Imp],
        Operand::Acc => &[Mode::Acc],
        Operand::Imm(_) => &[Mode::Imm],
        Operand::Abs(v) if small(v) => &[Mode::Zp, Mode::Abs],
        Operand::Abs(_) => &[Mode::Abs],
        Operand::AbsX(v) if small(v) => &[Mode::ZpX, Mode::AbsX],
        Operand::AbsX(_) => &[Mode::AbsX],
        Operand::AbsY(v) if small(v) => &[Mode::ZpY, Mode::AbsY],
        Operand::AbsY(_) => &[Mode::AbsY],
        Operand::Ind(_) => &[Mode::Ind],
        Operand::IndX(_) => &[Mode::IndX],
        Operand::IndY(_) => &[Mode::IndY],
    };
    for &m in candidates {
        if let Some(code) = opcode(op, m) {
            return Ok((m, code));
        }
    }
    Err(format!("invalid addressing mode for {op:?}: {operand:?}"))
}

fn mode_len(m: Mode) -> u16 {
    match m {
        Mode::Imp | Mode::Acc => 1,
        Mode::Abs | Mode::AbsX | Mode::AbsY | Mode::Ind => 3,
        _ => 2,
    }
}

fn resolve(v: &Val, symbols: &BTreeMap<String, u16>) -> Result<u16, String> {
    match v {
        Val::Num(n) => Ok(*n),
        Val::Label(l, off) => symbols
            .get(l)
            .map(|a| (*a as i32 + off) as u16)
            .ok_or_else(|| format!("undefined label `{l}`")),
    }
}

fn fmt_val(v: &Val) -> String {
    match v {
        Val::Num(n) if *n < 0x100 => format!("${n:02X}"),
        Val::Num(n) => format!("${n:04X}"),
        Val::Label(l, 0) => l.clone(),
        Val::Label(l, o) => format!("{l}{o:+}"),
    }
}

fn fmt_operand(o: &Operand) -> String {
    match o {
        Operand::Imp => String::new(),
        Operand::Acc => " A".into(),
        Operand::Imm(Imm::Num(n)) => format!(" #${n:02X}"),
        Operand::Imm(Imm::Lo(l)) => format!(" #<{l}"),
        Operand::Imm(Imm::Hi(l)) => format!(" #>{l}"),
        Operand::Abs(v) => format!(" {}", fmt_val(v)),
        Operand::AbsX(v) => format!(" {},X", fmt_val(v)),
        Operand::AbsY(v) => format!(" {},Y", fmt_val(v)),
        Operand::Ind(v) => format!(" ({})", fmt_val(v)),
        Operand::IndX(z) => format!(" (${z:02X},X)"),
        Operand::IndY(z) => format!(" (${z:02X}),Y"),
    }
}

/// Assembles `items` starting at `origin`.
pub fn assemble(items: &[Item], origin: u16) -> Result<Assembled, String> {
    // Pre-compute instruction modes once; they never depend on label values.
    let mut modes = Vec::with_capacity(items.len());
    for it in items {
        modes.push(match it {
            Item::Ins(op, operand) => Some(select_mode(*op, operand)?),
            _ => None,
        });
    }
    let mut long = vec![false; items.len()];
    let mut symbols;
    // Layout passes until no branch needs to grow (branches only ever grow, so this terminates).
    loop {
        symbols = BTreeMap::new();
        let mut addrs = Vec::with_capacity(items.len());
        let mut pc = origin as u32;
        for (i, it) in items.iter().enumerate() {
            addrs.push(pc);
            match it {
                Item::Label(l) => {
                    if symbols.insert(l.clone(), pc as u16).is_some() {
                        return Err(format!("duplicate label `{l}`"));
                    }
                }
                Item::Ins(..) => pc += mode_len(modes[i].unwrap().0) as u32,
                Item::Branch(..) => pc += if long[i] { 5 } else { 2 },
                Item::Bytes(b) => pc += b.len() as u32,
                Item::Words(w) => pc += 2 * w.len() as u32,
                Item::Comment(_) => {}
            }
            if pc > 0x10000 {
                return Err("program does not fit in the address space".into());
            }
        }
        let mut grew = false;
        for (i, it) in items.iter().enumerate() {
            if let Item::Branch(_, l) = it {
                if long[i] {
                    continue;
                }
                let target = *symbols.get(l).ok_or_else(|| format!("undefined label `{l}`"))? as i64;
                let disp = target - (addrs[i] as i64 + 2);
                if !(-128..=127).contains(&disp) {
                    long[i] = true;
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }

    let mut code = Vec::new();
    let mut listing = String::new();
    for (i, it) in items.iter().enumerate() {
        let pc = origin as u32 + code.len() as u32;
        let start = code.len();
        let text = match it {
            Item::Label(l) => {
                let _ = writeln!(listing, "{l}:");
                continue;
            }
            Item::Comment(c) => {
                let _ = writeln!(listing, "                ; {c}");
                continue;
            }
            Item::Ins(op, operand) => {
                let (mode, opc) = modes[i].unwrap();
                code.push(opc);
                match operand {
                    Operand::Imm(Imm::Num(n)) => code.push(*n),
                    Operand::Imm(Imm::Lo(l)) => code.push(resolve(&Val::Label(l.clone(), 0), &symbols)? as u8),
                    Operand::Imm(Imm::Hi(l)) => code.push((resolve(&Val::Label(l.clone(), 0), &symbols)? >> 8) as u8),
                    Operand::IndX(z) | Operand::IndY(z) => code.push(*z),
                    Operand::Abs(v) | Operand::AbsX(v) | Operand::AbsY(v) | Operand::Ind(v) => {
                        let a = resolve(v, &symbols)?;
                        if mode_len(mode) == 2 {
                            code.push(a as u8);
                        } else {
                            code.extend_from_slice(&a.to_le_bytes());
                        }
                    }
                    Operand::Imp | Operand::Acc => {}
                }
                format!("{op:?}{}", fmt_operand(operand))
            }
            Item::Branch(op, l) => {
                let target = symbols[l];
                if long[i] {
                    code.push(opcode(invert(*op), Mode::Rel).unwrap());
                    code.push(3);
                    code.push(0x4C);
                    code.extend_from_slice(&target.to_le_bytes());
                    format!("{:?} *+5 / JMP {l}", invert(*op))
                } else {
                    let disp = target as i64 - (pc as i64 + 2);
                    code.push(opcode(*op, Mode::Rel).unwrap());
                    code.push(disp as i8 as u8);
                    format!("{op:?} {l}")
                }
            }
            Item::Bytes(b) => {
                code.extend_from_slice(b);
                ".byte".to_string()
            }
            Item::Words(ws) => {
                let mut names = Vec::new();
                for w in ws {
                    code.extend_from_slice(&resolve(w, &symbols)?.to_le_bytes());
                    names.push(fmt_val(w));
                }
                format!(".word {}", names.join(", "))
            }
        };
        let bytes = &code[start..];
        let hex: Vec<String> = bytes.iter().take(8).map(|b| format!("{b:02X}")).collect();
        let more = if bytes.len() > 8 { " .." } else { "" };
        let _ = writeln!(listing, "  {pc:04X}  {:<26} {text}", format!("{}{more}", hex.join(" ")));
    }
    Ok(Assembled { origin, code, symbols, listing })
}

#[cfg(test)]
#[path = "asm_tests.rs"]
mod tests;
