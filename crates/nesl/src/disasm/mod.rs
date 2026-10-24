//! 6502 CPU 反汇编器
//!
//! 将 6502 机器码转换为汇编指令，用于真正的 NES ROM 反编译。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,        // INX
    Accumulator,    // ASL A
    Immediate,      // LDA #$12
    ZeroPage,       // LDA $12
    ZeroPageX,      // LDA $12,X
    ZeroPageY,      // LDX $12,Y
    Relative,       // BNE label
    Absolute,       // JMP $1234
    AbsoluteX,      // LDA $1234,X
    AbsoluteY,      // LDA $1234,Y
    Indirect,       // JMP ($1234)
    IndexedIndirect, // LDA ($12,X)
    IndirectIndexed, // LDA ($12),Y
}

#[derive(Debug, Clone)]
pub enum Instruction {
    // Load/Store
    LDA(AddressingMode, u16),
    LDX(AddressingMode, u16),
    LDY(AddressingMode, u16),
    STA(AddressingMode, u16),
    STX(AddressingMode, u16),
    STY(AddressingMode, u16),

    // Transfer
    TAX, TAY, TXA, TYA, TSX, TXS,

    // Stack
    PHA, PHP, PLA, PLP,

    // Arithmetic
    ADC(AddressingMode, u16),
    SBC(AddressingMode, u16),
    INC(AddressingMode, u16),
    INX, INY,
    DEC(AddressingMode, u16),
    DEX, DEY,

    // Logic
    AND(AddressingMode, u16),
    ORA(AddressingMode, u16),
    EOR(AddressingMode, u16),

    // Shift/Rotate
    ASL(AddressingMode, u16),
    LSR(AddressingMode, u16),
    ROL(AddressingMode, u16),
    ROR(AddressingMode, u16),

    // Compare
    CMP(AddressingMode, u16),
    CPX(AddressingMode, u16),
    CPY(AddressingMode, u16),

    // Branch
    BCC(i8), BCS(i8), BEQ(i8), BMI(i8),
    BNE(i8), BPL(i8), BVC(i8), BVS(i8),

    // Jump
    JMP(u16),
    JSR(u16),
    RTS,
    RTI,

    // Flags
    CLC, CLD, CLI, CLV,
    SEC, SED, SEI,

    // Misc
    BIT(AddressingMode, u16),
    NOP,
    BRK,

    // Invalid/Unknown
    Invalid(u8),
}

