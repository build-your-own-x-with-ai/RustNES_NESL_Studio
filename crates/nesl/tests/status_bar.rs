//! 006 US2: a fixed status bar via a sprite-0 split, checked against the test machine's
//! sprite-0 hit model and its log of PPUCTRL / PPUSCROLL writes.
mod common;
use common::*;
use nes_sim::nes::{pixel_cycle, CYCLES_PER_FRAME};

const PROG: &str = "
    var hero: u16; var target: u16; var cam: u16; var score: u8;
    fn main() {
        draw_map(W);
        status_bar(4);
        print(1, 1, \"SCORE\");
        ppu_on();
        loop {
            wait_vblank();
            if hero < target { hero += 3; } else if hero > target { hero -= 3; }
            camera_follow(hero, W);
            cam = camera_x();
            score += 1;
            print_u8(8, 1, score);
        }
    }";

fn set16(r: &mut Run, name: &str, v: u16) {
    let a = r.addr(name) as usize;
    r.nes.bus.ram[a] = v as u8;
    r.nes.bus.ram[a + 1] = (v >> 8) as u8;
}

/// `(cycle, kind, value)` entries from the test machine's register log.
type Log = Vec<(u64, u8, u8)>;

/// Register writes made during frame `k`, split at the start of the visible picture.
fn frame_writes(r: &Run, k: u64) -> (Log, Log) {
    let start = k * CYCLES_PER_FRAME;
    let vis = start + pixel_cycle(0, 0);
    let all: Vec<_> = r.nes.bus.reg_log.iter().copied().filter(|(c, _, _)| (start..start + CYCLES_PER_FRAME).contains(c)).collect();
    all.into_iter().partition(|(c, _, _)| *c < vis)
}

fn last(writes: &[(u64, u8, u8)], kind: u8) -> u8 {
    writes.iter().rev().find(|(_, k, _)| *k == kind).expect("write").2
}

#[test]
fn every_frame_shows_the_status_bar_then_splits_to_the_camera() {
    let width = 128;
    let mut r = run_frames(&format!("{}{PROG}", wide_map("W", width)), 6, |_| {});
    set16(&mut r, "target", 1020);
    for i in 0..300 {
        r.nes.run_frames(1).unwrap();
        let k = r.nes.frames - 1;
        let start = k * CYCLES_PER_FRAME;
        let (before, after) = frame_writes(&r, k);
        assert_eq!((last(&before, 0) & 1, last(&before, 1)), (0, 0), "frame {k}: status bar unscrolled");
        let hit = r.nes.bus.sprite0_at.expect("sprite 0 hit every frame");
        assert_eq!(hit, start + pixel_cycle(248, 30), "hit on the status bar's last row");
        assert_eq!(after.len(), 2, "exactly the split writes after rendering starts: {after:?}");
        let limit = start + pixel_cycle(0, 32);
        assert!(after.iter().all(|(c, _, _)| *c > hit && *c < limit), "split between the hit and line 32: {after:?}");
        let shown = ((last(&after, 0) & 1) as u16) << 8 | last(&after, 1) as u16;
        let cam = r.u16("cam");
        let world = (cam.saturating_sub(16)..=cam + 16).find(|w| w % 512 == shown).expect("scroll near camera");
        if i % 4 == 0 {
            let first = (world / 8) as usize;
            for c in first..(first + 33).min(width) {
                for row in 4..30 {
                    assert_eq!(nt(&r, c % 64, row), cell(c, row), "column {c} row {row}");
                }
            }
        }
    }
    assert_eq!(r.u16("cam"), 768);
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn status_rows_keep_their_text_while_the_level_scrolls() {
    let mut r = run_frames(&format!("{}{PROG}", wide_map("W", 128)), 6, |_| {});
    set16(&mut r, "target", 1020);
    r.nes.run_frames(320).unwrap();
    set16(&mut r, "target", 0);
    r.nes.run_frames(320).unwrap();
    assert_eq!(r.vram(0x2020 + 1, 5), b"SCORE".to_vec());
    let digits: Vec<u8> = r.vram(0x2020 + 8, 3);
    assert!(digits.iter().all(u8::is_ascii_digit), "{digits:?}");
    assert_eq!(r.vram(0x2000 + 3 * 32 + 31, 1), vec![0x7F], "sprite-0 marker");
    for row in 0..4u16 {
        assert!(r.vram(0x2400 + row * 32, 32).iter().all(|b| *b == b' '), "second nametable row {row} cleared");
    }
    for c in [0u16, 5, 20] {
        assert_eq!(r.vram(0x2000 + c, 1), vec![b' '], "row 0 column {c} untouched by streaming");
    }
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn status_bar_0_turns_the_split_off() {
    let mut r = run_frames(&format!("{} var t: u8;
        fn main() {{ draw_map(W); status_bar(2); ppu_on(); loop {{ wait_vblank(); t += 1; if t == 8 {{ status_bar(0); }} }} }}",
        wide_map("W", 64)), 0, |_| {});
    // start-up (drawing the map with rendering off) takes a few frames: wait for the loop
    let mut guard = 0;
    while r.u8("t") < 3 {
        r.nes.run_frames(1).unwrap();
        guard += 1;
        assert!(guard < 30, "main loop never started");
    }
    let k = r.nes.frames - 1;
    assert_eq!(r.nes.bus.sprite0_at, Some(k * CYCLES_PER_FRAME + pixel_cycle(248, 14)), "split on the 2-row bar");
    assert_eq!(frame_writes(&r, k).1.len(), 2);
    while r.u8("t") < 11 {
        r.nes.run_frames(1).unwrap();
    }
    let k = r.nes.frames - 1;
    assert_eq!(r.nes.bus.sprite0_at, None);
    assert!(frame_writes(&r, k).1.is_empty());
    assert_eq!(r.nes.peek(0x0200), 0xFF, "sprite 0 released");
}

#[test]
fn a_missing_marker_does_not_hang_the_game() {
    let r = run_frames(&format!("{} var t: u8;
        tile 0x7F = [\"........\", \"........\", \"........\", \"........\", \"........\", \"........\", \"........\", \"........\"];
        fn main() {{ draw_map(W); status_bar(3); ppu_on(); loop {{ wait_vblank(); t += 1; }} }}", wide_map("W", 32)), 30, |_| {});
    assert_eq!(r.nes.bus.sprite0_at, None);
    assert!(r.u8("t") >= 25, "main loop kept running: {}", r.u8("t"));
}

#[test]
fn status_bar_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("fn main() { status_bar(1); }").contains("2 to 8"));
    assert!(m("fn main() { status_bar(9); }").contains("2 to 8"));
    assert!(m("var n: u8; fn main() { status_bar(n); }").contains("constant"));
}
