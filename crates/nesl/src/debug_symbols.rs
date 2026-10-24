//! Debug symbol table generation for the NESL compiler.

use serde::{Deserialize, Serialize};

/// Type information for variables and parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Type {
    U8,
    U16,
    I8,
    Array { element_ty: Box<Type>, length: usize },
    Struct { name: String },
}

/// Runtime value representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    U8(u8),
    U16(u16),
    I8(i8),
    Array(Vec<Value>),
}

/// Variable declaration with address mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub ty: Type,
    pub addr: u16,
    pub init_value: Option<Value>,
}

/// Constant declaration with ROM data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub name: String,
    pub ty: Type,
    pub addr: u16,
    pub data: Vec<u8>,
}

/// Function signature and location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub addr: u16,
    pub params: Vec<(String, Type)>,
    pub return_ty: Option<Type>,
    pub local_vars: Vec<Variable>,
}

/// Resource type (tile, map, music, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Tile,
    Map { width: u8, height: u8 },
    Music,
    Metasprite,
    Sound,
}

/// Resource location in ROM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceLocation {
    ChrRom { offset: usize, length: usize },
    PrgRom { addr: u16, length: usize },
    Bank { bank: usize, addr: u16, length: usize },
}

/// Resource definition (tile, map, music, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub resource_ty: ResourceType,
    pub location: ResourceLocation,
}

/// Built-in function address mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builtin {
    pub name: String,
    pub addr: u16,
    pub signature: String,
}

/// Source code location mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub nes_addr: u16,
    pub source_line: usize,
    pub source_file: String,
}

/// Complete debug symbol table for a compiled ROM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    pub version: String,
    pub compiler_version: String,
    pub mapper: u8,
    pub prg_size: usize,
    pub chr_size: usize,
    pub variables: Vec<Variable>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub resources: Vec<Resource>,
    pub builtins: Vec<Builtin>,
    pub source_map: Vec<SourceMapEntry>,
}

/// Generates a debug symbol table from compiler output.
pub fn generate_symbols(out: &crate::CompileOutput) -> SymbolTable {
    let mut variables = Vec::new();
    let mut functions = Vec::new();
    let mut constants = Vec::new();

    // Extract variables from CompileOutput.vars
    for (name, info) in &out.vars {
        let elem = scalar(info.elem);
        let elem_size = if elem == Type::U16 { 2 } else { 1 };
        let ty = if info.size == elem_size {
            elem.clone()
        } else {
            Type::Array { element_ty: Box::new(elem.clone()), length: (info.size / elem_size) as usize }
        };
        let init_value = info.init.map(|val| match elem {
            Type::U16 => Value::U16(val),
            Type::I8 => Value::I8(val as u8 as i8),
            _ => Value::U8(val as u8),
        });
        variables.push(Variable {
            name: name.clone(),
            ty,
            addr: info.addr,
            init_value,
        });
    }

    // Extract functions and constants from CompileOutput.symbols
    for (name, addr) in &out.symbols {
        if let Some(func_name) = name.strip_prefix("f_") {
            functions.push(Function {
                name: func_name.to_string(),
                addr: *addr,
                params: Vec::new(), // TODO: extract from AST in later phases
                return_ty: None,
                local_vars: Vec::new(),
            });
        } else if let Some(const_name) = name.strip_prefix("d_") {
            let Some(info) = out.data.get(const_name) else { continue };
            let elem = scalar(info.elem);
            // u16 tables are stored as all low bytes, then all high bytes.
            let bytes = info.len as usize * if elem == Type::U16 { 2 } else { 1 };
            let data = prg_bytes(out, *addr, bytes);
            constants.push(Constant {
                name: const_name.to_string(),
                ty: Type::Array { element_ty: Box::new(elem), length: info.len as usize },
                addr: *addr,
                data,
            });
        }
    }

    // Calculate actual ROM sizes from the ROM structure
    // iNES format: 16-byte header + PRG-ROM + CHR-ROM
    let total_size = out.rom.len();
    let chr_size = if total_size >= 16 + 8192 {
        8192 // Standard CHR-ROM size for NROM
    } else {
        0
    };
    let prg_size = if total_size > 16 + chr_size {
        total_size - 16 - chr_size
    } else {
        0
    };

    // Extract builtin function addresses from the compiler's symbol table.
    // Every high-level builtin (wait_vblank, ppu_on, sprite, etc.) has a corresponding
    // `__name` label in the output symbol map.
    let builtins = crate::builtins::BUILTIN_NAMES
        .iter()
        .filter_map(|name| {
            let label = format!("__{name}");
            out.symbols.get(&label).map(|&addr| Builtin {
                name: name.to_string(),
                addr,
                signature: String::new(),
            })
        })
        .collect();

    SymbolTable {
        version: "1.0.0".into(),
        compiler_version: env!("CARGO_PKG_VERSION").into(),
        mapper: out.mapper,
        prg_size,
        chr_size,
        variables,
        constants,
        functions,
        resources: Vec::new(),
        builtins,
        source_map: Vec::new(),
    }
}

fn scalar(t: crate::ast::Type) -> Type {
    match t {
        crate::ast::Type::U8 => Type::U8,
        crate::ast::Type::I8 => Type::I8,
        crate::ast::Type::U16 => Type::U16,
    }
}

/// `len` bytes of the program at CPU address `addr`, for linearly mapped PRG (NROM, MMC3).
/// UNROM tables live in the fixed bank at $C000. Out-of-range reads come back empty.
fn prg_bytes(out: &crate::CompileOutput, addr: u16, len: usize) -> Vec<u8> {
    let base = if out.mapper == 2 { 0xC000 } else { 0x8000 };
    let prg_len = out.rom.get(4).map_or(0, |n| *n as usize * 0x4000);
    let prg_start = 16 + if out.mapper == 2 { prg_len - 0x4000 } else { 0 };
    let Some(off) = (addr as usize).checked_sub(base) else { return Vec::new() };
    out.rom.get(prg_start + off..prg_start + off + len).map(<[u8]>::to_vec).unwrap_or_default()
}
