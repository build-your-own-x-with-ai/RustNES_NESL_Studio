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

// ---------------------------------------------------------------- US2 camera

/// The picture's scroll position as the PPU sees it: nametable bit from PPUCTRL + $2005 x.
fn hardware_scroll(r: &Run) -> u16 {
    ((r.nes.bus.ppuctrl & 1) as u16) << 8 | r.nes.bus.scroll.0 as u16
}

/// Every one of the 33 columns that can be on screen must hold the right map column.
fn assert_visible_columns(r: &Run, cam: u16, width: usize, ctx: &str) {
    let first = (cam / 8) as usize;
    for c in first..(first + 33).min(width) {
        for row in 0..30 {
            assert_eq!(nt(r, c % 64, row), cell(c, row), "{ctx}: column {c} row {row} (camera {cam})");
        }
    }
}

const FOLLOW: &str = "
    var hero: u16; var target: u16; var cam: u16;
    fn main() {
        draw_map(W);
        ppu_on();
        loop {
            wait_vblank();
            if hero < target { hero += 3; } else if hero > target { hero -= 3; }
            camera_follow(hero, W);
            cam = camera_x();
        }
    }";

#[test]
fn camera_scrolls_across_four_screens_and_back_without_glitches() {
    let width = 128;
    let src = format!("{}{FOLLOW}", wide_map("W", width));
    let out = build(&src);
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    let addr = |n: &str| out.vars[n].addr;
    let poke16 = |nes: &mut nes_sim::TestNes, a: u16, v: u16| {
        nes.bus.ram[a as usize] = v as u8;
        nes.bus.ram[a as usize + 1] = (v >> 8) as u8;
    };
    nes.run_frames(3).unwrap();
    let mut max_cam = 0;
    for (target, frames) in [(1020u16, 420u64), (0, 420)] {
        poke16(&mut nes, addr("target"), target);
        for f in 0..frames {
            nes.run_frames(1).unwrap();
            let r = Run { nes, out: out.clone() };
            let cam = r.u16("cam");
            max_cam = max_cam.max(cam);
            if f % 3 == 0 {
                // the scroll the PPU used this frame matches the camera of the previous one
                assert_visible_columns(&r, cam, width, &format!("target {target}, frame {f}"));
            }
            nes = r.nes;
        }
    }
    let r = Run { nes, out };
    assert_eq!(max_cam, 128 * 8 - 256, "camera stops at the right edge");
    assert_eq!(r.u16("cam"), 0, "and comes all the way back");
    assert_eq!(hardware_scroll(&r), 0);
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn camera_keeps_hero_near_120_and_moves_at_most_8_pixels_a_frame() {
    let src = format!("{}{FOLLOW}", wide_map("W", 128));
    let mut r = run_frames(&src, 3, |_| {});
    let t = r.addr("target") as usize;
    let h = r.addr("hero") as usize;
    // teleport the hero to x = 600 (target too, so it stays there)
    for a in [t, h] {
        r.nes.bus.ram[a] = (600u16 & 0xFF) as u8;
        r.nes.bus.ram[a + 1] = (600u16 >> 8) as u8;
    }
    let mut prev = 0u16;
    for _ in 0..80 {
        r.nes.run_frames(1).unwrap();
        let cam = r.u16("cam");
        assert!(cam >= prev && cam - prev <= 8, "step {prev} -> {cam}");
        prev = cam;
    }
    assert_eq!(prev, 600 - 120);
    assert_eq!(hardware_scroll(&r), 480 % 512);
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn camera_follow_with_rendering_off_writes_directly() {
    let r = run(&format!("{} var c: u16;
        fn main() {{ draw_map(W); for i in 0..60 {{ camera_follow(400, W); }} c = camera_x(); }}", wide_map("W", 96)));
    assert_eq!(r.u16("c"), 280);
    assert_visible_columns(&r, 280, 96, "rendering off");
}

#[test]
fn narrow_maps_never_scroll() {
    let r = run(&format!("{} var c: u16; fn main() {{ draw_map(W); camera_follow(250, W); c = camera_x(); }}", wide_map("W", 32)));
    assert_eq!(r.u16("c"), 0);
}
