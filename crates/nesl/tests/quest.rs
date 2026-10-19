//! 009: COIN QUEST played on the test machine with scripted input and pokes.
mod common;
use common::*;

fn quest() -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quest.nesl")).unwrap()
}

fn frames(r: &mut Run, n: u64, buttons: u8) {
    r.nes.set_buttons(0, buttons);
    r.nes.run_frames(n).unwrap();
}

fn poke(r: &mut Run, name: &str, off: u16, v: u8) {
    let a = r.addr(name) + off;
    r.nes.bus.ram[a as usize] = v;
}

fn poke16(r: &mut Run, name: &str, v: u16) {
    poke(r, name, 0, v as u8);
    poke(r, name, 1, (v >> 8) as u8);
}

/// Power on, see the title, press START.
fn start() -> Run {
    let mut r = run_frames(&quest(), 10, |_| {});
    assert_eq!(r.u8("state"), 0);
    assert_eq!(r.vram(0x2000 + 7 * 32 + 11, 10), b"COIN QUEST".to_vec());
    frames(&mut r, 2, 0x10);
    frames(&mut r, 20, 0);
    assert_eq!((r.u8("state"), r.u8("world"), r.u8("lives")), (1, 0, 3));
    r
}

/// Teleports the hero over the first pit of world 1 and lets it fall.
fn fall_in_pit(r: &mut Run) {
    poke16(r, "hero.x", 244);
    frames(r, 60, 0);
}

#[test]
fn the_status_bar_shows_score_coins_lives_and_world() {
    let r = start();
    assert_eq!(r.vram(0x2020 + 1, 5), b"SCORE".to_vec());
    assert_eq!(r.vram(0x2020 + 7, 5), b"00000".to_vec());
    assert_eq!(r.vram(0x2020 + 24, 1), b"3".to_vec());
    assert_eq!(r.vram(0x2020 + 28, 1), b"1".to_vec());
}

#[test]
fn coins_score_and_stay_collected_after_dying() {
    let mut r = start();
    frames(&mut r, 50, 0x01); // run right along the row of ground coins
    let coins = r.u8("coins");
    assert!(coins >= 2, "picked up {coins}");
    assert_eq!(r.u16("score"), coins as u16 * 10);
    fall_in_pit(&mut r);
    assert_eq!(r.u8("lives"), 2);
    frames(&mut r, 50, 0x01); // the same stretch again
    assert_eq!(r.u8("coins"), coins, "collected coins do not come back");
}

#[test]
fn stomping_a_slime_kills_it_and_scores_50() {
    let mut r = start();
    // slime 0 right under a falling hero
    poke(&mut r, "foes.x", 0, 24);
    poke(&mut r, "foes.x", 8, 0);
    poke(&mut r, "foes.y", 0, 160);
    poke16(&mut r, "hero.x", 24);
    poke(&mut r, "hero.y", 0, 142);
    poke(&mut r, "hero.vy", 0, 3);
    frames(&mut r, 2, 0);
    assert_eq!(r.bytes("foes.alive")[0], 0);
    assert_eq!(r.u16("score"), 50);
    assert_eq!(r.u8("lives"), 3);
}

#[test]
fn touching_a_slime_costs_a_life_but_not_while_invulnerable() {
    let mut r = start();
    let hit = |r: &mut Run| {
        let hx = r.u16("hero.x");
        poke(r, "foes.x", 0, hx as u8);
        poke(r, "foes.x", 8, (hx >> 8) as u8);
        poke(r, "foes.y", 0, 160);
        frames(r, 2, 0);
    };
    hit(&mut r); // just spawned: still invulnerable
    assert_eq!(r.u8("lives"), 3);
    frames(&mut r, 100, 0);
    hit(&mut r);
    assert_eq!(r.u8("lives"), 2);
    assert_eq!(r.u16("hero.x"), 24, "respawned at the start");
    assert!(r.u8("hurt") > 0, "and invulnerable again");
}

