//! Semantic analysis and 6502 code generation.
//!
//! Expression results live in A (u8) or A:X (u16, low:high). Parameters and locals
//! are statically allocated, which is why recursion is rejected up front.

use std::collections::{BTreeMap, HashMap};

use crate::asm::{Op::*, Operand, Val};
use crate::ast::*;
use crate::builtins;
use crate::runtime::{self, Emitter, TMP0, TMP1, TMP2, TMP3, USER_RAM_END, USER_RAM_START, USER_ZP_START};
use crate::{Diagnostic, VarInfo};

pub(crate) type R<T> = Result<T, Diagnostic>;

#[derive(Clone, Debug)]
pub(crate) enum Sym {
    Var { addr: u16, ty: Type },
    Array { addr: u16, len: u16 },
    Const(i64),
    Data { label: String, len: u16 },
}

struct FnSig {
    params: Vec<(Type, u16)>,
    ret: Option<Type>,
    label: String,
}

pub(crate) struct Tile {
    pub index: u8,
    pub rows: Vec<(String, Pos)>,
    pub pos: Pos,
}

pub struct Codegen {
    pub(crate) e: Emitter,
    globals: HashMap<String, Sym>,
    pub vars: BTreeMap<String, VarInfo>,
    funcs: HashMap<String, FnSig>,
    scopes: Vec<HashMap<String, Sym>>,
    /// (continue label, break label) for each enclosing loop.
    loops: Vec<(String, String)>,
    cur_ret: Option<Type>,
    zp_next: u16,
    ram_next: u16,
    pub errors: Vec<Diagnostic>,
    /// Read-only data blocks emitted after all code.
    pub(crate) data: Vec<(String, Vec<u8>)>,
    pub(crate) tiles: Vec<Tile>,
    inits: Vec<(u16, Type, u16)>,
}

fn max_ty(a: Type, b: Type) -> Type {
    if a == Type::U16 || b == Type::U16 {
        Type::U16
    } else {
        Type::U8
    }
}

fn type_max(t: Type) -> i64 {
    match t {
        Type::U8 | Type::I8 => 0xFF,
        Type::U16 => 0xFFFF,
    }
}

fn negate(op: BinOp) -> BinOp {
    use BinOp::*;
    match op {
        Eq => Ne, Ne => Eq, Lt => Ge, Ge => Lt, Gt => Le, Le => Gt,
        other => other,
    }
}

pub(crate) fn contains_call(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Call(..) => true,
        ExprKind::Index(_, i) => contains_call(i),
        ExprKind::Unary(_, x) | ExprKind::Cast(x, _) => contains_call(x),
        ExprKind::Binary(_, l, r) => contains_call(l) || contains_call(r),
        _ => false,
    }
}

