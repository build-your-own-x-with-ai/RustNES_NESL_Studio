//! NESL source code generation from decompiled AST.

use crate::decompile::ast_builder::{DecompiledAst, ConstDecl, StructDecl, VarDecl, TileDecl, FunctionDecl, Stmt, Expr};
use crate::debug_symbols::{Type, Value};
use std::fmt::Write;

/// Generates formatted NESL source code from a decompiled AST.
pub fn generate_source(ast: &DecompiledAst) -> String {
    let mut out = String::new();

    // Constants
    for c in &ast.constants {
        write_const(&mut out, c);
    }
    if !ast.constants.is_empty() {
        out.push('\n');
    }

    // Structs (before the variables that use them)
    for st in &ast.structs {
        write_struct(&mut out, st);
    }
    if !ast.structs.is_empty() {
        out.push('\n');
    }

    // Variables
    for v in &ast.variables {
        write_var(&mut out, v);
    }
    if !ast.variables.is_empty() {
        out.push('\n');
    }

    // Tiles
    for t in &ast.tiles {
        write_tile(&mut out, t);
    }
    if !ast.tiles.is_empty() {
        out.push('\n');
    }

    // Functions
    for f in &ast.functions {
        write_function(&mut out, f);
        out.push('\n');
    }

    out
}

fn write_const(out: &mut String, c: &ConstDecl) {
    let _ = write!(out, "const {}: ", c.name);
    write_type(out, &c.ty);
    let _ = write!(out, " = ");
    write_value(out, &c.value);
    out.push_str(";\n");
}

fn write_struct(out: &mut String, st: &StructDecl) {
    let _ = write!(out, "struct {} {{ ", st.name);
    for (i, (name, ty)) in st.fields.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}: ", name);
        write_type(out, ty);
    }
    out.push_str(" }\n");
}

fn write_var(out: &mut String, v: &VarDecl) {
    let _ = write!(out, "var {}: ", v.name);
    write_type(out, &v.ty);
    if let Some(init) = &v.init {
        let _ = write!(out, " = ");
        write_value(out, init);
    }
    out.push_str(";\n");
}

fn write_tile(out: &mut String, t: &TileDecl) {
    let _ = writeln!(out, "tile {} = [", t.index);
    for (i, row) in t.pattern.iter().enumerate() {
        let _ = write!(out, "    \"{}\"", row);
        if i < t.pattern.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("];\n");
}

fn write_function(out: &mut String, f: &FunctionDecl) {
    let _ = write!(out, "fn {}(", f.name);
    for (i, (name, ty)) in f.params.iter().enumerate() {
        let _ = write!(out, "{}: ", name);
        write_type(out, ty);
        if i < f.params.len() - 1 {
            out.push_str(", ");
        }
    }
    out.push(')');
    if let Some(ret) = &f.return_ty {
        out.push_str(" -> ");
        write_type(out, ret);
    }
    out.push_str(" {\n");
    for stmt in &f.body {
        write_stmt(out, stmt, 1);
    }
    out.push_str("}\n");
}

fn write_stmt(out: &mut String, stmt: &Stmt, indent: usize) {
    let ind = "    ".repeat(indent);
    match stmt {
        Stmt::VarDecl { name, ty, init } => {
            let _ = write!(out, "{}let {}", ind, name);
            if let Some(t) = ty {
                out.push_str(": ");
                write_type(out, t);
            }
            if let Some(e) = init {
                out.push_str(" = ");
                write_expr(out, e);
            }
            out.push_str(";\n");
        }
        Stmt::Assign { target, value } => {
            let _ = write!(out, "{}{} = ", ind, target);
            write_expr(out, value);
            out.push_str(";\n");
        }
        Stmt::Call { func, args } => {
            let _ = write!(out, "{}{}(", ind, func);
            for (i, arg) in args.iter().enumerate() {
                write_expr(out, arg);
                if i < args.len() - 1 {
                    out.push_str(", ");
                }
            }
            out.push_str(");\n");
        }
        Stmt::If { cond, then_block, else_block } => {
            let _ = write!(out, "{}if ", ind);
            write_expr(out, cond);
            out.push_str(" {\n");
            for s in then_block {
                write_stmt(out, s, indent + 1);
            }
            if let Some(else_b) = else_block {
                let _ = writeln!(out, "{}}} else {{", ind);
                for s in else_b {
                    write_stmt(out, s, indent + 1);
                }
            }
            let _ = writeln!(out, "{}}}", ind);
        }
        Stmt::While { cond, body } => {
            let _ = write!(out, "{}while ", ind);
            write_expr(out, cond);
            out.push_str(" {\n");
            for s in body {
                write_stmt(out, s, indent + 1);
            }
            let _ = writeln!(out, "{}}}", ind);
        }
        Stmt::Return { value } => {
            let _ = write!(out, "{}return", ind);
            if let Some(v) = value {
                out.push(' ');
                write_expr(out, v);
            }
            out.push_str(";\n");
        }
    }
}

fn write_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::Var(name) => {
            let _ = write!(out, "{}", name);
        }
        Expr::Literal(val) => {
            write_value(out, val);
        }
        Expr::BinOp { op, left, right } => {
            write_expr(out, left);
            let _ = write!(out, " {} ", op);
            write_expr(out, right);
        }
        Expr::Call { func, args } => {
            let _ = write!(out, "{}(", func);
            for (i, arg) in args.iter().enumerate() {
                write_expr(out, arg);
                if i < args.len() - 1 {
                    out.push_str(", ");
                }
            }
            out.push(')');
        }
    }
}

fn write_type(out: &mut String, ty: &Type) {
    match ty {
        Type::U8 => out.push_str("u8"),
        Type::U16 => out.push_str("u16"),
        Type::I8 => out.push_str("i8"),
        Type::Array { element_ty, length } => {
            out.push('[');
            write_type(out, element_ty);
            if *length > 0 {
                let _ = write!(out, "; {}]", length);
            } else {
                out.push(']');
            }
        }
        Type::Struct { name } => {
            let _ = write!(out, "{}", name);
        }
    }
}

fn write_value(out: &mut String, val: &Value) {
    // A Value::Array of printable ASCII U8 bytes represents an inline string literal
    // (e.g. from `print(x, y, "HELLO")`). Emit it as a quoted string.
    if let Value::Array(vals) = val {
        let is_string = !vals.is_empty() && vals.iter().all(|v| matches!(v, Value::U8(b) if (0x20..=0x7E).contains(b)));
        if is_string {
            out.push('"');
            for v in vals {
                if let Value::U8(b) = v {
                    // Escape backslash and double-quote
                    if *b == b'\\' || *b == b'"' { out.push('\\'); }
                    out.push(*b as char);
                }
            }
            out.push('"');
            return;
        }
    }
    match val {
        Value::U8(n) => {
            let _ = write!(out, "{}", n);
        }
        Value::U16(n) => {
            let _ = write!(out, "{}", n);
        }
        Value::I8(n) => {
            let _ = write!(out, "{}", n);
        }
        Value::Array(vals) => {
            out.push('[');
            for (i, v) in vals.iter().enumerate() {
                write_value(out, v);
                if i < vals.len() - 1 {
                    out.push_str(", ");
                }
            }
            out.push(']');
        }
    }
}
