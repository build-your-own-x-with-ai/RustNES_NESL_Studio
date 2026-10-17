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
