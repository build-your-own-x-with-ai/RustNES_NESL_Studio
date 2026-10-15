//! US1: language semantics, verified by executing compiled ROMs.
mod common;
use common::*;

#[test]
fn globals_initializers_and_assignment() {
    let r = run("var a: u8 = 7; var b: u8; var c: u16 = 1000; var d: u8 = 0xFF;
                 fn main() { b = a + 3; }");
    assert_eq!((r.u8("a"), r.u8("b"), r.u16("c"), r.u8("d")), (7, 10, 1000, 255));
}

#[test]
fn u8_arithmetic_wraps_and_bitwise() {
    let r = run("
        var x: u8 = 200; var y: u8 = 100; var s: u8 = 5; var t: u8 = 10; var one: u8 = 1; var n: u8 = 3;
        var r1: u8; var r2: u8; var r3: u8; var r4: u8; var r5: u8; var r6: u8; var r7: u8;
        var r8: u8; var r9: u8; var r10: u8; var r11: u8; var r12: u8; var r13: u8; var r14: u8;
        fn main() {
            r1 = x + y;        r2 = s - t;         r3 = x & 0x3C;   r4 = x | 0x0F;
            r5 = x ^ 0xFF;     r6 = one << 5;      r7 = x >> 3;     r8 = -one;
            r9 = ~s;           r10 = !s;           r11 = !(s - 5);  r12 = one << n;
            r13 = x >> n;      r14 = (x + y) - (s + t) + (one + one);
        }");
    let got: Vec<u8> = (1..=14).map(|i| r.u8(&format!("r{i}"))).collect();
    assert_eq!(got, vec![44, 251, 200 & 0x3C, 200 | 0x0F, 200 ^ 0xFF, 32, 25, 255, !5u8, 0, 1, 8, 25, 31]);
}

#[test]
fn u8_multiply_divide_modulo_including_divide_by_zero() {
    let r = run("
        var a: u8 = 13; var b: u8 = 11; var c: u8 = 200; var d: u8 = 7; var z: u8 = 0;
        var m: u8; var big: u8; var q: u8; var rem: u8; var qz: u8; var rz: u8;
        fn main() { m = a * b; big = c * d; q = c / d; rem = c % d; qz = c / z; rz = c % z; }");
    assert_eq!(r.u8("m"), 143);
    assert_eq!(r.u8("big"), (200u32 * 7 % 256) as u8);
    assert_eq!((r.u8("q"), r.u8("rem")), (28, 4));
    assert_eq!((r.u8("qz"), r.u8("rz")), (0xFF, 200));
}

#[test]
fn u16_arithmetic_widening_and_casts() {
    let r = run("
        var a: u16 = 1000; var b: u16 = 300; var s: u8 = 250; var k: u8 = 4;
        var r1: u16; var r2: u16; var r3: u16; var r4: u16; var r5: u16; var r6: u16;
        var r7: u8; var r8: u16; var r9: u16; var r10: u16;
        fn main() {
            r1 = a + b; r2 = b - a; r3 = a + s; r4 = s + s; r5 = a << 2; r6 = a >> k;
            r7 = (a + 5) as u8; r8 = (a & 0xFF00) | 0x12; r9 = s as u16 + 6; r10 = a ^ 0xFFFF;
        }");
    assert_eq!(r.u16("r1"), 1300);
    assert_eq!(r.u16("r2"), 300u16.wrapping_sub(1000));
    assert_eq!(r.u16("r3"), 1250);
    assert_eq!(r.u16("r4"), 500, "operands are widened to the target type");
    assert_eq!(r.u16("r5"), 4000);
    assert_eq!(r.u16("r6"), 1000 >> 4);
    assert_eq!(r.u8("r7"), (1005u16 & 0xFF) as u8);
    assert_eq!(r.u16("r8"), 0x0312);
    assert_eq!(r.u16("r9"), 256);
    assert_eq!(r.u16("r10"), !1000u16);
}

#[test]
fn comparisons_as_values_u8_and_u16() {
    let r = run("
        var a: u8 = 3; var b: u8 = 7; var w: u16 = 0x1234; var v: u16 = 0x1300;
        var c: [u8; 12];
        fn main() {
            c[0] = a == b; c[1] = a != b; c[2] = a < b; c[3] = a <= b; c[4] = a > b; c[5] = a >= b;
            c[6] = w == v; c[7] = w != v; c[8] = w < v;  c[9] = w <= w; c[10] = w > v; c[11] = v >= w;
        }");
    assert_eq!(r.bytes("c"), vec![0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1]);
}

#[test]
fn if_else_chains_and_short_circuit_logic() {
    let r = run("
        var calls: u8; var r1: u8; var r2: u8; var r3: u8; var x: u8 = 5;
        fn side() -> u8 { calls += 1; return 1; }
        fn classify(v: u8) -> u8 {
            if v < 3 { return 1; } else if v < 6 { return 2; } else { return 3; }
        }
        fn main() {
            if x == 0 && side() { r1 = 9; }
            if x == 5 || side() { r2 = 1; }
            if x > 1 && side() == 1 { r3 = classify(x) * 10 + classify(1) + classify(200) * 100; }
        }");
    assert_eq!(r.u8("calls"), 1);
    assert_eq!((r.u8("r1"), r.u8("r2")), (0, 1));
    assert_eq!(r.u8("r3"), 300u32.wrapping_add(21) as u8);
}

#[test]
fn loops_break_continue_and_for() {
    let r = run("
        var sum: u16; var evens: u8; var n: u8; var nested: u16; var lp: u8;
        fn main() {
            let i: u8 = 1;
            while i <= 100 { sum += i; i += 1; }
            for k in 0..10 { if k % 2 == 1 { continue; } evens += 1; }
            loop { n += 3; if n > 20 { break; } }
            for a in 0..10 { for b in 0..10 { nested += 1; } }
            let lim: u8 = 4;
            for z in lim..lim { lp = 99; }
        }");
    assert_eq!(r.u16("sum"), 5050);
    assert_eq!(r.u8("evens"), 5);
    assert_eq!(r.u8("n"), 21);
    assert_eq!(r.u16("nested"), 100);
    assert_eq!(r.u8("lp"), 0);
}

#[test]
fn functions_params_locals_nested_calls_and_u16_returns() {
    let r = run("
        var r1: u8; var r2: u16; var r3: u8;
        fn add(a: u8, b: u8) -> u8 { let t = a + b; return t; }
        fn wide(a: u8, b: u16) -> u16 { return b + a; }
        fn twice(a: u8) -> u8 { return add(a, a); }
        fn main() {
            r1 = add(add(1, 2), add(3, twice(4)));
            r2 = wide(10, 60000);
            r3 = add(1, 1) + twice(add(2, 3));
        }");
    assert_eq!(r.u8("r1"), 14);
    assert_eq!(r.u16("r2"), 60010);
    assert_eq!(r.u8("r3"), 12);
}

#[test]
fn arrays_ram_and_const_data() {
    let r = run("
        const TABLE: [u8] = [10, 20, 30, 40];
        const MSG: [u8] = \"AB\";
        var buf: [u8; 8]; var total: u8; var m: u8;
        fn main() {
            for i in 0..4 { buf[i] = TABLE[i] + i; }
            buf[buf[0] - 8] += 5;
            for i in 0..4 { total += buf[i]; }
            m = MSG[1];
        }");
    assert_eq!(r.bytes("buf"), vec![10, 21, 37, 43, 0, 0, 0, 0]);
    assert_eq!(r.u8("total"), 111);
    assert_eq!(r.u8("m"), b'B');
}

#[test]
fn constants_are_folded_and_usable_everywhere() {
    let r = run("
        const W = 32; const H = W / 4 + 2; const BIG = W * 100;
        var area: u16 = W * H; var buf: [u8; H]; var b: u16;
        fn main() { b = BIG + 1; buf[H - 1] = W; }");
    assert_eq!(r.u16("area"), 320);
    assert_eq!(r.out.vars["buf"].size, 10);
    assert_eq!(r.bytes("buf")[9], 32);
    assert_eq!(r.u16("b"), 3201);
}

#[test]
fn let_inference_and_block_shadowing() {
    let r = run("
        var r1: u16; var r2: u8;
        fn main() {
            let w = 300;
            let x = 5;
            if true { let x = 9; r2 = x; }
            r1 = w + x;
        }");
    assert_eq!(r.u16("r1"), 305);
    assert_eq!(r.u8("r2"), 9);
}

#[test]
fn complex_operands_on_both_sides() {
    let r = run("
        var a: u8 = 9; var b: u8 = 4; var c: u8 = 20; var d: u8 = 6; var e: u8 = 17; var f: u8 = 5;
        var r1: u8; var r2: u16; var w: u16 = 500;
        fn main() {
            r1 = (a + b) * (c - d) / (e % f);
            r2 = (w - (a as u16)) - (w >> 2) + (b + (c - d));
        }");
    assert_eq!(r.u8("r1"), ((13u32 * 14) % 256 / 2) as u8);
    assert_eq!(r.u16("r2"), 500 - 9 - 125 + 18);
}

#[test]
fn long_bodies_need_far_branches() {
    let mut body = String::new();
    for _ in 0..80 {
        body.push_str("x = x + y; ");
    }
    let src = format!("var x: u8; var y: u8 = 1; var z: u8;
        fn main() {{ while z < 3 {{ {body} z += 1; }} if x == 240 {{ {body} }} }}");
    let r = run(&src);
    assert_eq!(r.u8("x"), (240u32 + 80) as u8);
}

#[test]
fn compilation_is_deterministic() {
    let src = "var a: u8; fn f(x: u8) -> u8 { return x * 2; } fn main() { a = f(3); }";
    assert_eq!(build(src).rom, build(src).rom);
}
