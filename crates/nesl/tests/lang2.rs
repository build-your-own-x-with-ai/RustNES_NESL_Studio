//! 002: struct, i8, typed arrays, local arrays and match — verified by execution.
mod common;
use common::*;

// ---------------------------------------------------------------- US1 struct

#[test]
fn struct_variables_and_struct_arrays() {
    let r = run("
        struct Boss { x: u8, hp: u8, score: u16 }
        struct Enemy { x: u8, y: u8, hp: u8 }
        var boss: Boss; var es: [Enemy; 8]; var total: u8;
        fn main() {
            boss.hp = 9; boss.x = boss.hp + 1; boss.score = 1000; boss.score += boss.x;
            for i in 0..8 { es[i].y += i; es[i].x = i * 2; }
            es[3].hp = 7;
            es[es[3].hp].hp += 1;
            for i in 0..8 { total += es[i].y; }
        }");
    assert_eq!((r.u8("boss.hp"), r.u8("boss.x"), r.u16("boss.score")), (9, 10, 1010));
    assert_eq!(r.bytes("es.y"), (0..8).collect::<Vec<u8>>());
    assert_eq!(r.bytes("es.x"), (0..8).map(|i| i * 2).collect::<Vec<u8>>());
    assert_eq!(r.bytes("es.hp"), vec![0, 0, 0, 7, 0, 0, 0, 1]);
    assert_eq!(r.u8("total"), 28);
}

#[test]
fn struct_array_field_read_is_a_single_indexed_load() {
    let out = build("struct E { x: u8, y: u8 } var es: [E; 4]; var i: u8; var a: u8; fn main() { a = es[i].y; }");
    let main = &out.listing[out.listing.find("f_main:").unwrap()..];
    let lines: Vec<&str> = main.lines().skip(1).take(3).collect();
    assert!(lines[0].contains("LDX"), "{main}");
    assert!(lines[1].contains("LDA") && lines[1].ends_with(",X"), "{main}");
    assert!(lines[2].contains("STA"), "{main}");
}

#[test]
fn struct_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    let pre = "struct Enemy { x: u8, y: u8 } var boss: Enemy; var es: [Enemy; 4]; var t: u8;";
    assert!(m(&format!("{pre} fn main() {{ boss.z = 1; }}")).contains("no field `z` in struct `Enemy`"));
    assert!(m(&format!("{pre} fn main() {{ t = es[0].q; }}")).contains("no field `q` in struct `Enemy`"));
    assert!(m(&format!("{pre} fn main() {{ t = boss; }}")).contains("is a struct"));
    assert!(m(&format!("{pre} fn main() {{ es[1] = 3; }}")).contains("access a field"));
    assert!(m(&format!("{pre} fn main() {{ t.x = 1; }}")).contains("`t` is not a struct"));
    assert!(m("var s: Nope; fn main() {}").contains("unknown struct `Nope`"));
    assert!(m("struct E { x: u8, x: u8 } fn main() {}").contains("duplicate field `x`"));
    assert!(m("struct E { x: u8 } struct E { y: u8 } fn main() {}").contains("already defined"));
}
