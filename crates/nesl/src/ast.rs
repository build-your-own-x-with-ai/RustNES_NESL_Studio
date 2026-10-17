//! Abstract syntax tree for NESL.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    U8,
    I8,
    U16,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Type::U8 => "u8",
            Type::I8 => "i8",
            Type::U16 => "u16",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VarType {
    Scalar(Type),
    /// `[T; N]` — RAM array.
    Array(Type, Expr),
    /// A variable of a user-defined struct type.
    Struct(String),
    /// `[S; N]` — array of structs (stored as one array per field).
    StructArray(String, Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub items: Vec<Decl>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Const { name: String, value: Expr, pos: Pos },
    /// `const NAME: [T] = [...]` or `= "text"` — read-only table in ROM.
    Data { name: String, elem: Type, elems: Vec<Expr>, pos: Pos },
    Struct { name: String, fields: Vec<(String, Type)>, pos: Pos },
    Var { name: String, ty: VarType, init: Option<Expr>, pos: Pos },
    Tile { index: Expr, rows: Vec<(String, Pos)>, pos: Pos },
    /// `map NAME = ["row", ...];` — a 32x30 background screen drawn from text.
    Map { name: String, rows: Vec<(String, Pos)>, pos: Pos },
    /// `music NAME = "MML";` or `= ["MML", ...]` — each string keeps its position for errors.
    Music { name: String, parts: Vec<(String, Pos)>, pos: Pos },
    /// `sound NAME = (pulse|noise, period, delta, frames[, volume[, duty]]);`
    Sound { name: String, channel: (String, Pos), args: Vec<Expr>, pos: Pos },
    /// `palette_zone MAP = ["0123", ...];` — background palette per 16x16 area of a map.
    PaletteZone { name: String, rows: Vec<(String, Pos)>, pos: Pos },
    /// `metasprite NAME = [(dx, dy, tile[, attr]), ...];`
    Metasprite { name: String, parts: Vec<MetaPart>, pos: Pos },
    Fn(Function),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Option<Type>,
    pub body: Block,
    pub pos: Pos,
}

pub type Block = Vec<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub enum Place {
    Var(String, Pos),
    Index(String, Expr, Pos),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let { name: String, ty: Option<Type>, init: Expr, pos: Pos },
    /// `let buf: [u8; 8];` / `let e: Enemy;` — zero-filled each time it executes.
    LetZeroed { name: String, ty: VarType, pos: Pos },
    Match { value: Expr, arms: Vec<MatchArm>, pos: Pos },
    Assign { target: Place, op: Option<BinOp>, value: Expr, pos: Pos },
    If { cond: Expr, then: Block, els: Option<Block>, pos: Pos },
    While { cond: Expr, body: Block, pos: Pos },
    Loop { body: Block, pos: Pos },
    For { var: String, start: Expr, end: Expr, body: Block, pos: Pos },
    Break(Pos),
    Continue(Pos),
    Return(Option<Expr>, Pos),
    Expr(Expr),
}

/// One 8x8 piece of a metasprite.
#[derive(Clone, Debug, PartialEq)]
pub struct MetaPart {
    pub dx: Expr,
    pub dy: Expr,
    pub tile: Expr,
    pub attr: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub pats: Vec<Pattern>,
    pub body: Block,
    pub pos: Pos,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Value(Expr),
    /// `a..=b`
    Range(Expr, Expr),
    Wild,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod, And, Or, Xor, Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge, LogAnd, LogOr,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        use BinOp::*;
        match self {
            Add => "+", Sub => "-", Mul => "*", Div => "/", Mod => "%", And => "&", Or => "|",
            Xor => "^", Shl => "<<", Shr => ">>", Eq => "==", Ne => "!=", Lt => "<", Le => "<=",
            Gt => ">", Ge => ">=", LogAnd => "&&", LogOr => "||",
        }
    }

    pub fn is_comparison(self) -> bool {
        use BinOp::*;
        matches!(self, Eq | Ne | Lt | Le | Gt | Ge)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Num(u32),
    Str(Vec<u8>),
    Var(String),
    Index(String, Box<Expr>),
    Call(String, Vec<Expr>),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Cast(Box<Expr>, Type),
}

/// S-expression rendering, used by tests and debugging.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ExprKind::Num(n) => write!(f, "{n}"),
            ExprKind::Str(s) => write!(f, "{:?}", String::from_utf8_lossy(s)),
            ExprKind::Var(v) => write!(f, "{v}"),
            ExprKind::Index(a, i) => write!(f, "(index {a} {i})"),
            ExprKind::Call(name, args) => {
                write!(f, "(call {name}")?;
                for a in args {
                    write!(f, " {a}")?;
                }
                write!(f, ")")
            }
            ExprKind::Unary(op, e) => {
                let s = match op {
                    UnOp::Neg => "neg",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                };
                write!(f, "({s} {e})")
            }
            ExprKind::Binary(op, l, r) => write!(f, "({} {l} {r})", op.symbol()),
            ExprKind::Cast(e, t) => write!(f, "(as {e} {t})"),
        }
    }
}
