//! AST reconstruction from symbols and ROM data.

use crate::debug_symbols::{SymbolTable, Type, Value};
use crate::decompile::DecompileError;

/// Reconstructed NESL AST (simplified representation).
#[derive(Debug, Clone)]
pub struct DecompiledAst {
    pub constants: Vec<ConstDecl>,
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

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub ty: Type,
    pub init: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct TileDecl {
    pub index: u8,
    pub pattern: Vec<String>,
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
pub fn build_ast(table: &SymbolTable, _rom: &[u8]) -> Result<DecompiledAst, DecompileError> {
    let mut constants = Vec::new();
    let mut variables = Vec::new();
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

    // Restore variables from symbol table
    for v in &table.variables {
        variables.push(VarDecl {
            name: v.name.clone(),
            ty: v.ty.clone(),
            init: v.init_value.clone(),
        });
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
        variables,
        tiles: Vec::new(), // Will be populated in US2
        functions,
    })
}