pub struct Disassembler<'a> {
    rom: &'a [u8],
    pc: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(rom: &'a [u8]) -> Self {
        Self { rom, pc: 0 }
    }

    /// 反汇编从指定地址开始的一条指令
    pub fn decode_at(&mut self, addr: usize) -> Option<(Instruction, usize)> {
        if addr >= self.rom.len() {
            return None;
        }

        self.pc = addr;
        let opcode = self.rom[self.pc];
        self.pc += 1;

        let instr = match opcode {
            // LDA
            0xA9 => Instruction::LDA(AddressingMode::Immediate, self.read_u8() as u16),
            0xA5 => Instruction::LDA(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xB5 => Instruction::LDA(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xAD => Instruction::LDA(AddressingMode::Absolute, self.read_u16()),
            0xBD => Instruction::LDA(AddressingMode::AbsoluteX, self.read_u16()),
            0xB9 => Instruction::LDA(AddressingMode::AbsoluteY, self.read_u16()),
            0xA1 => Instruction::LDA(AddressingMode::IndexedIndirect, self.read_u8() as u16),
            0xB1 => Instruction::LDA(AddressingMode::IndirectIndexed, self.read_u8() as u16),

            // LDX
            0xA2 => Instruction::LDX(AddressingMode::Immediate, self.read_u8() as u16),
            0xA6 => Instruction::LDX(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xB6 => Instruction::LDX(AddressingMode::ZeroPageY, self.read_u8() as u16),
            0xAE => Instruction::LDX(AddressingMode::Absolute, self.read_u16()),
            0xBE => Instruction::LDX(AddressingMode::AbsoluteY, self.read_u16()),

            // LDY
            0xA0 => Instruction::LDY(AddressingMode::Immediate, self.read_u8() as u16),
            0xA4 => Instruction::LDY(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xB4 => Instruction::LDY(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xAC => Instruction::LDY(AddressingMode::Absolute, self.read_u16()),
            0xBC => Instruction::LDY(AddressingMode::AbsoluteX, self.read_u16()),

            // STA
            0x85 => Instruction::STA(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x95 => Instruction::STA(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x8D => Instruction::STA(AddressingMode::Absolute, self.read_u16()),
            0x9D => Instruction::STA(AddressingMode::AbsoluteX, self.read_u16()),
            0x99 => Instruction::STA(AddressingMode::AbsoluteY, self.read_u16()),
            0x81 => Instruction::STA(AddressingMode::IndexedIndirect, self.read_u8() as u16),
            0x91 => Instruction::STA(AddressingMode::IndirectIndexed, self.read_u8() as u16),

            // STX/STY
            0x86 => Instruction::STX(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x96 => Instruction::STX(AddressingMode::ZeroPageY, self.read_u8() as u16),
            0x8E => Instruction::STX(AddressingMode::Absolute, self.read_u16()),
            0x84 => Instruction::STY(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x94 => Instruction::STY(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x8C => Instruction::STY(AddressingMode::Absolute, self.read_u16()),

            // Transfer
            0xAA => Instruction::TAX,
            0xA8 => Instruction::TAY,
            0x8A => Instruction::TXA,
            0x98 => Instruction::TYA,
            0xBA => Instruction::TSX,
            0x9A => Instruction::TXS,

            // Stack
            0x48 => Instruction::PHA,
            0x08 => Instruction::PHP,
            0x68 => Instruction::PLA,
            0x28 => Instruction::PLP,

            // Arithmetic
            0x69 => Instruction::ADC(AddressingMode::Immediate, self.read_u8() as u16),
            0x65 => Instruction::ADC(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x75 => Instruction::ADC(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x6D => Instruction::ADC(AddressingMode::Absolute, self.read_u16()),
            0x7D => Instruction::ADC(AddressingMode::AbsoluteX, self.read_u16()),
            0x79 => Instruction::ADC(AddressingMode::AbsoluteY, self.read_u16()),

            0xE9 => Instruction::SBC(AddressingMode::Immediate, self.read_u8() as u16),
            0xE5 => Instruction::SBC(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xF5 => Instruction::SBC(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xED => Instruction::SBC(AddressingMode::Absolute, self.read_u16()),
            0xFD => Instruction::SBC(AddressingMode::AbsoluteX, self.read_u16()),
            0xF9 => Instruction::SBC(AddressingMode::AbsoluteY, self.read_u16()),

            0xE8 => Instruction::INX,
            0xC8 => Instruction::INY,
            0xCA => Instruction::DEX,
            0x88 => Instruction::DEY,

            0xE6 => Instruction::INC(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xF6 => Instruction::INC(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xEE => Instruction::INC(AddressingMode::Absolute, self.read_u16()),
            0xFE => Instruction::INC(AddressingMode::AbsoluteX, self.read_u16()),

            0xC6 => Instruction::DEC(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xD6 => Instruction::DEC(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xCE => Instruction::DEC(AddressingMode::Absolute, self.read_u16()),
            0xDE => Instruction::DEC(AddressingMode::AbsoluteX, self.read_u16()),

            // Logic
            0x29 => Instruction::AND(AddressingMode::Immediate, self.read_u8() as u16),
            0x25 => Instruction::AND(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x35 => Instruction::AND(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x2D => Instruction::AND(AddressingMode::Absolute, self.read_u16()),
            0x3D => Instruction::AND(AddressingMode::AbsoluteX, self.read_u16()),
            0x39 => Instruction::AND(AddressingMode::AbsoluteY, self.read_u16()),

            0x09 => Instruction::ORA(AddressingMode::Immediate, self.read_u8() as u16),
            0x05 => Instruction::ORA(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x15 => Instruction::ORA(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x0D => Instruction::ORA(AddressingMode::Absolute, self.read_u16()),
            0x1D => Instruction::ORA(AddressingMode::AbsoluteX, self.read_u16()),
            0x19 => Instruction::ORA(AddressingMode::AbsoluteY, self.read_u16()),

            0x49 => Instruction::EOR(AddressingMode::Immediate, self.read_u8() as u16),
            0x45 => Instruction::EOR(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x55 => Instruction::EOR(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0x4D => Instruction::EOR(AddressingMode::Absolute, self.read_u16()),
            0x5D => Instruction::EOR(AddressingMode::AbsoluteX, self.read_u16()),
            0x59 => Instruction::EOR(AddressingMode::AbsoluteY, self.read_u16()),

            // Compare
            0xC9 => Instruction::CMP(AddressingMode::Immediate, self.read_u8() as u16),
            0xC5 => Instruction::CMP(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xD5 => Instruction::CMP(AddressingMode::ZeroPageX, self.read_u8() as u16),
            0xCD => Instruction::CMP(AddressingMode::Absolute, self.read_u16()),
            0xDD => Instruction::CMP(AddressingMode::AbsoluteX, self.read_u16()),
            0xD9 => Instruction::CMP(AddressingMode::AbsoluteY, self.read_u16()),

            0xE0 => Instruction::CPX(AddressingMode::Immediate, self.read_u8() as u16),
            0xE4 => Instruction::CPX(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xEC => Instruction::CPX(AddressingMode::Absolute, self.read_u16()),

            0xC0 => Instruction::CPY(AddressingMode::Immediate, self.read_u8() as u16),
            0xC4 => Instruction::CPY(AddressingMode::ZeroPage, self.read_u8() as u16),
            0xCC => Instruction::CPY(AddressingMode::Absolute, self.read_u16()),

            // Branch
            0x90 => Instruction::BCC(self.read_i8()),
            0xB0 => Instruction::BCS(self.read_i8()),
            0xF0 => Instruction::BEQ(self.read_i8()),
            0x30 => Instruction::BMI(self.read_i8()),
            0xD0 => Instruction::BNE(self.read_i8()),
            0x10 => Instruction::BPL(self.read_i8()),
            0x50 => Instruction::BVC(self.read_i8()),
            0x70 => Instruction::BVS(self.read_i8()),

            // Jump
            0x4C => Instruction::JMP(self.read_u16()),
            0x6C => {
                let addr = self.read_u16();
                Instruction::JMP(addr) // Indirect, but simplified
            },
            0x20 => Instruction::JSR(self.read_u16()),
            0x60 => Instruction::RTS,
            0x40 => Instruction::RTI,

            // Flags
            0x18 => Instruction::CLC,
            0xD8 => Instruction::CLD,
            0x58 => Instruction::CLI,
            0xB8 => Instruction::CLV,
            0x38 => Instruction::SEC,
            0xF8 => Instruction::SED,
            0x78 => Instruction::SEI,

            // Misc
            0x24 => Instruction::BIT(AddressingMode::ZeroPage, self.read_u8() as u16),
            0x2C => Instruction::BIT(AddressingMode::Absolute, self.read_u16()),
            0xEA => Instruction::NOP,
            0x00 => Instruction::BRK,

            _ => Instruction::Invalid(opcode),
        };

        let size = self.pc - addr;
        Some((instr, size))
    }

    fn read_u8(&mut self) -> u8 {
        let byte = self.rom[self.pc];
        self.pc += 1;
        byte
    }

    fn read_u16(&mut self) -> u16 {
        let lo = self.read_u8() as u16;
        let hi = self.read_u8() as u16;
        (hi << 8) | lo
    }

    fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }
}

impl Instruction {
    /// 格式化为汇编代码
    pub fn to_asm(&self, addr: u16) -> String {
        match self {
            Instruction::LDA(mode, op) => format!("LDA {}", format_operand(*mode, *op)),
            Instruction::LDX(mode, op) => format!("LDX {}", format_operand(*mode, *op)),
            Instruction::LDY(mode, op) => format!("LDY {}", format_operand(*mode, *op)),
            Instruction::STA(mode, op) => format!("STA {}", format_operand(*mode, *op)),
            Instruction::STX(mode, op) => format!("STX {}", format_operand(*mode, *op)),
            Instruction::STY(mode, op) => format!("STY {}", format_operand(*mode, *op)),

            Instruction::TAX => "TAX".into(),
            Instruction::TAY => "TAY".into(),
            Instruction::TXA => "TXA".into(),
            Instruction::TYA => "TYA".into(),
            Instruction::TSX => "TSX".into(),
            Instruction::TXS => "TXS".into(),

            Instruction::PHA => "PHA".into(),
            Instruction::PHP => "PHP".into(),
            Instruction::PLA => "PLA".into(),
            Instruction::PLP => "PLP".into(),

            Instruction::ADC(mode, op) => format!("ADC {}", format_operand(*mode, *op)),
            Instruction::SBC(mode, op) => format!("SBC {}", format_operand(*mode, *op)),
            Instruction::INC(mode, op) => format!("INC {}", format_operand(*mode, *op)),
            Instruction::INX => "INX".into(),
            Instruction::INY => "INY".into(),
            Instruction::DEC(mode, op) => format!("DEC {}", format_operand(*mode, *op)),
            Instruction::DEX => "DEX".into(),
            Instruction::DEY => "DEY".into(),

            Instruction::AND(mode, op) => format!("AND {}", format_operand(*mode, *op)),
            Instruction::ORA(mode, op) => format!("ORA {}", format_operand(*mode, *op)),
            Instruction::EOR(mode, op) => format!("EOR {}", format_operand(*mode, *op)),

            Instruction::ASL(mode, op) => format!("ASL {}", format_operand(*mode, *op)),
            Instruction::LSR(mode, op) => format!("LSR {}", format_operand(*mode, *op)),
            Instruction::ROL(mode, op) => format!("ROL {}", format_operand(*mode, *op)),
            Instruction::ROR(mode, op) => format!("ROR {}", format_operand(*mode, *op)),

            Instruction::CMP(mode, op) => format!("CMP {}", format_operand(*mode, *op)),
            Instruction::CPX(mode, op) => format!("CPX {}", format_operand(*mode, *op)),
            Instruction::CPY(mode, op) => format!("CPY {}", format_operand(*mode, *op)),

            Instruction::BCC(offset) => format!("BCC ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BCS(offset) => format!("BCS ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BEQ(offset) => format!("BEQ ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BMI(offset) => format!("BMI ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BNE(offset) => format!("BNE ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BPL(offset) => format!("BPL ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BVC(offset) => format!("BVC ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),
            Instruction::BVS(offset) => format!("BVS ${:04X}", (addr as i32 + 2 + *offset as i32) as u16),

            Instruction::JMP(addr) => format!("JMP ${:04X}", addr),
            Instruction::JSR(addr) => format!("JSR ${:04X}", addr),
            Instruction::RTS => "RTS".into(),
            Instruction::RTI => "RTI".into(),

            Instruction::CLC => "CLC".into(),
            Instruction::CLD => "CLD".into(),
            Instruction::CLI => "CLI".into(),
            Instruction::CLV => "CLV".into(),
            Instruction::SEC => "SEC".into(),
            Instruction::SED => "SED".into(),
            Instruction::SEI => "SEI".into(),

            Instruction::BIT(mode, op) => format!("BIT {}", format_operand(*mode, *op)),
            Instruction::NOP => "NOP".into(),
            Instruction::BRK => "BRK".into(),

            Instruction::Invalid(op) => format!(".db ${:02X}  ; invalid opcode", op),
        }
    }
}

fn format_operand(mode: AddressingMode, operand: u16) -> String {
    match mode {
        AddressingMode::Immediate => format!("#${:02X}", operand as u8),
        AddressingMode::ZeroPage => format!("${:02X}", operand as u8),
        AddressingMode::ZeroPageX => format!("${:02X},X", operand as u8),
        AddressingMode::ZeroPageY => format!("${:02X},Y", operand as u8),
        AddressingMode::Absolute => format!("${:04X}", operand),
        AddressingMode::AbsoluteX => format!("${:04X},X", operand),
        AddressingMode::AbsoluteY => format!("${:04X},Y", operand),
        AddressingMode::IndexedIndirect => format!("(${:02X},X)", operand as u8),
        AddressingMode::IndirectIndexed => format!("(${:02X}),Y", operand as u8),
        _ => format!("${:04X}", operand),
    }
}

/// Disassemble a complete NES ROM into 6502 assembly listing.
/// Works with any NES ROM, not just NESL-compiled ones.
pub fn disassemble_rom(rom_bytes: &[u8]) -> Result<String, String> {
    use std::collections::{BTreeMap, BTreeSet};

    if rom_bytes.len() < 16 {
        return Err("文件太小，不是有效的 NES ROM".to_string());
    }

    // Parse iNES header
    if &rom_bytes[0..4] != b"NES\x1a" {
        return Err("不是有效的 NES ROM（缺少 NES 头）".to_string());
    }

    let prg_banks = rom_bytes[4] as usize;
    let chr_banks = rom_bytes[5] as usize;
    let mapper = (rom_bytes[6] >> 4) | (rom_bytes[7] & 0xF0);

    if prg_banks == 0 {
        return Err("ROM 没有 PRG-ROM".to_string());
    }

    let prg_size = prg_banks * 16384;
    let chr_size = chr_banks * 8192;
    let header_size = 16;

    if rom_bytes.len() < header_size + prg_size {
        return Err(format!("ROM 文件被截断：期望 {} 字节 PRG", prg_size));
    }

    let prg_rom = &rom_bytes[header_size..header_size + prg_size];

    // 读取中断向量
    let (nmi_vec, reset_vec, irq_vec) = if prg_rom.len() >= 6 {
        let base = prg_rom.len() - 6;
        (
            u16::from_le_bytes([prg_rom[base], prg_rom[base + 1]]),
            u16::from_le_bytes([prg_rom[base + 2], prg_rom[base + 3]]),
            u16::from_le_bytes([prg_rom[base + 4], prg_rom[base + 5]]),
        )
    } else {
        (0x8000, 0x8000, 0x8000)
    };

    // 追踪代码块
    let mut visited = BTreeSet::new();
    let mut to_visit = vec![reset_vec, nmi_vec, irq_vec];
    let mut blocks: BTreeMap<u16, Vec<String>> = BTreeMap::new();

    let mut disasm = Disassembler::new(prg_rom);

    while let Some(addr) = to_visit.pop() {
        if visited.contains(&addr) || addr < 0x8000 {
            continue;
        }

        let offset = (addr - 0x8000) as usize;
        if offset >= prg_rom.len() {
            continue;
        }

        visited.insert(addr);
        let mut lines = Vec::new();
        let mut pc = offset;

        // 反汇编直到 RTS/RTI/JMP/无效指令
        for _ in 0..500 {
            if pc >= prg_rom.len() {
                break;
            }

            let Some((instr, size)) = disasm.decode_at(pc) else { break };
            let cpu_addr = 0x8000u16.wrapping_add(pc as u16);

            // 格式化指令
            let line = format!("    {}", instr.to_asm(cpu_addr));
            lines.push(line);

            // 追踪控制流
            match &instr {
                Instruction::JSR(target) => {
                    if *target >= 0x8000 {
                        to_visit.push(*target);
                    }
                    pc += size;
                }
                Instruction::JMP(target) => {
                    if *target >= 0x8000 {
                        to_visit.push(*target);
                    }
                    break;
                }
                Instruction::RTS | Instruction::RTI => break,
                Instruction::BCC(off) | Instruction::BCS(off) |
                Instruction::BEQ(off) | Instruction::BNE(off) |
                Instruction::BMI(off) | Instruction::BPL(off) |
                Instruction::BVC(off) | Instruction::BVS(off) => {
                    let target = cpu_addr.wrapping_add(size as u16).wrapping_add(*off as u16);
                    if target >= 0x8000 {
                        to_visit.push(target);
                    }
                    pc += size;
                }
                Instruction::Invalid(_) => break,
                _ => pc += size,
            }
        }

        if !lines.is_empty() {
            blocks.insert(addr, lines);
        }
    }

    // 生成输出
    let mut out = String::new();
    out.push_str(&"; NES ROM 反汇编\n".to_string());
    out.push_str(&format!("; PRG: {} 个 bank ({} 字节)\n", prg_banks, prg_size));
    out.push_str(&format!("; CHR: {} 个 bank ({} 字节)\n", chr_banks, chr_size));
    out.push_str(&format!("; Mapper: {}\n", mapper));
    out.push_str(&format!("; Reset 向量: ${:04X}\n\n", reset_vec));

    for (addr, lines) in &blocks {
        out.push_str(&format!("\nL{:04X}:  ; 代码块起始\n", addr));
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
    }

    out.push_str("\n; 中断向量表\n");
    out.push_str(&".org $FFFA\n".to_string());
    out.push_str(&format!("    .dw ${:04X}  ; NMI\n", nmi_vec));
    out.push_str(&format!("    .dw ${:04X}  ; RESET\n", reset_vec));
    out.push_str(&format!("    .dw ${:04X}  ; IRQ/BRK\n", irq_vec));

    Ok(out)
}
