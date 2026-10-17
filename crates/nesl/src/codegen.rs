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

/// Symbols to bind for a struct allocation, plus its contiguous (addr, size) region.
type Allocation = (Vec<(String, Sym)>, u16, u16);

#[derive(Clone, Debug)]
pub(crate) enum Sym {
    Var { addr: u16, ty: Type },
    Array { addr: u16, len: u16, elem: Type },
    Const(i64),
    Data { label: String, len: u16, elem: Type },
    /// Marker for struct variables; field access routes to Var("name.field").
    Struct(String),
    /// Marker for struct arrays; field access routes to Array("name.field", ...).
    StructArray(String),
    /// A background map in ROM (column-major tiles at `label`, attributes at `label_a`).
    Map { label: String, width: u8 },
    /// A metasprite table in ROM: `[count, (dx, dy, tile, attr)...]`.
    Metasprite { label: String },
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
    structs: HashMap<String, Vec<(String, Type)>>,
    zones: HashMap<String, (Vec<(String, Pos)>, Pos)>,
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
    let min = if t == Type::I8 { -128 } else { -(max + 1) };
    if v > max || v < min {
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
            structs: HashMap::new(),
            zones: HashMap::new(),
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
            if let Decl::Struct { name, fields, pos } = d {
                if let Err(e) = self.declare_struct(name, fields, *pos) {
                    self.errors.push(e);
                }
            }
        }
        for d in &prog.items {
            if let Decl::PaletteZone { name, rows, pos } = d {
                if self.zones.insert(name.clone(), (rows.clone(), *pos)).is_some() {
                    self.errors.push(Diagnostic::new(*pos, format!("`{name}` already has a palette_zone")));
                }
            }
        }
        for d in &prog.items {
            if let Err(e) = self.declare(d) {
                // Keep the name defined so later uses do not cascade into
                // misleading "unknown" errors.
                if let Decl::Map { name, .. } = d {
                    self.globals.entry(name.clone()).or_insert(Sym::Map { label: "__bad_map".into(), width: 32 });
                }
                self.errors.push(e);
            }
        }
        let mut orphans: Vec<_> = self
            .zones
            .iter()
            .filter(|(n, _)| !matches!(self.globals.get(*n), Some(Sym::Map { .. })))
            .map(|(n, (_, pos))| Diagnostic::new(*pos, format!("palette_zone `{n}` does not name a map")))
            .collect();
        orphans.sort_by_key(|d| (d.line, d.col));
        self.errors.extend(orphans);
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

    fn declare_struct(&mut self, name: &str, fields: &[(String, Type)], pos: Pos) -> R<()> {
        if self.structs.contains_key(name) {
            return self.err(pos, format!("struct `{name}` is already defined"));
        }
        self.define(name, pos)?;
        if fields.is_empty() {
            return self.err(pos, "struct cannot be empty");
        }
        let mut seen = std::collections::HashSet::new();
        for (f, _) in fields {
            if !seen.insert(f) {
                return self.err(pos, format!("duplicate field `{f}` in struct `{name}`"));
            }
        }
        self.structs.insert(name.to_string(), fields.to_vec());
        Ok(())
    }

    /// Allocates `[elem; len]`; u16 arrays get a low-byte table then a high-byte table.
    fn alloc_array(&mut self, elem: Type, len: &Expr, pos: Pos) -> R<(Sym, u16, u16)> {
        let n = self.const_expr(len, "array length")?;
        if !(1..=256).contains(&n) {
            return self.err(len.pos, "array length must be between 1 and 256");
        }
        let size = n as u16 * if elem == Type::U16 { 2 } else { 1 };
        let addr = self.alloc(size, false, pos)?;
        Ok((Sym::Array { addr, len: n as u16, elem }, addr, size))
    }

