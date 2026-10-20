//! 010: compiler automatically switches to UNROM banking when code+data exceed 32KB.
mod common;
use common::*;

fn big_maps() -> String {
    // 9 maps of 128 columns = ~36KB of map data, forcing UNROM
    let mut s = String::from("const MAPS: [map] = [");
    for i in 0..9 {
        s.push_str(&format!("M{},", i));
    }
    s.push_str("];\n");
    for i in 0..9 {
        s.push_str(&wide_map(&format!("M{i}"), 128));
    }
    s
}

#[test]
fn large_programs_use_unrom_mapper2() {
    let src = format!("{} var a: u8; var b: u8; var c: u8; var d: u8; var e: u8; var f: u8; var g: u8; var h: u8; var j: u8;
        fn main() {{
            draw_map(MAPS[0]);
            a = map_tile(M0, 50, 2);
            b = map_tile(M8, 50, 2);
            c = map_tile(MAPS[1], 50, 2);
            d = map_tile(MAPS[8], 50, 2);
            camera_follow(1000, M7);
            e = camera_x() as u8;
            for i in 0..20 {{ camera_follow(1000, MAPS[i % 9]); }}
            f = camera_x() as u8;
            set_map_tile(M3, 10, 5, '#');
            g = map_tile(M3, 10, 5);
            reset_map(MAPS[4]);
            h = map_tile(M4, 10, 5);
            j = map_tile(M4, 11, 5);
        }}", big_maps());
    let out = nesl::compile(&src).unwrap();
    assert_eq!(out.mapper, 2, "mapper 2 (UNROM)");
    assert_eq!(out.rom[5], 0, "CHR count = 0 (CHR-RAM)");
    assert_eq!(out.rom[6] >> 4, 2, "mapper low nibble");
    
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(10).unwrap();
    
    let v = |n: &str| out.vars[n].addr;
    assert_eq!(nes.peek(v("a")), cell(50, 2), "map_tile M0");
    assert_eq!(nes.peek(v("b")), cell(50, 2), "map_tile M8");
    assert_eq!(nes.peek(v("c")), cell(50, 2), "map_tile MAPS[1]");
    assert_eq!(nes.peek(v("d")), cell(50, 2), "map_tile MAPS[8]");
    assert!(nes.peek(v("e")) > 0, "camera_follow M7 moved the camera");
    assert!(nes.peek(v("f")) > 0, "camera_follow MAPS[i] loop");
    assert_eq!(nes.peek(v("g")), b'#', "set_map_tile M3");
    assert_eq!(nes.peek(v("h")), cell(10, 5), "reset_map cleared the change");
    assert_eq!(nes.peek(v("j")), cell(11, 5));
    
    assert_eq!(nes.bus.bus_conflicts, 0, "no bus conflicts");
    assert_eq!(nes.bus.render_conflicts, 0);
}

#[test]
fn small_programs_still_use_nrom() {
    let out = nesl::compile("fn main() {}").unwrap();
    assert_eq!(out.mapper, 0, "mapper 0 (NROM)");
}

#[test]
fn chr_data_is_copied_into_vram() {
    let src = format!("{} tile 5 = [\"11111111\",\"22222222\",\"33333333\",\"........\",\"11111111\",\"22222222\",\"33333333\",\"........\"];
        fn main() {{ draw_map(M0); }}", big_maps());
    let out = nesl::compile(&src).unwrap();
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(10).unwrap();
    // tile 5's pattern should be in CHR (PPU $0000-$1FFF)
    assert_eq!(nes.bus.vram[5 * 16], 0xFF, "tile 5 plane 0 row 0");
    assert_eq!(nes.bus.vram[5 * 16 + 8], 0x00, "tile 5 plane 1 row 0");
}

#[test]
fn bank_counts_that_need_padding_keep_the_fixed_bank_last() {
    // 11 maps -> 4 switchable banks + fixed = 5, padded to 8 PRG banks
    let mut src = String::new();
    for i in 0..11 {
        src.push_str(&wide_map(&format!("M{i}"), 128));
    }
    src.push_str("var a: u8; var b: u8; fn main() { draw_map(M10); a = map_tile(M10, 50, 2); b = map_tile(M0, 7, 3); }");
    let out = nesl::compile(&src).unwrap();
    assert_eq!(out.mapper, 2);
    assert_eq!(out.rom[4], 8, "PRG banks rounded up to a power of two");
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(10).unwrap();
    assert_eq!(nes.peek(out.vars["a"].addr), cell(50, 2), "reset vector found: the program ran");
    assert_eq!(nes.peek(out.vars["b"].addr), cell(7, 3));
    assert_eq!(nes.bus.bus_conflicts, 0);
}

#[test]
fn programs_up_to_32kb_of_prg_stay_nrom() {
    // 6 wide maps (~24KB) + code: more than 24KB but under 32KB of PRG
    let mut src = String::new();
    for i in 0..6 {
        src.push_str(&wide_map(&format!("M{i}"), 128));
    }
    src.push_str("fn main() { draw_map(M5); }");
    let out = nesl::compile(&src).unwrap();
    assert!(out.prg_used > 24 * 1024, "{}", out.prg_used);
    assert_eq!(out.mapper, 0, "CHR is a separate chip in NROM; only PRG counts");
}

#[test]
fn unrom_roms_use_vertical_mirroring_for_horizontal_scrolling() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/quest.nesl")).unwrap();
    let out = nesl::compile(&src).unwrap();
    assert_eq!(out.mapper, 2);
    assert_eq!(out.rom[6] & 0x01, 0x01, "flags6 bit 0 must be set (vertical mirroring)");
}

/// Regression for the FCEUX bug: with the wrong mirroring the streamed columns land in
/// the visible nametable, so the screen stops matching the map while collision still
/// reads the real map. Every visible column must hold the map's real tiles.
#[test]
fn scrolling_an_unrom_level_keeps_the_screen_in_sync_with_the_map() {
    let src = format!("{} var hero: u16; var cam: u16;
        fn main() {{
            draw_map(MAPS[5]);
            ppu_on();
            loop {{
                wait_vblank();
                if hero < 700 {{ hero += 3; }}
                camera_follow(hero, MAPS[5]);
                cam = camera_x();
            }}
        }}", big_maps());
    let r = run_frames(&src, 300, |_| {});
    assert_eq!(r.out.mapper, 2);
    let cam = r.u16("cam") as usize;
    assert!(cam > 300, "camera should have scrolled, cam={cam}");
    for c in cam / 8..cam / 8 + 32 {
        for row in 0..30 {
            assert_eq!(nt(&r, c % 64, row), cell(c, row), "map column {c} row {row} (cam={cam})");
        }
    }
}
