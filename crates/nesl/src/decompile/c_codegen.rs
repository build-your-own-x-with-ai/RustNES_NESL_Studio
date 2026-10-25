//! Feature 020：SMB → C 静态重编译。
//!
//! 把 019 追踪到的 6502 程序翻译为自包含 C 源码：JSR 目标 → 函数，
//! JumpEngine 跳转表 → switch，指令 → 等价 C 语句（含与 nes-sim
//! cpu.rs 对齐的周期计数），I/O → 移植自 nes-sim NesBus 的运行时。
//!
//! 周期策略：基本块内累计、控制转移 / 标签 / I/O 访问前结清；
//! 索引寻址跨页 +1 在读助手内判定；分支命中 +1（跨页 +2）翻译期算出。
//! $2002 轮询（vblank 等待、sprite-0 等待）天然重同步，块内误差
//! 不会跨扫描线，帧渲染与 nes-sim 一致。

use crate::asm::{Mode, Op};
use crate::disasm::roundtrip::{operand_len, trace_rom, RomImage};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// 经行为验证的 RAM 符号（2026-10-25 在 nes-sim 中实测，命名沿用
/// 社区反汇编 smbdis 惯例）。
const SYMBOLS: &[(u16, &str)] = &[
    (0x09, "FrameCounter"),
    (0x1D, "Player_State"),
    (0x57, "Player_X_Speed"),
    (0x6D, "Player_PageLoc"),
    (0x86, "Player_X_Position"),
    (0xCE, "Player_Y_Position"),
    (0x06FC, "SavedJoypadBits"),
    (0x0770, "OperMode"),
    (0x073F, "HorizontalScroll"),
    (0x075A, "NumberofLives"),
    (0x075F, "WorldNumber"),
    (0x0760, "AreaNumber"),
    (0x0776, "GamePauseStatus"),
];

/// 指令基础周期（与 nes-sim cpu.rs 的 decode 表一致）。
fn cost(op: Op, mode: Mode) -> u32 {
    use Mode::*;
    use Op::*;
    match op {
        ASL | LSR | ROL | ROR | INC | DEC => match mode {
            Acc => 2,
            Zp => 5,
            ZpX => 6,
            Abs => 6,
            _ => 7, // AbsX
        },
        STA => match mode {
            Zp => 3,
            ZpX => 4,
            Abs => 4,
            AbsX | AbsY => 5,
            _ => 6, // IndX/IndY
        },
        STX | STY => match mode {
            Zp => 3,
            _ => 4,
        },
        JMP => {
            if mode == Ind {
                5
            } else {
                3
            }
        }
        JSR | RTS | RTI => 6,
        BRK => 7,
        PHA | PHP => 3,
        PLA | PLP => 4,
        LDA | LDX | LDY | ADC | SBC | AND | ORA | EOR | CMP | CPX | CPY | BIT => match mode {
            Imm => 2,
            Zp => 3,
            ZpX | ZpY => 4,
            Abs | AbsX | AbsY => 4,
            IndX => 6,
            IndY => 5,
            _ => 2,
        },
        _ => 2, // 隐含 / 标志 / 传送 / 分支基础 / NOP
    }
}

/// 索引寻址跨页 +1 的指令（与 cpu.rs penalty 标记一致）。
fn penalized(op: Op) -> bool {
    matches!(
        op,
        Op::LDA | Op::LDX | Op::LDY | Op::ADC | Op::SBC | Op::AND | Op::ORA | Op::EOR | Op::CMP
    )
}

fn is_branch(op: Op) -> bool {
    matches!(
        op,
        Op::BCC | Op::BCS | Op::BEQ | Op::BMI | Op::BNE | Op::BPL | Op::BVC | Op::BVS
    )
}

fn branch_cond(op: Op) -> &'static str {
    match op {
        Op::BCC => "!flag_c",
        Op::BCS => "flag_c",
        Op::BEQ => "flag_z",
        Op::BNE => "!flag_z",
        Op::BMI => "flag_n",
        Op::BPL => "!flag_n",
        Op::BVS => "flag_v",
        _ => "!flag_v",
    }
}

/// 单个翻译单元（C 函数）的指令集合与内部跳转目标。
struct FnBody {
    seen: BTreeSet<usize>,
    labels: BTreeSet<usize>,
}

