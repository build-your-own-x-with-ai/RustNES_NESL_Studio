//! US1: diagnostics carry positions and readable messages.
mod common;
use common::errors;

fn first(src: &str) -> (u32, u32, String) {
    let e = &errors(src)[0];
    (e.line, e.col, e.message.clone())
}

#[test]
fn undefined_names() {
    let (l, c, m) = first("fn main() {\n  x = 1;\n}");
    assert_eq!((l, c), (2, 3));
    assert!(m.contains("unknown variable `x`"), "{m}");
    let (_, _, m) = first("fn main() { foo(); }");
    assert!(m.contains("unknown function `foo`"), "{m}");
}

#[test]
fn type_errors() {
    let (_, _, m) = first("var a: u8; var w: u16; fn main() { a = w; }");
    assert!(m.contains("as u8"), "{m}");
    let (_, _, m) = first("var a: u8 = 300; fn main() {}");
    assert!(m.contains("does not fit"), "{m}");
    let (_, _, m) = first("var w: u16; fn main() { w = w * w; }");
    assert!(m.contains("u16"), "{m}");
    let (_, _, m) = first("var a: u8; fn main() { a = a[1]; }");
    assert!(m.contains("not an array"), "{m}");
    let (_, _, m) = first("const K = 3; fn main() { K = 1; }");
    assert!(m.contains("cannot assign"), "{m}");
}

#[test]
fn call_errors() {
    let (_, _, m) = first("fn f(a: u8) {} fn main() { f(1, 2); }");
    assert!(m.contains("expects 1 argument"), "{m}");
    let (_, _, m) = first("fn f() {} var a: u8; fn main() { a = f(); }");
    assert!(m.contains("does not return a value"), "{m}");
    let (_, _, m) = first("fn f() -> u8 { return; } fn main() {}");
    assert!(m.contains("must return"), "{m}");
}

#[test]
fn recursion_is_rejected() {
    let (_, _, m) = first("fn f() { f(); } fn main() { f(); }");
    assert!(m.contains("recursion"), "{m}");
    let (_, _, m) = first("fn a() { b(); } fn b() { a(); } fn main() { a(); }");
    assert!(m.contains("recursion"), "{m}");
}

#[test]
fn structural_errors() {
    let (_, _, m) = first("fn helper() {}");
    assert!(m.contains("main"), "{m}");
    let (_, _, m) = first("var a: u8; var a: u8; fn main() {}");
    assert!(m.contains("already defined"), "{m}");
    let (_, _, m) = first("fn main() { break; }");
    assert!(m.contains("outside of a loop"), "{m}");
}

#[test]
fn multiple_errors_are_reported() {
    let errs = errors("fn main() {\n x = 1;\n y = 2;\n}");
    assert_eq!(errs.len(), 2);
    assert_eq!(errs[1].line, 3);
}

#[test]
fn syntax_errors_surface_through_compile() {
    let (l, _, m) = first("fn main() {\n let = 3;\n}");
    assert_eq!(l, 2);
    assert!(m.contains("expected identifier"), "{m}");
}
