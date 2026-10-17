//! 004: wide maps, camera and palette zones — verified on the test machine.
mod common;
use common::*;

/// A map `width` columns wide whose cell (c, r) is a printable character derived from
/// both coordinates, so any misplaced column or row is detected.
fn cell(c: usize, r: usize) -> u8 {
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[(c * 7 + r * 3) % 36]
}

fn wide_map(name: &str, width: usize) -> String {
    let rows: Vec<String> = (0..30)
        .map(|r| format!("    \"{}\"", (0..width).map(|c| cell(c, r) as char).collect::<String>()))
        .collect();
    format!("map {name} = [\n{}\n];\n", rows.join(",\n"))
}

/// Nametable byte for ring column `slot` (0-63) and row `r`.
fn nt(r: &Run, slot: usize, row: usize) -> u8 {
    let base = if slot < 32 { 0x2000 } else { 0x2400 };
    r.nes.bus.vram[base + row * 32 + slot % 32]
}

// ---------------------------------------------------------------- US1 wide maps

#[test]
fn draw_map_fills_both_nametables_for_a_wide_map() {
    let r = run(&format!("{} fn main() {{ draw_map(W); }}", wide_map("W", 128)));
    for c in 0..64 {
        for row in 0..30 {
            assert_eq!(nt(&r, c, row), cell(c, row), "column {c} row {row}");
        }
    }
}

#[test]
fn map_tile_takes_u16_columns() {
    let r = run(&format!("{} var a: u8; var b: u8; var c: u8; var d: u8; var e: u8; var far: u16 = 300; var col: u16 = 100;
        fn main() {{ a = map_tile(W, col, 5); b = map_tile(W, 127, 29); c = map_tile(W, 128, 0); d = map_tile(W, far, 1); e = map_tile(W, 3, 30); }}",
        wide_map("W", 128)));
    assert_eq!((r.u8("a"), r.u8("b"), r.u8("c"), r.u8("d"), r.u8("e")), (cell(100, 5), cell(127, 29), 0, 0, 0));
}

#[test]
fn width_limit_is_255_columns() {
    build(&format!("{} fn main() {{}}", wide_map("W", 255)));
    let e = errors(&format!("{} fn main() {{}}", wide_map("W", 256)));
    assert!(e[0].message.contains("255"), "{}", e[0].message);
}