/// 生成自包含的 C 源码（含运行时与快照驱动 main）。
pub fn generate_c(rom: &RomImage) -> Result<String, String> {
    let (t, base) = trace_rom(rom)?;
    let prg = &rom.prg;
    let le16 = |o: usize| u16::from_le_bytes([prg[o], prg[o + 1]]);
    let nmi = le16(prg.len() - 6);
    let reset = le16(prg.len() - 4);
    let irq = le16(prg.len() - 2);
    let off = |a: u16| -> Option<usize> {
        if a >= base {
            Some((a - base) as usize)
        } else {
            None
        }
    };
    let traced = |a: u16| off(a).map(|o| t.instrs.contains_key(&o)).unwrap_or(false);

    // ---- 入口集合：向量 + JSR 目标 + 跳转表项 ----
    let mut entries: BTreeSet<u16> = BTreeSet::new();
    for v in [reset, nmi, irq] {
        if traced(v) {
            entries.insert(v);
        }
    }
    if !traced(reset) || !traced(nmi) {
        return Err("Reset/NMI 向量未能追踪为代码".into());
    }
    for (&o, &(op, mode, val)) in &t.instrs {
        if op == Op::JSR && mode == Mode::Abs && !t.jump_tables.contains_key(&o) && traced(val) {
            entries.insert(val);
        }
    }
    for es in t.jump_tables.values() {
        for &e in es {
            if traced(e) {
                entries.insert(e);
            }
        }
    }

    // ---- 符号 ----
    let sym: BTreeMap<u16, &str> = SYMBOLS.iter().copied().collect();
    let ram_idx = |a: u16| -> String {
        let fold = a & 0x7FF;
        match sym.get(&fold) {
            Some(n) => format!("ram[{n}]"),
            None => format!("ram[0x{fold:03X}]"),
        }
    };
    let zp_base = |z: u8| -> String {
        match sym.get(&(z as u16)) {
            Some(n) => (*n).to_string(),
            None => format!("0x{z:02X}"),
        }
    };

    // 读表达式：(表达式, 是否需先结清周期)
    let abs_read = |a: u16| -> (String, bool) {
        if a < 0x2000 {
            (ram_idx(a), false)
        } else if a < 0x4020 {
            (format!("io_rd(0x{a:04X})"), true)
        } else if (0x6000..0x8000).contains(&a) {
            (format!("sram[0x{:04X}]", a - 0x6000), false)
        } else if a >= 0x8000 {
            (format!("prg[0x{:04X}]", a - 0x8000), false)
        } else {
            ("0 /* open bus */".into(), false)
        }
    };
    let read_expr = |op: Op, mode: Mode, val: u16| -> (String, bool) {
        match mode {
            Mode::Imm => (format!("0x{:02X}", val as u8), false),
            Mode::Zp => (ram_idx(val), false),
            Mode::ZpX => (format!("ram[(uint8_t)({} + cpu_x)]", zp_base(val as u8)), false),
            Mode::ZpY => (format!("ram[(uint8_t)({} + cpu_y)]", zp_base(val as u8)), false),
            Mode::Abs => abs_read(val),
            Mode::AbsX | Mode::AbsY => {
                let idx = if mode == Mode::AbsX { "cpu_x" } else { "cpu_y" };
                let pen = penalized(op);
                if (val as u32) + 255 < 0x2000 {
                    if pen {
                        (format!("rdp_ram(0x{val:04X}, {idx})"), false)
                    } else {
                        (format!("ram[(uint16_t)(0x{val:04X} + {idx}) & 0x7FF]"), false)
                    }
                } else if val >= 0x8000 && val <= 0xFF00 {
                    if pen {
                        (format!("rdp_prg(0x{val:04X}, {idx})"), false)
                    } else {
                        (format!("prg[(uint16_t)(0x{val:04X} + {idx}) - 0x8000]"), false)
                    }
                } else if pen {
                    (format!("rdp(0x{val:04X}, {idx})"), true)
                } else {
                    (format!("rd((uint16_t)(0x{val:04X} + {idx}))"), true)
                }
            }
            Mode::IndX => (
                format!("rd(zp_ptr((uint8_t)({} + cpu_x)))", zp_base(val as u8)),
                true,
            ),
            Mode::IndY => (format!("rdp_zpy({})", zp_base(val as u8)), true),
            _ => ("0".into(), false),
        }
    };
    // 写语句：(语句, 是否需先结清周期)
    let write_stmt = |mode: Mode, val: u16, src: &str| -> (String, bool) {
        match mode {
            Mode::Zp => (format!("{} = {src};", ram_idx(val)), false),
            Mode::ZpX => (
                format!("ram[(uint8_t)({} + cpu_x)] = {src};", zp_base(val as u8)),
                false,
            ),
            Mode::ZpY => (
                format!("ram[(uint8_t)({} + cpu_y)] = {src};", zp_base(val as u8)),
                false,
            ),
            Mode::Abs => {
                if val < 0x2000 {
                    (format!("{} = {src};", ram_idx(val)), false)
                } else if val < 0x4020 {
                    (format!("io_wr(0x{val:04X}, {src});"), true)
                } else if (0x6000..0x8000).contains(&val) {
                    (format!("sram[0x{:04X}] = {src};", val - 0x6000), false)
                } else {
                    (format!("; /* 写 ROM ${val:04X}：忽略 */"), false)
                }
            }
            Mode::AbsX | Mode::AbsY => {
                let idx = if mode == Mode::AbsX { "cpu_x" } else { "cpu_y" };
                if (val as u32) + 255 < 0x2000 {
                    (
                        format!("ram[(uint16_t)(0x{val:04X} + {idx}) & 0x7FF] = {src};"),
                        false,
                    )
                } else {
                    (format!("wr((uint16_t)(0x{val:04X} + {idx}), {src});"), true)
                }
            }
            Mode::IndX => (
                format!("wr(zp_ptr((uint8_t)({} + cpu_x)), {src});", zp_base(val as u8)),
                true,
            ),
            Mode::IndY => (
                format!("wr((uint16_t)(zp_ptr({}) + cpu_y), {src});", zp_base(val as u8)),
                true,
            ),
            _ => (";".into(), false),
        }
    };

    // ---- 函数体收集 ----
    let collect = |entry: u16| -> FnBody {
        let e_off = off(entry).unwrap();
        let mut seen = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut stack = vec![e_off];
        while let Some(start) = stack.pop() {
            let mut pc = start;
            loop {
                if seen.contains(&pc) {
                    break;
                }
                let addr = base.wrapping_add(pc as u16);
                if pc != e_off && entries.contains(&addr) {
                    break; // 落入其他入口：发射时翻译为尾调用
                }
                let Some(&(op, mode, val)) = t.instrs.get(&pc) else { break };
                seen.insert(pc);
                let len = 1 + operand_len(mode);
                if is_branch(op) {
                    if val == entry {
                        labels.insert(e_off);
                    } else if !entries.contains(&val) {
                        if let Some(to) = off(val) {
                            if t.instrs.contains_key(&to) {
                                labels.insert(to);
                                stack.push(to);
                            }
                        }
                    }
                    pc += len;
                } else if op == Op::JMP && mode == Mode::Abs {
                    if val == addr {
                        break; // JMP self：主线程停机
                    }
                    if val == entry {
                        labels.insert(e_off);
                        break;
                    }
                    if entries.contains(&val) {
                        break; // 尾调用
                    }
                    if let Some(to) = off(val) {
                        if t.instrs.contains_key(&to) {
                            labels.insert(to);
                            stack.push(to);
                        }
                    }
                    break;
                } else if op == Op::JMP {
                    break; // 间接：dispatch
                } else if op == Op::JSR {
                    if t.jump_tables.contains_key(&pc) {
                        break; // switch 分派后返回
                    }
                    pc += len;
                } else if matches!(op, Op::RTS | Op::RTI | Op::BRK) {
                    break;
                } else {
                    pc += len;
                }
            }
        }
        FnBody { seen, labels }
    };

    // ---- 函数发射 ----
    let mut fwd = String::new();
    let mut fns = String::new();
    for &entry in &entries {
        let mut body = collect(entry);
        let e_off = off(entry).unwrap();
        // 函数体按地址升序发射；入口若不是最低地址，开头必须先跳到入口
        let entry_first = body.seen.iter().next() == Some(&e_off);
        if !entry_first {
            body.labels.insert(e_off);
        }
        let _ = writeln!(fwd, "static void sub_{entry:04X}(void);");
        let role = if entry == reset {
            " /* Reset 入口 */"
        } else if entry == nmi {
            " /* NMI 入口（每帧游戏逻辑）*/"
        } else if entry == irq {
            " /* IRQ 入口 */"
        } else {
            ""
        };
        let _ = writeln!(fns, "\nstatic void sub_{entry:04X}(void) {{{role}");
        if !entry_first {
            let _ = writeln!(fns, "    goto l_{entry:04X}; /* 入口在函数体中部 */");
        }
        let mut pending: u32 = 0;
        let flush = |out: &mut String, pending: &mut u32| {
            if *pending > 0 {
                let _ = writeln!(out, "    cyc += {pending};");
                *pending = 0;
            }
        };
        let mut cursor = 0usize;
        let offs: Vec<usize> = body.seen.iter().copied().collect();
        for &pc in &offs {
            if pc < cursor {
                continue; // 交叠代码：字节已由前一条覆盖
            }
            let addr = base.wrapping_add(pc as u16);
            if body.labels.contains(&pc) {
                flush(&mut fns, &mut pending);
                let _ = writeln!(fns, "l_{addr:04X}:;");
            }
            let (op, mode, val) = t.instrs[&pc];
            let len = 1 + operand_len(mode);
            pending += cost(op, mode);
            let mut falls = true;

            // 跳转目标的发射形式
            let target_kind = |tv: u16| -> u8 {
                // 0 = 内部 goto, 1 = 尾调用, 2 = fatal
                if tv == entry && body.labels.contains(&off(entry).unwrap()) {
                    0
                } else if tv != entry && entries.contains(&tv) {
                    1
                } else if off(tv).map(|o| body.seen.contains(&o)).unwrap_or(false) {
                    0
                } else {
                    2
                }
            };

            use Op::*;
            match op {
                LDA | LDX | LDY => {
                    let (e, fl) = read_expr(op, mode, val);
                    if fl {
                        flush(&mut fns, &mut pending);
                    }
                    let r = match op {
                        LDA => "cpu_a",
                        LDX => "cpu_x",
                        _ => "cpu_y",
                    };
                    let _ = writeln!(fns, "    {r} = {e}; set_zn({r});");
                }
                STA | STX | STY => {
                    let src = match op {
                        STA => "cpu_a",
                        STX => "cpu_x",
                        _ => "cpu_y",
                    };
                    let (s, fl) = write_stmt(mode, val, src);
                    if fl {
                        flush(&mut fns, &mut pending);
                    }
                    let _ = writeln!(fns, "    {s}");
                }
                ADC | SBC | AND | ORA | EOR | CMP | CPX | CPY | BIT => {
                    let (e, fl) = read_expr(op, mode, val);
                    if fl {
                        flush(&mut fns, &mut pending);
                    }
                    let s = match op {
                        ADC => format!("do_adc({e});"),
                        SBC => format!("do_sbc({e});"),
                        AND => format!("cpu_a &= {e}; set_zn(cpu_a);"),
                        ORA => format!("cpu_a |= {e}; set_zn(cpu_a);"),
                        EOR => format!("cpu_a ^= {e}; set_zn(cpu_a);"),
                        CMP => format!("do_cmp(cpu_a, {e});"),
                        CPX => format!("do_cmp(cpu_x, {e});"),
                        CPY => format!("do_cmp(cpu_y, {e});"),
                        _ => format!("do_bit({e});"),
                    };
                    let _ = writeln!(fns, "    {s}");
                }
                ASL | LSR | ROL | ROR | INC | DEC => {
                    let f = match op {
                        ASL => "do_asl",
                        LSR => "do_lsr",
                        ROL => "do_rol",
                        ROR => "do_ror",
                        INC => "do_inc",
                        _ => "do_dec",
                    };
                    match mode {
                        Mode::Acc => {
                            let _ = writeln!(fns, "    cpu_a = {f}(cpu_a);");
                        }
                        Mode::Zp => {
                            let r = ram_idx(val);
                            let _ = writeln!(fns, "    {r} = {f}({r});");
                        }
                        Mode::ZpX => {
                            let _ = writeln!(
                                fns,
                                "    {{ uint8_t z__ = (uint8_t)({} + cpu_x); ram[z__] = {f}(ram[z__]); }}",
                                zp_base(val as u8)
                            );
                        }
                        Mode::Abs if val < 0x2000 => {
                            let r = ram_idx(val);
                            let _ = writeln!(fns, "    {r} = {f}({r});");
                        }
                        Mode::Abs if val >= 0x8000 => {
                            // 对 ROM 的 RMW：读出算标志，写入忽略
                            let _ = writeln!(fns, "    (void){f}(prg[0x{:04X}]);", val - 0x8000);
                        }
                        Mode::AbsX if (val as u32) + 255 < 0x2000 => {
                            let _ = writeln!(
                                fns,
                                "    {{ uint16_t ea__ = (uint16_t)(0x{val:04X} + cpu_x) & 0x7FF; ram[ea__] = {f}(ram[ea__]); }}"
                            );
                        }
                        _ => {
                            flush(&mut fns, &mut pending);
                            let idx = if mode == Mode::AbsX { " + cpu_x" } else { "" };
                            let _ = writeln!(
                                fns,
                                "    {{ uint16_t ea__ = (uint16_t)(0x{val:04X}{idx}); wr(ea__, {f}(rd(ea__))); }}"
                            );
                        }
                    }
                }
                INX => {
                    let _ = writeln!(fns, "    cpu_x++; set_zn(cpu_x);");
                }
                INY => {
                    let _ = writeln!(fns, "    cpu_y++; set_zn(cpu_y);");
                }
                DEX => {
                    let _ = writeln!(fns, "    cpu_x--; set_zn(cpu_x);");
                }
                DEY => {
                    let _ = writeln!(fns, "    cpu_y--; set_zn(cpu_y);");
                }
                TAX => {
                    let _ = writeln!(fns, "    cpu_x = cpu_a; set_zn(cpu_x);");
                }
                TAY => {
                    let _ = writeln!(fns, "    cpu_y = cpu_a; set_zn(cpu_y);");
                }
                TXA => {
                    let _ = writeln!(fns, "    cpu_a = cpu_x; set_zn(cpu_a);");
                }
                TYA => {
                    let _ = writeln!(fns, "    cpu_a = cpu_y; set_zn(cpu_a);");
                }
                TSX => {
                    let _ = writeln!(fns, "    cpu_x = cpu_sp; set_zn(cpu_x);");
                }
                TXS => {
                    let _ = writeln!(fns, "    cpu_sp = cpu_x;");
                }
                PHA => {
                    let _ = writeln!(fns, "    push(cpu_a);");
                }
                PHP => {
                    let _ = writeln!(fns, "    push((uint8_t)(pack_p() | 0x10));");
                }
                PLA => {
                    let _ = writeln!(fns, "    cpu_a = pull(); set_zn(cpu_a);");
                }
                PLP => {
                    let _ = writeln!(fns, "    unpack_p(pull());");
                }
                CLC => {
                    let _ = writeln!(fns, "    flag_c = 0;");
                }
                SEC => {
                    let _ = writeln!(fns, "    flag_c = 1;");
                }
                CLI => {
                    let _ = writeln!(fns, "    flag_i = 0;");
                }
                SEI => {
                    let _ = writeln!(fns, "    flag_i = 1;");
                }
                CLV => {
                    let _ = writeln!(fns, "    flag_v = 0;");
                }
                CLD => {
                    let _ = writeln!(fns, "    flag_d = 0;");
                }
                SED => {
                    let _ = writeln!(fns, "    flag_d = 1;");
                }
                NOP => {}
                BCC | BCS | BEQ | BNE | BMI | BPL | BVC | BVS => {
                    flush(&mut fns, &mut pending);
                    let cond = branch_cond(op);
                    let next = addr.wrapping_add(len as u16);
                    let k = 1 + u32::from(val & 0xFF00 != next & 0xFF00);
                    match target_kind(val) {
                        0 => {
                            let _ =
                                writeln!(fns, "    if ({cond}) {{ cyc += {k}; goto l_{val:04X}; }}");
                        }
                        1 => {
                            let _ = writeln!(
                                fns,
                                "    if ({cond}) {{ cyc += {k}; sub_{val:04X}(); return; }}"
                            );
                        }
                        _ => {
                            let _ = writeln!(
                                fns,
                                "    if ({cond}) fatal(\"${addr:04X}: 分支到未追踪代码 ${val:04X}\");"
                            );
                        }
                    }
                }
                JMP if mode == Mode::Abs => {
                    flush(&mut fns, &mut pending);
                    falls = false;
                    if val == addr {
                        let _ = writeln!(fns, "    return; /* 主线程停机循环（等待 NMI）*/");
                    } else {
                        match target_kind(val) {
                            0 => {
                                let _ = writeln!(fns, "    goto l_{val:04X};");
                            }
                            1 => {
                                let _ = writeln!(fns, "    sub_{val:04X}(); return;");
                            }
                            _ => {
                                let _ = writeln!(
                                    fns,
                                    "    fatal(\"${addr:04X}: JMP 到未追踪代码 ${val:04X}\");"
                                );
                            }
                        }
                    }
                }
                JMP => {
                    // 间接 JMP（含 6502 页内回绕取高字节）
                    flush(&mut fns, &mut pending);
                    falls = false;
                    let _ = writeln!(
                        fns,
                        "    dispatch((uint16_t)(rd(0x{val:04X}) | rd((0x{val:04X} & 0xFF00) | ((0x{val:04X} + 1) & 0xFF)) << 8)); return;"
                    );
                }
                JSR => {
                    if let Some(table) = t.jump_tables.get(&pc) {
                        pending += 43; // JumpEngine 本体（asl…jmp(ind)）
                        flush(&mut fns, &mut pending);
                        falls = false;
                        // JumpEngine 副作用：$04/$05 = 表指针（JSR 返回地址），
                        // $06/$07 = 目标例程地址，Y = 2A+2，A = 目标高字节（带 NZ）。
                        // SMB 的部分例程依赖这些残留（如标题画面解码），必须复现。
                        let ret = addr.wrapping_add(2);
                        let _ = writeln!(
                            fns,
                            "    ram[0x04] = 0x{:02X}; ram[0x05] = 0x{:02X}; /* JumpEngine 残留：表指针 */",
                            ret as u8,
                            ret >> 8
                        );
                        let _ = writeln!(
                            fns,
                            "    cpu_y = (uint8_t)(cpu_a * 2 + 2);\n    switch (cpu_a) {{ /* JSR JumpEngine，表 ${:04X} */",
                            addr.wrapping_add(3)
                        );
                        for (i, &e) in table.iter().enumerate() {
                            if traced(e) {
                                let _ = writeln!(
                                    fns,
                                    "    case {i}: ram[0x06] = 0x{:02X}; ram[0x07] = 0x{:02X}; cpu_a = 0x{:02X}; set_zn(cpu_a); sub_{e:04X}(); return;",
                                    e as u8,
                                    e >> 8,
                                    e >> 8
                                );
                            } else {
                                let _ = writeln!(
                                    fns,
                                    "    case {i}: fatal(\"跳转表 ${addr:04X} 项 {i} 未追踪\");"
                                );
                            }
                        }
                        let _ = writeln!(
                            fns,
                            "    default: fatal(\"跳转表 ${addr:04X}：A=%u 超界\", cpu_a);\n    }}"
                        );
                    } else {
                        flush(&mut fns, &mut pending);
                        if entries.contains(&val) {
                            let _ = writeln!(fns, "    sub_{val:04X}();");
                        } else {
                            let _ = writeln!(
                                fns,
                                "    fatal(\"${addr:04X}: JSR 到未追踪代码 ${val:04X}\");"
                            );
                        }
                    }
                }
                RTS | RTI => {
                    flush(&mut fns, &mut pending);
                    falls = false;
                    let _ = writeln!(fns, "    return;");
                }
                BRK => {
                    flush(&mut fns, &mut pending);
                    falls = false;
                    let _ = writeln!(fns, "    fatal(\"${addr:04X}: BRK\");");
                }
            }

            cursor = pc + len;
            if falls && !body.seen.contains(&cursor) {
                flush(&mut fns, &mut pending);
                let next_addr = base.wrapping_add(cursor as u16);
                if entries.contains(&next_addr) {
                    let _ = writeln!(fns, "    sub_{next_addr:04X}(); return; /* 顺序落入 */");
                } else {
                    let _ = writeln!(fns, "    fatal(\"${next_addr:04X}: 落入未追踪代码\");");
                }
            }
        }
        flush(&mut fns, &mut pending);
        let _ = writeln!(fns, "}}");
    }

    // ---- dispatch（间接 JMP 用）----
    let mut dispatch = String::from("\nstatic void dispatch(uint16_t a) {\n    switch (a) {\n");
    for &e in &entries {
        let _ = writeln!(dispatch, "    case 0x{e:04X}: sub_{e:04X}(); return;");
    }
    dispatch.push_str("    default: fatal(\"dispatch: $%04X 不是已知入口\", a);\n    }\n}\n");

    // ---- 组装 ----
    let mut out = String::new();
    out.push_str("/* Super Mario Bros 静态重编译 — 由 nesc decompile-c 自动生成（Feature 020）\n");
    out.push_str(" * 验证：快照经 nes-sim 渲染器重建帧，与原版 ROM 逐像素对比一致。\n");
    out.push_str(" * 用法：./smb <script.txt>，脚本命令 btn/dump/end（见 main）。 */\n");
    out.push_str("#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n");
    out.push_str("/* 经行为验证的 RAM 符号（命名沿用社区反汇编惯例） */\n");
    for (a, n) in SYMBOLS {
        let _ = writeln!(out, "#define {n} 0x{a:02X}");
    }
    let _ = writeln!(out, "\n#define MIRROR_H {}", u8::from(rom.flags6 & 0x01 == 0 && rom.flags6 & 0x08 == 0));
    out.push_str(RUNTIME_STATE);

    let _ = writeln!(out, "\nstatic const uint8_t prg[0x{:04X}] = {{", prg.len());
    push_bytes(&mut out, prg);
    out.push_str("};\n");
    let _ = writeln!(out, "static const uint8_t chr_rom[0x{:04X}] = {{", rom.chr.len());
    push_bytes(&mut out, &rom.chr);
    out.push_str("};\n");

    out.push_str(RUNTIME_HELPERS);
    out.push_str("\n/* ---- 前向声明 ---- */\n");
    out.push_str(&fwd);
    out.push_str("static void dispatch(uint16_t a);\n");
    out.push_str("\n/* ---- 翻译后的游戏代码 ---- */\n");
    out.push_str(&fns);
    out.push_str(&dispatch);

    out.push_str(RUNTIME_MAIN_A);
    let _ = writeln!(out, "    sub_{reset:04X}(); /* 执行到主线程停机 */");
    out.push_str(RUNTIME_MAIN_B);
    let _ = writeln!(
        out,
        "        if (ppuctrl & 0x80) {{ cpu_sp -= 3; cyc += 7; sub_{nmi:04X}(); cpu_sp += 3; }}"
    );
    out.push_str(RUNTIME_MAIN_C);
    Ok(out)
}

