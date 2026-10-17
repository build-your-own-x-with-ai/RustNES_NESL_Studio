//! 003: game development API — verified on the test machine, including that no VRAM
//! write ever happens while the picture is being drawn.
mod common;
use common::*;

fn conflicts(r: &Run) -> u64 {
    r.nes.bus.render_conflicts
}

// ---------------------------------------------------------------- US1 VRAM queue

#[test]
fn print_anywhere_while_rendering_is_safe() {
    let r = run_frames("
        var n: u8; var spin: u16;
        fn main() {
            ppu_on();
            loop {
                spin = 0;
                while spin < 300 { spin += 1; }   // land mid-frame
                print(2, 5, \"SCORE\");
                print_u8(8, 5, n);
                set_tile(30, 1, 'X');
                n += 1;
                wait_vblank();
            }
        }", 12, |_| {});
    assert_eq!(conflicts(&r), 0);
    assert_eq!(r.vram(0x2000 + 5 * 32 + 2, 5), b"SCORE".to_vec());
    assert_eq!(r.vram(0x2000 + 32 + 30, 1), b"X".to_vec());
    let shown: String = r.vram(0x2000 + 5 * 32 + 8, 3).iter().map(|b| *b as char).collect();
    let n: u32 = shown.parse().unwrap();
    assert!(n + 2 >= r.u8("n") as u32 && n <= r.u8("n") as u32, "shown {n}, n = {}", r.u8("n"));
}

#[test]
fn queue_overflow_waits_instead_of_dropping() {
    let r = run_frames("
        fn main() {
            ppu_on();
            for row in 0..24 { print(0, row, \"ABCDEFGHIJKLMNOPQRSTUVWXYZ012345\"); }
            print(0, 25, \"DONE\");
            loop { wait_vblank(); }
        }", 40, |_| {});
    assert_eq!(conflicts(&r), 0);
    for row in 0..24u16 {
        assert_eq!(r.vram(0x2000 + row * 32, 32), b"ABCDEFGHIJKLMNOPQRSTUVWXYZ012345".to_vec(), "row {row}");
    }
    assert_eq!(r.vram(0x2000 + 25 * 32, 4), b"DONE".to_vec());
}

#[test]
fn ppu_off_flushes_pending_writes() {
    let r = run("fn main() { ppu_on(); print(1, 1, \"KEEP\"); ppu_off(); }");
    assert_eq!(r.vram(0x2021, 4), b"KEEP".to_vec());
    assert_eq!(conflicts(&r), 0);
}

#[test]
fn palette_changes_while_rendering_go_through_the_queue() {
    let r = run_frames("
        const A: [u8] = [0x0F, 0x11, 0x12, 0x13];
        const B: [u8] = [0x0F, 0x21, 0x22, 0x23];
        fn main() {
            load_palette(A);
            ppu_on();
            loop { wait_vblank(); load_palette(B); }
        }", 6, |_| {});
    assert_eq!(conflicts(&r), 0);
    assert_eq!(r.vram(0x3F00, 4), vec![0x0F, 0x21, 0x22, 0x23]);
}

#[test]
fn rendering_off_writes_are_immediate() {
    let r = run("fn main() { print(3, 3, \"NOW\"); set_tile(0, 0, 7); }");
    assert_eq!(r.vram(0x2063, 3), b"NOW".to_vec());
    assert_eq!(r.vram(0x2000, 1), vec![7]);
}

#[test]
fn bundled_examples_never_write_vram_while_rendering() {
    for name in ["hello", "sprite", "game", "shooter"] {
        let path = format!("{}/../../examples/{name}.nesl", env!("CARGO_MANIFEST_DIR"));
        let r = run_frames(&std::fs::read_to_string(path).unwrap(), 200, |nes| nes.set_buttons(0, 0x81));
        assert_eq!(conflicts(&r), 0, "{name}");
    }
}

// ---------------------------------------------------------------- US2 map

const LEVEL: &str = "map L = [
    \"################################\",
    \"#..............................#\",
    \"#....@.........*\",
    \"####\",
];";

#[test]
fn draw_map_fills_the_nametable_and_clears_attributes() {
    let r = run(&format!("{LEVEL} fn main() {{ vram_addr(0x23C0); vram_fill(0xFF, 64); draw_map(L); }}"));
    let nt = r.vram(0x2000, 960);
    assert_eq!(&nt[..32], &[b'#'; 32]);
    assert_eq!(nt[32], b'#');
    assert_eq!(nt[33], b' ', "`.` is drawn as a space");
    assert_eq!(nt[64 + 5], b'@');
    assert_eq!(nt[64 + 15], b'*');
    assert_eq!(nt[64 + 16], b' ', "short rows are padded");
    assert_eq!(&nt[96..100], b"####");
    assert!(nt[128..].iter().all(|b| *b == b' '), "missing rows are blank");
    assert!(r.vram(0x23C0, 64).iter().all(|b| *b == 0));
}

#[test]
fn draw_map_while_rendering_does_not_glitch() {
    let r = run_frames(&format!("{LEVEL} var done: u8;
        fn main() {{ ppu_on(); wait_vblank(); draw_map(L); done = 1; loop {{ wait_vblank(); }} }}"), 10, |_| {});
    assert_eq!(r.u8("done"), 1);
    assert_eq!(r.nes.bus.render_conflicts, 0);
    assert_eq!(r.nes.bus.ppumask, 0x1E, "rendering is turned back on");
    assert_eq!(r.vram(0x2000 + 64 + 5, 1), b"@".to_vec());
}

#[test]
fn map_tile_reads_cells_and_returns_zero_out_of_bounds() {
    let r = run(&format!("{LEVEL} var a: u8; var b: u8; var c: u8; var d: u8; var e: u8; var x: u8 = 5;
        fn main() {{ a = map_tile(L, x, 2); b = map_tile(L, 1, 1); c = map_tile(L, 32, 0); d = map_tile(L, 0, 30); e = map_tile(L, 31, 29); }}"));
    assert_eq!((r.u8("a"), r.u8("b"), r.u8("c"), r.u8("d"), r.u8("e")), (b'@', b' ', 0, 0, b' '));
}

#[test]
fn map_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    let long_row = format!("map M = [\"{}\"]; fn main() {{}}", "#".repeat(256));
    assert!(m(&long_row).contains("255"));
    let many_rows = format!("map M = [{}]; fn main() {{}}", vec!["\"#\""; 31].join(","));
    assert!(m(&many_rows).contains("30"));
    assert!(m("const P: [u8] = [1]; fn main() { draw_map(P); }").contains("map"));
}

// ---------------------------------------------------------------- US3 metasprite

fn oam_shadow(r: &Run, slot: u16) -> [u8; 4] {
    let a = 0x0200 + slot * 4;
    [r.nes.peek(a), r.nes.peek(a + 1), r.nes.peek(a + 2), r.nes.peek(a + 3)]
}

#[test]
fn metasprite_lays_out_pieces_and_returns_next_slot() {
    let r = run("
        metasprite HERO = [(0, 0, 1), (8, 0, 2), (0, 8, 3), (8, 8, 4, 0x40)];
        var next: u8; var x: u8 = 100;
        fn main() { next = metasprite(2, x, 50, HERO); }");
    assert_eq!(r.u8("next"), 6);
    // OAM y = requested y - 1 (the PPU draws sprites one line lower)
    assert_eq!(oam_shadow(&r, 2), [49, 1, 0, 100]);
    assert_eq!(oam_shadow(&r, 3), [49, 2, 0, 108]);
    assert_eq!(oam_shadow(&r, 4), [57, 3, 0, 100]);
    assert_eq!(oam_shadow(&r, 5), [57, 4, 0x40, 108]);
}

#[test]
fn metasprite_negative_offsets_and_offscreen_pieces() {
    let r = run("
        metasprite M = [(-8, 0, 1), (0, -8, 2), (8, 0, 3)];
        fn main() { metasprite(0, 100, 100, M); metasprite(3, 4, 4, M); metasprite(6, 250, 20, M); }");
    assert_eq!(oam_shadow(&r, 0), [99, 1, 0, 92]);
    assert_eq!(oam_shadow(&r, 1), [91, 2, 0, 100]);
    assert_eq!(oam_shadow(&r, 3)[0], 0xFF, "x = 4 - 8 wraps: hidden");
    assert_eq!(oam_shadow(&r, 4)[0], 0xFF, "y = 4 - 8 wraps: hidden");
    assert_eq!(oam_shadow(&r, 5), [3, 3, 0, 12]);
    assert_eq!(oam_shadow(&r, 8)[0], 0xFF, "x = 250 + 8 wraps: hidden");
    assert_eq!(oam_shadow(&r, 6), [19, 1, 0, 242]);
}

#[test]
fn metasprite_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("metasprite M = [(200, 0, 1)]; fn main() {}").contains("does not fit"));
    assert!(m("const P: [u8] = [1]; fn main() { metasprite(0, 0, 0, P); }").contains("metasprite"));
}

// ---------------------------------------------------------------- US4 collide

#[test]
fn collide_handles_overlap_touching_containment_and_255_edge() {
    let r = run("
        var c: [u8; 8]; var calls: u8;
        fn n(v: u8) -> u8 { calls += 1; return v; }
        fn main() {
            c[0] = collide(10, 10, 8, 8, 14, 14, 8, 8);
            c[1] = collide(10, 10, 8, 8, 18, 10, 8, 8);
            c[2] = collide(10, 10, 8, 8, 10, 18, 8, 8);
            c[3] = collide(0, 0, 100, 100, 40, 40, 8, 8);
            c[4] = collide(250, 10, 16, 8, 2, 10, 8, 8);
            c[5] = collide(250, 10, 16, 8, 252, 12, 8, 8);
            c[6] = collide(n(10), n(10), n(8), n(8), n(12), n(12), n(4), n(4));
            if collide(1, 1, 1, 1, 5, 5, 1, 1) { c[7] = 9; }
        }");
    assert_eq!(r.bytes("c"), vec![1, 0, 0, 1, 0, 1, 1, 0]);
    assert_eq!(r.u8("calls"), 8, "each argument is evaluated exactly once");
}

// ---------------------------------------------------------------- US5 sfx

fn writes_to(r: &Run, reg: u16) -> Vec<u8> {
    r.nes.bus.apu_writes.iter().filter(|(a, _)| *a == reg).map(|(_, v)| *v).collect()
}

#[test]
fn sfx_jump_slides_pitch_fades_and_stops() {
    let r = run_frames("fn main() { ppu_on(); sfx(SFX_JUMP); loop { wait_vblank(); } }", 40, |_| {});
    let vols: Vec<u8> = writes_to(&r, 0x4004).iter().map(|v| v & 0x0F).collect();
    assert!(vols.len() >= 10, "{vols:?}");
    assert!(vols.windows(2).all(|w| w[1] <= w[0]), "volume never rises: {vols:?}");
    let lows = writes_to(&r, 0x4006);
    assert!(lows.len() >= 10 && lows.windows(2).all(|w| w[1] < w[0]), "pitch rises (period falls): {lows:x?}");
    let en = writes_to(&r, 0x4015);
    assert_eq!(en.first().map(|v| v & 0x02), Some(0x02), "pulse 2 enabled: {en:x?}");
    assert_eq!(en.last().map(|v| v & 0x02), Some(0), "and disabled when done: {en:x?}");
}

#[test]
fn sfx_explode_uses_the_noise_channel() {
    let r = run_frames("fn main() { ppu_on(); sfx(SFX_EXPLODE); loop { wait_vblank(); } }", 5, |_| {});
    assert!(!writes_to(&r, 0x400C).is_empty());
    assert!(writes_to(&r, 0x4015).iter().any(|v| v & 0x08 != 0));
    assert!(writes_to(&r, 0x4004).is_empty(), "pulse 2 untouched");
}

#[test]
fn sfx_and_play_tone_play_together_and_stop_sound_stops_both() {
    let r = run_frames("var t: u8;
        fn main() { ppu_on(); play_tone(0x1AB); sfx(SFX_COIN);
            loop { wait_vblank(); t += 1; if t == 3 { stop_sound(); } } }", 8, |_| {});
    let en = writes_to(&r, 0x4015);
    assert!(en.iter().any(|v| v & 0x03 == 0x03), "both channels on at once: {en:x?}");
    assert_eq!(en.last(), Some(&0), "{en:x?}");
    let after_stop = r.nes.bus.apu_writes.iter().rposition(|w| *w == (0x4015, 0)).unwrap();
    assert!(!r.nes.bus.apu_writes[after_stop..].iter().any(|(a, _)| *a == 0x4004), "sfx stopped too");
}

#[test]
fn every_sfx_constant_compiles() {
    build("fn main() { sfx(SFX_JUMP); sfx(SFX_COIN); sfx(SFX_SHOOT); sfx(SFX_HIT); sfx(SFX_EXPLODE); }");
}

#[test]
fn a_bad_map_reports_one_error_not_one_per_use() {
    let errs = errors(&format!("map M = [\"{}\"]; fn main() {{ draw_map(M); draw_map(M); }}", "#".repeat(256)));
    assert_eq!(errs.len(), 1, "{errs:?}");
}
