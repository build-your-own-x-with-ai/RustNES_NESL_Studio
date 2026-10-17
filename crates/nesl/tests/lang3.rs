//! 009: language fixes found while writing COIN QUEST.
mod common;
use common::*;

// ---------------------------------------------------------------- print_u16

#[test]
fn print_u16_shows_five_digits() {
    let r = run("var a: u16 = 65535; var b: u16 = 12345; var z: u16;
        fn main() {
            print_u16(0, 0, z); print_u16(0, 1, 7); print_u16(0, 2, a);
            print_u16(0, 3, b); print_u16(0, 4, 100); print_u16(6, 5, b + 1);
        }");
    for (row, want) in [(0u16, "00000"), (1, "00007"), (2, "65535"), (3, "12345"), (4, "00100")] {
        assert_eq!(r.vram(0x2000 + row * 32, 5), want.as_bytes().to_vec(), "row {row}");
    }
    assert_eq!(r.vram(0x2000 + 5 * 32 + 6, 5), b"12346".to_vec());
}

#[test]
fn print_u16_while_rendering_goes_through_the_queue() {
    let r = run_frames("var s: u16 = 4321;
        fn main() { ppu_on(); loop { wait_vblank(); s += 1; print_u16(10, 2, s); } }", 8, |_| {});
    let shown: u16 = String::from_utf8(r.vram(0x2000 + 2 * 32 + 10, 5)).unwrap().parse().unwrap();
    assert!(shown > 4321 && shown <= r.u16("s"), "{shown}");
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

// ---------------------------------------------------------------- set_map_tile with u16 columns

#[test]
fn set_map_tile_takes_u16_columns() {
    let r = run(&format!("{} var c: u16 = 150; var a: u8; var ok: u8; var edge: u8; var past: u8; var far: u8;
        fn main() {{
            ok = set_map_tile(W, c, 3, '#');
            a = map_tile(W, 150, 3);
            edge = set_map_tile(W, 199, 0, '#');
            past = set_map_tile(W, 200, 0, '#');
            far = set_map_tile(W, 300, 0, '#');
        }}", wide_map("W", 200)));
    assert_eq!((r.u8("ok"), r.u8("a"), r.u8("edge"), r.u8("past"), r.u8("far")), (1, b'#', 1, 0, 0));
}

// ---------------------------------------------------------------- map lists

fn two_maps() -> String {
    // the list comes first: maps may be declared after it
    format!("const L: [map] = [W, V];\n{}{}", wide_map("W", 64), wide_map("V", 40))
}

#[test]
fn map_lists_pick_a_map_at_run_time() {
    let r = run(&format!("{} var i: u8 = 1; var a: u8; var b: u8; var d: u8; var ok: u8; var calls: u8; var c: u16;
        fn pick() -> u8 {{ calls += 1; return 1; }}
        fn main() {{
            draw_map(L[i]);
            a = map_tile(L[i], 50, 2);
            b = map_tile(L[0], 50, 2);
            ok = set_map_tile(L[pick()], 3, 4, '#');
            d = map_tile(V, 3, 4);
            for k in 0..80 {{ camera_follow(1000, L[i]); }}
            c = camera_x();
        }}", two_maps()));
    assert_eq!(r.u8("a"), 0, "V is only 40 columns wide");
    assert_eq!(r.u8("b"), cell(50, 2));
    assert_eq!((r.u8("ok"), r.u8("d"), r.u8("calls")), (1, b'#', 1), "index evaluated once");
    assert_eq!(r.u16("c"), 40 * 8 - 256, "camera limit of V");
    assert_eq!(nt(&r, 39, 2), cell(39, 2));
    assert_eq!(nt(&r, 45, 2), 0, "V draws only its 40 columns");
}

#[test]
fn reset_map_and_static_indexes_work_through_a_list() {
    let r = run(&format!("{} var a: u8; var b: u8;
        fn main() {{ set_map_tile(W, 1, 1, '@'); a = map_tile(L[0], 1, 1); reset_map(L[0]); b = map_tile(W, 1, 1); }}", two_maps()));
    assert_eq!((r.u8("a"), r.u8("b")), (b'@', cell(1, 1)));
}

#[test]
fn an_out_of_range_index_uses_the_first_map() {
    let r = run(&format!("{} var i: u8 = 5; var a: u8; fn main() {{ a = map_tile(L[i], 50, 2); }}", two_maps()));
    assert_eq!(r.u8("a"), cell(50, 2));
}

#[test]
fn map_list_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    let w = wide_map("W", 32);
    assert!(m("const L: [map] = [Q]; fn main() {}").contains("`Q`"));
    assert!(m(&format!("var x: u8; {w} const L: [map] = [W, x]; fn main() {{}}")).contains("not a map"));
    assert!(m(&format!("{w} const L: [map] = [W]; fn main() {{ draw_map(L); }}")).contains("L[i]"));
    assert!(m(&format!("{w} const L: [map] = [W]; fn main() {{ draw_map(L[1]); }}")).contains("out of bounds"));
    assert!(m(&format!("{w} const L: [map] = [W]; var a: u8; fn main() {{ a = L[0]; }}")).contains("list of maps"));
}
