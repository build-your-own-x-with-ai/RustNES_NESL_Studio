//! Enhanced pseudocode generator with control flow reconstruction
//!
//! Phase 2: Apply control flow reconstruction to generate better pseudocode

use std::collections::{HashMap, BTreeMap, HashSet};
use crate::disasm::{Disassembler, Instruction, AddressingMode};
use crate::decompile::advanced::{Variable, Function, AccessPattern};

/// Enhanced pseudocode generator
pub struct EnhancedPseudocodeGenerator {
    prg_rom: Vec<u8>,
    variables: HashMap<u8, Variable>,
    functions: BTreeMap<u16, Function>,
    prg_size: usize,
}

impl EnhancedPseudocodeGenerator {
    pub fn new(
        prg_rom: &[u8],
        variables: HashMap<u8, Variable>,
        functions: BTreeMap<u16, Function>,
    ) -> Self {
        Self {
            prg_rom: prg_rom.to_vec(),
            variables,
            functions,
            prg_size: prg_rom.len(),
        }
    }

    /// Generate enhanced pseudocode with control flow reconstruction
    pub fn generate(&self) -> String {
        let mut code = String::new();

        code.push_str("// NES ROM Enhanced Decompilation\n");
        code.push_str("// With Control Flow Reconstruction\n");
        code.push_str("// Phase 2: Improved Quality\n\n");

        // Variables
        code.push_str(&self.generate_variables());
        code.push_str("\n");

        // Functions with control flow
        code.push_str(&self.generate_functions());

        code
    }

    fn generate_variables(&self) -> String {
        let mut code = String::new();
        code.push_str("// ========== Variables ==========\n\n");

        let mut vars: Vec<_> = self.variables.values().collect();
        vars.sort_by_key(|v| v.addr);

        for var in vars.iter().take(50) {
            let var_type = self.infer_variable_type(var);
            let comment = self.variable_comment(var);

            code.push_str(&format!("{} {};  // ${:02X} {}\n",
                var_type, var.name, var.addr, comment));
        }

        code
    }

    fn infer_variable_type(&self, var: &Variable) -> String {
        match var.access_pattern {
            AccessPattern::Pointer => "u16".to_string(),
            AccessPattern::Flag if var.access_count > 10 => "bool".to_string(),
            _ => "u8".to_string(),
        }
    }

    fn variable_comment(&self, var: &Variable) -> String {
        let pattern = match var.access_pattern {
            AccessPattern::Counter => "counter",
            AccessPattern::Flag => "flag",
            AccessPattern::Pointer => "pointer",
            AccessPattern::Coordinate => "coordinate",
            _ => "",
        };

        format!("{} ({} accesses)", pattern, var.access_count)
    }

    fn generate_functions(&self) -> String {
        let mut code = String::new();
        code.push_str("// ========== Functions ==========\n\n");

        for (addr, func) in self.functions.iter().take(20) {
            code.push_str(&self.generate_function_enhanced(*addr, func));
            code.push_str("\n");
        }

        code
    }

    fn generate_function_enhanced(&self, addr: u16, func: &Function) -> String {
        let mut code = format!("void {}() {{  // ${:04X}\n", func.name, addr);

        // Get function body
        let offset = self.addr_to_offset(addr);
        if offset >= self.prg_rom.len() {
            code.push_str("    // Out of bounds\n");
            code.push_str("}\n");
            return code;
        }

        // Analyze function structure
        let structure = self.analyze_function_structure(addr, func);

        match structure {
            FunctionStructure::Simple => {
                code.push_str(&self.generate_simple_function(addr, func));
            }
            FunctionStructure::HasLoop => {
                code.push_str(&self.generate_loop_function(addr, func));
            }
            FunctionStructure::HasBranching => {
                code.push_str(&self.generate_branching_function(addr, func));
            }
            FunctionStructure::Complex => {
                code.push_str("    // Complex function - needs manual analysis\n");
                code.push_str(&self.generate_fallback(addr, func));
            }
        }

        code.push_str("}\n");
        code
    }

