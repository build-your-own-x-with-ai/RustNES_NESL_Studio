//! Feature 019：文本 6502 汇编器。
//!
//! 解析 `roundtrip::disassemble` 生成的源码格式（.ines / .org / .chr /
//! 标签 / 指令 / .byte / .word），两遍汇编，输出完整 iNES 文件。
//! 约定：`$XX`（两位十六进制）= 零页寻址，`$XXX`/`$XXXX` = 绝对寻址。

use crate::asm::{encode_opcode, Mode, Op};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
enum Val {
    Num(u16),
    Label(String),
}

#[derive(Clone, Debug)]
enum Arg {
    Imp,
    Acc,
    Imm(u8),
    Zp(u8),
    ZpX(u8),
    ZpY(u8),
    Abs(Val),
    AbsX(Val),
    AbsY(Val),
    Ind(Val),
    IndX(u8),
    IndY(u8),
    /// 分支：操作数是目标地址（标签或字面量），编码为相对偏移。
    Rel(Val),
}

#[derive(Debug)]
enum Line {
    Label(String),
    Ins(Op, Arg),
    Bytes(Vec<u8>),
    Words(Vec<Val>),
}

fn arg_len(arg: &Arg) -> u32 {
    match arg {
        Arg::Imp | Arg::Acc => 0,
        Arg::Abs(_) | Arg::AbsX(_) | Arg::AbsY(_) | Arg::Ind(_) => 2,
        _ => 1,
    }
}

/// 去掉注释（`;` 之后），忽略双引号字符串内的 `;`。
fn strip_comment(s: &str) -> &str {
    let mut in_quotes = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ';' if !in_quotes => return &s[..i],
            _ => {}
        }
    }
    s
}

fn parse_num(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('$') {
        u16::from_str_radix(hex, 16).map_err(|_| format!("无效的十六进制数 `{s}`"))
    } else {
        s.parse::<u16>().map_err(|_| format!("无效的数字 `{s}`"))
    }
}

fn parse_val(s: &str) -> Result<Val, String> {
    let s = s.trim();
    if s.starts_with('$') || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        Ok(Val::Num(parse_num(s)?))
    } else if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(Val::Label(s.to_string()))
    } else {
        Err(format!("无效的值 `{s}`"))
    }
}

fn parse_op(s: &str) -> Option<Op> {
    use Op::*;
    const OPS: [Op; 56] = [
        ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV,
        CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA, LDX, LDY, LSR, NOP,
        ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI, RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX,
        TAY, TSX, TXA, TXS, TYA,
    ];
    let upper = s.to_ascii_uppercase();
    OPS.into_iter().find(|op| format!("{op:?}") == upper)
}

fn is_branch(op: Op) -> bool {
    matches!(op, Op::BCC | Op::BCS | Op::BEQ | Op::BMI | Op::BNE | Op::BPL | Op::BVC | Op::BVS)
}

/// `$XX` 两位 → 零页；更多位 → 绝对。标签 → 绝对。
fn parse_mem_val(s: &str) -> Result<(Val, bool), String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('$') {
        let zp = hex.len() <= 2;
        Ok((Val::Num(parse_num(s)?), zp))
    } else {
        Ok((parse_val(s)?, false))
    }
}

fn parse_arg(op: Op, s: &str) -> Result<Arg, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Arg::Imp);
    }
    if s.eq_ignore_ascii_case("A") {
        return Ok(Arg::Acc);
    }
    if let Some(rest) = s.strip_prefix('#') {
        let n = parse_num(rest)?;
        if n > 0xFF {
            return Err(format!("立即数超出范围 `{s}`"));
        }
        return Ok(Arg::Imm(n as u8));
    }
    if let Some(inner) = s.strip_prefix('(') {
        if let Some(core) = inner.strip_suffix(",X)") {
            let n = parse_num(core)?;
            return Ok(Arg::IndX(n as u8));
        }
        if let Some(core) = inner.strip_suffix("),Y") {
            let n = parse_num(core)?;
            return Ok(Arg::IndY(n as u8));
        }
        if let Some(core) = inner.strip_suffix(')') {
            return Ok(Arg::Ind(parse_val(core)?));
        }
        return Err(format!("无效的间接寻址 `{s}`"));
    }
    if is_branch(op) {
        return Ok(Arg::Rel(parse_val(s)?));
    }
    let (core, idx) = if let Some(c) = s.strip_suffix(",X").or_else(|| s.strip_suffix(",x")) {
        (c, Some('X'))
    } else if let Some(c) = s.strip_suffix(",Y").or_else(|| s.strip_suffix(",y")) {
        (c, Some('Y'))
    } else {
        (s, None)
    };
    let (val, zp) = parse_mem_val(core)?;
    Ok(match (idx, zp) {
        (None, true) => Arg::Zp(num_u8(&val)),
        (Some('X'), true) => Arg::ZpX(num_u8(&val)),
        (Some('Y'), true) => Arg::ZpY(num_u8(&val)),
        (None, false) => Arg::Abs(val),
        (Some('X'), false) => Arg::AbsX(val),
        _ => Arg::AbsY(val),
    })
}

