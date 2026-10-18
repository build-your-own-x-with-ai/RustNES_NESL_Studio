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
