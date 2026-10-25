//! Feature 019：字节级往返反汇编器。
//!
//! 把 NROM ROM 完整反汇编为可重新汇编的文本源码，不变式：
//! 输出覆盖每个 PRG 字节恰好一次，重汇编后与原 ROM 逐字节一致。
//!
//! 代码/数据分离只影响可读性，不影响正确性：追踪误判（数据被当代码、
//! 跳转表扫描过头）产生的指令仍按原字节编码；指令与标签冲突时在发射
//! 阶段退化为 `.byte`。

use crate::asm::{opcode_table, Mode, Op};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// 解析后的 iNES 镜像（本 feature 只支持 mapper 0 / NROM）。
pub struct RomImage {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub mapper: u8,
    pub flags6: u8,
}

/// 解析 iNES 文件。
pub fn parse_ines(bytes: &[u8]) -> Result<RomImage, String> {
    if bytes.len() < 16 || &bytes[0..4] != b"NES\x1a" {
        return Err("不是有效的 iNES 文件（缺少 NES\\x1a 头）".into());
    }
    let prg_banks = bytes[4] as usize;
    let chr_banks = bytes[5] as usize;
    let flags6 = bytes[6];
    let mapper = (bytes[6] >> 4) | (bytes[7] & 0xF0);
    if mapper != 0 {
        return Err(format!("往返反汇编只支持 mapper 0（NROM），该 ROM 是 mapper {mapper}"));
    }
    if prg_banks == 0 || prg_banks > 2 {
        return Err(format!("NROM 需要 1 或 2 个 PRG bank，实际 {prg_banks}"));
    }
    let trainer = if flags6 & 0x04 != 0 { 512 } else { 0 };
    let prg_start = 16 + trainer;
    let prg_end = prg_start + prg_banks * 0x4000;
    let chr_end = prg_end + chr_banks * 0x2000;
    if bytes.len() < chr_end {
        return Err(format!("iNES 文件被截断：期望至少 {chr_end} 字节，实际 {}", bytes.len()));
    }
    Ok(RomImage {
        prg: bytes[prg_start..prg_end].to_vec(),
        chr: bytes[prg_end..chr_end].to_vec(),
        mapper,
        flags6,
    })
}

impl RomImage {
    /// 重建 iNES 文件字节（16 字节头 + PRG + CHR，无 trainer）。
    pub fn build_ines(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.prg.len() + self.chr.len());
        out.extend_from_slice(b"NES\x1a");
        out.push((self.prg.len() / 0x4000) as u8);
        out.push((self.chr.len() / 0x2000) as u8);
        out.push((self.flags6 & 0x0F) | ((self.mapper & 0x0F) << 4));
        out.push(self.mapper & 0xF0);
        out.extend_from_slice(&[0; 8]);
        out.extend_from_slice(&self.prg);
        out.extend_from_slice(&self.chr);
        out
    }
}

fn operand_len(mode: Mode) -> usize {
    match mode {
        Mode::Imp | Mode::Acc => 0,
        Mode::Abs | Mode::AbsX | Mode::AbsY | Mode::Ind => 2,
        _ => 1,
    }
}

/// 追踪结果：已识别指令、标签、`.word` 数据位置。
struct Trace {
    /// PRG 偏移 → (指令, 寻址模式, 操作数)。Rel 的操作数存目标地址。
    instrs: BTreeMap<usize, (Op, Mode, u16)>,
    /// CPU 地址 → 标签名。
    labels: BTreeMap<u16, String>,
    /// 以 `.word` 发射的 PRG 偏移（跳转表项、中断向量）。
    words: BTreeSet<usize>,
}

impl Trace {
    fn add_label(&mut self, addr: u16, base: u16, prg_len: usize) {
        if addr >= base && (addr as usize) < base as usize + prg_len {
            self.labels.entry(addr).or_insert_with(|| format!("L{addr:04X}"));
        }
    }
}