fn push_bytes(out: &mut String, bytes: &[u8]) {
    for chunk in bytes.chunks(16) {
        out.push_str("    ");
        for b in chunk {
            let _ = write!(out, "0x{b:02X},");
        }
        out.push('\n');
    }
}

/* ------------------------------------------------------------------ */
/* C 运行时模板（移植自 nes-sim 的 mapper 0 子集）                      */
/* ------------------------------------------------------------------ */

const RUNTIME_STATE: &str = r#"
#define CPF 29781ULL      /* 每帧 CPU 周期（NTSC），与 nes-sim 一致 */
#define VBLANK 2273ULL
#define DMA_CYCLES 513ULL

#define fatal(...) do { fprintf(stderr, "fatal: " __VA_ARGS__); fprintf(stderr, "\n"); exit(2); } while (0)

/* ---- 机器状态 ---- */
static uint8_t cpu_a, cpu_x, cpu_y, cpu_sp = 0xFD;
static uint8_t flag_c, flag_z, flag_n, flag_v, flag_i = 1, flag_d;
static uint64_t cyc = 7;   /* 复位 7 周期，对齐 nes-sim */
static uint64_t frames = 0;
static uint8_t ram[0x800], sram[0x2000], vram[0x4000], oam[256];
static uint8_t ppuctrl, ppumask, scroll_x, scroll_y, oam_addr;
static uint8_t nt_sel; /* t 寄存器的命名表位：$2000 或 $2006 最后写入者决定 */
static uint8_t write_toggle, read_buffer;
static uint16_t ppu_addr;
static uint8_t buttons[2], pad_shift[2], pad_reads[2], pad_strobe;

