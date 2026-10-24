//! AST reconstruction from symbols and ROM data.

use crate::debug_symbols::{SymbolTable, Type, Value};
use crate::decompile::{DecompileError, rom_parser::RomImage};

/// Reconstructed NESL AST (simplified representation).
#[derive(Debug, Clone)]
pub struct DecompiledAst {
    pub constants: Vec<ConstDecl>,
    pub structs: Vec<StructDecl>,
    pub variables: Vec<VarDecl>,
    pub tiles: Vec<TileDecl>,
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Type,
    pub value: Value,
}

/// A struct type recovered from flattened `name.field` variables.
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub ty: Type,
    pub init: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct TileDecl {
    pub index: u8,
    pub pattern: [String; 8],
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_ty: Option<Type>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { name: String, ty: Option<Type>, init: Option<Expr> },
    Assign { target: String, value: Expr },
    Call { func: String, args: Vec<Expr> },
    If { cond: Expr, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    Return { value: Option<Expr> },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Literal(Value),
    BinOp { op: String, left: Box<Expr>, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
}

/// Builds a decompiled AST from symbol table and ROM data.
pub fn build_ast(table: &SymbolTable, rom: &RomImage) -> Result<DecompiledAst, DecompileError> {
    let mut constants = Vec::new();
    let mut variables = Vec::new();
    let mut tiles = Vec::new();
    let mut functions = Vec::new();

    // Restore constants from symbol table
    for c in &table.constants {
        let Type::Array { element_ty, length } = &c.ty else { continue };
        // Older symbol files carry no table contents; a table must have at least one element.
        let stride = if **element_ty == Type::U16 { 2 } else { 1 };
        if *length == 0 || c.data.len() != length * stride {
            continue;
        }
        let values = (0..*length)
            .map(|i| match **element_ty {
                Type::I8 => Value::I8(c.data[i] as i8),
                // u16 tables: all low bytes, then all high bytes
                Type::U16 => Value::U16(c.data[i] as u16 | (c.data[length + i] as u16) << 8),
                _ => Value::U8(c.data[i]),
            })
            .collect();
        constants.push(ConstDecl {
            name: c.name.clone(),
            // `const NAME: [T] = [...]` takes its length from the elements
            ty: Type::Array { element_ty: element_ty.clone(), length: 0 },
            value: Value::Array(values),
        });
    }

    // Restore variables. The compiler flattens `var rocks: [Rock; 6]` into one variable per
    // field (`rocks.x`, `rocks.y`, ...), allocated in field order, so sorting a group by
    // address recovers the struct's field order.
    let mut groups: Vec<(String, Vec<&crate::debug_symbols::Variable>)> = Vec::new();
    for v in &table.variables {
        match v.name.split_once('.') {
            Some((base, _)) => match groups.iter_mut().find(|(b, _)| b == base) {
                Some((_, g)) => g.push(v),
                None => groups.push((base.to_string(), vec![v])),
            },
            None => variables.push(VarDecl { name: v.name.clone(), ty: v.ty.clone(), init: v.init_value.clone() }),
        }
    }
    let mut structs: Vec<StructDecl> = Vec::new();
    for (base, mut fields) in groups {
        fields.sort_by_key(|v| v.addr);
        // Field types are scalars; struct arrays store each field as an array of the same length.
        let len = match &fields[0].ty {
            Type::Array { length, .. } => Some(*length),
            _ => None,
        };
        let field_list: Vec<(String, Type)> = fields
            .iter()
            .map(|v| {
                let fname = v.name.split_once('.').unwrap().1.to_string();
                let ty = match &v.ty {
                    Type::Array { element_ty, .. } => (**element_ty).clone(),
                    t => t.clone(),
                };
                (fname, ty)
            })
            .collect();
        // Reuse an identical struct shape (e.g. two `Rock` variables) instead of duplicating it.
        let sname = match structs.iter().find(|s| s.fields == field_list) {
            Some(s) => s.name.clone(),
            None => {
                let sname = struct_name(&base);
                structs.push(StructDecl { name: sname.clone(), fields: field_list });
                sname
            }
        };
        let ty = match len {
            Some(n) => Type::Array { element_ty: Box::new(Type::Struct { name: sname }), length: n },
            None => Type::Struct { name: sname },
        };
        variables.push(VarDecl { name: base, ty, init: None });
    }

    // Extract tiles from CHR-ROM
    if !rom.chr_rom.is_empty() {
        // Extract user-defined tiles (non-font tiles)
        let user_tiles = super::chr_decode::extract_user_tiles(&rom.chr_rom);
        for (index, pattern) in user_tiles {
            tiles.push(TileDecl { index, pattern });
        }
    }

    // Restore function signatures and decompile function bodies
    for f in &table.functions {
        let body = decompile_function(f, rom, table)?;
        functions.push(FunctionDecl {
            name: f.name.clone(),
            params: f.params.clone(),
            return_ty: f.return_ty.clone(),
            body,
        });
    }

    Ok(DecompiledAst {
        constants,
        structs,
        variables,
        tiles,
        functions,
    })
}

/// Struct type name for a variable: `rocks` -> `Rocks`, `enemy_list` -> `EnemyList`.
fn struct_name(var: &str) -> String {
    var.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
        })
        .collect()
}

