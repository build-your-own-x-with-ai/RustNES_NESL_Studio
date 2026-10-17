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
