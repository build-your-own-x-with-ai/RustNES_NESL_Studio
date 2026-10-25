//! Advanced decompiler: Static analysis for commercial NES ROMs without debug symbols.
//!
//! Converts 6502 assembly to high-level pseudo-code through:
//! - Variable inference (Zero Page analysis)
//! - Function identification (JSR/RTS analysis)
//! - Control flow reconstruction
//! - Pseudo-code generation

use crate::disasm::{Disassembler, Instruction, AddressingMode};
use std::collections::{HashMap, BTreeMap};

/// Memory access record
#[derive(Debug, Clone)]
pub struct MemoryAccess {
    pub addr: u8,           // Zero-page address
    pub is_write: bool,     // Read or write
    pub instruction: String, // Instruction type (LDA, STA, INC, etc.)
    pub pc: u16,            // Program counter when accessed
}

/// Inferred variable information
#[derive(Debug, Clone)]
pub struct Variable {
    pub addr: u8,
    pub name: String,
    pub var_type: VarType,
    pub access_count: usize,
    pub read_count: usize,
    pub write_count: usize,
    pub access_pattern: AccessPattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarType {
    U8,
    U16Low,     // Low byte of u16
    U16High,    // High byte of u16
    Array,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessPattern {
    ReadOnly,       // Only LDA
    WriteOnly,      // Only STA
    ReadWrite,      // Both LDA and STA
    Counter,        // INC/DEC pattern
    Flag,           // AND/ORA/BIT pattern
    Pointer,        // Indirect addressing
    Coordinate,     // Pattern suggests X/Y position
}

/// Scan all Zero Page memory accesses in PRG-ROM
pub fn scan_zeropage_accesses(prg_rom: &[u8]) -> HashMap<u8, Vec<MemoryAccess>> {
    let mut accesses: HashMap<u8, Vec<MemoryAccess>> = HashMap::new();
    let mut disasm = Disassembler::new(prg_rom);

    let cpu_base = 0x8000u16;
    let mut pc = 0;

    // Scan through PRG-ROM
    while pc < prg_rom.len().min(32768) {  // Scan first 32KB
        let Some((instr, size)) = disasm.decode_at(pc) else {
            pc += 1;
            continue;
        };

        let cpu_addr = cpu_base.wrapping_add(pc as u16);

        // Check if instruction accesses Zero Page
        if let Some((zp_addr, is_write, instr_name)) = extract_zp_access(&instr) {
            accesses.entry(zp_addr).or_default().push(MemoryAccess {
                addr: zp_addr,
                is_write,
                instruction: instr_name,
                pc: cpu_addr,
            });
        }

        pc += size;
    }

    accesses
}

/// Extract Zero Page address from instruction if it accesses ZP
fn extract_zp_access(instr: &Instruction) -> Option<(u8, bool, String)> {
    use Instruction as I;
    use AddressingMode as M;

    match instr {
        // Read operations
        I::LDA(M::ZeroPage, addr) => Some((*addr as u8, false, "LDA".into())),
        I::LDX(M::ZeroPage, addr) => Some((*addr as u8, false, "LDX".into())),
        I::LDY(M::ZeroPage, addr) => Some((*addr as u8, false, "LDY".into())),
        I::LDA(M::ZeroPageX, addr) => Some((*addr as u8, false, "LDA,X".into())),
        I::LDA(M::ZeroPageY, addr) => Some((*addr as u8, false, "LDA,Y".into())),
        I::LDX(M::ZeroPageY, addr) => Some((*addr as u8, false, "LDX,Y".into())),
        I::LDY(M::ZeroPageX, addr) => Some((*addr as u8, false, "LDY,X".into())),

        // Write operations
        I::STA(M::ZeroPage, addr) => Some((*addr as u8, true, "STA".into())),
        I::STX(M::ZeroPage, addr) => Some((*addr as u8, true, "STX".into())),
        I::STY(M::ZeroPage, addr) => Some((*addr as u8, true, "STY".into())),
        I::STA(M::ZeroPageX, addr) => Some((*addr as u8, true, "STA,X".into())),
        I::STX(M::ZeroPageY, addr) => Some((*addr as u8, true, "STX,Y".into())),
        I::STY(M::ZeroPageX, addr) => Some((*addr as u8, true, "STY,X".into())),

        // Read-modify-write
        I::INC(M::ZeroPage, addr) => Some((*addr as u8, true, "INC".into())),
        I::DEC(M::ZeroPage, addr) => Some((*addr as u8, true, "DEC".into())),
        I::ASL(M::ZeroPage, addr) => Some((*addr as u8, true, "ASL".into())),
        I::LSR(M::ZeroPage, addr) => Some((*addr as u8, true, "LSR".into())),
        I::ROL(M::ZeroPage, addr) => Some((*addr as u8, true, "ROL".into())),
        I::ROR(M::ZeroPage, addr) => Some((*addr as u8, true, "ROR".into())),

        // Bit test (read-only)
        I::BIT(M::ZeroPage, addr) => Some((*addr as u8, false, "BIT".into())),

        // Arithmetic/logic (read)
        I::ADC(M::ZeroPage, addr) => Some((*addr as u8, false, "ADC".into())),
        I::SBC(M::ZeroPage, addr) => Some((*addr as u8, false, "SBC".into())),
        I::AND(M::ZeroPage, addr) => Some((*addr as u8, false, "AND".into())),
        I::ORA(M::ZeroPage, addr) => Some((*addr as u8, false, "ORA".into())),
        I::EOR(M::ZeroPage, addr) => Some((*addr as u8, false, "EOR".into())),
        I::CMP(M::ZeroPage, addr) => Some((*addr as u8, false, "CMP".into())),
        I::CPX(M::ZeroPage, addr) => Some((*addr as u8, false, "CPX".into())),
        I::CPY(M::ZeroPage, addr) => Some((*addr as u8, false, "CPY".into())),

        // Indirect addressing (pointer usage)
        I::LDA(M::IndexedIndirect, addr) => Some((*addr as u8, false, "LDA,(X)".into())),
        I::STA(M::IndexedIndirect, addr) => Some((*addr as u8, true, "STA,(X)".into())),
        I::LDA(M::IndirectIndexed, addr) => Some((*addr as u8, false, "LDA,()Y".into())),
        I::STA(M::IndirectIndexed, addr) => Some((*addr as u8, true, "STA,()Y".into())),

        _ => None,
    }
}

/// Analyze accesses to infer variable types and patterns
pub fn infer_variables(accesses: &HashMap<u8, Vec<MemoryAccess>>) -> HashMap<u8, Variable> {
    let mut variables = HashMap::new();

    for (addr, access_list) in accesses {
        // Skip if too few accesses (noise)
        if access_list.len() < 2 {
            continue;
        }

        // Count reads vs writes
        let read_count = access_list.iter().filter(|a| !a.is_write).count();
        let write_count = access_list.iter().filter(|a| a.is_write).count();

        // Determine access pattern
        let access_pattern = determine_access_pattern(access_list);

        // Infer type
        let var_type = infer_type(*addr, access_list, accesses);

        // Generate name
        let name = generate_variable_name(*addr, &access_pattern, &var_type);

        variables.insert(*addr, Variable {
            addr: *addr,
            name,
            var_type,
            access_count: access_list.len(),
            read_count,
            write_count,
            access_pattern,
        });
    }

    variables
}

/// Determine the primary access pattern
fn determine_access_pattern(accesses: &[MemoryAccess]) -> AccessPattern {
    let has_read = accesses.iter().any(|a| !a.is_write);
    let has_write = accesses.iter().any(|a| a.is_write);
    let has_inc_dec = accesses.iter().any(|a| a.instruction == "INC" || a.instruction == "DEC");
    let has_bitwise = accesses.iter().any(|a| {
        matches!(a.instruction.as_str(), "AND" | "ORA" | "EOR" | "BIT")
    });
    let has_indirect = accesses.iter().any(|a| a.instruction.contains('('));

    if has_indirect {
        AccessPattern::Pointer
    } else if has_inc_dec {
        AccessPattern::Counter
    } else if has_bitwise {
        AccessPattern::Flag
    } else if !has_write {
        AccessPattern::ReadOnly
    } else if !has_read {
        AccessPattern::WriteOnly
    } else {
        AccessPattern::ReadWrite
    }
}

/// Infer variable type (u8, u16, array)
fn infer_type(addr: u8, accesses: &[MemoryAccess], all_accesses: &HashMap<u8, Vec<MemoryAccess>>) -> VarType {
    // Check if this and next address are accessed together (u16 pattern)
    if addr < 255 {
        if let Some(next_accesses) = all_accesses.get(&(addr + 1)) {
            // Count how many times addr and addr+1 are accessed close together
            let mut pair_count = 0;
            for a1 in accesses {
                for a2 in next_accesses {
                    // If accessed within 10 bytes of code, likely a u16
                    if a1.pc.abs_diff(a2.pc) < 10 {
                        pair_count += 1;
                    }
                }
            }

            // If frequently paired, this is the low byte of a u16
            if pair_count > accesses.len() / 2 {
                return VarType::U16Low;
            }
        }
    }

    // Check if previous address might be u16 low byte
    if addr > 0 {
        if let Some(prev_accesses) = all_accesses.get(&(addr - 1)) {
            let mut pair_count = 0;
            for a1 in prev_accesses {
                for a2 in accesses {
                    if a1.pc.abs_diff(a2.pc) < 10 {
                        pair_count += 1;
                    }
                }
            }
            if pair_count > accesses.len() / 2 {
                return VarType::U16High;
            }
        }
    }

    // Check for indexed access (array pattern)
    if accesses.iter().any(|a| a.instruction.contains(",X") || a.instruction.contains(",Y")) {
        return VarType::Array;
    }

    VarType::U8
}

/// Generate a meaningful variable name
fn generate_variable_name(addr: u8, pattern: &AccessPattern, var_type: &VarType) -> String {
    // NES-specific conventions
    match (addr, pattern, var_type) {
        // Common pointer pairs
        (0x00 | 0x02 | 0x04, AccessPattern::Pointer, _) => format!("ptr{}", addr / 2),

        // Common coordinate pairs
        (0x02, _, VarType::U8) => "player_x".to_string(),
        (0x03, _, VarType::U8) => "player_y".to_string(),
        (0x04, _, VarType::U8) => "enemy_x".to_string(),
        (0x05, _, VarType::U8) => "enemy_y".to_string(),

        // Counters
        (_, AccessPattern::Counter, _) => format!("counter_{:02X}", addr),

        // Flags
        (_, AccessPattern::Flag, _) => format!("flags_{:02X}", addr),

        // U16 pairs
        (_, _, VarType::U16Low) => format!("var_{:02X}_lo", addr),
        (_, _, VarType::U16High) => format!("var_{:02X}_hi", addr),

        // Default
        _ => format!("var_{:02X}", addr),
    }
}

// ============================================================================
// Phase 2: Function Identification
// ============================================================================

/// Inferred function information
#[derive(Debug, Clone)]
pub struct Function {
    pub entry: u16,
    pub name: String,
    pub exit_points: Vec<u16>,    // RTS/RTI locations
    pub calls_to: Vec<u16>,        // Functions this calls
    pub called_from: Vec<u16>,     // Where this is called from
    pub size_bytes: usize,         // Approximate size
}

/// Scan PRG-ROM for function calls (JSR instructions)
pub fn scan_function_calls(prg_rom: &[u8]) -> HashMap<u16, Vec<u16>> {
    let mut calls: HashMap<u16, Vec<u16>> = HashMap::new();  // callee -> [callers]
    let mut disasm = Disassembler::new(prg_rom);
    let cpu_base = 0x8000u16;

    let mut pc = 0;
    while pc < prg_rom.len().min(32768) {
        let Some((instr, size)) = disasm.decode_at(pc) else {
            pc += 1;
            continue;
        };

        if let Instruction::JSR(target) = instr {
            let caller = cpu_base.wrapping_add(pc as u16);
            calls.entry(target).or_default().push(caller);
        }

        pc += size;
    }

    calls
}

/// Identify interrupt vector functions (NMI, RESET, IRQ)
pub fn identify_interrupt_handlers(prg_rom: &[u8]) -> HashMap<String, u16> {
    let mut handlers = HashMap::new();

    // Interrupt vectors are at the last 6 bytes of PRG-ROM
    if prg_rom.len() >= 6 {
        let vectors_offset = prg_rom.len() - 6;

        let nmi = u16::from_le_bytes([
            prg_rom[vectors_offset],
            prg_rom[vectors_offset + 1]
        ]);
        let reset = u16::from_le_bytes([
            prg_rom[vectors_offset + 2],
            prg_rom[vectors_offset + 3]
        ]);
        let irq = u16::from_le_bytes([
            prg_rom[vectors_offset + 4],
            prg_rom[vectors_offset + 5]
        ]);

        handlers.insert("nmi".to_string(), nmi);
        handlers.insert("reset".to_string(), reset);
        handlers.insert("irq".to_string(), irq);
    }

    handlers
}

/// Find function exit points (RTS/RTI) starting from entry address
pub fn find_function_exits(entry: u16, prg_rom: &[u8], max_size: usize) -> Vec<u16> {
    let cpu_base = 0x8000u16;
    let offset = entry.wrapping_sub(cpu_base) as usize;

    if offset >= prg_rom.len() {
        return Vec::new();
    }

    let mut exits = Vec::new();
    let mut disasm = Disassembler::new(prg_rom);
    let mut pc = offset;

    // Scan forward for RTS/RTI, stop after max_size bytes or hitting one
    for _ in 0..max_size {
        if pc >= prg_rom.len() {
            break;
        }

        let Some((instr, size)) = disasm.decode_at(pc) else {
            pc += 1;
            continue;
        };

        match instr {
            Instruction::RTS | Instruction::RTI => {
                exits.push(cpu_base.wrapping_add(pc as u16));
                break;  // Stop at first exit for simple analysis
            }
            Instruction::JMP(_) => {
                // Tail call or jump table, stop here
                break;
            }
            _ => {}
        }

        pc += size;
    }

    exits
}

/// Infer all functions from call graph and interrupt vectors
pub fn infer_functions(prg_rom: &[u8]) -> BTreeMap<u16, Function> {
    let mut functions = BTreeMap::new();
    let calls = scan_function_calls(prg_rom);
    let handlers = identify_interrupt_handlers(prg_rom);

    // Add interrupt handlers first
    for (handler_name, entry) in &handlers {
        if *entry >= 0x8000 {  // Valid ROM address
            let exits = find_function_exits(*entry, prg_rom, 1000);
            let size_bytes = if let Some(first_exit) = exits.first() {
                first_exit.wrapping_sub(*entry) as usize
            } else {
                256  // Default estimate
            };

            functions.insert(*entry, Function {
                entry: *entry,
                name: handler_name.clone(),
                exit_points: exits,
                calls_to: Vec::new(),
                called_from: Vec::new(),
                size_bytes,
            });
        }
    }

    // Add all called functions
    for (callee, callers) in &calls {
        if *callee < 0x8000 {
            continue;  // Skip non-ROM addresses
        }

        let name = generate_function_name(*callee, &handlers);
        let exits = find_function_exits(*callee, prg_rom, 1000);
        let size_bytes = if let Some(first_exit) = exits.first() {
            first_exit.wrapping_sub(*callee) as usize
        } else {
            256
        };

        functions.entry(*callee).or_insert_with(|| Function {
            entry: *callee,
            name,
            exit_points: exits,
            calls_to: Vec::new(),
            called_from: callers.clone(),
            size_bytes,
        });
    }

    // Build call graph (what each function calls)
    for func in functions.values_mut() {
        let offset = func.entry.wrapping_sub(0x8000) as usize;
        if offset >= prg_rom.len() {
            continue;
        }

        let mut disasm = Disassembler::new(prg_rom);
        let mut pc = offset;

        // Scan function body for JSR calls
        for _ in 0..func.size_bytes {
            if pc >= prg_rom.len() {
                break;
            }

            let Some((instr, size)) = disasm.decode_at(pc) else {
                pc += 1;
                continue;
            };

            if let Instruction::JSR(target) = instr {
                if target >= 0x8000 {
                    func.calls_to.push(target);
                }
            }

            if matches!(instr, Instruction::RTS | Instruction::RTI) {
                break;
            }

            pc += size;
        }
    }

    functions
}

/// Generate a meaningful function name
fn generate_function_name(addr: u16, handlers: &HashMap<String, u16>) -> String {
    // Check if it's an interrupt handler
    for (name, &handler_addr) in handlers {
        if addr == handler_addr {
            return name.clone();
        }
    }

    // Check for common library function patterns
    // (This is heuristic-based and may not always be accurate)
    match addr {
        0x8000..=0x8010 => format!("init_{:04X}", addr),
        _ => format!("sub_{:04X}", addr),
    }
}