/// Converts a constant to the bit pattern of `t`, rejecting values that do not fit.
pub(crate) fn fit(v: i64, t: Type, pos: Pos) -> R<u16> {
    let max = type_max(t);
    if v > max || v < -(max + 1) {
        return Err(Diagnostic::new(pos, format!("constant {v} does not fit in {t}")));
    }
    Ok((v & max) as u16)
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            e: Emitter::default(),
            globals: HashMap::new(),
            vars: BTreeMap::new(),
            funcs: HashMap::new(),
            scopes: Vec::new(),
            loops: Vec::new(),
            cur_ret: None,
            zp_next: USER_ZP_START,
            ram_next: USER_RAM_START,
            errors: Vec::new(),
            data: Vec::new(),
            tiles: Vec::new(),
            inits: Vec::new(),
        }
    }

    pub(crate) fn err<T>(&self, pos: Pos, msg: impl Into<String>) -> R<T> {
        Err(Diagnostic::new(pos, msg))
    }

    // ------------------------------------------------------------------ program

    pub fn program(&mut self, prog: &Program) {
        runtime::emit_core(&mut self.e);
        runtime::emit_math(&mut self.e);
        crate::builtins::emit_runtime(&mut self.e);

        for d in &prog.items {
            if let Err(e) = self.declare(d) {
                self.errors.push(e);
            }
        }
        match self.funcs.get("main") {
            None => self.errors.push(Diagnostic::new(Pos { line: 1, col: 1 }, "missing entry point `fn main()`")),
            Some(sig) if !sig.params.is_empty() || sig.ret.is_some() => {
                let pos = prog.items.iter().find_map(|d| match d {
                    Decl::Fn(f) if f.name == "main" => Some(f.pos),
                    _ => None,
                });
                self.errors.push(Diagnostic::new(pos.unwrap_or_default(), "`main` must take no parameters and return nothing"));
            }
            _ => {}
        }
        if let Err(e) = self.check_recursion(prog) {
            self.errors.push(e);
        }

        self.e.comment("---- global initializers ----");
        self.e.label("__init_globals");
        for (addr, ty, v) in std::mem::take(&mut self.inits) {
            self.e.imm(LDA, v as u8);
            self.e.abs(STA, addr);
            if ty == Type::U16 {
                self.e.imm(LDA, (v >> 8) as u8);
                self.e.abs(STA, addr + 1);
            }
        }
        self.e.imp(RTS);

        for d in &prog.items {
            if let Decl::Fn(f) = d {
                self.function(f);
            }
        }

        self.e.comment("---- read-only data ----");
        for (label, bytes) in std::mem::take(&mut self.data) {
            self.e.label(&label);
            self.e.items.push(crate::asm::Item::Bytes(bytes));
        }
    }

    fn define(&mut self, name: &str, pos: Pos) -> R<()> {
        if self.globals.contains_key(name) || self.funcs.contains_key(name) {
            return self.err(pos, format!("`{name}` is already defined"));
        }
        if builtins::constant(name).is_some() || builtins::is_builtin_fn(name) {
            return self.err(pos, format!("`{name}` is already defined as a built-in"));
        }
        Ok(())
    }

    fn alloc(&mut self, size: u16, zero_page: bool, pos: Pos) -> R<u16> {
        if zero_page && self.zp_next + size <= 0x100 {
            let a = self.zp_next;
            self.zp_next += size;
            return Ok(a);
        }
        if self.ram_next + size > USER_RAM_END {
            return self.err(pos, "out of RAM: too many variables (2KB NES RAM)");
        }
        let a = self.ram_next;
        self.ram_next += size;
        Ok(a)
    }

    fn const_expr(&self, e: &Expr, what: &str) -> R<i64> {
        self.const_val(e).ok_or_else(|| Diagnostic::new(e.pos, format!("{what} must be a constant expression")))
    }

    fn declare(&mut self, d: &Decl) -> R<()> {
        match d {
            Decl::Const { name, value, pos } => {
                self.define(name, *pos)?;
                let v = self.const_expr(value, "constant value")?;
                self.globals.insert(name.clone(), Sym::Const(v));
            }
            Decl::Data { name, elems, pos, .. } => {
                self.define(name, *pos)?;
                let mut bytes = Vec::with_capacity(elems.len());
                for el in elems {
                    bytes.push(fit(self.const_expr(el, "array element")?, Type::U8, el.pos)? as u8);
                }
                if bytes.is_empty() || bytes.len() > 0xFFFF {
                    return self.err(*pos, "constant array must have between 1 and 65535 elements");
                }
                let label = format!("d_{name}");
                self.globals.insert(name.clone(), Sym::Data { label: label.clone(), len: bytes.len() as u16 });
                self.data.push((label, bytes));
            }
            Decl::Struct { pos, .. } => return self.err(*pos, "not implemented yet"),
            Decl::Var { name, ty, init, pos } => {
                self.define(name, *pos)?;
                match ty {
                    VarType::Struct(_) | VarType::StructArray(..) => {
                        return self.err(*pos, "not implemented yet");
                    }
                    VarType::Scalar(t) => {
                        let size = if *t == Type::U16 { 2 } else { 1 };
                        let addr = self.alloc(size, true, *pos)?;
                        if let Some(init) = init {
                            let v = fit(self.const_expr(init, "global initializer")?, *t, init.pos)?;
                            if v != 0 {
                                self.inits.push((addr, *t, v));
                            }
                        }
                        self.globals.insert(name.clone(), Sym::Var { addr, ty: *t });
                        self.vars.insert(name.clone(), VarInfo { addr, size });
                    }
                    VarType::Array(_, len) => {
                        let n = self.const_expr(len, "array length")?;
                        if !(1..=256).contains(&n) {
                            return self.err(len.pos, "array length must be between 1 and 256");
                        }
                        if init.is_some() {
                            return self.err(*pos, "RAM arrays cannot have an initializer (they start zeroed)");
                        }
                        let addr = self.alloc(n as u16, false, *pos)?;
                        self.globals.insert(name.clone(), Sym::Array { addr, len: n as u16 });
                        self.vars.insert(name.clone(), VarInfo { addr, size: n as u16 });
                    }
                }
            }
            Decl::Tile { index, rows, pos } => {
                let idx = self.const_expr(index, "tile index")?;
                if !(0..=255).contains(&idx) {
                    return self.err(index.pos, "tile index must be between 0 and 255");
                }
                self.tiles.push(Tile { index: idx as u8, rows: rows.clone(), pos: *pos });
            }
            Decl::Fn(f) => {
                self.define(&f.name, f.pos)?;
                let mut params = Vec::new();
                for (_, t) in &f.params {
                    let size = if *t == Type::U16 { 2 } else { 1 };
                    params.push((*t, self.alloc(size, true, f.pos)?));
                }
                self.funcs.insert(f.name.clone(), FnSig { params, ret: f.ret, label: format!("f_{}", f.name) });
            }
        }
        Ok(())
    }

    fn check_recursion(&self, prog: &Program) -> R<()> {
        fn calls(b: &Block, out: &mut Vec<String>) {
            fn ex(e: &Expr, out: &mut Vec<String>) {
                match &e.kind {
                    ExprKind::Call(n, args) => {
                        out.push(n.clone());
                        args.iter().for_each(|a| ex(a, out));
                    }
                    ExprKind::Index(_, i) => ex(i, out),
                    ExprKind::Unary(_, x) | ExprKind::Cast(x, _) => ex(x, out),
                    ExprKind::Binary(_, l, r) => {
                        ex(l, out);
                        ex(r, out);
                    }
                    _ => {}
                }
            }
            for s in b {
                match s {
                    Stmt::Let { init, .. } => ex(init, out),
                    Stmt::Assign { target, value, .. } => {
                        if let Place::Index(_, i, _) = target {
                            ex(i, out);
                        }
                        ex(value, out);
                    }
                    Stmt::If { cond, then, els, .. } => {
                        ex(cond, out);
                        calls(then, out);
                        if let Some(e) = els {
                            calls(e, out);
                        }
                    }
                    Stmt::While { cond, body, .. } => {
                        ex(cond, out);
                        calls(body, out);
                    }
                    Stmt::Loop { body, .. } => calls(body, out),
                    Stmt::For { start, end, body, .. } => {
                        ex(start, out);
                        ex(end, out);
                        calls(body, out);
                    }
                    Stmt::Return(Some(e), _) | Stmt::Expr(e) => ex(e, out),
                    _ => {}
                }
            }
        }
        let mut graph: BTreeMap<&str, (Vec<String>, Pos)> = BTreeMap::new();
        for d in &prog.items {
            if let Decl::Fn(f) = d {
                let mut out = Vec::new();
                calls(&f.body, &mut out);
                graph.insert(&f.name, (out, f.pos));
            }
        }
        // 0 = unvisited, 1 = on stack, 2 = done
        fn dfs<'a>(
            n: &'a str,
            g: &'a BTreeMap<&'a str, (Vec<String>, Pos)>,
            state: &mut HashMap<&'a str, u8>,
            path: &mut Vec<&'a str>,
        ) -> Option<Vec<&'a str>> {
            state.insert(n, 1);
            path.push(n);
            for c in &g[n].0 {
                let Some((key, _)) = g.get_key_value(c.as_str()) else { continue };
                match state.get(key).copied().unwrap_or(0) {
                    1 => {
                        let start = path.iter().position(|p| p == key).unwrap();
                        let mut cycle = path[start..].to_vec();
                        cycle.push(key);
                        return Some(cycle);
                    }
                    0 => {
                        if let Some(c) = dfs(key, g, state, path) {
                            return Some(c);
                        }
                    }
                    _ => {}
                }
            }
            path.pop();
            state.insert(n, 2);
            None
        }
        let mut state = HashMap::new();
        for n in graph.keys() {
            if state.get(n).copied().unwrap_or(0) == 0 {
                if let Some(cycle) = dfs(n, &graph, &mut state, &mut Vec::new()) {
                    return self.err(
                        graph[cycle[0]].1,
                        format!("recursion is not supported: {}", cycle.join(" -> ")),
                    );
                }
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------ lookup & types

    pub(crate) fn lookup(&self, name: &str) -> Option<Sym> {
        for s in self.scopes.iter().rev() {
            if let Some(sym) = s.get(name) {
                return Some(sym.clone());
            }
        }
        if let Some(s) = self.globals.get(name) {
            return Some(s.clone());
        }
        builtins::constant(name).map(|v| Sym::Const(v as i64))
    }

    pub(crate) fn const_val(&self, e: &Expr) -> Option<i64> {
        use BinOp::*;
        match &e.kind {
            ExprKind::Num(n) => Some(*n as i64),
            ExprKind::Var(name) => match self.lookup(name) {
                Some(Sym::Const(v)) => Some(v),
                _ => None,
            },
            ExprKind::Unary(op, x) => {
                let v = self.const_val(x)?;
                Some(match op {
                    UnOp::Neg => -v,
                    UnOp::Not => (v == 0) as i64,
                    UnOp::BitNot => !v,
                })
            }
            ExprKind::Cast(x, t) => Some(self.const_val(x)? & type_max(*t)),
            ExprKind::Binary(op, l, r) => {
                let a = self.const_val(l)?;
                let b = self.const_val(r)?;
                Some(match op {
                    Add => a + b,
                    Sub => a - b,
                    Mul => a.checked_mul(b)?,
                    Div => a.checked_div(b)?,
                    Mod => a.checked_rem(b)?,
                    And => a & b,
                    Or => a | b,
                    Xor => a ^ b,
                    Shl => a.checked_shl(u32::try_from(b).ok().filter(|s| *s < 32)?)?,
                    Shr => a.checked_shr(u32::try_from(b).ok().filter(|s| *s < 64)?)?,
                    Eq => (a == b) as i64,
                    Ne => (a != b) as i64,
                    Lt => (a < b) as i64,
                    Le => (a <= b) as i64,
                    Gt => (a > b) as i64,
                    Ge => (a >= b) as i64,
                    LogAnd => (a != 0 && b != 0) as i64,
                    LogOr => (a != 0 || b != 0) as i64,
                })
            }
            _ => None,
        }
    }

    fn unknown_var(&self, name: &str, pos: Pos) -> Diagnostic {
        Diagnostic::new(pos, format!("unknown variable `{name}`"))
    }

    pub(crate) fn call_ret(&self, name: &str, pos: Pos) -> R<Option<Type>> {
        if let Some(sig) = builtins::signature(name) {
            return Ok(sig.ret);
        }
        match self.funcs.get(name) {
            Some(f) => Ok(f.ret),
            None => self.err(pos, format!("unknown function `{name}`")),
        }
    }

    /// The natural type of an expression.
    pub(crate) fn ty(&self, e: &Expr) -> R<Type> {
        if let Some(v) = self.const_val(e) {
            if v > 0xFFFF {
                return self.err(e.pos, format!("constant {v} does not fit in u16"));
            }
            return Ok(if v > 0xFF { Type::U16 } else { Type::U8 });
        }
        match &e.kind {
            ExprKind::Num(_) => unreachable!("numbers are constant"),
            ExprKind::Str(_) => self.err(e.pos, "string literals can only be used in constant arrays and `print`"),
            ExprKind::Var(name) => match self.lookup(name) {
                Some(Sym::Var { ty, .. }) => Ok(ty),
                Some(Sym::Array { .. }) | Some(Sym::Data { .. }) => {
                    self.err(e.pos, format!("`{name}` is an array; use `{name}[i]` to read an element"))
                }
                Some(Sym::Const(_)) => unreachable!(),
                None => Err(self.unknown_var(name, e.pos)),
            },
            ExprKind::Index(name, idx) => {
                self.array_base(name, e.pos)?;
                if self.ty(idx)? != Type::U8 {
                    return self.err(idx.pos, "array index must be u8");
                }
                Ok(Type::U8)
            }
            ExprKind::Call(name, _) => match self.call_ret(name, e.pos)? {
                Some(t) => Ok(t),
                None => self.err(e.pos, format!("function `{name}` does not return a value")),
            },
            ExprKind::Unary(UnOp::Not, x) => {
                self.ty(x)?;
                Ok(Type::U8)
            }
            ExprKind::Unary(_, x) => self.ty(x),
            ExprKind::Binary(op, l, r) => {
                let (a, b) = (self.ty(l)?, self.ty(r)?);
                if op.is_comparison() || matches!(op, BinOp::LogAnd | BinOp::LogOr) {
                    Ok(Type::U8)
                } else {
                    Ok(max_ty(a, b))
                }
            }
            ExprKind::Cast(x, t) => {
                self.ty(x)?;
                Ok(*t)
            }
        }
    }

    /// Checks that `e` can be stored into a `want`-typed location.
    pub(crate) fn check_fits(&self, e: &Expr, want: Type) -> R<()> {
        if want == Type::U8 && self.ty(e)? == Type::U16 {
            return self.err(e.pos, "mismatched types: expected u8, found u16 (use `as u8` to truncate)");
        }
        Ok(())
    }

    fn array_base(&self, name: &str, pos: Pos) -> R<(Val, u16)> {
        match self.lookup(name) {
            Some(Sym::Array { addr, len }) => Ok((Val::Num(addr), len)),
            Some(Sym::Data { label, len }) => Ok((Val::Label(label, 0), len)),
            Some(_) => self.err(pos, format!("`{name}` is not an array")),
            None => Err(self.unknown_var(name, pos)),
        }
    }

    fn offset(base: &Val, i: u16) -> Val {
        match base {
            Val::Num(a) => Val::Num(a + i),
            Val::Label(l, o) => Val::Label(l.clone(), o + i as i32),
        }
    }

    fn const_index(&self, name: &str, idx: &Expr, pos: Pos) -> R<Option<Val>> {
        let Some(i) = self.const_val(idx) else { return Ok(None) };
        let (base, len) = self.array_base(name, pos)?;
        if i < 0 || i >= len as i64 {
            return self.err(idx.pos, format!("index {i} is out of bounds for `{name}` (length {len})"));
        }
        Ok(Some(Self::offset(&base, i as u16)))
    }

    /// A directly addressable u8 operand for `e`, if it is that simple.
    pub(crate) fn operand8(&self, e: &Expr) -> R<Option<Operand>> {
        if let Some(v) = self.const_val(e) {
            return Ok(Some(Operand::imm(fit(v, Type::U8, e.pos)? as u8)));
        }
        Ok(match &e.kind {
            ExprKind::Var(name) => match self.lookup(name) {
                Some(Sym::Var { addr, ty: Type::U8 }) => Some(Operand::abs(addr)),
                _ => None,
            },
            ExprKind::Index(name, idx) => self.const_index(name, idx, e.pos)?.map(Operand::Abs),
            _ => None,
        })
    }

    /// Directly addressable (low, high) operands for `e` widened to u16.
    fn operand16(&self, e: &Expr) -> R<Option<(Operand, Operand)>> {
        if let Some(v) = self.const_val(e) {
            let v = fit(v, Type::U16, e.pos)?;
            return Ok(Some((Operand::imm(v as u8), Operand::imm((v >> 8) as u8))));
        }
        if let ExprKind::Var(name) = &e.kind {
            if let Some(Sym::Var { addr, ty: Type::U16 }) = self.lookup(name) {
                return Ok(Some((Operand::abs(addr), Operand::abs(addr + 1))));
            }
        }
        Ok(self.operand8(e)?.map(|o| (o, Operand::imm(0))))
    }

    // ------------------------------------------------------------------ expressions

    fn widen(&mut self, from: Type, want: Type) {
        if from == Type::U8 && want == Type::U16 {
            self.e.imm(LDX, 0);
        }
    }

    pub(crate) fn store(&mut self, addr: u16, ty: Type) {
        self.e.abs(STA, addr);
        if ty == Type::U16 {
            self.e.abs(STX, addr + 1);
        }
    }

    /// Evaluates `e` into A (u8) or A:X (u16).
    pub(crate) fn expr(&mut self, e: &Expr, want: Type) -> R<()> {
        if let Some(v) = self.const_val(e) {
            let v = fit(v, want, e.pos)?;
            self.e.imm(LDA, v as u8);
            if want == Type::U16 {
                self.e.imm(LDX, (v >> 8) as u8);
            }
            return Ok(());
        }
        let nat = self.ty(e)?;
        if nat == Type::U16 && want == Type::U8 {
            return self.check_fits(e, want);
        }
        match &e.kind {
            ExprKind::Var(name) => {
                let Some(Sym::Var { addr, ty }) = self.lookup(name) else { unreachable!() };
                self.e.abs(LDA, addr);
                if ty == Type::U16 {
                    self.e.abs(LDX, addr + 1);
                } else {
                    self.widen(ty, want);
                }
            }
            ExprKind::Index(name, idx) => {
                self.index(name, idx, e.pos)?;
                self.widen(Type::U8, want);
            }
            ExprKind::Call(name, args) => {
                let ret = self.call(name, args, e.pos)?;
                self.widen(ret.expect("checked by ty()"), want);
            }
            ExprKind::Unary(UnOp::Not, _) | ExprKind::Binary(BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::LogAnd | BinOp::LogOr, ..) => {
                let f = self.e.new_label();
                let end = self.e.new_label();
                self.cond(e, &f, false)?;
                self.e.imm(LDA, 1);
                self.e.jmp(&end);
                self.e.label(&f);
                self.e.imm(LDA, 0);
                self.e.label(&end);
                self.widen(Type::U8, want);
            }
            ExprKind::Unary(op, x) => {
                self.expr(x, want)?;
                let neg = *op == UnOp::Neg;
                self.e.imm(EOR, 0xFF);
                if neg {
                    self.e.imp(CLC);
                    self.e.imm(ADC, 1);
                }
                if want == Type::U16 {
                    self.e.imp(TAY);
                    self.e.imp(TXA);
                    self.e.imm(EOR, 0xFF);
                    if neg {
                        self.e.imm(ADC, 0);
                    }
                    self.e.imp(TAX);
                    self.e.imp(TYA);
                }
            }
            ExprKind::Binary(op, l, r) => self.arith(*op, l, r, want)?,
            ExprKind::Cast(x, t) => {
                let inner = self.ty(x)?;
                self.expr(x, inner)?;
                if want == Type::U16 && (inner == Type::U8 || *t == Type::U8) {
                    self.e.imm(LDX, 0);
                }
            }
            ExprKind::Num(_) | ExprKind::Str(_) => unreachable!("handled by const_val / ty"),
        }
        Ok(())
    }

    /// Loads `name[idx]` into A.
    fn index(&mut self, name: &str, idx: &Expr, pos: Pos) -> R<()> {
        if let Some(v) = self.const_index(name, idx, pos)? {
            self.e.ins(LDA, Operand::Abs(v));
            return Ok(());
        }
        let (base, _) = self.array_base(name, pos)?;
        self.index_to_x(idx)?;
        self.e.ins(LDA, Operand::AbsX(base));
        Ok(())
    }

    fn index_to_x(&mut self, idx: &Expr) -> R<()> {
        if self.ty(idx)? != Type::U8 {
            return self.err(idx.pos, "array index must be u8");
        }
        match self.operand8(idx)? {
            Some(o) => self.e.ins(LDX, o),
            None => {
                self.expr(idx, Type::U8)?;
                self.e.imp(TAX);
            }
        }
        Ok(())
    }

    fn arith(&mut self, op: BinOp, l: &Expr, r: &Expr, want: Type) -> R<()> {
        use BinOp::*;
        let pow2 = self.const_val(r).filter(|v| *v > 0 && v & (v - 1) == 0).map(|v| v.trailing_zeros());
        match (op, want) {
            (Add | Sub | And | Or | Xor, Type::U8) => {
                let opc = match op { Add => ADC, Sub => SBC, And => AND, Or => ORA, _ => EOR };
                let rhs = match self.operand8(r)? {
                    Some(o) => {
                        self.expr(l, Type::U8)?;
                        o
                    }
                    None => {
                        self.expr(l, Type::U8)?;
                        self.e.imp(PHA);
                        self.expr(r, Type::U8)?;
                        self.e.abs(STA, TMP0);
                        self.e.imp(PLA);
                        Operand::abs(TMP0)
                    }
                };
                match op {
                    Add => self.e.imp(CLC),
                    Sub => self.e.imp(SEC),
                    _ => {}
                }
                self.e.ins(opc, rhs);
            }
            (Add | Sub | And | Or | Xor, Type::U16) => {
                let (lo, hi) = match self.operand16(r)? {
                    Some(ops) => {
                        self.expr(l, Type::U16)?;
                        ops
                    }
                    None => {
                        self.expr(l, Type::U16)?;
                        self.e.imp(PHA);
                        self.e.imp(TXA);
                        self.e.imp(PHA);
                        self.expr(r, Type::U16)?;
                        self.e.abs(STA, TMP0);
                        self.e.abs(STX, TMP1);
                        self.e.imp(PLA);
                        self.e.imp(TAX);
                        self.e.imp(PLA);
                        (Operand::abs(TMP0), Operand::abs(TMP1))
                    }
                };
                let opc = match op { Add => ADC, Sub => SBC, And => AND, Or => ORA, _ => EOR };
                match op {
                    Add => self.e.imp(CLC),
                    Sub => self.e.imp(SEC),
                    _ => {}
                }
                self.e.ins(opc, lo);
                self.e.imp(TAY);
                self.e.imp(TXA);
                self.e.ins(opc, hi);
                self.e.imp(TAX);
                self.e.imp(TYA);
            }
            (Shl | Shr, _) => {
                if let Some(n) = self.const_val(r) {
                    self.expr(l, want)?;
                    self.shift_const(op == Shl, n, want);
                } else {
                    if self.ty(r)? != Type::U8 {
                        return self.err(r.pos, "shift amount must be u8");
                    }
                    self.shift_var(op == Shl, l, r, want)?;
                }
            }
            (Mul, _) if pow2.is_some() => {
                self.expr(l, want)?;
                self.shift_const(true, pow2.unwrap() as i64, want);
            }
            (Div, _) if pow2.is_some() => {
                self.expr(l, want)?;
                self.shift_const(false, pow2.unwrap() as i64, want);
            }
            (Mod, _) if pow2.is_some() => {
                let mask = (1i64 << pow2.unwrap()) - 1;
                self.expr(l, want)?;
                self.e.imm(AND, mask as u8);
                if want == Type::U16 {
                    self.e.imp(TAY);
                    self.e.imp(TXA);
                    self.e.imm(AND, (mask >> 8) as u8);
                    self.e.imp(TAX);
                    self.e.imp(TYA);
                }
            }
            (Mul | Div | Mod, Type::U16) => {
                return self.err(
                    l.pos,
                    format!("`{}` is not supported for u16 values (only u8, or a power-of-two constant)", op.symbol()),
                );
            }
            (Mul | Div | Mod, Type::U8) => {
                self.expr(l, Type::U8)?;
                self.e.imp(PHA);
                self.expr(r, Type::U8)?;
                self.e.abs(STA, TMP1);
                self.e.imp(PLA);
                self.e.abs(STA, TMP0);
                self.e.jsr(if op == Mul { "__mul8" } else { "__div8" });
                if op == Mod {
                    self.e.abs(LDA, TMP2);
                }
            }
            _ => unreachable!("comparison and logic handled by cond"),
        }
        Ok(())
    }

    fn shift_const(&mut self, left: bool, n: i64, ty: Type) {
        let bits = if ty == Type::U16 { 16 } else { 8 };
        if n >= bits {
            self.e.imm(LDA, 0);
            if ty == Type::U16 {
                self.e.imm(LDX, 0);
            }
            return;
        }
        if ty == Type::U8 {
            for _ in 0..n {
                self.e.acc(if left { ASL } else { LSR });
            }
            return;
        }
        if n == 0 {
            return;
        }
        self.e.abs(STX, TMP1);
        for _ in 0..n {
            if left {
                self.e.acc(ASL);
                self.e.abs(ROL, TMP1);
            } else {
                self.e.abs(LSR, TMP1);
                self.e.acc(ROR);
            }
        }
        self.e.abs(LDX, TMP1);
    }

    fn shift_var(&mut self, left: bool, l: &Expr, r: &Expr, ty: Type) -> R<()> {
        let top = self.e.new_label();
        let done = self.e.new_label();
        self.expr(l, ty)?;
        self.e.imp(PHA);
        if ty == Type::U16 {
            self.e.imp(TXA);
            self.e.imp(PHA);
        }
        self.expr(r, Type::U8)?;
        self.e.imp(TAY);
        if ty == Type::U16 {
            self.e.imp(PLA);
            self.e.abs(STA, TMP1);
        }
        self.e.imp(PLA);
        self.e.imp(INY);
        self.e.label(&top);
        self.e.imp(DEY);
        self.e.br(BEQ, &done);
        match (left, ty) {
            (true, Type::U8 | Type::I8) => self.e.acc(ASL),
            (false, Type::U8 | Type::I8) => self.e.acc(LSR),
            (true, Type::U16) => {
                self.e.acc(ASL);
                self.e.abs(ROL, TMP1);
            }
            (false, Type::U16) => {
                self.e.abs(LSR, TMP1);
                self.e.acc(ROR);
            }
        }
        self.e.jmp(&top);
        self.e.label(&done);
        if ty == Type::U16 {
            self.e.abs(LDX, TMP1);
        }
        Ok(())
    }

    /// Emits a jump to `label` taken when the truth of `e` equals `jump_if`.
    pub(crate) fn cond(&mut self, e: &Expr, label: &str, jump_if: bool) -> R<()> {
        if let Some(v) = self.const_val(e) {
            if (v != 0) == jump_if {
                self.e.jmp(label);
            }
            return Ok(());
        }
        match &e.kind {
            ExprKind::Unary(UnOp::Not, x) => self.cond(x, label, !jump_if),
            ExprKind::Binary(op @ (BinOp::LogAnd | BinOp::LogOr), a, b) => {
                // `a && b` is false if either is false; `a || b` is true if either is true.
                let short_circuits_to_label = (*op == BinOp::LogAnd) != jump_if;
                if short_circuits_to_label {
                    self.cond(a, label, jump_if)?;
                    self.cond(b, label, jump_if)
                } else {
                    let skip = self.e.new_label();
                    self.cond(a, &skip, !jump_if)?;
                    self.cond(b, label, jump_if)?;
                    self.e.label(&skip);
                    Ok(())
                }
            }
            ExprKind::Binary(op, l, r) if op.is_comparison() => {
                let op = if jump_if { *op } else { negate(*op) };
                self.compare(op, l, r, label)
            }
            _ => {
                let t = self.ty(e)?;
                self.expr(e, t)?;
                if t == Type::U16 {
                    self.e.abs(STX, TMP0);
                    self.e.abs(ORA, TMP0);
                } else {
                    self.e.imm(CMP, 0);
                }
                self.e.br(if jump_if { BNE } else { BEQ }, label);
                Ok(())
            }
        }
    }

    /// Jumps to `label` if `l op r` holds.
    fn compare(&mut self, op: BinOp, l: &Expr, r: &Expr, label: &str) -> R<()> {
        use BinOp::*;
        let t = max_ty(self.ty(l)?, self.ty(r)?);
        if t == Type::U8 {
            match self.operand8(r)? {
                Some(o) => {
                    self.expr(l, Type::U8)?;
                    self.e.ins(CMP, o);
                }
                None => {
                    self.expr(l, Type::U8)?;
                    self.e.imp(PHA);
                    self.expr(r, Type::U8)?;
                    self.e.abs(STA, TMP0);
                    self.e.imp(PLA);
                    self.e.abs(CMP, TMP0);
                }
            }
            match op {
                Eq => self.e.br(BEQ, label),
                Ne => self.e.br(BNE, label),
                Lt => self.e.br(BCC, label),
                Ge => self.e.br(BCS, label),
                Gt => {
                    let skip = self.e.new_label();
                    self.e.br(BEQ, &skip);
                    self.e.br(BCS, label);
                    self.e.label(&skip);
                }
                Le => {
                    self.e.br(BCC, label);
                    self.e.br(BEQ, label);
                }
                _ => unreachable!(),
            }
            return Ok(());
        }
        let (lo, hi) = match self.operand16(r)? {
            Some(ops) => {
                self.expr(l, Type::U16)?;
                self.e.abs(STA, TMP2);
                self.e.abs(STX, TMP3);
                ops
            }
            None => {
                self.expr(l, Type::U16)?;
                self.e.imp(PHA);
                self.e.imp(TXA);
                self.e.imp(PHA);
                self.expr(r, Type::U16)?;
                self.e.abs(STA, TMP0);
                self.e.abs(STX, TMP1);
                self.e.imp(PLA);
                self.e.abs(STA, TMP3);
                self.e.imp(PLA);
                self.e.abs(STA, TMP2);
                (Operand::abs(TMP0), Operand::abs(TMP1))
            }
        };
        match op {
            Eq | Ne => {
                let skip = self.e.new_label();
                self.e.abs(LDA, TMP2);
                self.e.ins(CMP, lo);
                self.e.br(BNE, &skip);
                self.e.abs(LDA, TMP3);
                self.e.ins(CMP, hi);
                self.e.label(&skip);
                self.e.br(if op == Eq { BEQ } else { BNE }, label);
            }
            Lt | Ge => {
                self.e.abs(LDA, TMP2);
                self.e.ins(CMP, lo);
                self.e.abs(LDA, TMP3);
                self.e.ins(SBC, hi);
                self.e.br(if op == Lt { BCC } else { BCS }, label);
            }
            Gt | Le => {
                // l > r  <=>  r < l
                self.e.ins(LDA, lo);
                self.e.abs(CMP, TMP2);
                self.e.ins(LDA, hi);
                self.e.abs(SBC, TMP3);
                self.e.br(if op == Gt { BCC } else { BCS }, label);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Emits a call; returns the callee's return type.
    pub(crate) fn call(&mut self, name: &str, args: &[Expr], pos: Pos) -> R<Option<Type>> {
        if builtins::is_builtin_fn(name) {
            return self.builtin_call(name, args, pos);
        }
        let Some(sig) = self.funcs.get(name) else {
            return self.err(pos, format!("unknown function `{name}`"));
        };
        let (params, ret, label) = (sig.params.clone(), sig.ret, sig.label.clone());
        if args.len() != params.len() {
            let s = if params.len() == 1 { "" } else { "s" };
            return self.err(pos, format!("function `{name}` expects {} argument{s}, found {}", params.len(), args.len()));
        }
        for (a, (t, _)) in args.iter().zip(&params) {
            self.check_fits(a, *t)?;
        }
        // Later arguments containing calls could clobber parameter slots that were
        // already written, so stage everything on the stack in that case.
        if args.iter().skip(1).any(contains_call) {
            for (a, (t, _)) in args.iter().zip(&params) {
                self.expr(a, *t)?;
                self.e.imp(PHA);
                if *t == Type::U16 {
                    self.e.imp(TXA);
                    self.e.imp(PHA);
                }
            }
            for (t, addr) in params.iter().rev() {
                if *t == Type::U16 {
                    self.e.imp(PLA);
                    self.e.abs(STA, addr + 1);
                }
                self.e.imp(PLA);
                self.e.abs(STA, *addr);
            }
        } else {
            for (a, (t, addr)) in args.iter().zip(&params) {
                self.expr(a, *t)?;
                self.store(*addr, *t);
            }
        }
        self.e.jsr(&label);
        Ok(ret)
    }

    // ------------------------------------------------------------------ statements

    fn function(&mut self, f: &Function) {
        // Functions whose declaration failed (e.g. name clash) were already reported.
        let Some(sig) = self.funcs.get(&f.name) else { return };
        let mut scope = HashMap::new();
        for ((name, ty), (_, addr)) in f.params.iter().zip(&sig.params) {
            scope.insert(name.clone(), Sym::Var { addr: *addr, ty: *ty });
        }
        let label = sig.label.clone();
        self.cur_ret = f.ret;
        self.e.comment(format!("---- fn {} ----", f.name));
        self.e.label(&label);
        self.scopes = vec![scope];
        self.block(&f.body);
        self.scopes.clear();
        self.e.imp(RTS);
    }

    fn block(&mut self, b: &Block) {
        self.scopes.push(HashMap::new());
        for s in b {
            if let Err(d) = self.stmt(s) {
                self.errors.push(d);
            }
        }
        self.scopes.pop();
    }

    fn declare_local(&mut self, name: &str, ty: Type, pos: Pos) -> R<u16> {
        let addr = self.alloc(if ty == Type::U16 { 2 } else { 1 }, true, pos)?;
        self.scopes.last_mut().unwrap().insert(name.to_string(), Sym::Var { addr, ty });
        Ok(addr)
    }

    fn stmt(&mut self, s: &Stmt) -> R<()> {
        match s {
            Stmt::Let { name, ty, init, pos } => {
                let ty = match ty {
                    Some(t) => {
                        self.check_fits(init, *t)?;
                        *t
                    }
                    None => self.ty(init)?,
                };
                self.expr(init, ty)?;
                let addr = self.declare_local(name, ty, *pos)?;
                self.store(addr, ty);
            }
            Stmt::Assign { target: Place::Var(name, vpos), op, value, .. } => {
                let (addr, ty) = match self.lookup(name) {
                    Some(Sym::Var { addr, ty }) => (addr, ty),
                    Some(Sym::Const(_)) => return self.err(*vpos, format!("cannot assign to constant `{name}`")),
                    Some(_) => return self.err(*vpos, format!("cannot assign to array `{name}`; assign to `{name}[i]`")),
                    None => return Err(self.unknown_var(name, *vpos)),
                };
                if ty == Type::U8 && matches!(op, Some(BinOp::Add | BinOp::Sub)) && self.const_val(value) == Some(1) {
                    self.e.abs(if *op == Some(BinOp::Add) { INC } else { DEC }, addr);
                    return Ok(());
                }
                let full = self.compound(Expr { kind: ExprKind::Var(name.clone()), pos: *vpos }, *op, value);
                self.check_fits(&full, ty)?;
                self.expr(&full, ty)?;
                self.store(addr, ty);
            }
            Stmt::Assign { target: Place::Index(name, idx, ipos), op, value, .. } => {
                let addr = match self.lookup(name) {
                    Some(Sym::Array { addr, .. }) => addr,
                    Some(Sym::Data { .. }) => return self.err(*ipos, format!("cannot assign to constant array `{name}`")),
                    Some(_) => return self.err(*ipos, format!("`{name}` is not an array")),
                    None => return Err(self.unknown_var(name, *ipos)),
                };
                let elem = Expr { kind: ExprKind::Index(name.clone(), Box::new(idx.clone())), pos: *ipos };
                let full = self.compound(elem, *op, value);
                self.check_fits(&full, Type::U8)?;
                if let Some(Val::Num(a)) = self.const_index(name, idx, *ipos)? {
                    self.expr(&full, Type::U8)?;
                    self.e.abs(STA, a);
                } else if self.operand8(idx)?.is_some() {
                    self.expr(&full, Type::U8)?;
                    self.index_to_x(idx)?;
                    self.e.ins(STA, Operand::abs_x(addr));
                } else {
                    self.expr(&full, Type::U8)?;
                    self.e.imp(PHA);
                    self.index_to_x(idx)?;
                    self.e.imp(PLA);
                    self.e.ins(STA, Operand::abs_x(addr));
                }
            }
            Stmt::If { cond, then, els, .. } => {
                let else_l = self.e.new_label();
                self.cond(cond, &else_l, false)?;
                self.block(then);
                match els {
                    Some(els) => {
                        let end = self.e.new_label();
                        self.e.jmp(&end);
                        self.e.label(&else_l);
                        self.block(els);
                        self.e.label(&end);
                    }
                    None => self.e.label(&else_l),
                }
            }
            Stmt::While { cond, body, .. } => {
                let top = self.e.new_label();
                let end = self.e.new_label();
                self.e.label(&top);
                self.cond(cond, &end, false)?;
                self.loop_body(body, &top, &end);
                self.e.jmp(&top);
                self.e.label(&end);
            }
            Stmt::Loop { body, .. } => {
                let top = self.e.new_label();
                let end = self.e.new_label();
                self.e.label(&top);
                self.loop_body(body, &top, &end);
                self.e.jmp(&top);
                self.e.label(&end);
            }
            Stmt::For { var, start, end, body, pos } => {
                for b in [start, end] {
                    if self.ty(b)? != Type::U8 {
                        return self.err(b.pos, "`for` range bounds must be u8");
                    }
                }
                self.scopes.push(HashMap::new());
                let r = self.for_loop(var, start, end, body, *pos);
                self.scopes.pop();
                r?;
            }
            Stmt::Break(pos) | Stmt::Continue(pos) => {
                let Some((cont, brk)) = self.loops.last().cloned() else {
                    let kw = if matches!(s, Stmt::Break(_)) { "break" } else { "continue" };
                    return self.err(*pos, format!("`{kw}` outside of a loop"));
                };
                self.e.jmp(if matches!(s, Stmt::Break(_)) { &brk } else { &cont });
            }
            Stmt::Return(value, pos) => {
                match (value, self.cur_ret) {
                    (Some(v), Some(t)) => {
                        self.check_fits(v, t)?;
                        self.expr(v, t)?;
                    }
                    (Some(v), None) => return self.err(v.pos, "this function has no return type; declare one with `-> u8`"),
                    (None, Some(t)) => return self.err(*pos, format!("this function must return a value of type {t}")),
                    (None, None) => {}
                }
                self.e.imp(RTS);
            }
            Stmt::LetZeroed { pos, .. } | Stmt::Match { pos, .. } => {
                return self.err(*pos, "not implemented yet");
            }
            Stmt::Expr(e) => {
                if let ExprKind::Call(name, args) = &e.kind {
                    self.call(name, args, e.pos)?;
                } else {
                    let t = self.ty(e)?;
                    self.expr(e, t)?;
                }
            }
        }
        Ok(())
    }

    fn compound(&self, target: Expr, op: Option<BinOp>, value: &Expr) -> Expr {
        match op {
            None => value.clone(),
            Some(op) => Expr { pos: value.pos, kind: ExprKind::Binary(op, Box::new(target), Box::new(value.clone())) },
        }
    }

    fn loop_body(&mut self, body: &Block, cont: &str, brk: &str) {
        self.loops.push((cont.to_string(), brk.to_string()));
        self.block(body);
        self.loops.pop();
    }

    fn for_loop(&mut self, var: &str, start: &Expr, end: &Expr, body: &Block, pos: Pos) -> R<()> {
        self.expr(start, Type::U8)?;
        let addr = self.declare_local(var, Type::U8, pos)?;
        self.e.abs(STA, addr);
        let limit = match self.const_val(end) {
            Some(v) => Operand::imm(fit(v, Type::U8, end.pos)? as u8),
            None => {
                let hidden = self.alloc(1, true, pos)?;
                self.expr(end, Type::U8)?;
                self.e.abs(STA, hidden);
                Operand::abs(hidden)
            }
        };
        let top = self.e.new_label();
        let cont = self.e.new_label();
        let brk = self.e.new_label();
        self.e.label(&top);
        self.e.abs(LDA, addr);
        self.e.ins(CMP, limit);
        self.e.br(BCS, &brk);
        self.loop_body(body, &cont, &brk);
        self.e.label(&cont);
        self.e.abs(INC, addr);
        self.e.jmp(&top);
        self.e.label(&brk);
        Ok(())
    }

}
