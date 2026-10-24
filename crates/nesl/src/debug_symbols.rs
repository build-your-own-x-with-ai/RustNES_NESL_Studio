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
pub fn generate_symbols(_out: &crate::CompileOutput) -> SymbolTable {
    // Stub implementation - will be filled in Phase 2
    SymbolTable {
        version: "1.0.0".into(),
        compiler_version: env!("CARGO_PKG_VERSION").into(),
        mapper: 0,
        prg_size: 0,
        chr_size: 0,
        variables: Vec::new(),
        constants: Vec::new(),
        functions: Vec::new(),
        resources: Vec::new(),
        builtins: Vec::new(),
        source_map: Vec::new(),
    }
}