    /// Allocates a struct (`len == None`) or struct array; returns the symbols to bind
    /// (`name` plus one per field) and the contiguous (addr, size) region.
    fn alloc_struct(&mut self, name: &str, sname: &str, len: Option<&Expr>, pos: Pos) -> R<Allocation> {
        let Some(fields) = self.structs.get(sname).cloned() else {
            return self.err(pos, format!("unknown struct `{sname}`"));
        };
        let mut syms = Vec::new();
        let mut start = None;
        let mut total = 0u16;
        for (fname, fty) in &fields {
            let (sym, addr, size) = match len {
                None => {
                    let size = if *fty == Type::U16 { 2 } else { 1 };
                    let addr = self.alloc(size, false, pos)?;
                    (Sym::Var { addr, ty: *fty }, addr, size)
                }
                Some(len) => self.alloc_array(*fty, len, pos)?,
            };
            start.get_or_insert(addr);
            total += size;
            self.vars.insert(format!("{name}.{fname}"), VarInfo { addr, size });
            syms.push((format!("{name}.{fname}"), sym));
        }
        let marker = match len {
            None => Sym::Struct(sname.to_string()),
            Some(_) => Sym::StructArray(sname.to_string()),
        };
        syms.push((name.to_string(), marker));
        Ok((syms, start.unwrap(), total))
    }