/// Decompiles a function's body from its 6502 machine code.
fn decompile_function(
    func: &crate::debug_symbols::Function,
    rom: &RomImage,
    table: &SymbolTable,
) -> Result<Vec<Stmt>, DecompileError> {
    use crate::disasm::{AddressingMode, Disassembler, Instruction};

    let cpu_base: u16 = if rom.header.mapper == 2 { 0xC000 } else { 0x8000 };
    let func_offset = func.addr.wrapping_sub(cpu_base) as usize;

    if func_offset >= rom.prg_rom.len() {
        return Ok(vec![]);
    }

    let mut dis = Disassembler::new(&rom.prg_rom);
    let mut body = Vec::new();

    // First pass: collect loop targets (backward JMPs).
    let mut loop_targets = std::collections::HashSet::<usize>::new();
    {
        let mut pc = func_offset;
        for _ in 0..500 {
            let Some((instr, size)) = dis.decode_at(pc) else { break };
            if let Instruction::JMP(target) = instr {
                let off = target.wrapping_sub(cpu_base) as usize;
                if off <= pc { loop_targets.insert(off); }
            }
            if matches!(instr, Instruction::RTS | Instruction::RTI) { break; }
            pc += size;
        }
    }

    // Second pass: emit NESL statements.
    // `arg_slots` tracks the last value written to ARG0-ARG4 ($10-$14) since the
    // previous JSR; used to reconstruct call arguments.
    // Slot 0 = ARG0 ($10) … slot 4 = ARG4 ($14); None = unknown.
    let mut arg_slots: [Option<Expr>; 5] = Default::default();
    // ptr_slot: last absolute address whose lo byte was stored at $04 (PTR) and hi at $05
    // For load_palette: ARG2 holds the length, but we resolve by const name via the data table.
    // Track the last data pointer address seen (set by `pointer(&label)` which writes to $04/$05).
    let mut data_ptr_addr: Option<u16> = None;

    let start_pc = func_offset;
    let mut pc = func_offset;
    let mut loop_start: Option<usize> = None;
    let mut loop_body: Vec<Stmt> = Vec::new();
    let mut in_loop = false;

    for _ in 0..2000 {
        if pc >= rom.prg_rom.len() { break; }

        // Entering a loop?
        if loop_targets.contains(&pc) && pc > start_pc && !in_loop {
            in_loop = true;
            loop_start = Some(pc);
        }

        let Some((instr, size)) = dis.decode_at(pc) else { break };

        let stmts = if in_loop { &mut loop_body } else { &mut body };

        match &instr {
            // STA into ARG0-ARG4 ($10-$14): the concrete value was already captured by the
            // LDA #imm lookahead below, so nothing extra to do here.
            // STA $04/$05 (PTR): the data_ptr_addr was set by the 4-instruction lookahead on
            // the first LDA of that sequence — do NOT reset it here.
            Instruction::STA(AddressingMode::ZeroPage, _) |
            Instruction::STA(AddressingMode::Absolute, _) => {
                pc += size;
                continue;
            }
            // Track LDA #imm followed by STA slot — handled as a two-instruction window.
            _ => {}
        }

        // Two-instruction window: LDA #n ; STA $1x
        // We look ahead one instruction to see if the next is STA into ARG0-4.
        if let Instruction::LDA(AddressingMode::Immediate, imm) = &instr {
            let imm = *imm as u8;
            let next_off = pc + size;
            if let Some((Instruction::STA(AddressingMode::ZeroPage, slot_addr), _)) =
                dis.decode_at(next_off)
            {
                if (0x10..=0x14).contains(&slot_addr) {
                    let slot = (slot_addr - 0x10) as usize;
                    arg_slots[slot] = Some(Expr::Literal(Value::U8(imm)));
                }
            }
        }

        // Four-instruction lookahead: LDA #lo ; STA $04 ; LDA #hi ; STA $05
        // This is the `pointer(&label)` sequence the emitter emits for Data/Text args.
        // Capture the reconstructed address so we can match it to a const table entry.
        if let Instruction::LDA(AddressingMode::Immediate, lo) = &instr {
            let next_off = pc + size;
            if let Some((Instruction::STA(AddressingMode::ZeroPage, 0x04), sz2)) =
                dis.decode_at(next_off)
            {
                let next2 = next_off + sz2;
                if let Some((Instruction::LDA(AddressingMode::Immediate, hi), sz3)) =
                    dis.decode_at(next2)
                {
                    let next3 = next2 + sz3;
                    if let Some((Instruction::STA(AddressingMode::ZeroPage, 0x05), _)) =
                        dis.decode_at(next3)
                    {
                        let addr = *lo | (hi << 8);
                        data_ptr_addr = Some(addr);
                    }
                }
            }
        }

        match &instr {
            Instruction::JSR(target_addr) => {
                let target = *target_addr;
                let name_opt = detect_builtin(target, table);

                if let Some(name) = name_opt {
                    let args = reconstruct_args(&name, &arg_slots, data_ptr_addr, table, cpu_base, &rom.prg_rom);
                    stmts.push(Stmt::Call { func: name, args });
                    // Reset tracking after JSR
                    arg_slots = Default::default();
                    data_ptr_addr = None;
                } else if let Some(fname) = find_function_by_addr(target, table) {
                    stmts.push(Stmt::Call { func: fname, args: vec![] });
                    arg_slots = Default::default();
                    data_ptr_addr = None;
                }
                pc += size;
            }
            Instruction::RTS => break,
            Instruction::JMP(target) => {
                let off = target.wrapping_sub(cpu_base) as usize;
                if in_loop && loop_start.is_some_and(|s| off <= s) {
                    // Close the loop
                    let finished = std::mem::take(&mut loop_body);
                    body.push(Stmt::While {
                        cond: Expr::Literal(Value::U8(1)),
                        body: finished,
                    });
                    break;
                }
                pc += size;
            }
            _ => { pc += size; }
        }
    }

    // Flush any open loop body
    if in_loop && !loop_body.is_empty() {
        body.push(Stmt::While {
            cond: Expr::Literal(Value::U8(1)),
            body: loop_body,
        });
    }

    Ok(body)
}

