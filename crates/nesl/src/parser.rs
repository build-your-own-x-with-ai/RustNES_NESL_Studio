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
            Tok::Ident(s) if s == "u16" => {
                self.advance();
                Ok(Type::U16)
            }
            t => self.err(format!("expected type `u8` or `u16`, found {}", describe(t))),
        }
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
                        match self.scalar_type()? {
                            Type::U8 => {}
                            Type::U16 => return self.err("only `[u8]` constant arrays are supported"),
                        }
                        self.expect_sym("]")?;
                        self.expect_sym("=")?;
                        let elems = self.byte_list()?;
                        self.expect_sym(";")?;
                        return Ok(Decl::Data { name, elems, pos });
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
                let ty = if self.eat_sym("[") {
                    if self.scalar_type()? != Type::U8 {
                        return self.err("only `[u8; N]` arrays are supported");
                    }
                    self.expect_sym(";")?;
                    let len = self.expr()?;
                    self.expect_sym("]")?;
                    VarType::Array(len)
                } else {
                    VarType::Scalar(self.scalar_type()?)
                };
                let init = if self.eat_sym("=") { Some(self.expr()?) } else { None };
                self.expect_sym(";")?;
                Ok(Decl::Var { name, ty, init, pos })
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
            t => self.err(format!("expected `fn`, `var`, `const` or `tile`, found {}", describe(t))),
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
                let ty = if self.eat_sym(":") { Some(self.scalar_type()?) } else { None };
                self.expect_sym("=")?;
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
            Tok::Kw("break") => {
                self.advance();
                self.expect_sym(";")?;
                Ok(Stmt::Break(pos))
            }
            Tok::Kw("continue") => {
                self.advance();
                self.expect_sym(";")?;
                Ok(Stmt::Continue(pos))
            }
            Tok::Kw("return") => {
                self.advance();
                let value = if self.is_sym(";") { None } else { Some(self.expr()?) };
                self.expect_sym(";")?;
                Ok(Stmt::Return(value, pos))
            }
            _ => {
                let e = self.expr()?;
                let op = match self.peek() {
                    Tok::Sym("=") => Some(None),
                    Tok::Sym(s) => compound_op(s).map(Some),
                    _ => None,
                };
                if let Some(op) = op {
                    let target = match e.kind {
                        ExprKind::Var(name) => Place::Var(name, e.pos),
                        ExprKind::Index(name, idx) => Place::Index(name, *idx, e.pos),
                        _ => return self.err("left side of assignment must be a variable or array element"),
                    };
                    self.advance();
                    let value = self.expr()?;
                    self.expect_sym(";")?;
                    return Ok(Stmt::Assign { target, op, value, pos });
                }
                self.expect_sym(";")?;
                Ok(Stmt::Expr(e))
            }
        }
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
        loop {
            let Some((op, level)) = (match self.peek() {
                Tok::Sym(s) => binop(s),
                _ => None,
            }) else {
                break;
            };
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
                    Ok(Expr { kind: ExprKind::Index(name, Box::new(idx)), pos })
                } else {
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
