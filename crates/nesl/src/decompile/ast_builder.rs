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
        // Skip constants with zero length (no data yet)
        if let Type::Array { length, .. } = &c.ty {
            if *length == 0 {
                continue;
            }
        }
        constants.push(ConstDecl {
            name: c.name.clone(),
            ty: c.ty.clone(),
            value: Value::Array(vec![]), // TODO: read from ROM in later phases
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

    // Restore function signatures (body will be empty stubs for now)
    for f in &table.functions {
        functions.push(FunctionDecl {
            name: f.name.clone(),
            params: f.params.clone(),
            return_ty: f.return_ty.clone(),
            body: vec![
                // Placeholder body - actual disassembly will come in later phases
                Stmt::Call {
                    func: "wait_vblank".to_string(),
                    args: vec![],
                }
            ],
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