fn num_u8(v: &Val) -> u8 {
    match v {
        Val::Num(n) => *n as u8,
        Val::Label(_) => unreachable!("零页操作数不支持标签"),
    }
}

fn arg_mode(arg: &Arg) -> Mode {
    match arg {
        Arg::Imp => Mode::Imp,
        Arg::Acc => Mode::Acc,
        Arg::Imm(_) => Mode::Imm,
        Arg::Zp(_) => Mode::Zp,
        Arg::ZpX(_) => Mode::ZpX,
        Arg::ZpY(_) => Mode::ZpY,
        Arg::Abs(_) => Mode::Abs,
        Arg::AbsX(_) => Mode::AbsX,
        Arg::AbsY(_) => Mode::AbsY,
        Arg::Ind(_) => Mode::Ind,
        Arg::IndX(_) => Mode::IndX,
        Arg::IndY(_) => Mode::IndY,
        Arg::Rel(_) => Mode::Rel,
    }
}

/// 源码中 `.chr` 指令引用的文件名（供 CLI 在汇编前读取）。
pub fn chr_file_name(src: &str) -> Option<String> {
    for raw in src.lines() {
        let line = strip_comment(raw).trim();
        if let Some(rest) = line.strip_prefix(".chr") {
            let rest = rest.trim();
            if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
                return Some(rest[1..rest.len() - 1].to_string());
            }
        }
    }
    None
}