#[test]
fn three_falls_end_the_game_and_start_returns_to_play() {
    let mut r = start();
    for lives in [2, 1] {
        fall_in_pit(&mut r);
        assert_eq!(r.u8("lives"), lives);
    }
    fall_in_pit(&mut r);
    assert_eq!(r.u8("state"), 3);
    frames(&mut r, 5, 0);
    assert_eq!(r.vram(0x2000 + 11 * 32 + 11, 9), b"GAME OVER".to_vec());
    frames(&mut r, 2, 0x10);
    // a new game redraws the status bar and the map (several frames); wait for it
    let mut waited = 0;
    while r.u8("state") != 1 {
        frames(&mut r, 1, 0);
        waited += 1;
        assert!(waited < 30, "START on the game-over screen did not start a new game");
    }
    assert_eq!((r.u8("lives"), r.u16("score")), (3, 0));
}

#[test]
fn flags_lead_through_all_three_worlds_to_the_win_screen() {
    let mut r = start();
    for w in 0..6u8 {
        assert_eq!((r.u8("state"), r.u8("world")), (1, w));
        frames(&mut r, 30, 0);
        poke16(&mut r, "hero.x", 980); // at the flag (column 123)
        frames(&mut r, 2, 0);
        assert_eq!(r.u8("state"), 2, "world {w} cleared");
        frames(&mut r, 100, 0);
    }
    assert_eq!(r.u8("state"), 4);
    frames(&mut r, 5, 0);
    assert_eq!(r.vram(0x2000 + 10 * 32 + 12, 8), b"YOU WIN!".to_vec());
    assert_eq!(r.nes.bus.render_conflicts, 0);
    assert_eq!(r.nes.bus.bus_conflicts, 0);
}

#[test]
fn six_worlds_are_built_as_unrom() {
    let out = build(&quest());
    assert_eq!((out.mapper, out.rom[5]), (2, 0), "UNROM with CHR-RAM");
}

#[test]
fn a_long_scripted_run_never_glitches() {
    let mut r = start();
    for f in 0..1500u32 {
        frames(&mut r, 1, if f % 30 < 4 { 0x81 } else { 0x01 });
        if r.u8("state") == 3 {
            frames(&mut r, 2, 0x10);
        }
    }
    assert_eq!(r.nes.bus.render_conflicts, 0);
    assert_eq!(r.nes.bus.bus_conflicts, 0);
    assert!(r.nes.bus.sprite0_at.is_some() || r.u8("state") != 1, "status bar split active while playing");
}

#[test]
fn the_first_platform_can_be_reached_by_jumping() {
    let mut r = start();
    frames(&mut r, 60, 0); // land, invulnerability over
    // stand on the ground just left of world 1's first platform (row 17, columns 40-45)
    poke16(&mut r, "hero.x", 37 * 8);
    frames(&mut r, 10, 0);
    assert_eq!((r.u8("hero.y"), r.u8("hero.ground")), (152, 1), "on the ground");
    frames(&mut r, 3, 0x81); // jump while running right
    frames(&mut r, 12, 0x01);
    frames(&mut r, 10, 0);
    assert_eq!((r.u8("hero.y"), r.u8("hero.ground")), (17 * 8 - 16, 1), "standing on the platform");
}

#[test]
fn pits_let_the_hero_fall_through() {
    let mut r = start();
    frames(&mut r, 60, 0);
    // Teleport to the edge of pit 1 (cols 30-32) at ground level
    poke16(&mut r, "hero.x", 29 * 8);
    r.nes.poke(r.addr("hero.y"), 152);
    frames(&mut r, 5, 0);
    assert_eq!(r.u8("hero.ground"), 1, "standing before the pit");
    
    // Walk into the pit
    frames(&mut r, 10, 0x01); // RIGHT for 10 frames
    let y_in_pit = r.u8("hero.y");
    assert!(y_in_pit > 152, "should be falling (y > 152), got y={}", y_in_pit);
    assert_eq!(r.u8("hero.ground"), 0, "should be airborne in the pit");
}