/// Given a builtin name and the current ARG slot state, reconstruct the call arguments.
fn reconstruct_args(
    name: &str,
    arg_slots: &[Option<Expr>; 5],
    data_ptr_addr: Option<u16>,
    table: &SymbolTable,
    cpu_base: u16,
    prg_rom: &[u8],
) -> Vec<Expr> {
    use crate::builtins::signature;
    use crate::builtins::Param;

    let Some(sig) = signature(name) else { return vec![] };

    let mut args = Vec::new();
    let mut slot = 0usize; // ARG0-based index

    for param in sig.params {
        match param {
            Param::Data => {
                // load_palette(PAL): identify the const by address.
                if let Some(ptr_addr) = data_ptr_addr {
                    if let Some(const_name) = find_const_by_addr(ptr_addr, table, cpu_base, prg_rom) {
                        args.push(Expr::Var(const_name));
                    } else {
                        args.push(slot_or_unknown(arg_slots, slot));
                        slot += 1;
                    }
                } else {
                    args.push(slot_or_unknown(arg_slots, slot));
                    slot += 1;
                }
            }
            Param::Text => {
                // print(x, y, MSG): pointer in PTR ($04/$05), length in ARG2 (slot).
                // First check if it's a named const, otherwise read the raw bytes as a string.
                if let Some(ptr_addr) = data_ptr_addr {
                    if let Some(const_name) = find_const_by_addr(ptr_addr, table, cpu_base, prg_rom) {
                        args.push(Expr::Var(const_name));
                    } else {
                        let len = match arg_slots.get(slot).and_then(|e| e.as_ref()) {
                            Some(Expr::Literal(Value::U8(n))) => *n as usize,
                            _ => 0,
                        };
                        if let Some(s) = read_string_from_prg(ptr_addr, len, cpu_base, prg_rom) {
                            let bytes: Vec<Value> = s.bytes().map(Value::U8).collect();
                            args.push(Expr::Literal(Value::Array(bytes)));
                        } else {
                            args.push(slot_or_unknown(arg_slots, slot));
                            slot += 1;
                        }
                    }
                } else {
                    args.push(slot_or_unknown(arg_slots, slot));
                    slot += 1;
                }
            }
            Param::U8 => {
                args.push(slot_or_unknown(arg_slots, slot));
                slot += 1;
            }
            Param::U16 => {
                // U16 arg: lo byte in slot, hi byte in slot+1
                let lo = match arg_slots.get(slot).and_then(|e| e.as_ref()) {
                    Some(Expr::Literal(Value::U8(n))) => *n as u16,
                    _ => 0,
                };
                let hi = match arg_slots.get(slot + 1).and_then(|e| e.as_ref()) {
                    Some(Expr::Literal(Value::U8(n))) => *n as u16,
                    _ => 0,
                };
                args.push(Expr::Literal(Value::U16(lo | (hi << 8))));
                slot += 2;
            }
            Param::Map | Param::Meta | Param::Music => {
                // Resource args — pointer-based but not yet tracked; emit a placeholder.
                args.push(slot_or_unknown(arg_slots, slot));
                slot += 1;
            }
        }
    }

    args
}