    /// Attribute bytes for a map, column-group-major: group g (4 tile columns), block row r
    /// at `g * 8 + r`. Each byte packs the palettes of four 16x16 areas.
    fn palette_zone_attrs(&self, map: &str, width: usize) -> R<Vec<u8>> {
        let groups = width.div_ceil(4);
        let mut zone = vec![[0u8; 128]; 16];
        if let Some((rows, pos)) = self.zones.get(map) {
            if rows.len() > 15 {
                return self.err(rows[15].1, format!("a palette_zone has at most 15 rows, found {}", rows.len()));
            }
            let max_cols = width.div_ceil(2);
            for (zr, (row, rpos)) in rows.iter().enumerate() {
                if row.chars().count() > max_cols {
                    return self.err(*rpos, format!("palette_zone row {} is wider than the map ({max_cols} areas)", zr + 1));
                }
                for (zc, ch) in row.chars().enumerate() {
                    zone[zr][zc] = match ch {
                        '.' | '0' => 0,
                        '1' => 1,
                        '2' => 2,
                        '3' => 3,
                        other => {
                            return self.err(*rpos, format!("invalid palette `{other}` in palette_zone row {} (use 0-3)", zr + 1))
                        }
                    };
                }
            }
            let _ = pos;
        }
        let mut out = Vec::with_capacity(groups * 8);
        for g in 0..groups {
            for r in 0..8 {
                let (zr, zc) = (r * 2, g * 2);
                out.push(zone[zr][zc] | zone[zr][zc + 1] << 2 | zone[zr + 1][zc] << 4 | zone[zr + 1][zc + 1] << 6);
            }
        }
        Ok(out)
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
            Decl::Data { name, elem, elems, pos } => {
                self.define(name, *pos)?;
                let max = if *elem == Type::U16 { 256 } else { 0xFFFF };
                if elems.is_empty() || elems.len() > max {
                    return self.err(*pos, format!("constant {elem} array must have between 1 and {max} elements"));
                }
                // u16 tables are split into a low-byte table followed by a high-byte table.
                let mut lo = Vec::with_capacity(elems.len());
                let mut hi = Vec::new();
                for el in elems {
                    let v = fit(self.const_expr(el, "array element")?, *elem, el.pos)?;
                    lo.push(v as u8);
                    hi.push((v >> 8) as u8);
                }
                if *elem == Type::U16 {
                    lo.extend(hi);
                }
                let label = format!("d_{name}");
                self.globals.insert(name.clone(), Sym::Data { label: label.clone(), len: elems.len() as u16, elem: *elem });
                self.data.push((label, lo));
            }
            Decl::Struct { .. } => {}
            Decl::Var { name, ty, init, pos } => {
                self.define(name, *pos)?;
                match ty {
                    VarType::Struct(sname) | VarType::StructArray(sname, _) => {
                        if init.is_some() {
                            return self.err(*pos, "struct variables cannot have an initializer (they start zeroed)");
                        }
                        let len = match ty {
                            VarType::StructArray(_, len) => Some(len),
                            _ => None,
                        };
                        let (syms, _, _) = self.alloc_struct(name, sname, len, *pos)?;
                        self.globals.extend(syms);
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
                    VarType::Array(elem, len) => {
                        if init.is_some() {
                            return self.err(*pos, "RAM arrays cannot have an initializer (they start zeroed)");
                        }
                        let (sym, addr, size) = self.alloc_array(*elem, len, *pos)?;
                        self.globals.insert(name.clone(), sym);
                        self.vars.insert(name.clone(), VarInfo { addr, size });
                    }
                }
            }
            Decl::Map { name, rows, pos } => {
                self.define(name, *pos)?;
                if rows.len() > 30 {
                    return self.err(rows[30].1, format!("a map has at most 30 rows, found {}", rows.len()));
                }
                for (y, (row, rpos)) in rows.iter().enumerate() {
                    if !row.is_ascii() {
                        return self.err(*rpos, "map rows may only contain ASCII characters");
                    }
                    if row.len() > 255 {
                        return self.err(*rpos, format!("map row {} has {} columns; the limit is 255", y + 1, row.len()));
                    }
                }
                let width = rows.iter().map(|(r, _)| r.len()).max().unwrap_or(0).max(32);
                // Column-major: column c, row r at c * 30 + r, so a column is one vertical write.
                let mut tiles = vec![b' '; width * 30];
                for (y, (row, _)) in rows.iter().enumerate() {
                    for (x, c) in row.bytes().enumerate() {
                        tiles[x * 30 + y] = if c == b'.' { b' ' } else { c };
                    }
                }
                let attrs = self.palette_zone_attrs(name, width)?;
                let label = format!("m_{name}");
                self.globals.insert(name.clone(), Sym::Map { label: label.clone(), width: width as u8 });
                self.data.push((format!("{label}_a"), attrs));
                self.data.push((label, tiles));
            }
            Decl::PaletteZone { .. } => {}
            Decl::Metasprite { name, parts, pos } => {
                self.define(name, *pos)?;
                if parts.is_empty() || parts.len() > 63 {
                    return self.err(*pos, format!("a metasprite needs 1 to 63 pieces, found {}", parts.len()));
                }
                let mut bytes = vec![parts.len() as u8];
                for part in parts {
                    for (e, signed) in [(&part.dx, true), (&part.dy, true), (&part.tile, false)] {
                        let v = self.const_expr(e, "metasprite value")?;
                        let range = if signed { -128..=127 } else { 0..=255 };
                        if !range.contains(&v) {
                            let what = if signed { "an offset (-128..=127)" } else { "a tile (0..=255)" };
                            return self.err(e.pos, format!("constant {v} does not fit as {what}"));
                        }
                        bytes.push(v as u8);
                    }
                    let attr = match &part.attr {
                        Some(a) => fit(self.const_expr(a, "metasprite attribute")?, Type::U8, a.pos)? as u8,
                        None => 0,
                    };
                    bytes.push(attr);
                }
                let label = format!("ms_{name}");
                self.globals.insert(name.clone(), Sym::Metasprite { label: label.clone() });
                self.data.push((label, bytes));
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
                    Stmt::Match { value, arms, .. } => {
                        ex(value, out);
                        for arm in arms {
                            calls(&arm.body, out);
                        }
                    }
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
        if let Some(dot) = name.find('.') {
            let base = &name[..dot];
            let field = &name[dot + 1..];
            if let Some(Sym::Struct(sname) | Sym::StructArray(sname)) = self.lookup(base) {
                return Diagnostic::new(pos, format!("no field `{field}` in struct `{sname}`"));
            }
            return Diagnostic::new(pos, format!("`{base}` is not a struct or struct array"));
        }
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
                Some(Sym::Struct(_)) => self.err(e.pos, format!("`{name}` is a struct; access a field with `{name}.field`")),
                Some(Sym::StructArray(_)) => self.err(e.pos, format!("`{name}` is a struct array; access a field with `{name}[i].field`")),
                Some(Sym::Map { .. }) => self.err(e.pos, format!("`{name}` is a map; use `draw_map({name})` or `map_tile({name}, x, y)`")),
                Some(Sym::Metasprite { .. }) => self.err(e.pos, format!("`{name}` is a metasprite; draw it with `metasprite(slot, x, y, {name})`")),
                Some(Sym::Const(_)) => unreachable!(),
                None => Err(self.unknown_var(name, e.pos)),
            },
            ExprKind::Index(name, idx) => {
                let (_, _, elem) = self.array_base(name, e.pos)?;
                if self.ty(idx)? != Type::U8 {
                    return self.err(idx.pos, "array index must be u8");
                }
                Ok(elem)
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
                } else if a == Type::U16 || b == Type::U16 {
                    Ok(Type::U16)
                } else if matches!(op, BinOp::Shl | BinOp::Shr) {
                    Ok(a)
                } else if (a == Type::I8 && (b == Type::I8 || self.const_val(r).is_some()))
                    || (b == Type::I8 && self.const_val(l).is_some())
                {
                    Ok(Type::I8)
                } else {
                    Ok(Type::U8)
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
        if want != Type::U16 && self.ty(e)? == Type::U16 {
            return self.err(e.pos, format!("mismatched types: expected {want}, found u16 (use `as {want}` to truncate)"));
        }
        Ok(())
    }

    fn array_base(&self, name: &str, pos: Pos) -> R<(Val, u16, Type)> {
        match self.lookup(name) {
            Some(Sym::Array { addr, len, elem }) => Ok((Val::Num(addr), len, elem)),
            Some(Sym::Data { label, len, elem }) => Ok((Val::Label(label.clone(), 0), len, elem)),
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
        let (base, len, _) = self.array_base(name, pos)?;
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
                Some(Sym::Var { addr, ty: Type::U8 | Type::I8 }) => Some(Operand::abs(addr)),
                _ => None,
            },
            ExprKind::Index(name, idx) if self.array_base(name, e.pos)?.2 != Type::U16 => {
                self.const_index(name, idx, e.pos)?.map(Operand::Abs)
            }
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
        if self.ty(e)? == Type::I8 {
            return Ok(None);
        }
        Ok(self.operand8(e)?.map(|o| (o, Operand::imm(0))))
    }

    // ------------------------------------------------------------------ expressions

    /// Extends an 8-bit value in A to A:X when `want` is u16 (sign-extending i8).
    fn widen(&mut self, from: Type, want: Type) {
        if from == Type::U16 || want != Type::U16 {
            return;
        }
        self.e.imm(LDX, 0);
        if from == Type::I8 {
            let pos = self.e.new_label();
            self.e.imm(CMP, 0x80);
            self.e.br(BCC, &pos);
            self.e.imp(DEX);
            self.e.label(&pos);
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
        if nat == Type::U16 && want != Type::U16 {
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
                self.widen(nat, want);
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
            ExprKind::Binary(op, l, r) => {
                // Operate at the target width when widening, otherwise at the natural
                // type so i8 keeps arithmetic shifts.
                let width = if want == Type::U16 { Type::U16 } else { nat };
                self.arith(*op, l, r, width)?;
            }
            ExprKind::Cast(x, t) => {
                let inner = self.ty(x)?;
                self.expr(x, inner)?;
                if *t == Type::U16 {
                    self.widen(inner, Type::U16);
                } else {
                    self.widen(*t, want);
                }
            }
            ExprKind::Num(_) | ExprKind::Str(_) => unreachable!("handled by const_val / ty"),
        }
        Ok(())
    }

    /// Loads `name[idx]` into A (8-bit elements) or A:X (u16 elements).
    fn index(&mut self, name: &str, idx: &Expr, pos: Pos) -> R<()> {
        let (base, len, elem) = self.array_base(name, pos)?;
        let hi = Self::offset(&base, len);
        if let Some(v) = self.const_index(name, idx, pos)? {
            self.e.ins(LDA, Operand::Abs(v.clone()));
            if elem == Type::U16 {
                self.e.ins(LDX, Operand::Abs(Self::offset(&v, len)));
            }
            return Ok(());
        }
        if elem == Type::U16 {
            self.index_to_x(idx)?;
            self.e.imp(TXA);
            self.e.imp(TAY);
            self.e.ins(LDA, Operand::AbsY(base));
            self.e.ins(LDX, Operand::AbsY(hi));
            return Ok(());
        }
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
        if want == Type::I8 && matches!(op, Div | Mod) || want == Type::I8 && op == Mul && pow2.is_none() {
            return self.err(
                l.pos,
                format!("`{}` is not supported for i8 values (only `*` by a power of 2)", op.symbol()),
            );
        }
        match (op, want) {
            (Add | Sub | And | Or | Xor, Type::U8 | Type::I8) => {
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
        if n >= bits && !(ty == Type::I8 && !left) {
            self.e.imm(LDA, 0);
            if ty == Type::U16 {
                self.e.imm(LDX, 0);
            }
            return;
        }
        if ty == Type::I8 && !left {
            for _ in 0..n.min(7) {
                self.e.imm(CMP, 0x80);
                self.e.acc(ROR);
            }
            return;
        }
        if ty != Type::U16 {
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
            (false, Type::U8) => self.e.acc(LSR),
            (false, Type::I8) => {
                self.e.imm(CMP, 0x80);
                self.e.acc(ROR);
            }
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
        let (a, b) = (self.ty(l)?, self.ty(r)?);
        let (lc, rc) = (self.const_val(l).is_some(), self.const_val(r).is_some());
        if (a == Type::I8 && (b == Type::I8 || rc)) || (b == Type::I8 && lc) {
            return self.compare_signed(op, l, r, label);
        }
        if (a == Type::I8) != (b == Type::I8) && !lc && !rc {
            let (s, u) = if a == Type::I8 { (a, b) } else { (b, a) };
            return self.err(
                l.pos,
                format!("mismatched types: cannot compare {s} with {u} (convert one side with `as`)"),
            );
        }
        let t = max_ty(a, b);
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

    /// Signed 8-bit comparison: after `SEC; SBC r` the true sign of l - r is N ^ V.
    fn compare_signed(&mut self, op: BinOp, l: &Expr, r: &Expr, label: &str) -> R<()> {
        use BinOp::*;
        // l > r  <=>  r < l,  l <= r  <=>  r >= l
        let (l, r) = if matches!(op, Gt | Le) { (r, l) } else { (l, r) };
        let rhs = match self.operand8(r)? {
            Some(o) => {
                self.expr(l, Type::I8)?;
                o
            }
            None => {
                self.expr(l, Type::I8)?;
                self.e.imp(PHA);
                self.expr(r, Type::I8)?;
                self.e.abs(STA, TMP0);
                self.e.imp(PLA);
                Operand::abs(TMP0)
            }
        };
        if matches!(op, Eq | Ne) {
            self.e.ins(CMP, rhs);
            self.e.br(if op == Eq { BEQ } else { BNE }, label);
            return Ok(());
        }
        let no_overflow = self.e.new_label();
        self.e.imp(SEC);
        self.e.ins(SBC, rhs);
        self.e.br(BVC, &no_overflow);
        self.e.imm(EOR, 0x80);
        self.e.label(&no_overflow);
        // N is now set exactly when (the possibly swapped) l < r.
        self.e.br(if matches!(op, Lt | Gt) { BMI } else { BPL }, label);
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
                    Some(Sym::StructArray(_)) => {
                        return self.err(*ipos, format!("`{name}` is a struct array; access a field with `{name}[i].field`"));
                    }
                    Some(_) => return self.err(*ipos, format!("`{name}` is not an array")),
                    None => return Err(self.unknown_var(name, *ipos)),
                };
                let (_, len, ety) = self.array_base(name, *ipos)?;
                let elem = Expr { kind: ExprKind::Index(name.clone(), Box::new(idx.clone())), pos: *ipos };
                let full = self.compound(elem, *op, value);
                self.check_fits(&full, ety)?;
                if ety == Type::U16 {
                    self.store_u16_elem(addr, len, idx, &full)?;
                } else if let Some(Val::Num(a)) = self.const_index(name, idx, *ipos)? {
                    self.expr(&full, ety)?;
                    self.e.abs(STA, a);
                } else if self.operand8(idx)?.is_some() {
                    self.expr(&full, ety)?;
                    self.index_to_x(idx)?;
                    self.e.ins(STA, Operand::abs_x(addr));
                } else {
                    self.expr(&full, ety)?;
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
            Stmt::LetZeroed { name, ty, pos } => {
                let (syms, addr, size) = match ty {
                    VarType::Array(elem, len) => {
                        let (sym, addr, size) = self.alloc_array(*elem, len, *pos)?;
                        (vec![(name.clone(), sym)], addr, size)
                    }
                    VarType::Struct(sname) => self.alloc_struct(name, sname, None, *pos)?,
                    VarType::StructArray(sname, len) => self.alloc_struct(name, sname, Some(len), *pos)?,
                    VarType::Scalar(_) => unreachable!("parser requires an initializer"),
                };
                self.zero_fill(addr, size);
                self.scopes.last_mut().unwrap().extend(syms);
            }
            Stmt::Match { value, arms, pos } => {
                self.scopes.push(HashMap::new());
                let r = self.match_stmt(value, arms, *pos);
                self.scopes.pop();
                r?;
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

    /// Binds `names` to fresh static u8 slots in a new scope (caller pops it).
    pub(crate) fn hidden_u8_scope(&mut self, names: &[&str], pos: Pos) -> R<Vec<u16>> {
        let mut scope = HashMap::new();
        let mut addrs = Vec::new();
        for n in names {
            let addr = self.alloc(1, true, pos)?;
            scope.insert(n.to_string(), Sym::Var { addr, ty: Type::U8 });
            addrs.push(addr);
        }
        self.scopes.push(scope);
        Ok(addrs)
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Lowers `match` to a hidden local holding the scrutinee and one condition per arm.
    fn match_stmt(&mut self, value: &Expr, arms: &[MatchArm], pos: Pos) -> R<()> {
        let ty = self.ty(value)?;
        let hidden = format!("$match{}", self.e.new_label());
        self.expr(value, ty)?;
        let addr = self.declare_local(&hidden, ty, pos)?;
        self.store(addr, ty);
        let var = Expr { kind: ExprKind::Var(hidden), pos };
        let end = self.e.new_label();
        for arm in arms {
            let mut cond: Option<Expr> = None;
            for pat in &arm.pats {
                let test = match pat {
                    Pattern::Wild => Expr { kind: ExprKind::Num(1), pos: arm.pos },
                    Pattern::Value(v) => {
                        self.const_expr(v, "match pattern")?;
                        Expr { kind: ExprKind::Binary(BinOp::Eq, Box::new(var.clone()), Box::new(v.clone())), pos: v.pos }
                    }
                    Pattern::Range(lo, hi) => {
                        if self.const_expr(lo, "match pattern")? > self.const_expr(hi, "match pattern")? {
                            return self.err(lo.pos, "empty range pattern: the start is greater than the end");
                        }
                        let ge = Expr { kind: ExprKind::Binary(BinOp::Ge, Box::new(var.clone()), Box::new(lo.clone())), pos: lo.pos };
                        let le = Expr { kind: ExprKind::Binary(BinOp::Le, Box::new(var.clone()), Box::new(hi.clone())), pos: hi.pos };
                        Expr { kind: ExprKind::Binary(BinOp::LogAnd, Box::new(ge), Box::new(le)), pos: lo.pos }
                    }
                };
                cond = Some(match cond {
                    None => test,
                    Some(c) => Expr { pos: c.pos, kind: ExprKind::Binary(BinOp::LogOr, Box::new(c), Box::new(test)) },
                });
            }
            let next = self.e.new_label();
            self.cond(cond.as_ref().expect("arm has a pattern"), &next, false)?;
            self.block(&arm.body);
            self.e.jmp(&end);
            self.e.label(&next);
        }
        self.e.label(&end);
        Ok(())
    }

    /// `arr[idx] = value` for u16 arrays (low table at `addr`, high table at `addr + len`).
    fn store_u16_elem(&mut self, addr: u16, len: u16, idx: &Expr, value: &Expr) -> R<()> {
        if let Some(i) = self.const_val(idx) {
            if i < 0 || i >= len as i64 {
                return self.err(idx.pos, format!("index {i} is out of bounds (length {len})"));
            }
            self.expr(value, Type::U16)?;
            self.store(addr + i as u16, Type::U8);
            self.e.abs(STX, addr + len + i as u16);
            return Ok(());
        }
        self.expr(value, Type::U16)?;
        self.e.imp(PHA);
        self.e.imp(TXA);
        self.e.imp(PHA);
        self.index_to_x(idx)?;
        self.e.imp(TXA);
        self.e.imp(TAY);
        self.e.imp(PLA);
        self.e.ins(STA, Operand::abs_y(addr + len));
        self.e.imp(PLA);
        self.e.ins(STA, Operand::abs_y(addr));
        Ok(())
    }

    /// Emits code clearing `size` bytes at `addr`.
    fn zero_fill(&mut self, addr: u16, size: u16) {
        self.e.imm(LDA, 0);
        if size <= 4 {
            for i in 0..size {
                self.e.abs(STA, addr + i);
            }
            return;
        }
        let mut off = 0;
        while off < size {
            let n = (size - off).min(255);
            let top = self.e.new_label();
            self.e.imm(LDX, n as u8);
            self.e.label(&top);
            self.e.ins(STA, Operand::abs_x(addr + off - 1));
            self.e.imp(DEX);
            self.e.br(BNE, &top);
            off += n;
        }
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