/// SMB 风格 JumpEngine 特征：asl / tay / pla / sta zp / pla / sta zp。
/// 命中表示 `JSR` 后紧跟地址表且不会返回到表上。
fn is_jump_engine(prg: &[u8], off: usize) -> bool {
    prg.len() >= off + 7
        && prg[off] == 0x0A
        && prg[off + 1] == 0xA8
        && prg[off + 2] == 0x68
        && prg[off + 3] == 0x85
        && prg[off + 5] == 0x68
        && prg[off + 6] == 0x85
}

/// 追踪式反汇编：返回可重新汇编的 .asm 文本（含 .ines 头部指令）。
pub fn disassemble(rom: &RomImage) -> Result<String, String> {
    let prg = &rom.prg;
    if prg.len() < 6 || prg.len() % 0x4000 != 0 {
        return Err(format!("PRG 大小异常：{} 字节", prg.len()));
    }
    let base = (0x10000 - prg.len()) as u16;
    let table = opcode_table();
    let le16 = |o: usize| u16::from_le_bytes([prg[o], prg[o + 1]]);

    let nmi = le16(prg.len() - 6);
    let reset = le16(prg.len() - 4);
    let irq = le16(prg.len() - 2);

    let mut t = Trace { instrs: BTreeMap::new(), labels: BTreeMap::new(), words: BTreeSet::new() };
    for (addr, name) in [(reset, "Reset"), (nmi, "NMI"), (irq, "IRQ")] {
        if addr >= base {
            t.labels.entry(addr).or_insert_with(|| name.to_string());
        }
    }
    // 中断向量区以 .word 发射
    for o in [prg.len() - 6, prg.len() - 4, prg.len() - 2] {
        t.words.insert(o);
    }

    let mut worklist: Vec<u16> = vec![reset, nmi, irq];
    while let Some(entry) = worklist.pop() {
        if entry < base {
            continue;
        }
        let mut pc = (entry - base) as usize;
        loop {
            if pc >= prg.len() || t.instrs.contains_key(&pc) {
                break;
            }
            let Some((op, mode)) = table[prg[pc] as usize] else { break };
            let len = 1 + operand_len(mode);
            if pc + len > prg.len() {
                break;
            }
            let addr = base.wrapping_add(pc as u16);
            let operand = match operand_len(mode) {
                0 => 0,
                1 => prg[pc + 1] as u16,
                _ => le16(pc + 1),
            };
            match mode {
                Mode::Rel => {
                    let target = addr.wrapping_add(len as u16).wrapping_add(prg[pc + 1] as i8 as u16);
                    t.instrs.insert(pc, (op, mode, target));
                    t.add_label(target, base, prg.len());
                    worklist.push(target);
                    pc += len;
                }
                Mode::Abs if op == Op::JMP => {
                    t.instrs.insert(pc, (op, mode, operand));
                    t.add_label(operand, base, prg.len());
                    worklist.push(operand);
                    break;
                }
                Mode::Ind if op == Op::JMP => {
                    t.instrs.insert(pc, (op, mode, operand));
                    break;
                }
                Mode::Abs if op == Op::JSR => {
                    t.instrs.insert(pc, (op, mode, operand));
                    t.add_label(operand, base, prg.len());
                    worklist.push(operand);
                    if operand >= base && is_jump_engine(prg, (operand - base) as usize) {
                        // JSR JumpEngine 后紧跟地址表：逐字收录为代码入口，
                        // 遇到越界值 / 已追踪代码 / 已有标签（即表后第一个例程）为止。
                        let mut p = pc + len;
                        while p + 2 <= prg.len()
                            && !t.instrs.contains_key(&p)
                            && !t.labels.contains_key(&base.wrapping_add(p as u16))
                        {
                            let w = le16(p);
                            if w < base || w >= 0xFFFA {
                                break;
                            }
                            t.words.insert(p);
                            t.add_label(w, base, prg.len());
                            worklist.push(w);
                            p += 2;
                        }
                        break;
                    }
                    pc += len;
                }
                _ => {
                    t.instrs.insert(pc, (op, mode, operand));
                    match op {
                        Op::RTS | Op::RTI | Op::BRK => break,
                        _ => pc += len,
                    }
                }
            }
        }
    }

    // ---- 发射：按地址线性扫描，每个字节恰好输出一次 ----
    let mut out = String::new();
    let _ = writeln!(out, "; 字节级往返反汇编（Feature 019），由 nesc disasm-full 生成");
    let _ = writeln!(out, "; 重汇编：nesc asm <本文件> → 与原 ROM 逐字节一致");
    let _ = writeln!(
        out,
        ".ines prg={} chr={} mapper={} flags6=${:02X}",
        prg.len() / 0x4000,
        rom.chr.len() / 0x2000,
        rom.mapper,
        rom.flags6
    );
    if !rom.chr.is_empty() {
        let _ = writeln!(out, ".chr \"chr.bin\"");
    }
    let _ = writeln!(out, ".org ${base:04X}");

    let mut pos = 0usize;
    while pos < prg.len() {
        let addr = base.wrapping_add(pos as u16);
        if let Some(name) = t.labels.get(&addr) {
            let _ = writeln!(out, "{name}:");
        }
        // 跳转表 / 向量：.word（表项中间有标签时退化为 .byte）
        if t.words.contains(&pos)
            && pos + 2 <= prg.len()
            && !t.labels.contains_key(&addr.wrapping_add(1))
        {
            let w = le16(pos);
            let text = match t.labels.get(&w) {
                Some(l) => l.clone(),
                None => format!("${w:04X}"),
            };
            let _ = writeln!(out, "    .word {text}");
            pos += 2;
            continue;
        }
        // 追踪到的指令：体内无标签才按指令发射
        if let Some(&(op, mode, val)) = t.instrs.get(&pos) {
            let len = 1 + operand_len(mode);
            let spans_label =
                (1..len).any(|i| t.labels.contains_key(&addr.wrapping_add(i as u16)));
            if pos + len <= prg.len() && !spans_label {
                let _ = writeln!(out, "    {}", fmt_instr(op, mode, val, &t.labels));
                pos += len;
                continue;
            }
        }
        // 数据：.byte，遇到标签 / 指令起点 / 表项起点 / 16 字节换行
        let mut end = pos + 1;
        while end < prg.len() && end - pos < 16 {
            let a = base.wrapping_add(end as u16);
            if t.labels.contains_key(&a) || t.instrs.contains_key(&end) || t.words.contains(&end) {
                break;
            }
            end += 1;
        }
        let bytes: Vec<String> = prg[pos..end].iter().map(|b| format!("${b:02X}")).collect();
        let _ = writeln!(out, "    .byte {}", bytes.join(", "));
        pos = end;
    }
    Ok(out)
}