typedef struct { uint64_t cyc; uint8_t kind, val; } RegEvent;
static RegEvent reg_log[1 << 20];
static uint32_t reg_log_n;
"#;

const RUNTIME_HELPERS: &str = r#"
/* ---- CPU 助手（语义对齐 nes-sim cpu.rs）---- */
static void set_zn(uint8_t v) { flag_z = v == 0; flag_n = v >> 7; }
static uint8_t pack_p(void) {
    return (uint8_t)(flag_c | flag_z << 1 | flag_i << 2 | flag_d << 3 | 0x20 | flag_v << 6 | flag_n << 7);
}
static void unpack_p(uint8_t p) {
    flag_c = p & 1; flag_z = (p >> 1) & 1; flag_i = (p >> 2) & 1;
    flag_d = (p >> 3) & 1; flag_v = (p >> 6) & 1; flag_n = p >> 7;
}
static void push(uint8_t v) { ram[0x100 | cpu_sp] = v; cpu_sp--; }
static uint8_t pull(void) { cpu_sp++; return ram[0x100 | cpu_sp]; }
static void do_adc(uint8_t m) {
    uint16_t sum = (uint16_t)cpu_a + m + flag_c;
    uint8_t r = (uint8_t)sum;
    flag_c = sum > 0xFF;
    flag_v = ((uint8_t)~(cpu_a ^ m) & (uint8_t)(cpu_a ^ r) & 0x80) != 0;
    cpu_a = r; set_zn(r);
}
static void do_sbc(uint8_t m) { do_adc((uint8_t)~m); }
static void do_cmp(uint8_t reg, uint8_t m) { flag_c = reg >= m; set_zn((uint8_t)(reg - m)); }
static void do_bit(uint8_t m) { flag_z = (cpu_a & m) == 0; flag_v = (m >> 6) & 1; flag_n = m >> 7; }
static uint8_t do_asl(uint8_t v) { flag_c = v >> 7; v = (uint8_t)(v << 1); set_zn(v); return v; }
static uint8_t do_lsr(uint8_t v) { flag_c = v & 1; v >>= 1; set_zn(v); return v; }
static uint8_t do_rol(uint8_t v) { uint8_t c = flag_c; flag_c = v >> 7; v = (uint8_t)((v << 1) | c); set_zn(v); return v; }
static uint8_t do_ror(uint8_t v) { uint8_t c = flag_c; flag_c = v & 1; v = (uint8_t)((v >> 1) | (c << 7)); set_zn(v); return v; }
static uint8_t do_inc(uint8_t v) { v++; set_zn(v); return v; }
static uint8_t do_dec(uint8_t v) { v--; set_zn(v); return v; }
static uint16_t zp_ptr(uint8_t z) { return (uint16_t)(ram[z] | ram[(uint8_t)(z + 1)] << 8); }

