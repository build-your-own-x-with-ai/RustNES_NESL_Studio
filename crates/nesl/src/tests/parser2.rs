//! 002: struct, i8, typed arrays, zeroed lets and match.
use crate::ast::*;
use crate::lexer::{lex, Tok};
use crate::parser::parse;

fn main_body(body: &str) -> Block {
    let p = parse(&format!("fn main() {{ {body} }}")).expect("parse");
    match p.items.into_iter().last() {
        Some(Decl::Fn(f)) => f.body,
        d => panic!("unexpected {d:?}"),
    }
}

fn assigned(body: &str) -> (Place, String) {
    match main_body(body).remove(0) {
        Stmt::Assign { target, value, .. } => (target, value.to_string()),
        s => panic!("unexpected {s:?}"),
    }
}

#[test]
fn lexes_dot_and_new_keywords() {
    let toks: Vec<Tok> = lex("a.b struct match ..= ..").unwrap().into_iter().map(|t| t.tok).collect();
    assert_eq!(
        toks,
        vec![
            Tok::Ident("a".into()), Tok::Sym("."), Tok::Ident("b".into()), Tok::Kw("struct"),
            Tok::Kw("match"), Tok::Sym("..="), Tok::Sym(".."), Tok::Eof
        ]
    );
}

#[test]
fn struct_declarations() {
    let p = parse("struct Enemy { x: u8, y: u8, vx: i8, score: u16, }").unwrap();
    match &p.items[0] {
        Decl::Struct { name, fields, .. } => {
            assert_eq!(name, "Enemy");
            assert_eq!(
                fields,
                &vec![("x".into(), Type::U8), ("y".into(), Type::U8), ("vx".into(), Type::I8), ("score".into(), Type::U16)]
            );
        }
        d => panic!("{d:?}"),
    }
}

#[test]
fn typed_and_struct_variables() {
    let p = parse(
        "var vx: i8 = -2; var s: [u16; 5]; var o: [i8; 4]; var boss: Enemy; var es: [Enemy; 8];
         const N: [u16] = [0x1AB, 2]; const D: [i8] = [-1, 1]; const B: [u8] = [1];",
    )
    .unwrap();
    assert!(matches!(&p.items[0], Decl::Var { ty: VarType::Scalar(Type::I8), init: Some(_), .. }));
    assert!(matches!(&p.items[1], Decl::Var { ty: VarType::Array(Type::U16, _), .. }));
    assert!(matches!(&p.items[2], Decl::Var { ty: VarType::Array(Type::I8, _), .. }));
    assert!(matches!(&p.items[3], Decl::Var { ty: VarType::Struct(n), .. } if n == "Enemy"));
    assert!(matches!(&p.items[4], Decl::Var { ty: VarType::StructArray(n, _), .. } if n == "Enemy"));
    assert!(matches!(&p.items[5], Decl::Data { elem: Type::U16, elems, .. } if elems.len() == 2));
    assert!(matches!(&p.items[6], Decl::Data { elem: Type::I8, .. }));
    assert!(matches!(&p.items[7], Decl::Data { elem: Type::U8, .. }));
}

#[test]
fn field_access_lowers_to_dotted_names() {
    let (t, v) = assigned("boss.hp = es[i + 1].x + boss.y;");
    assert!(matches!(&t, Place::Var(n, _) if n == "boss.hp"), "{t:?}");
    assert_eq!(v, "(+ (index es.x (+ i 1)) boss.y)");
    let (t, _) = assigned("es[i].vx += 1;");
    assert!(matches!(&t, Place::Index(n, _, _) if n == "es.vx"), "{t:?}");
}

#[test]
fn zeroed_lets_for_arrays_and_structs() {
    let b = main_body("let buf: [u8; 8]; let w: [u16; 2]; let e: Enemy; let v: i8 = -1;");
    assert!(matches!(&b[0], Stmt::LetZeroed { name, ty: VarType::Array(Type::U8, _), .. } if name == "buf"));
    assert!(matches!(&b[1], Stmt::LetZeroed { ty: VarType::Array(Type::U16, _), .. }));
    assert!(matches!(&b[2], Stmt::LetZeroed { ty: VarType::Struct(n), .. } if n == "Enemy"));
    assert!(matches!(&b[3], Stmt::Let { ty: Some(Type::I8), .. }));
}

#[test]
fn match_arms() {
    let b = main_body(
        "match state {
            0 => title(),
            1 | 2 => { play(); }
            3..=9 => score += 1,
            -1 => {}
            _ => {}
        }",
    );
    let Stmt::Match { value, arms, .. } = &b[0] else { panic!("{:?}", b[0]) };
    assert_eq!(value.to_string(), "state");
    assert_eq!(arms.len(), 5);
    assert!(matches!(&arms[0].pats[..], [Pattern::Value(e)] if e.to_string() == "0"));
    assert_eq!(arms[1].pats.len(), 2);
    assert!(matches!(&arms[2].pats[..], [Pattern::Range(a, b)] if a.to_string() == "3" && b.to_string() == "9"));
    assert!(matches!(&arms[2].body[0], Stmt::Assign { op: Some(BinOp::Add), .. }));
    assert!(matches!(&arms[3].pats[..], [Pattern::Value(e)] if e.to_string() == "(neg 1)"));
    assert!(matches!(&arms[4].pats[..], [Pattern::Wild]));
    assert!(arms[4].body.is_empty());
}

#[test]
fn match_last_arm_comma_is_optional_and_errors_are_positioned() {
    let b = main_body("match x { _ => f() }");
    assert!(matches!(&b[0], Stmt::Match { arms, .. } if arms.len() == 1));
    let e = parse("fn main() { match x { 1 => f() 2 => g(), } }").unwrap_err();
    assert!(e.message.contains("expected `,`"), "{}", e.message);
    assert_eq!(e.col, 32);
    let e = parse("fn main() { let a: u8; }").unwrap_err();
    assert!(e.message.contains("initializer"), "{}", e.message);
    let e = parse("struct S { a: [u8; 2] }").unwrap_err();
    assert!(e.message.contains("field"), "{}", e.message);
}
