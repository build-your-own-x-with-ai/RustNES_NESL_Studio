//! Recursive-descent parser producing the AST.

use crate::ast::*;
use crate::lexer::{lex, Tok, Token};
use crate::Diagnostic;

type PResult<T> = Result<T, Diagnostic>;

pub fn parse(src: &str) -> PResult<Program> {
    let toks = lex(src)?;
    let mut p = Parser { toks, i: 0 };
    let mut items = Vec::new();
    while p.peek() != &Tok::Eof {
        items.push(p.decl()?);
    }
    Ok(Program { items })
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Int(n) => format!("number `{n}`"),
        Tok::Str(_) => "string literal".into(),
        Tok::Ident(s) => format!("identifier `{s}`"),
        Tok::Kw(k) => format!("keyword `{k}`"),
        Tok::Sym(s) => format!("`{s}`"),
        Tok::Eof => "end of file".into(),
    }
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.i].tok
    }

    fn pos(&self) -> Pos {
        self.toks[self.i].pos
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.i].clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        t
    }

    fn err<T>(&self, msg: impl Into<String>) -> PResult<T> {
        Err(Diagnostic::new(self.pos(), msg))
    }

    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Sym(x) if *x == s)
    }

    fn is_kw(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Kw(x) if *x == s)
    }

    fn eat_sym(&mut self, s: &str) -> bool {
        if self.is_sym(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_sym(&mut self, s: &str) -> PResult<()> {
        if self.eat_sym(s) {
            Ok(())
        } else {
            self.err(format!("expected `{s}`, found {}", describe(self.peek())))
        }
    }

    fn expect_kw(&mut self, s: &str) -> PResult<()> {
        if self.is_kw(s) {
            self.advance();
            Ok(())
        } else {
            self.err(format!("expected `{s}`, found {}", describe(self.peek())))
        }
    }

    fn ident(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.advance();
                Ok(s)
            }
            t => self.err(format!("expected identifier, found {}", describe(&t))),
        }
    }

    fn scalar_type(&mut self) -> PResult<Type> {
        match self.peek() {
            Tok::Ident(s) if s == "u8" => {
                self.advance();
                Ok(Type::U8)
            }
            Tok::Ident(s) if s == "i8" => {
                self.advance();
                Ok(Type::I8)
            }
            Tok::Ident(s) if s == "u16" => {
                self.advance();
                Ok(Type::U16)
            }
            t => self.err(format!("expected type `u8`, `i8` or `u16`, found {}", describe(t))),
        }
    }

    fn is_scalar_type(&self) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == "u8" || s == "i8" || s == "u16")
    }

    /// A struct type name; rejects things that look like unsupported integer types.
    fn struct_name(&mut self) -> PResult<String> {
        if let Tok::Ident(s) = self.peek() {
            let b = s.as_bytes();
            if b.len() > 1 && (b[0] == b'u' || b[0] == b'i') && b[1..].iter().all(u8::is_ascii_digit) {
                return self.err(format!("unknown type `{s}` (expected u8, i8, u16 or a struct name)"));
            }
        }
        self.ident()
    }

    /// `T`, `[T; N]`, `S` or `[S; N]`.
    fn var_type(&mut self) -> PResult<VarType> {
        if self.eat_sym("[") {
            let elem = if self.is_scalar_type() { Ok(self.scalar_type()?) } else { Err(self.struct_name()?) };
            self.expect_sym(";")?;
            let len = self.expr()?;
            self.expect_sym("]")?;
            return Ok(match elem {
                Ok(t) => VarType::Array(t, len),
                Err(name) => VarType::StructArray(name, len),
            });
        }
        if self.is_scalar_type() {
            return Ok(VarType::Scalar(self.scalar_type()?));
        }
        Ok(VarType::Struct(self.struct_name()?))
    }

    // ---------------------------------------------------------------- declarations

    fn decl(&mut self) -> PResult<Decl> {
        let pos = self.pos();
        match self.peek() {
            Tok::Kw("const") => {
                self.advance();
                let name = self.ident()?;
                if self.eat_sym(":") {
                    if self.eat_sym("[") {
                        let elem = self.scalar_type()?;
                        self.expect_sym("]")?;
                        self.expect_sym("=")?;
                        let elems = self.byte_list()?;
                        self.expect_sym(";")?;
                        return Ok(Decl::Data { name, elem, elems, pos });
                    }
                    self.scalar_type()?;
                }
                self.expect_sym("=")?;
                let value = self.expr()?;
                self.expect_sym(";")?;
                Ok(Decl::Const { name, value, pos })
            }
            Tok::Kw("var") => {
                self.advance();
                let name = self.ident()?;
                self.expect_sym(":")?;
                let ty = self.var_type()?;
                let init = if self.eat_sym("=") { Some(self.expr()?) } else { None };
                self.expect_sym(";")?;
                Ok(Decl::Var { name, ty, init, pos })
            }
            Tok::Kw("struct") => {
                self.advance();
                let name = self.ident()?;
                self.expect_sym("{")?;
                let mut fields = Vec::new();
                while !self.is_sym("}") {
                    let f = self.ident()?;
                    self.expect_sym(":")?;
                    if !self.is_scalar_type() {
                        return self.err(format!("struct field `{f}` must be u8, i8 or u16, found {}", describe(self.peek())));
                    }
                    fields.push((f, self.scalar_type()?));
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("}")?;
                Ok(Decl::Struct { name, fields, pos })
            }
            Tok::Kw("tile") => {
                self.advance();
                let index = self.expr()?;
                self.expect_sym("=")?;
                self.expect_sym("[")?;
                let mut rows = Vec::new();
                while !self.is_sym("]") {
                    let rpos = self.pos();
                    match self.advance().tok {
                        Tok::Str(b) => rows.push((String::from_utf8_lossy(&b).into_owned(), rpos)),
                        t => {
                            return Err(Diagnostic::new(
                                rpos,
                                format!("expected tile row string, found {}", describe(&t)),
                            ))
                        }
                    }
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("]")?;
                self.expect_sym(";")?;
                Ok(Decl::Tile { index, rows, pos })
            }
            Tok::Kw("fn") => {
                self.advance();
                let name = self.ident()?;
                self.expect_sym("(")?;
                let mut params = Vec::new();
                while !self.is_sym(")") {
                    let p = self.ident()?;
                    self.expect_sym(":")?;
                    params.push((p, self.scalar_type()?));
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym(")")?;
                let ret = if self.eat_sym("->") { Some(self.scalar_type()?) } else { None };
                let body = self.block()?;
                Ok(Decl::Fn(Function { name, params, ret, body, pos }))
            }
            t => self.err(format!("expected `fn`, `var`, `const`, `struct` or `tile`, found {}", describe(t))),
        }
    }

    /// `[e, e, ...]` or a string literal.
    fn byte_list(&mut self) -> PResult<Vec<Expr>> {
        let pos = self.pos();
        if let Tok::Str(bytes) = self.peek().clone() {
            self.advance();
            return Ok(bytes.into_iter().map(|b| Expr { kind: ExprKind::Num(b as u32), pos }).collect());
        }
        self.expect_sym("[")?;
        let mut elems = Vec::new();
        while !self.is_sym("]") {
            if let Tok::Str(bytes) = self.peek().clone() {
                let p = self.pos();
                self.advance();
                elems.extend(bytes.into_iter().map(|b| Expr { kind: ExprKind::Num(b as u32), pos: p }));
            } else {
                elems.push(self.expr()?);
            }
            if !self.eat_sym(",") {
                break;
            }
        }
        self.expect_sym("]")?;
        Ok(elems)
    }

    // ---------------------------------------------------------------- statements

    fn block(&mut self) -> PResult<Block> {
        self.expect_sym("{")?;
        let mut stmts = Vec::new();
        while !self.is_sym("}") {
            if self.peek() == &Tok::Eof {
                return self.err("expected `}`, found end of file");
            }
            stmts.push(self.stmt()?);
        }
        self.advance();
        Ok(stmts)
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        match self.peek() {
            Tok::Kw("let") => {
                self.advance();
                let name = self.ident()?;
                let ty = if self.eat_sym(":") {
                    match self.var_type()? {
                        VarType::Scalar(t) => Some(t),
                        vt => {
                            if self.is_sym("=") {
                                return self.err("local arrays and structs cannot have an initializer (they start zeroed)");
                            }
                            self.expect_sym(";")?;
                            return Ok(Stmt::LetZeroed { name, ty: vt, pos });
                        }
                    }
                } else {
                    None
                };
                if !self.is_sym("=") {
                    return self.err(format!("`let {name}` needs an initializer (`= value`)"));
                }
                self.advance();
                let init = self.expr()?;
                self.expect_sym(";")?;
                Ok(Stmt::Let { name, ty, init, pos })
            }
            Tok::Kw("if") => self.if_stmt(),
            Tok::Kw("while") => {
                self.advance();
                let cond = self.expr()?;
                let body = self.block()?;
                Ok(Stmt::While { cond, body, pos })
            }
            Tok::Kw("loop") => {
                self.advance();
                Ok(Stmt::Loop { body: self.block()?, pos })
            }
            Tok::Kw("for") => {
                self.advance();
                let var = self.ident()?;
                self.expect_kw("in")?;
                let start = self.expr()?;
                self.expect_sym("..")?;
                let end = self.expr()?;
                let body = self.block()?;
                Ok(Stmt::For { var, start, end, body, pos })
            }
            Tok::Kw("match") => self.match_stmt(),
            _ => {
                let st = self.simple_stmt()?;
                self.expect_sym(";")?;
                Ok(st)
            }
        }
    }

    /// `break`, `continue`, `return [e]`, assignment or expression — without the trailing `;`.
    fn simple_stmt(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        match self.peek() {
            Tok::Kw("break") => {
                self.advance();
                Ok(Stmt::Break(pos))
            }
            Tok::Kw("continue") => {
                self.advance();
                Ok(Stmt::Continue(pos))
            }
            Tok::Kw("return") => {
                self.advance();
                let value = if self.is_sym(";") || self.is_sym(",") || self.is_sym("}") { None } else { Some(self.expr()?) };
                Ok(Stmt::Return(value, pos))
            }
            _ => {
                let e = self.expr()?;
                let op = match self.peek() {
                    Tok::Sym("=") => Some(None),
                    Tok::Sym(s) => compound_op(s).map(Some),
                    _ => None,
                };
                let Some(op) = op else { return Ok(Stmt::Expr(e)) };
                let target = match e.kind {
                    ExprKind::Var(name) => Place::Var(name, e.pos),
                    ExprKind::Index(name, idx) => Place::Index(name, *idx, e.pos),
                    _ => return self.err("left side of assignment must be a variable or array element"),
                };
                self.advance();
                let value = self.expr()?;
                Ok(Stmt::Assign { target, op, value, pos })
            }
        }
    }

    fn match_stmt(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        self.expect_kw("match")?;
        let value = self.expr()?;
        self.expect_sym("{")?;
        let mut arms = Vec::new();
        while !self.is_sym("}") {
            let apos = self.pos();
            let mut pats = vec![self.pattern()?];
            while self.eat_sym("|") {
                pats.push(self.pattern()?);
            }
            self.expect_sym("=>")?;
            if self.is_sym("{") {
                let body = self.block()?;
                self.eat_sym(",");
                arms.push(MatchArm { pats, body, pos: apos });
            } else {
                let body = vec![self.simple_stmt()?];
                arms.push(MatchArm { pats, body, pos: apos });
                if !self.is_sym("}") {
                    self.expect_sym(",")?;
                }
            }
        }
        self.advance();
        Ok(Stmt::Match { value, arms, pos })
    }

    /// An optional `.field` suffix.
    fn field(&mut self) -> PResult<Option<String>> {
        if !self.eat_sym(".") {
            return Ok(None);
        }
        let f = self.ident()?;
        if self.is_sym(".") {
            return self.err("nested struct fields are not supported");
        }
        Ok(Some(f))
    }

    fn pattern(&mut self) -> PResult<Pattern> {
        if matches!(self.peek(), Tok::Ident(s) if s == "_") {
            self.advance();
            return Ok(Pattern::Wild);
        }
        // Parse above `|` precedence so `|` separates alternatives.
        let lo = self.binary(5)?;
        if self.eat_sym("..=") {
            let hi = self.binary(5)?;
            return Ok(Pattern::Range(lo, hi));
        }
        Ok(Pattern::Value(lo))
    }

    fn if_stmt(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        self.expect_kw("if")?;
        let cond = self.expr()?;
        let then = self.block()?;
        let els = if self.is_kw("else") {
            self.advance();
            if self.is_kw("if") {
                Some(vec![self.if_stmt()?])
            } else {
                Some(self.block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If { cond, then, els, pos })
    }

    // ---------------------------------------------------------------- expressions

    pub fn expr(&mut self) -> PResult<Expr> {
        self.binary(0)
    }

    fn binary(&mut self, min_level: u8) -> PResult<Expr> {
        let mut lhs = self.cast()?;
        while let Some((op, level)) = match self.peek() {
            Tok::Sym(s) => binop(s),
            _ => None,
        } {
            if level < min_level {
                break;
            }
            let pos = self.pos();
            self.advance();
            let rhs = self.binary(level + 1)?;
            lhs = Expr { kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)), pos };
        }
        Ok(lhs)
    }

    fn cast(&mut self) -> PResult<Expr> {
        let mut e = self.unary()?;
        while self.is_kw("as") {
            let pos = self.pos();
            self.advance();
            let ty = self.scalar_type()?;
            e = Expr { kind: ExprKind::Cast(Box::new(e), ty), pos };
        }
        Ok(e)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        let op = match self.peek() {
            Tok::Sym("-") => Some(UnOp::Neg),
            Tok::Sym("!") => Some(UnOp::Not),
            Tok::Sym("~") => Some(UnOp::BitNot),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let e = self.unary()?;
            return Ok(Expr { kind: ExprKind::Unary(op, Box::new(e)), pos });
        }
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        match self.peek().clone() {
            Tok::Int(n) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Num(n), pos })
            }
            Tok::Str(s) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Str(s), pos })
            }
            Tok::Sym("(") => {
                self.advance();
                let e = self.expr()?;
                self.expect_sym(")")?;
                Ok(e)
            }
            Tok::Ident(name) => {
                self.advance();
                if self.eat_sym("(") {
                    let mut args = Vec::new();
                    while !self.is_sym(")") {
                        args.push(self.expr()?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")")?;
                    Ok(Expr { kind: ExprKind::Call(name, args), pos })
                } else if self.eat_sym("[") {
                    let idx = self.expr()?;
                    self.expect_sym("]")?;
                    // `a[i].f` is lowered to the per-field array `a.f` (SoA layout).
                    let name = match self.field()? {
                        Some(f) => format!("{name}.{f}"),
                        None => name,
                    };
                    Ok(Expr { kind: ExprKind::Index(name, Box::new(idx)), pos })
                } else {
                    let name = match self.field()? {
                        Some(f) => format!("{name}.{f}"),
                        None => name,
                    };
                    Ok(Expr { kind: ExprKind::Var(name), pos })
                }
            }
            t => self.err(format!("expected expression, found {}", describe(&t))),
        }
    }
}

fn binop(s: &str) -> Option<(BinOp, u8)> {
    use BinOp::*;
    Some(match s {
        "||" => (LogOr, 1),
        "&&" => (LogAnd, 2),
        "==" => (Eq, 3),
        "!=" => (Ne, 3),
        "<" => (Lt, 3),
        "<=" => (Le, 3),
        ">" => (Gt, 3),
        ">=" => (Ge, 3),
        "|" => (Or, 4),
        "^" => (Xor, 5),
        "&" => (And, 6),
        "<<" => (Shl, 7),
        ">>" => (Shr, 7),
        "+" => (Add, 8),
        "-" => (Sub, 8),
        "*" => (Mul, 9),
        "/" => (Div, 9),
        "%" => (Mod, 9),
        _ => return None,
    })
}

fn compound_op(s: &str) -> Option<BinOp> {
    use BinOp::*;
    Some(match s {
        "+=" => Add,
        "-=" => Sub,
        "*=" => Mul,
        "/=" => Div,
        "%=" => Mod,
        "&=" => And,
        "|=" => Or,
        "^=" => Xor,
        "<<=" => Shl,
        ">>=" => Shr,
        _ => return None,
    })
}