/* ---- PPU 地址折叠（移植 nes-sim nes.rs）---- */
static unsigned vram_index(uint16_t addr) {
    unsigned a = addr & 0x3FFF;
    if (a >= 0x3F00) { a = 0x3F00 | (a & 0x1F); if ((a & 0x13) == 0x10) a &= ~0x10u; }
    return a;
}
static unsigned nt_index(uint16_t addr) {
    unsigned i = vram_index(addr);
    if (i < 0x2000 || i >= 0x3F00) return i;
    { unsigned a = 0x2000 | (i & 0x0FFF); return MIRROR_H ? (a & ~0x400u) : (a & ~0x800u); }
}
static unsigned nt_base(unsigned wx) { return MIRROR_H ? 0x2000 : 0x2000 + ((wx >> 8) & 1) * 0x400; }

/* ---- sprite-0 命中（移植 nes-sim；惰性锁存，周期判定一致）---- */
static uint64_t pixel_cycle(uint8_t px, uint8_t py) { return VBLANK + (341ULL * (1 + py) + px) / 3; }
static int bg_opaque(uint8_t sx_, uint8_t sy_) {
    unsigned wx = (nt_sel & 1u) * 256 + scroll_x + sx_;
    unsigned wy = (unsigned)scroll_y + sy_;
    unsigned nt, tile, basea, a;
    uint8_t bit;
    if (wy >= 240) wy -= 240;
    nt = nt_base(wx);
    tile = vram[nt + (wy / 8) * 32 + (wx & 0xFF) / 8];
    basea = (ppuctrl & 0x10) ? 0x1000 : 0;
    a = basea + tile * 16 + wy % 8;
    bit = (uint8_t)(0x80 >> (wx % 8));
    return ((vram[a] | vram[a + 8]) & bit) != 0;
}
static int sprite0_hit(void) { /* 返回 (x<<8)|y，无命中 -1 */
    uint8_t y0, tile, attr, x0;
    unsigned basea, r;
    if ((ppumask & 0x18) != 0x18) return -1;
    y0 = oam[0]; tile = oam[1]; attr = oam[2]; x0 = oam[3];
    basea = (ppuctrl & 0x08) ? 0x1000 : 0;
    for (r = 0; r < 8; r++) {
        unsigned sy_ = (unsigned)y0 + 1 + r, row, a, ccol;
        uint8_t bits;
        if (sy_ >= 240) break;
        row = (attr & 0x80) ? 7 - r : r;
        a = basea + tile * 16u + row;
        bits = vram[a] | vram[a + 8];
        for (ccol = 0; ccol < 8; ccol++) {
            unsigned sx_ = (unsigned)x0 + ccol;
            int clipped;
            uint8_t bit;
            if (sx_ >= 255) break;
            clipped = sx_ < 8 && (ppumask & 0x06) != 0x06;
            bit = (attr & 0x40) ? (uint8_t)(1u << ccol) : (uint8_t)(0x80u >> ccol);
            if (!clipped && (bits & bit) && bg_opaque((uint8_t)sx_, (uint8_t)sy_))
                return (int)((sx_ << 8) | sy_);
        }
    }
    return -1;
}
static uint64_t s0_latched_frame = ~0ULL, s0_at, s0_clear;
static int s0_valid;
static void maybe_latch_sprite0(void) {
    uint64_t f = cyc / CPF;
    uint64_t latch_at = f * CPF + pixel_cycle(0, 0);
    if (cyc >= latch_at && s0_latched_frame != f) {
        int hit = sprite0_hit();
        s0_valid = hit >= 0;
        if (s0_valid) s0_at = f * CPF + pixel_cycle((uint8_t)(hit >> 8), (uint8_t)(hit & 0xFF));
        s0_clear = f * CPF + CPF + VBLANK;
        s0_latched_frame = f;
    }
}
/* 渲染在可见区内重新开启后重判（对齐 nes-sim recheck_sprite0）：
   命中像素仍在未来才算数 */
