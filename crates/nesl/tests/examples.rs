//! Every program in examples/ must compile and run without crashing the CPU.
mod common;
use common::*;
use nes_sim::TestNes;

fn example(name: &str) -> String {
    let path = format!("{}/../../examples/{name}.nesl", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn hello_prints_its_message() {
    let r = run_frames(&example("hello"), 10, |_| {});
    assert_eq!(r.vram(0x2000 + 13 * 32 + 11, 10), b"HELLO NES!".to_vec());
    assert_eq!(r.nes.bus.ppumask, 0x1E);
}

#[test]
fn sprite_moves_with_the_dpad() {
    let r = run_frames(&example("sprite"), 20, |nes| nes.set_buttons(0, 0x01 | 0x04));
    assert!(r.u8("x") > 124 && r.u8("y") > 112);
    assert_eq!(r.nes.peek(0x0203), r.u8("x"), "OAM shadow is updated this frame");
    assert_eq!(r.nes.bus.oam[3], r.u8("x") - 1, "PPU OAM lags one frame (DMA happens in NMI)");
}

#[test]
fn game_runs_until_game_over_and_restarts() {
    let out = build(&example("game"));
    let mut nes = TestNes::from_ines(&out.rom).unwrap();
    // Nobody plays: all 3 stars eventually hit the floor.
    nes.run_frames(1200).unwrap();
    let over = out.vars["over"].addr;
    let lives = out.vars["lives"].addr;
    assert_eq!((nes.peek(over), nes.peek(lives)), (1, 0));
    assert_eq!(&nes.bus.vram[0x2000 + 14 * 32 + 11..][..9], b"GAME OVER");
    nes.set_buttons(0, 0x10);
    nes.run_frames(3).unwrap();
    assert_eq!((nes.peek(over), nes.peek(lives)), (0, 3));
}

#[test]
fn shooter_fires_and_moves() {
    let out = build(&example("shooter"));
    let mut nes = TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(5).unwrap();
    nes.set_buttons(0, 0x80 | 0x01);
    nes.run_frames(3).unwrap();
    assert!(nes.peek(out.vars["ship_x"].addr) > 120);
    assert_eq!(nes.peek(out.vars["shot.active"].addr), 1);
}

#[test]
fn maze_hero_walks_collects_coins_and_never_glitches() {
    let out = build(&example("maze"));
    let mut nes = TestNes::from_ines(&out.rom).unwrap();
    let v = |nes: &TestNes, n: &str| nes.peek(out.vars[n].addr);
    // Start-up scans all 960 map cells for coins; wait for it instead of guessing a frame count.
    let mut waited = 0;
    while v(&nes, "left") == 0 {
        nes.run_frames(1).unwrap();
        waited += 1;
        assert!(waited < 60, "coin scan never finished");
    }
    assert_eq!(&nes.bus.vram[0x2000 + 3 * 32..][..32], &[b'#'; 32], "map drawn");
    assert_eq!(v(&nes, "left"), v(&nes, "ncoins"));
    assert!(v(&nes, "ncoins") >= 10);
    // Walk up-left into the top-left coin at tile (1, 4).
    nes.set_buttons(0, 0x08 | 0x02);
    nes.run_frames(30).unwrap();
    assert_eq!(v(&nes, "score"), 1, "coin collected");
    assert_eq!(nes.bus.vram[0x2000 + 4 * 32 + 1], b' ', "coin erased from the screen");
    assert!(v(&nes, "hero.x") >= 8 && v(&nes, "hero.y") >= 32, "walls stop the hero");
    // Wander for a while: whatever happens, VRAM is only written in vblank.
    nes.set_buttons(0, 0x01 | 0x04);
    nes.run_frames(400).unwrap();
    assert_eq!(nes.bus.render_conflicts, 0);
}

#[test]
fn runner_scrolls_through_the_level_without_glitches() {
    let out = build(&example("runner"));
    let mut nes = TestNes::from_ines(&out.rom).unwrap();
    let u16v = |nes: &TestNes, n: &str| {
        let a = out.vars[n].addr;
        nes.peek(a) as u16 | (nes.peek(a + 1) as u16) << 8
    };
    nes.run_frames(10).unwrap();
    let start_y = nes.peek(out.vars["hero.y"].addr);
    assert!(start_y > 120 && start_y < 168, "hero landed on the ground: {start_y}");
    // Run right, jumping now and then (pressing A every 20 frames).
    let mut furthest = 0;
    for f in 0..900u32 {
        nes.set_buttons(0, if f % 20 < 10 { 0x81 } else { 0x01 });
        nes.run_frames(1).unwrap();
        furthest = furthest.max(u16v(&nes, "hero.x"));
    }
    assert!(furthest > 300, "hero ran past the first screen: {furthest}");
    let cam = u16v(&nes, "cam");
    assert_eq!(cam, 128 * 8 - 256, "camera reached the end of the level");
    assert_eq!(nes.peek(out.vars["won"].addr), 1, "the flag was reached (x = {furthest})");
    let w = &nes.bus.apu_writes;
    for (reg, voice) in [(0x4002, "melody"), (0x400A, "bass"), (0x4006, "harmony")] {
        assert!(w.iter().filter(|(a, _)| *a == reg).count() > 20, "{voice} plays");
    }
    let score = nes.peek(out.vars["score"].addr);
    assert!(score > 0, "coins collected");
    // every coin taken is recorded in the map's change list, so none can come back
    assert_eq!(nes.peek(0x039B), score, "one recorded change per coin");
    assert_eq!(&nes.bus.vram[0x2020 + 2..][..5], b"COINS", "status bar survived the whole level");
    assert_eq!(&nes.bus.vram[0x2020 + 20..][..8], b"YOU WIN!");
    assert!(cam > 0, "camera followed");
    assert_eq!(nes.bus.render_conflicts, 0);
}

#[test]
fn runner_hero_stands_on_the_ground_with_no_gap() {
    let out = build(&example("runner"));
    let mut nes = TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(40).unwrap(); // long enough to land
    let (hx, hy) = (nes.peek(out.vars["hero.x"].addr) as usize, nes.peek(out.vars["hero.y"].addr) as usize);
    assert_eq!(hy, 168 - 16, "physics puts the hero's 16-pixel box on the ground");
    // Sprite pixels = where the picture changes when sprites are turned off.
    let with = nes_sim::render::frame(&nes.bus, nes.frames - 1);
    let mut bg_only = nes.bus.clone();
    bg_only.ppumask &= !0x10;
    let without = nes_sim::render::frame(&bg_only, nes.frames - 1);
    let lowest = (0..240)
        .rev()
        .find(|&y| (hx..hx + 8).any(|x| with[y * 256 + x] != without[y * 256 + x]))
        .expect("hero is drawn");
    assert_eq!(lowest, 167, "the hero's feet touch the ground (row 168)");
    let highest = (0..240).find(|&y| (hx..hx + 8).any(|x| with[y * 256 + x] != without[y * 256 + x])).unwrap();
    assert_eq!(highest, hy, "the top of the hero is exactly at hero.y");
}