    fn analyze_function_structure(&self, addr: u16, func: &Function) -> FunctionStructure {
        let offset = self.addr_to_offset(addr);
        let mut disasm = Disassembler::new(&self.prg_rom);

        let mut has_loop = false;
        let mut has_branch = false;
        let mut branch_count = 0;

        let end_offset = offset + func.size_bytes.min(256);

        for pc in offset..end_offset.min(self.prg_rom.len()) {
            if let Some((instr, _)) = disasm.decode_at(pc) {
                match instr {
                    Instruction::BCC(_) | Instruction::BCS(_) |
                    Instruction::BEQ(_) | Instruction::BNE(_) |
                    Instruction::BMI(_) | Instruction::BPL(_) |
                    Instruction::BVC(_) | Instruction::BVS(_) => {
                        has_branch = true;
                        branch_count += 1;

                        // Check if it's a backward branch (loop)
                        if let Some(target) = Self::extract_branch_offset(&instr) {
                            if target < 0 {
                                has_loop = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if branch_count > 5 {
            FunctionStructure::Complex
        } else if has_loop {
            FunctionStructure::HasLoop
        } else if has_branch {
            FunctionStructure::HasBranching
        } else {
            FunctionStructure::Simple
        }
    }

    fn extract_branch_offset(instr: &Instruction) -> Option<i8> {
        match instr {
            Instruction::BCC(offset) | Instruction::BCS(offset) |
            Instruction::BEQ(offset) | Instruction::BNE(offset) |
            Instruction::BMI(offset) | Instruction::BPL(offset) |
            Instruction::BVC(offset) | Instruction::BVS(offset) => Some(*offset),
            _ => None,
        }
    }

    fn generate_simple_function(&self, addr: u16, func: &Function) -> String {
        let mut code = String::new();
        code.push_str("    // Simple sequential function\n");

        let offset = self.addr_to_offset(addr);
        let mut disasm = Disassembler::new(&self.prg_rom);
        let end = offset + func.size_bytes.min(100);

        let mut pc = offset;
        while pc < end && pc < self.prg_rom.len() {
            if let Some((instr, size)) = disasm.decode_at(pc) {
                let stmt = self.instruction_to_statement(&instr, pc as u16);
                if !stmt.is_empty() {
                    code.push_str(&format!("    {}\n", stmt));
                }

                if matches!(instr, Instruction::RTS | Instruction::RTI) {
                    break;
                }

                pc += size;
            } else {
                break;
            }
        }

        code
    }

    fn generate_loop_function(&self, _addr: u16, _func: &Function) -> String {
        let mut code = String::new();
        code.push_str("    // Function with loop detected\n");
        code.push_str("    // TODO: Reconstruct loop structure\n");
        code
    }

    fn generate_branching_function(&self, _addr: u16, _func: &Function) -> String {
        let mut code = String::new();
        code.push_str("    // Function with conditional branching\n");
        code.push_str("    // TODO: Reconstruct if statements\n");
        code
    }

    fn generate_fallback(&self, _addr: u16, func: &Function) -> String {
        format!("    // Complex: {} bytes, {} calls\n",
            func.size_bytes, func.calls_to.len())
    }

    fn instruction_to_statement(&self, instr: &Instruction, _pc: u16) -> String {
        match instr {
            Instruction::LDA(mode, val) => {
                format!("A = {};", self.addressing_mode_to_expr(mode, *val))
            }
            Instruction::STA(mode, val) => {
                format!("{} = A;", self.addressing_mode_to_expr(mode, *val))
            }
            Instruction::CMP(mode, val) => {
                format!("compare(A, {});", self.addressing_mode_to_expr(mode, *val))
            }
            Instruction::RTS => "return;".to_string(),
            Instruction::JSR(addr) => {
                if let Some(func) = self.functions.get(addr) {
                    format!("{}();", func.name)
                } else {
                    format!("call_${:04X}();", addr)
                }
            }
            _ => String::new(),
        }
    }

    fn addressing_mode_to_expr(&self, mode: &AddressingMode, val: u16) -> String {
        match mode {
            AddressingMode::Immediate => format!("0x{:02X}", val as u8),
            AddressingMode::ZeroPage => {
                let addr = val as u8;
                if let Some(var) = self.variables.get(&addr) {
                    var.name.clone()
                } else {
                    format!("mem[0x{:02X}]", addr)
                }
            }
            AddressingMode::Absolute => {
                format!("mem[0x{:04X}]", val)
            }
            _ => "...".to_string(),
        }
    }

    fn addr_to_offset(&self, addr: u16) -> usize {
        if addr >= 0xC000 && self.prg_size <= 16384 {
            (addr - 0xC000) as usize
        } else if addr >= 0x8000 {
            (addr - 0x8000) as usize
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FunctionStructure {
    Simple,       // Sequential, no branches
    HasLoop,      // Contains loops
    HasBranching, // Has if statements
    Complex,      // Too complex
}

/// Generate enhanced pseudocode
pub fn generate_enhanced_pseudocode(
    prg_rom: &[u8],
    variables: &HashMap<u8, Variable>,
    functions: &BTreeMap<u16, Function>,
) -> String {
    let generator = EnhancedPseudocodeGenerator::new(prg_rom, variables.clone(), functions.clone());
    generator.generate()
}
