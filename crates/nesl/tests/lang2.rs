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
    assert!(m(&format!("{pre} fn main() {{ t.x = 1; }}")).contains("not a struct"));
    assert!(m("var s: Nope; fn main() {}").contains("unknown struct `Nope`"));
    assert!(m("struct E { x: u8, x: u8 } fn main() {}").contains("duplicate field `x`"));
    assert!(m("struct E { x: u8 } struct E { y: u8 } fn main() {}").contains("already defined"));
}

// ---------------------------------------------------------------- US2 i8

#[test]
fn signed_i8_arithmetic_and_comparisons() {
    let r = run("
        var x: u8 = 10; var vx: i8 = -3; var w: u16 = 1000; var v: i8 = -1;
        var s: i8 = -8; var sh: i8; var a: i8 = -5; var b: i8 = 3;
        var lt: u8; var gt: u8; var min: i8 = -128; var max: i8 = 127;
        fn main() {
            x += vx; w += v; sh = s >> 1;
            if a < b { lt = 1; } if a > b { gt = 1; }
            if min < max { lt += 1; }
        }");
    assert_eq!(r.u8("x"), 7);
    assert_eq!(r.u16("w"), 999);
    assert_eq!(r.i8("sh"), -4);
    assert_eq!((r.u8("lt"), r.u8("gt")), (2, 0));
}

#[test]
fn i8_constant_comparisons_negation_and_wrapping() {
    let r = run("
        var vx: i8 = -2; var c: [u8; 6]; var n: i8; var m: i8 = 127; var k: i8 = 100;
        fn main() {
            c[0] = vx < 0; c[1] = vx > -3; c[2] = vx <= -2; c[3] = 0 > vx; c[4] = vx == -2; c[5] = vx >= 1;
            n = -vx; m += 1; k = k + k;
        }");
    assert_eq!(r.bytes("c"), vec![1, 1, 1, 1, 1, 0]);
    assert_eq!((r.i8("n"), r.i8("m"), r.i8("k")), (2, -128, -56));
}

#[test]
fn i8_sign_extension_to_u16() {
    let r = run("var v: i8 = -2; var w: u16 = 100; fn main() { w = v as u16; }");
    assert_eq!(r.u16("w"), 0xFFFE);
}

#[test]
fn i8_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("var a: i8; var b: u8; fn main() { if a < b {} }").contains("type"));
    assert!(m("var a: i8; fn main() { a = a / 3; }").contains("power of 2"));
}

// ---------------------------------------------------------------- US3 typed and local arrays

#[test]
fn u16_and_i8_arrays_ram_and_const() {
    let r = run("
        const NOTES: [u16] = [0x1AB, 0x17C, 300];
        const DIRS: [i8] = [-1, 0, 1];
        var t: [u16; 4]; var o: [i8; 3]; var i: u8 = 1; var sum: u16; var x: u8 = 10;
        fn main() {
            t[0] = 1000; t[i] = NOTES[2] + 1; t[2] += 2000; t[i + 2] = t[0] + t[1];
            for k in 0..3 { o[k] = DIRS[k]; x += DIRS[k] * 4; sum += NOTES[k]; }
            play_tone(NOTES[i]);
        }");
    let t = r.bytes("t");
    let get = |k: usize| t[k] as u16 | (t[k + 4] as u16) << 8;
    assert_eq!((get(0), get(1), get(2), get(3)), (1000, 301, 2000, 1301));
    assert_eq!(r.bytes("o"), vec![0xFF, 0, 1]);
    assert_eq!(r.u16("sum"), 0x1AB + 0x17C + 300);
    assert_eq!(r.u8("x"), 10);
    let w = &r.nes.bus.apu_writes;
    assert!(w.contains(&(0x4002, 0x7C)) && w.contains(&(0x4003, 0x01)), "{w:x?}");
}

#[test]
fn local_arrays_and_structs_are_zeroed_each_call() {
    let r = run("
        struct P { a: u8, b: u16 }
        var first: u8; var second: u8; var pb: u16;
        fn f() -> u8 {
            let buf: [u8; 4]; let p: P;
            let s: u8 = 0;
            for i in 0..4 { s += buf[i]; buf[i] = i + 1; }
            p.b += 500; pb = p.b;
            return s;
        }
        fn main() { first = f(); second = f(); }");
    assert_eq!((r.u8("first"), r.u8("second")), (0, 0));
    assert_eq!(r.u16("pb"), 500);
}

#[test]
fn typed_array_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("const N: [u16] = [1]; fn main() { print(0, 0, N); }").contains("string"));
    assert!(m("var t: [u16; 2]; var a: u8; fn main() { a = t[0]; }").contains("as u8"));
    assert!(m("fn main() { let b: [u8; 2] = 3; }").contains("initializer"));
}

