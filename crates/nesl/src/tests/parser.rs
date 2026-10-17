use crate::ast::*;
use crate::parser::parse;

fn expr(src: &str) -> String {
    let p = parse(&format!("fn main() {{ x = {src}; }}")).expect("parse");
    match &p.items[0] {
        Decl::Fn(f) => match &f.body[0] {
            Stmt::Assign { value, .. } => value.to_string(),
            s => panic!("unexpected {s:?}"),
        },
        d => panic!("unexpected {d:?}"),
    }
}

#[test]
fn arithmetic_precedence() {
    assert_eq!(expr("1 + 2 * 3"), "(+ 1 (* 2 3))");
    assert_eq!(expr("(1 + 2) * 3"), "(* (+ 1 2) 3)");
    assert_eq!(expr("a - b - c"), "(- (- a b) c)");
}

#[test]
fn rust_style_bitwise_vs_comparison_precedence() {
    assert_eq!(expr("j & BTN_A != 0"), "(!= (& j BTN_A) 0)");
    assert_eq!(expr("a < b && c || d"), "(|| (&& (< a b) c) d)");
    assert_eq!(expr("1 << 2 + 1"), "(<< 1 (+ 2 1))");
    assert_eq!(expr("a | b ^ c & d"), "(| a (^ b (& c d)))");
}

#[test]
fn unary_cast_call_and_index() {
    assert_eq!(expr("-a + !b"), "(+ (neg a) (! b))");
    assert_eq!(expr("~f(1, g(2)) as u8"), "(as (~ (call f 1 (call g 2))) u8)");
    assert_eq!(expr("arr[i + 1] * 2"), "(* (index arr (+ i 1)) 2)");
}

#[test]
fn top_level_declarations() {
    let p = parse(
        r#"
        const SPEED = 2 + 1;
        const PAL: [u8] = [1, 2, SPEED];
        const MSG: [u8] = "AB";
        var x: u8 = 5;
        var score: u16;
        var buf: [u8; 16];
        tile 1 = ["..3333..", "........", "........", "........",
                  "........", "........", "........", "........"];
        fn add(a: u8, b: u16) -> u16 { return a + b; }
        fn main() {}
        "#,
    )
    .unwrap();
    assert_eq!(p.items.len(), 9);
    assert!(matches!(&p.items[0], Decl::Const { name, .. } if name == "SPEED"));
    assert!(matches!(&p.items[1], Decl::Data { elems, .. } if elems.len() == 3));
    assert!(matches!(&p.items[2], Decl::Data { elems, .. } if elems.len() == 2));
    assert!(matches!(&p.items[3], Decl::Var { ty: VarType::Scalar(Type::U8), init: Some(_), .. }));
    assert!(matches!(&p.items[4], Decl::Var { ty: VarType::Scalar(Type::U16), init: None, .. }));
    assert!(matches!(&p.items[5], Decl::Var { ty: VarType::Array(Type::U8, _), .. }));
    assert!(matches!(&p.items[6], Decl::Tile { rows, .. } if rows.len() == 8));
    match &p.items[7] {
        Decl::Fn(f) => {
            assert_eq!(f.name, "add");
            assert_eq!(f.params, vec![("a".into(), Type::U8), ("b".into(), Type::U16)]);
            assert_eq!(f.ret, Some(Type::U16));
        }
        d => panic!("{d:?}"),
    }
}

#[test]
fn statements() {
    let p = parse(
        r#"fn main() {
            let a: u8 = 1;
            let b = 2;
            a += 3;
            buf[a] = 4;
            if a == 1 { a = 2; } else if a == 2 { a = 3; } else { a = 4; }
            while a < 10 { a = a + 1; continue; }
            loop { break; }
            for i in 0..8 { f(i); }
            return;
        }"#,
    )
    .unwrap();
    let Decl::Fn(f) = &p.items[0] else { panic!() };
    let b = &f.body;
    assert!(matches!(&b[0], Stmt::Let { ty: Some(Type::U8), .. }));
    assert!(matches!(&b[1], Stmt::Let { ty: None, .. }));
    assert!(matches!(&b[2], Stmt::Assign { op: Some(BinOp::Add), target: Place::Var(..), .. }));
    assert!(matches!(&b[3], Stmt::Assign { target: Place::Index(..), .. }));
    match &b[4] {
        Stmt::If { els: Some(els), .. } => assert!(matches!(&els[0], Stmt::If { els: Some(_), .. })),
        s => panic!("{s:?}"),
    }
    assert!(matches!(&b[5], Stmt::While { .. }));
    assert!(matches!(&b[6], Stmt::Loop { .. }));
    assert!(matches!(&b[7], Stmt::For { var, .. } if var == "i"));
    assert!(matches!(&b[8], Stmt::Return(None, _)));
}

#[test]
fn syntax_errors_report_position() {
    let e = parse("fn main() {\n  let a = 1\n}").unwrap_err();
    assert_eq!((e.line, e.col), (3, 1));
    assert!(e.message.contains("expected `;`"), "{}", e.message);
    let e = parse("fn main() { 1 + ; }").unwrap_err();
    assert_eq!((e.line, e.col), (1, 17));
    let e = parse("var x: u32;").unwrap_err();
    assert!(e.message.contains("type"), "{}", e.message);
}