fn slot_or_unknown(slots: &[Option<Expr>; 5], i: usize) -> Expr {
    slots.get(i).and_then(|e| e.clone()).unwrap_or(Expr::Literal(Value::U8(0)))
}

/// Finds the name of a constant whose ROM address matches `ptr_addr`.
fn find_const_by_addr(ptr_addr: u16, table: &SymbolTable, cpu_base: u16, prg_rom: &[u8]) -> Option<String> {
    let _ = (cpu_base, prg_rom);
    for c in &table.constants {
        if c.addr == ptr_addr {
            return Some(c.name.clone());
        }
    }
    None
}

/// Reads up to `len` bytes from PRG-ROM at the given CPU address and returns them as a
/// printable ASCII string if every byte is a printable ASCII character (0x20-0x7E).
/// Inline string literals are stored in PRG-ROM right where the pointer points.
fn read_string_from_prg(ptr_addr: u16, len: usize, cpu_base: u16, prg_rom: &[u8]) -> Option<String> {
    if len == 0 || len > 64 {
        return None;
    }
    let off = ptr_addr.checked_sub(cpu_base)? as usize;
    let bytes = prg_rom.get(off..off + len)?;
    let mut s = String::with_capacity(len);
    for &b in bytes {
        if !(0x20..=0x7E).contains(&b) {
            return None;
        }
        s.push(b as char);
    }
    Some(s)
}

/// Detects if an address is a builtin function based on the symbol table.
fn detect_builtin(addr: u16, table: &SymbolTable) -> Option<String> {
    // Check builtin table for known addresses
    for builtin in &table.builtins {
        if builtin.addr == addr {
            return Some(builtin.name.clone());
        }
    }
    None
}

/// Finds a user function by its address.
fn find_function_by_addr(addr: u16, table: &SymbolTable) -> Option<String> {
    for func in &table.functions {
        if func.addr == addr {
            return Some(func.name.clone());
        }
    }
    None
}