// ---------------------------------------------------------------- US4 match

#[test]
fn match_values_alternatives_ranges_and_wildcard() {
    let r = run("
        var out: [u8; 12]; var calls: u8;
        fn classify(v: u8) -> u8 {
            let r: u8 = 0;
            match v {
                0 => r = 10,
                1 | 2 => { r = 20; }
                3..=9 => r = 30,
                200..=255 => { r = 40; }
                _ => r = 50,
            }
            return r;
        }
        fn next() -> u8 { calls += 1; return calls; }
        fn main() {
            let vals: [u8; 8];
            vals[0] = 0; vals[1] = 1; vals[2] = 2; vals[3] = 3; vals[4] = 9; vals[5] = 10; vals[6] = 200; vals[7] = 255;
            for i in 0..8 { out[i] = classify(vals[i]); }
            match next() { 1 => out[8] = 1, 2 => out[8] = 2, _ => out[8] = 3 }
            match 7 { 1 => out[9] = 1 }
        }");
    assert_eq!(&r.bytes("out")[..10], &[10, 20, 20, 30, 30, 50, 40, 40, 1, 0]);
    assert_eq!(r.u8("calls"), 1, "scrutinee is evaluated exactly once");
}

#[test]
fn match_on_i8_with_negative_patterns() {
    let r = run("
        var out: [u8; 4]; var d: [i8; 4];
        fn main() {
            d[0] = -1; d[1] = 0; d[2] = 1; d[3] = -100;
            for i in 0..4 {
                match d[i] { -1 => out[i] = 1, 0 => out[i] = 2, -128..=-2 => out[i] = 4, _ => out[i] = 3 }
            }
        }");
    assert_eq!(r.bytes("out"), vec![1, 2, 3, 4]);
}

#[test]
fn match_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("var a: u8; var b: u8; fn main() { match a { b => {} } }").contains("constant"));
    assert!(m("var a: u8; fn main() { match a { 5..=1 => {} } }").contains("empty range"));
    assert!(m("fn f() { match 1 { _ => f() } } fn main() { f(); }").contains("recursion"));
}

#[test]
fn preview_enemy_loop_compiles_and_runs() {
    let r = run_frames("
        struct Enemy { x: u8, y: u8, hp: u8 }
        var enemies: [Enemy; 8];
        fn main() {
            for i in 0..8 { enemies[i].x = i * 24; enemies[i].hp = i % 2; }
            ppu_on();
            loop {
                wait_vblank();
                for i in 0..8 {
                    enemies[i].y += 1;
                    match enemies[i].hp {
                        0 => hide_sprite(i),
                        _ => sprite(i, enemies[i].x, enemies[i].y, 2, 0),
                    }
                }
            }
        }", 6, |_| {});
    assert_eq!(r.nes.peek(0x0200), 0xFF, "enemy 0 has hp 0 and is hidden");
    assert_eq!(r.nes.peek(0x0207), 24, "enemy 1 sprite x");
    // OAM holds y - 1; the enemy moves one pixel per frame after setup
    let oam_y = r.nes.peek(0x0204);
    assert!(oam_y >= 3, "enemy 1 moved down (OAM y = {oam_y})");
}
