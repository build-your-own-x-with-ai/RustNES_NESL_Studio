//! 013: `fn irq()` builds an MMC3 ROM; `scanline_irq(n)` fires it once per frame at line n,
//! and `split_scroll(x)` changes the horizontal scroll from there down.
mod common;
use common::*;
use nes_sim::nes::CYCLES_PER_FRAME;
use nes_sim::render;

const PAL: &str = "const PAL: [u8] = [0x0F, 0x30, 0x16, 0x27, 0x0F, 0x21, 0x31, 0x11, 0x0F, 0x29, 0x19, 0x09, 0x0F, 0x26, 0x16, 0x06];";

#[test]
fn programs_with_fn_irq_build_as_mmc3() {
    let out = build("var n: u8; fn irq() { n += 1; } fn main() { }");
    assert_eq!(out.mapper, 4);
    assert_eq!(&out.rom[4..8], &[2, 1, 0x43, 0x00], "32KB PRG, 8KB CHR, mapper 4, battery, vertical");
    let chr = &out.rom[16 + 0x8000..];
    assert_eq!(chr[..0x1000], chr[0x1000..0x2000], "sprite table at $1000 holds the same tiles");
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_until_pc(out.symbols["__halt"], 2_000_000).unwrap();
}

#[test]
fn programs_without_fn_irq_stay_nrom() {
    assert_eq!(build("fn main() { }").mapper, 0);
}

#[test]
fn scanline_irq_fires_once_per_frame_near_the_requested_line() {
    let src = "var n: u8; var t: u8;
        fn irq() { n += 1; }
        fn main() { scanline_irq(100); ppu_on(); loop { wait_vblank(); t += 1; } }";
    let mut r = run_frames(src, 2, |_| {});
    let irq = r.out.symbols["f_irq"];
    r.nes.run_until_pc(irq, 2 * CYCLES_PER_FRAME).unwrap();
    let at = r.nes.cpu.cycles % CYCLES_PER_FRAME;
    let line = (at.saturating_sub(nes_sim::nes::VBLANK_CYCLES) * 3 / 341) as i64 - 1;
    assert!((97..=101).contains(&line), "irq() entered on line {line}, expected just before 100");
    let (n0, t0) = (r.u8("n"), r.u8("t"));
    r.nes.run_frames(60).unwrap();
    let (dn, dt) = (r.u8("n").wrapping_sub(n0), r.u8("t").wrapping_sub(t0));
    assert!(dn.abs_diff(dt) <= 1 && (59..=61).contains(&dn), "{dn} IRQs in {dt} frames");
}

#[test]
fn scanline_irq_zero_turns_it_off() {
    let src = "var n: u8;
        fn irq() { n += 1; }
        fn main() { scanline_irq(50); ppu_on(); wait_vblank(); wait_vblank(); scanline_irq(0); loop { wait_vblank(); } }";
    let mut r = run_frames(src, 10, |_| {});
    let n = r.u8("n");
    r.nes.run_frames(10).unwrap();
    assert_eq!(r.u8("n"), n, "no IRQs after scanline_irq(0)");
}

#[test]
fn irq_does_not_disturb_the_main_program() {
    // main multiplies in a tight loop while irq() multiplies too (shared runtime scratch)
    let src = "var i: u8; var r: u8; var bad: u8; var k: u8; var n: u8;
        fn irq() { k = k * 5 + 1; n += 1; }
        fn main() {
            scanline_irq(60);
            ppu_on();
            loop { i += 1; r = i * 3; if r != i * 3 { bad += 1; } }
        }";
    let r = run_frames(src, 30, |_| {});
    assert!(r.u8("n") >= 25, "IRQ ran ({} times)", r.u8("n"));
    assert_eq!(r.u8("bad"), 0, "main's arithmetic was corrupted by the IRQ");
}

fn picture(src: &str) -> Vec<u8> {
    let r = run_frames(src, 20, |_| {});
    render::frame(&r.nes.bus, r.nes.frames - 1)
}

#[test]
fn split_scroll_scrolls_the_picture_below_the_irq_line() {
    let body = |irq: &str, setup: &str| {
        format!(
            "{PAL} {} {irq}
            fn main() {{ load_palette(PAL); draw_map(W); {setup} ppu_on(); loop {{ wait_vblank(); }} }}",
            wide_map("W", 64)
        )
    };
    let flat = picture(&body("", "scroll(0, 0);"));
    let moved = picture(&body("", "scroll(40, 0);"));
    let split = picture(&body("fn irq() { split_scroll(40); }", "scroll(0, 0); scanline_irq(120);"));
    let row = |p: &[u8], y: usize| p[y * 256..(y + 1) * 256].to_vec();
    for y in [0, 50, 117] {
        assert_eq!(row(&split, y), row(&flat, y), "line {y} above the split is unscrolled");
    }
    for y in [122, 160, 239] {
        assert_eq!(row(&split, y), row(&moved, y), "line {y} below the split is scrolled by 40");
    }
    assert!((118..=121).any(|y| row(&split, y) != row(&flat, y)), "the split starts by line 121");
}

#[test]
fn irq_errors() {
    let msg = |src: &str| errors(src)[0].message.clone();
    assert!(msg("fn irq(a: u8) { } fn main() { }").contains("`irq` must take no parameters"));
    assert!(msg("fn f() { } fn irq() { f(); } fn main() { }").contains("cannot call"));
    assert!(msg("fn irq() { } fn main() { irq(); }").contains("cannot be called"));
    assert!(msg("fn main() { split_scroll(3); }").contains("only inside `fn irq()`"));
    assert!(msg("fn main() { scanline_irq(10); }").contains("needs `fn irq()`"));
}