static void recheck_sprite0(void) {
    uint64_t f = cyc / CPF, frame_start = f * CPF;
    int hit;
    if (cyc < frame_start + pixel_cycle(0, 0)) return;
    if (s0_valid && s0_at >= frame_start) return;
    hit = sprite0_hit();
    if (hit >= 0) {
        uint64_t at = frame_start + pixel_cycle((uint8_t)(hit >> 8), (uint8_t)(hit & 0xFF));
        if (at >= cyc) { s0_valid = 1; s0_at = at; s0_clear = frame_start + CPF + VBLANK; }
    }
}

/* ---- I/O（移植 nes-sim NesBus，mapper 0）---- */
static void log_reg(uint8_t kind, uint8_t val) {
    if (reg_log_n < (1u << 20)) {
        reg_log[reg_log_n].cyc = cyc; reg_log[reg_log_n].kind = kind; reg_log[reg_log_n].val = val;
        reg_log_n++;
    }
}
static void pads_latch(void) {
    pad_shift[0] = buttons[0]; pad_shift[1] = buttons[1];
    pad_reads[0] = pad_reads[1] = 0;
}
static uint8_t pad_read(int port) {
    uint8_t bit;
    if (pad_strobe) { pads_latch(); return (uint8_t)(0x40 | (buttons[port] >> 7)); }
    if (pad_reads[port] >= 8) bit = 1;
    else { bit = pad_shift[port] >> 7; pad_shift[port] = (uint8_t)(pad_shift[port] << 1); pad_reads[port]++; }
    return (uint8_t)(0x40 | bit);
}
static uint8_t rd(uint16_t addr);
static uint8_t effective_ctrl(void) { return (uint8_t)((ppuctrl & ~3u) | (nt_sel & 3)); }
/* $2006 写入同时装载 t（对齐 nes-sim vram_addr_write_loads_t）：
   第一字节 = 命名表位 / fine y / coarse y 高位，第二字节 = coarse x / coarse y 低位 */
