//! 012: save_game() / load_game() persist variables to SRAM across power cycles.
mod common;
use common::*;

#[test]
fn save_game_writes_variables_to_sram() {
    let src = "var a: u8 = 10; var b: u16 = 500; var c: i8 = -3;
        fn main() { a = 99; b = 12345; c = -50; save_game(); }";
    let out = build(src);
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(5).unwrap();

    let save = nes.sram_snapshot();
    // Magic header "NESL" at $6000
    assert_eq!(&save[0..4], b"NESL", "SRAM magic header");

    // Variables stored after the header
    assert_eq!(save[4], 99, "a");
    assert_eq!((save[5] as u16) | ((save[6] as u16) << 8), 12345, "b (u16, little-endian)");
    assert_eq!(save[7] as i8, -50, "c (i8)");
}

#[test]
fn load_game_restores_variables_from_sram() {
    let src = "var score: u16 = 0; var level: u8 = 1;
        fn main() {
            if load_game() {
                // save exists: variables were restored
            } else {
                // no save: use defaults
                score = 1000;
                level = 5;
                save_game();
            }
        }";
    let out = build(src);

    // First run: no save, uses defaults
    let mut nes1 = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes1.run_frames(5).unwrap();
    assert_eq!(nes1.peek(out.vars["score"].addr), (1000 & 0xFF) as u8);
    assert_eq!(nes1.peek(out.vars["score"].addr + 1), (1000 >> 8) as u8);
    assert_eq!(nes1.peek(out.vars["level"].addr), 5);

    let save = nes1.sram_snapshot();
    assert_eq!(&save[0..4], b"NESL");

    // Second run: restore from save
    let mut nes2 = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes2.restore_sram(&save);
    nes2.run_frames(5).unwrap();

    // load_game() returned true, so score/level stayed at the saved values
    assert_eq!(nes2.peek(out.vars["score"].addr), (1000 & 0xFF) as u8);
    assert_eq!(nes2.peek(out.vars["score"].addr + 1), (1000 >> 8) as u8);
    assert_eq!(nes2.peek(out.vars["level"].addr), 5);
}

#[test]
fn save_game_with_no_variables_still_writes_magic() {
    let src = "fn main() { save_game(); }";
    let out = build(src);
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(5).unwrap();
    let save = nes.sram_snapshot();
    assert_eq!(&save[0..4], b"NESL");
}

#[test]
fn load_game_returns_false_when_magic_is_missing() {
    let src = "var found: u8;
        fn main() {
            if load_game() { found = 1; } else { found = 0; }
        }";
    let out = build(src);
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(5).unwrap();
    assert_eq!(nes.peek(out.vars["found"].addr), 0, "load_game() returned false (no magic)");
}

#[test]
fn multiple_save_load_cycles() {
    let src = "var x: u8 = 0;
        fn main() {
            if load_game() {
                x += 10;
            } else {
                x = 5;
            }
            save_game();
        }";
    let out = build(src);

    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(5).unwrap();
    assert_eq!(nes.peek(out.vars["x"].addr), 5);

    let s1 = nes.sram_snapshot();
    let mut nes2 = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes2.restore_sram(&s1);
    nes2.run_frames(5).unwrap();
    assert_eq!(nes2.peek(out.vars["x"].addr), 15);

    let s2 = nes2.sram_snapshot();
    let mut nes3 = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes3.restore_sram(&s2);
    nes3.run_frames(5).unwrap();
    assert_eq!(nes3.peek(out.vars["x"].addr), 25);
}
