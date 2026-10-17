//! US2: built-in NES API, verified against the test machine's PPU/OAM/APU state.
mod common;
use common::*;

const PAL: &str = "const PAL: [u8] = [
    0x0F,0x30,0x16,0x27, 0x0F,0x01,0x02,0x03, 0x0F,0x04,0x05,0x06, 0x0F,0x07,0x08,0x09,
    0x0F,0x11,0x12,0x13, 0x0F,0x14,0x15,0x16, 0x0F,0x17,0x18,0x19, 0x0F,0x1A,0x1B,0x1C];";

#[test]
fn load_palette_writes_palette_ram() {
    let r = run(&format!("{PAL} fn main() {{ load_palette(PAL); }}"));
    assert_eq!(r.vram(0x3F00, 16), vec![0x0F, 0x30, 0x16, 0x27, 0x0F, 1, 2, 3, 0x0F, 4, 5, 6, 0x0F, 7, 8, 9]);
    assert_eq!(r.vram(0x3F11, 3), vec![0x11, 0x12, 0x13]);
    assert_eq!(r.vram(0x3F1D, 3), vec![0x1A, 0x1B, 0x1C]);
}

#[test]
fn print_text_and_numbers_to_the_nametable() {
    let r = run("const MSG: [u8] = \"SCORE\"; var v: u8 = 7; var w: u8 = 255;
        fn main() { print(2, 3, \"HELLO\"); print(0, 29, MSG); print_u8(10, 0, v); print_u8(14, 0, w); print_u8(18, 0, 42); }");
    assert_eq!(r.vram(0x2000 + 3 * 32 + 2, 5), b"HELLO".to_vec());
    assert_eq!(r.vram(0x2000 + 29 * 32, 5), b"SCORE".to_vec());
    assert_eq!(r.vram(0x200A, 3), b"007".to_vec());
    assert_eq!(r.vram(0x200E, 3), b"255".to_vec());
    assert_eq!(r.vram(0x2012, 3), b"042".to_vec());
}

#[test]
fn vram_addr_put_and_fill() {
    let r = run("var a: u16 = 0x2100;
        fn main() { vram_addr(a); vram_put(1); vram_put(2); vram_fill(9, 300); }");
    let v = r.vram(0x2100, 303);
    assert_eq!(&v[..2], &[1, 2]);
    assert!(v[2..302].iter().all(|b| *b == 9));
    assert_eq!(v[302], 0);
}

#[test]
fn sprites_reach_oam_through_nmi_dma() {
    let r = run_frames("var sx: u8 = 100;
        fn main() {
            ppu_on();
            sprite(1, sx, 50, 3, 0x41);
            sprite(63, 1, 2, 3, 4);
            hide_sprite(0);
            loop { wait_vblank(); }
        }", 5, |_| {});
    // OAM y is one less than the requested y (the PPU draws sprites one line lower)
    assert_eq!(&r.nes.bus.oam[4..8], &[49, 3, 0x41, 100]);
    assert_eq!(&r.nes.bus.oam[252..256], &[1, 3, 4, 1]);
    assert_eq!(r.nes.bus.oam[0], 0xFF);
    assert_eq!(r.nes.bus.ppumask, 0x1E);
}

#[test]
fn joypad_input_drives_game_state() {
    let r = run_frames("var x: u8; var a2: u8; var a1: u8;
        fn main() {
            ppu_on();
            loop {
                wait_vblank();
                if joypad(0) & BTN_RIGHT { x += 1; }
                if joypad(0) & BTN_A != 0 { a1 += 1; }
                if joypad(1) & BTN_A { a2 += 1; }
            }
        }", 12, |nes| {
        nes.set_buttons(0, 0x01);
        nes.set_buttons(1, 0x80);
    });
    assert!((9..=12).contains(&r.u8("x")), "x = {}", r.u8("x"));
    assert_eq!(r.u8("a1"), 0);
    assert_eq!(r.u8("a2"), r.u8("x"));
}

#[test]
fn wait_vblank_waits_one_frame_each() {
    let r = run("var f0: u8; var f1: u8; fn main() { f0 = frame_count(); for i in 0..5 { wait_vblank(); } f1 = frame_count(); }");
    assert_eq!(r.u8("f1").wrapping_sub(r.u8("f0")), 5);
}

#[test]
fn rand_has_full_period_of_distinct_nonzero_values() {
    let r = run("var buf: [u8; 255]; fn main() { for i in 0..255 { buf[i] = rand(); } }");
    let mut v = r.bytes("buf");
    v.sort();
    v.dedup();
    assert_eq!(v.len(), 255);
    assert!(!v.contains(&0));
}

#[test]
fn scroll_and_ppu_off() {
    let r = run_frames("fn main() { scroll(12, 34); ppu_on(); wait_vblank(); wait_vblank(); ppu_off(); loop {} }", 6, |_| {});
    assert_eq!(r.nes.bus.scroll, (12, 34));
    assert_eq!(r.nes.bus.ppumask, 0);
}

#[test]
fn peek_and_poke() {
    let r = run("var p: u16 = 0x07F1; var a: u8; var b: u8;
        fn main() { poke(0x07F0, 77); poke(p, 88); a = peek(0x07F0); b = peek(p) + 1; }");
    assert_eq!(r.nes.peek(0x07F0), 77);
    assert_eq!(r.nes.peek(0x07F1), 88);
    assert_eq!((r.u8("a"), r.u8("b")), (77, 89));
}

#[test]
fn play_tone_and_stop_sound_program_the_apu() {
    let r = run("fn main() { play_tone(0x1AB); stop_sound(); }");
    let w = &r.nes.bus.apu_writes;
    assert!(w.contains(&(0x4015, 0x01)), "{w:x?}");
    assert!(w.contains(&(0x4002, 0xAB)), "{w:x?}");
    assert!(w.contains(&(0x4003, 0x01)), "{w:x?}");
    assert_eq!(w.last(), Some(&(0x4015, 0x00)));
}

#[test]
fn chr_contains_font_and_user_tiles() {
    let out = build("tile 1 = [
        \"..3333..\", \".1....2.\", \"........\", \"........\",
        \"........\", \"........\", \"........\", \"33333333\"];
        fn main() {}");
    let chr = &out.rom[16 + 32768..];
    let t1 = &chr[16..32];
    assert_eq!(t1[0], 0b0011_1100, "plane 0 row 0");
    assert_eq!(t1[8], 0b0011_1100, "plane 1 row 0");
    assert_eq!((t1[1], t1[9]), (0b0100_0000, 0b0000_0010));
    assert_eq!((t1[7], t1[15]), (0xFF, 0xFF));
    let a = &chr[0x41 * 16..0x42 * 16];
    assert!(a[..8].iter().any(|b| *b != 0), "glyph A has pixels");
    assert!(a[8..].iter().all(|b| *b == 0), "font uses color 1 only");
    assert_eq!(&chr[0x1000..0x2000], &chr[..0x1000], "pattern table 1 mirrors table 0");
}

#[test]
fn builtin_misuse_is_reported() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("fn main() { load_palette(5); }").contains("constant array"));
    assert!(m("var a: u8; fn main() { print(1, 2, a); }").contains("string"));
    assert!(m("fn main() { joypad(); }").contains("expects 1 argument"));
    assert!(m("fn main() { let r = wait_vblank(); }").contains("does not return a value"));
    assert!(m("tile 1 = [\"...\"]; fn main() {}").contains("8 rows"));
    assert!(m("tile 1 = [\"..x.....\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"]; fn main() {}").contains("pixel"));
    assert!(m("fn sprite() {} fn main() {}").contains("built-in"));
}

#[test]
fn sprites_appear_exactly_at_the_given_y() {
    // tile 1 is solid; sprite palette 0 colour 1 = $16. The NES shows a sprite one line
    // below its OAM y, so the runtime must compensate for `sprite(i, x, y, ...)` to mean y.
    let r = run_frames("
        const PAL: [u8] = [0x0F,0x30,0x30,0x30, 0x0F,0x30,0x30,0x30, 0x0F,0x30,0x30,0x30, 0x0F,0x30,0x30,0x30,
                           0x0F,0x16,0x16,0x16, 0x0F,0x16,0x16,0x16, 0x0F,0x16,0x16,0x16, 0x0F,0x16,0x16,0x16];
        tile 1 = [\"11111111\", \"11111111\", \"11111111\", \"11111111\", \"11111111\", \"11111111\", \"11111111\", \"11111111\"];
        metasprite M = [(0, 0, 1), (0, 8, 1)];
        fn main() { load_palette(PAL); ppu_on(); sprite(1, 100, 50, 1, 0); metasprite(2, 120, 60, M); loop { wait_vblank(); } }",
        4, |_| {});
    let f = nes_sim::render::frame(&r.nes.bus, r.nes.frames - 1);
    let px = |x: usize, y: usize| f[y * 256 + x];
    assert_eq!((px(100, 49), px(100, 50), px(100, 57), px(100, 58)), (0x0F, 0x16, 0x16, 0x0F), "sprite rows 50-57");
    assert_eq!((px(120, 59), px(120, 60), px(120, 75), px(120, 76)), (0x0F, 0x16, 0x16, 0x0F), "metasprite rows 60-75");
}