static void vram_addr_write_loads_t(uint8_t v, int second) {
    if (second) {
        uint8_t coarse_y = (uint8_t)(((scroll_y >> 3) & 0x18) | (v >> 5));
        scroll_x = (uint8_t)(((v & 0x1F) << 3) | (scroll_x & 7));
        scroll_y = (uint8_t)((coarse_y << 3) | (scroll_y & 7));
    } else {
        uint8_t coarse_y = (uint8_t)(((scroll_y >> 3) & 7) | ((v & 3) << 3));
        nt_sel = (v >> 2) & 3;
        scroll_y = (uint8_t)((coarse_y << 3) | ((v >> 4) & 3));
    }
    log_reg(0, effective_ctrl()); log_reg(1, scroll_x); log_reg(2, scroll_y);
}
static void ppu_inc(void) { ppu_addr = (uint16_t)((ppu_addr + ((ppuctrl & 0x04) ? 32 : 1)) & 0x3FFF); }
static uint8_t io_rd(uint16_t addr) {
    maybe_latch_sprite0(); /* 任何 I/O 前先用当前状态完成本帧锁存 */
    if (addr < 0x4000) {
        switch (addr & 7) {
        case 2: {
            int hit;
            write_toggle = 0;
            hit = s0_valid && cyc >= s0_at && cyc < s0_clear;
            return (uint8_t)(0x80 | (hit ? 0x40 : 0));
        }
        case 4: return oam[oam_addr];
        case 7: {
            uint16_t a = ppu_addr;
            uint8_t val = vram[a < 0x2000 ? a : nt_index(a)], out_;
            if (a >= 0x3F00) { read_buffer = vram[nt_index((uint16_t)(a - 0x1000))]; out_ = val; }
            else { out_ = read_buffer; read_buffer = val; }
            ppu_inc();
            return out_;
        }
        default: return 0;
        }
    }
    if (addr == 0x4016) return pad_read(0);
    if (addr == 0x4017) return pad_read(1);
    return 0;
}
static void oam_dma(uint8_t page) {
    uint16_t basea = (uint16_t)page << 8;
    unsigned i;
    for (i = 0; i < 256; i++) oam[(uint8_t)(oam_addr + i)] = rd((uint16_t)(basea | i));
    cyc += DMA_CYCLES;
}
static void io_wr(uint16_t addr, uint8_t v) {
    maybe_latch_sprite0(); /* 写入前先用旧状态完成本帧锁存 */
    if (addr < 0x4000) {
        switch (addr & 7) {
        case 0: ppuctrl = v; nt_sel = v & 3; log_reg(0, v); break;
        case 1: ppumask = v; recheck_sprite0(); break;
        case 3: oam_addr = v; break;
        case 4: oam[oam_addr++] = v; break;
        case 5:
            log_reg(write_toggle ? 2 : 1, v);
            if (write_toggle) scroll_y = v; else scroll_x = v;
            write_toggle ^= 1;
            break;
        case 6:
            if (write_toggle) ppu_addr = (uint16_t)((ppu_addr & 0xFF00) | v);
            else ppu_addr = (uint16_t)(((v & 0x3F) << 8) | (ppu_addr & 0xFF));
            vram_addr_write_loads_t(v, write_toggle);
            write_toggle ^= 1;
            break;
        case 7: vram[nt_index(ppu_addr)] = v; ppu_inc(); break;
        }
        return;
    }
    if (addr == 0x4014) { oam_dma(v); return; }
    if (addr == 0x4016) { uint8_t nw = v & 1; if (pad_strobe || nw) pads_latch(); pad_strobe = nw; return; }
    /* APU $4000-$4013/$4015/$4017：忽略（声音不验证） */
}
static uint8_t rd(uint16_t addr) {
    if (addr < 0x2000) return ram[addr & 0x7FF];
    if (addr < 0x4020) return io_rd(addr);
    if (addr >= 0x8000) return prg[(size_t)(addr - 0x8000) % sizeof prg];
    if (addr >= 0x6000) return sram[addr - 0x6000];
    return 0;
}
static void wr(uint16_t addr, uint8_t v) {
    if (addr < 0x2000) { ram[addr & 0x7FF] = v; return; }
    if (addr < 0x4020) { io_wr(addr, v); return; }
    if (addr >= 0x6000 && addr < 0x8000) { sram[addr - 0x6000] = v; return; }
    /* 写 ROM：忽略 */
}
/* 索引读 + 跨页罚时（对齐 cpu.rs penalty 语义） */
static uint8_t rdp(uint16_t base, uint8_t idx) {
    uint16_t ea = (uint16_t)(base + idx);
    if ((ea ^ base) & 0xFF00) cyc++;
    return rd(ea);
}
static uint8_t rdp_ram(uint16_t base, uint8_t idx) {
    uint16_t ea = (uint16_t)(base + idx);
    if ((ea ^ base) & 0xFF00) cyc++;
    return ram[ea & 0x7FF];
}
static uint8_t rdp_prg(uint16_t base, uint8_t idx) {
    uint16_t ea = (uint16_t)(base + idx);
    if ((ea ^ base) & 0xFF00) cyc++;
    return prg[(size_t)(ea - 0x8000) % sizeof prg];
}
static uint8_t rdp_zpy(uint8_t z) {
    uint16_t base = zp_ptr(z), ea = (uint16_t)(base + cpu_y);
    if ((ea ^ base) & 0xFF00) cyc++;
    return rd(ea);
}
"#;