/// 汇编源码为完整 .nes 镜像。`chr_override` 优先于源码中的 `.chr` 指令。
pub fn assemble_text(src: &str, chr_override: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut prg_banks: Option<usize> = None;
    let mut chr_banks: usize = 0;
    let mut mapper: u8 = 0;
    let mut flags6: u8 = 0;
    let mut org: Option<u16> = None;
    let mut has_chr_directive = false;
    let mut lines: Vec<Line> = Vec::new();

    for (ln, raw) in src.lines().enumerate() {
        let n = ln + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix(".ines") {
            for tok in rest.split_whitespace() {
                let (k, v) = tok.split_once('=').ok_or(format!("第 {n} 行：.ines 参数 `{tok}` 需要 k=v"))?;
                let v = parse_num(v).map_err(|e| format!("第 {n} 行：{e}"))?;
                match k {
                    "prg" => prg_banks = Some(v as usize),
                    "chr" => chr_banks = v as usize,
                    "mapper" => mapper = v as u8,
                    "flags6" => flags6 = v as u8,
                    _ => return Err(format!("第 {n} 行：未知的 .ines 参数 `{k}`")),
                }
            }
        } else if line.starts_with(".chr") {
            has_chr_directive = true;
        } else if let Some(rest) = line.strip_prefix(".org") {
            org = Some(parse_num(rest).map_err(|e| format!("第 {n} 行：{e}"))?);
        } else if let Some(rest) = line.strip_prefix(".byte") {
            let mut bytes = Vec::new();
            for part in rest.split(',') {
                let v = parse_num(part).map_err(|e| format!("第 {n} 行：{e}"))?;
                if v > 0xFF {
                    return Err(format!("第 {n} 行：.byte 值超出范围 `{part}`"));
                }
                bytes.push(v as u8);
            }
            lines.push(Line::Bytes(bytes));
        } else if let Some(rest) = line.strip_prefix(".word") {
            let mut vals = Vec::new();
            for part in rest.split(',') {
                vals.push(parse_val(part).map_err(|e| format!("第 {n} 行：{e}"))?);
            }
            lines.push(Line::Words(vals));
        } else if let Some(name) = line.strip_suffix(':') {
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(format!("第 {n} 行：无效的标签名 `{name}`"));
            }
            lines.push(Line::Label(name.to_string()));
        } else {
            let (mn, rest) = match line.find(char::is_whitespace) {
                Some(i) => (&line[..i], &line[i..]),
                None => (line, ""),
            };
            let op = parse_op(mn).ok_or(format!("第 {n} 行：未知指令 `{mn}`"))?;
            let arg = parse_arg(op, rest).map_err(|e| format!("第 {n} 行：{e}"))?;
            // 提前校验寻址模式合法
            encode_opcode(op, arg_mode(&arg))
                .ok_or(format!("第 {n} 行：{op:?} 不支持该寻址模式"))?;
            lines.push(Line::Ins(op, arg));
        }
    }

    let prg_banks = prg_banks.ok_or("缺少 .ines prg=N 指令")?;
    let org = org.ok_or("缺少 .org 指令")?;

    // 第一遍：计算标签地址
    let mut labels: BTreeMap<String, u16> = BTreeMap::new();
    let mut pc = org as u32;
    for line in &lines {
        match line {
            Line::Label(name) => {
                if labels.insert(name.clone(), pc as u16).is_some() {
                    return Err(format!("重复的标签 `{name}`"));
                }
            }
            Line::Ins(_, arg) => pc += 1 + arg_len(arg),
            Line::Bytes(b) => pc += b.len() as u32,
            Line::Words(w) => pc += 2 * w.len() as u32,
        }
        if pc > 0x10000 {
            return Err("程序超出 64KB 地址空间".into());
        }
    }

    let resolve = |v: &Val| -> Result<u16, String> {
        match v {
            Val::Num(n) => Ok(*n),
            Val::Label(l) => labels.get(l).copied().ok_or(format!("未定义的标签 `{l}`")),
        }
    };

    // 第二遍：编码
    let mut code: Vec<u8> = Vec::new();
    for line in &lines {
        let pc = org as u32 + code.len() as u32;
        match line {
            Line::Label(_) => {}
            Line::Bytes(b) => code.extend_from_slice(b),
            Line::Words(ws) => {
                for w in ws {
                    code.extend_from_slice(&resolve(w)?.to_le_bytes());
                }
            }
            Line::Ins(op, arg) => {
                let opc = encode_opcode(*op, arg_mode(arg)).unwrap();
                code.push(opc);
                match arg {
                    Arg::Imp | Arg::Acc => {}
                    Arg::Imm(n) | Arg::Zp(n) | Arg::ZpX(n) | Arg::ZpY(n) | Arg::IndX(n)
                    | Arg::IndY(n) => code.push(*n),
                    Arg::Abs(v) | Arg::AbsX(v) | Arg::AbsY(v) | Arg::Ind(v) => {
                        code.extend_from_slice(&resolve(v)?.to_le_bytes());
                    }
                    Arg::Rel(v) => {
                        let target = resolve(v)? as i64;
                        let disp = target - (pc as i64 + 2);
                        if !(-128..=127).contains(&disp) {
                            return Err(format!("分支目标超出范围：{op:?} → {target:04X}（偏移 {disp}）"));
                        }
                        code.push(disp as i8 as u8);
                    }
                }
            }
        }
    }

    let prg_size = prg_banks * 0x4000;
    if code.len() != prg_size {
        return Err(format!("PRG 大小不匹配：声明 {prg_size} 字节，实际生成 {} 字节", code.len()));
    }
    let chr: Vec<u8> = match chr_override {
        Some(c) => c.to_vec(),
        None if has_chr_directive => {
            return Err("源码引用了 .chr 文件但未提供 CHR 数据（CLI 会自动读取）".into());
        }
        None => Vec::new(),
    };
    if chr.len() != chr_banks * 0x2000 {
        return Err(format!(
            "CHR 大小不匹配：声明 {} 字节，实际 {} 字节",
            chr_banks * 0x2000,
            chr.len()
        ));
    }

    let img = super::roundtrip::RomImage { prg: code, chr, mapper, flags6 };
    Ok(img.build_ines())
}