/// 指令文本。约定：零页两位 `$XX`，绝对四位 `$XXXX`（见 asm_text 解析规则）。
/// 仅 JMP/JSR/分支目标使用标签，数据引用一律字面量。
fn fmt_instr(op: Op, mode: Mode, val: u16, labels: &BTreeMap<u16, String>) -> String {
    let name = format!("{op:?}");
    let target = |val: u16| match labels.get(&val) {
        Some(l) => l.clone(),
        None => format!("${val:04X}"),
    };
    match mode {
        Mode::Imp => name,
        Mode::Acc => format!("{name} A"),
        Mode::Imm => format!("{name} #${val:02X}"),
        Mode::Zp => format!("{name} ${val:02X}"),
        Mode::ZpX => format!("{name} ${val:02X},X"),
        Mode::ZpY => format!("{name} ${val:02X},Y"),
        Mode::Rel => format!("{name} {}", target(val)),
        Mode::Abs if op == Op::JMP || op == Op::JSR => format!("{name} {}", target(val)),
        Mode::Abs => format!("{name} ${val:04X}"),
        Mode::AbsX => format!("{name} ${val:04X},X"),
        Mode::AbsY => format!("{name} ${val:04X},Y"),
        Mode::Ind => format!("{name} (${val:04X})"),
        Mode::IndX => format!("{name} (${val:02X},X)"),
        Mode::IndY => format!("{name} (${val:02X}),Y"),
    }
}