const RUNTIME_MAIN_A: &str = r#"
/* ---- 快照与驱动 ---- */
static void dump_snapshot(const char *path) {
    FILE *f = fopen(path, "wb");
    uint32_t n;
    uint32_t i;
    if (!f) fatal("无法写快照 %s", path);
    fwrite("SMBC", 1, 4, f);
    fwrite(&frames, 8, 1, f); fwrite(&cyc, 8, 1, f);
    fwrite(ram, 1, sizeof ram, f); fwrite(vram, 1, sizeof vram, f); fwrite(oam, 1, sizeof oam, f);
    fwrite(&ppuctrl, 1, 1, f); fwrite(&ppumask, 1, 1, f);
    n = reg_log_n; fwrite(&n, 4, 1, f);
    for (i = 0; i < n; i++) {
        fwrite(&reg_log[i].cyc, 8, 1, f);
        fwrite(&reg_log[i].kind, 1, 1, f);
        fwrite(&reg_log[i].val, 1, 1, f);
    }
    fclose(f);
}

typedef struct { uint64_t frame; uint8_t mask; } BtnEv;
typedef struct { uint64_t frame; char path[1024]; } DumpEv;
static BtnEv btn_ev[4096]; static unsigned btn_n;
static DumpEv dump_ev[256]; static unsigned dump_n;
static uint64_t end_frame;

int main(int argc, char **argv) {
    FILE *sf;
    char cmd[16];
    unsigned i;
    if (argc < 2) fatal("用法：%s <script.txt>", argv[0]);
    sf = fopen(argv[1], "r");
    if (!sf) fatal("无法读脚本 %s", argv[1]);
    while (fscanf(sf, "%15s", cmd) == 1) {
        unsigned long long fr; unsigned m;
        if (!strcmp(cmd, "btn")) {
            if (fscanf(sf, "%llu %u", &fr, &m) != 2) fatal("btn 参数错误");
            btn_ev[btn_n].frame = fr; btn_ev[btn_n].mask = (uint8_t)m; btn_n++;
        } else if (!strcmp(cmd, "dump")) {
            if (fscanf(sf, "%llu %1023s", &fr, dump_ev[dump_n].path) != 2) fatal("dump 参数错误");
            dump_ev[dump_n].frame = fr; dump_n++;
        } else if (!strcmp(cmd, "end")) {
            if (fscanf(sf, "%llu", &fr) != 1) fatal("end 参数错误");
            end_frame = fr;
        } else fatal("未知脚本命令 %s", cmd);
    }
    fclose(sf);
    memcpy(vram, chr_rom, sizeof chr_rom < 0x2000 ? sizeof chr_rom : 0x2000);
"#;

const RUNTIME_MAIN_B: &str = r#"    frames = cyc / CPF;
    for (;;) {
        while (cyc < (frames + 1) * CPF) cyc += 3; /* 停机 JMP 空转 */
        frames++;
        /* 此处 = nes-sim run_frames 返回点：本帧 NMI 已触发、体未执行 */
        for (i = 0; i < dump_n; i++) if (dump_ev[i].frame == frames) dump_snapshot(dump_ev[i].path);
        if (end_frame && frames >= end_frame) break;
        for (i = 0; i < btn_n; i++) if (btn_ev[i].frame == frames) buttons[0] = btn_ev[i].mask;
"#;

const RUNTIME_MAIN_C: &str = r#"        while ((frames + 1) * CPF <= cyc) frames++; /* NMI 体内跨帧界的滞后帧 */
    }
    return 0;
}
"#;
