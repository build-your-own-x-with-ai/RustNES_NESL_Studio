//! 006 US1: set_map_tile / reset_map — map_tile, the screen and scroll streaming all
//! see the changes.
mod common;
use common::*;

fn poke16(r: &mut Run, name: &str, v: u16) {
    let a = r.addr(name) as usize;
    r.nes.bus.ram[a] = v as u8;
    r.nes.bus.ram[a + 1] = (v >> 8) as u8;
}

#[test]
fn changes_are_read_back_and_neighbours_untouched() {
    let r = run(&format!("{} var a: u8; var b: u8; var c: u8; var ok: u8;
        fn main() {{ ok = set_map_tile(W, 5, 10, ' '); a = map_tile(W, 5, 10); b = map_tile(W, 6, 10); c = map_tile(W, 5, 11); }}",
        wide_map("W", 32)));
    assert_eq!((r.u8("ok"), r.u8("a"), r.u8("b"), r.u8("c")), (1, b' ', cell(6, 10), cell(5, 11)));
}

#[test]
fn visible_cells_update_on_screen_without_glitches() {
    let r = run_frames(&format!("{} var t: u8;
        fn main() {{ draw_map(W); ppu_on(); loop {{ wait_vblank(); t += 1; if t == 5 {{ set_map_tile(W, 4, 6, '#'); }} }} }}",
        wide_map("W", 64)), 12, |_| {});
    assert_eq!(nt(&r, 4, 6), b'#');
    assert_eq!(nt(&r, 5, 6), cell(5, 6));
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn changes_survive_scrolling_away_and_back_and_stream_in_off_screen() {
    let src = format!("{} var hero: u16; var target: u16; var cam: u16; var ok: u8;
        fn main() {{
            draw_map(W);
            ok = set_map_tile(W, 10, 5, '#') + set_map_tile(W, 90, 7, '@');
            ppu_on();
            loop {{
                wait_vblank();
                if hero < target {{ hero += 4; }} else if hero > target {{ hero -= 4; }}
                camera_follow(hero, W);
                cam = camera_x();
            }}
        }}", wide_map("W", 128));
    let mut r = run_frames(&src, 4, |_| {});
    assert_eq!(r.u8("ok"), 2);
    assert_eq!(nt(&r, 10, 5), b'#', "visible change written directly while rendering was off");
    poke16(&mut r, "target", 1020);
    let mut saw_90 = false;
    for _ in 0..300 {
        r.nes.run_frames(1).unwrap();
        let cam = r.u16("cam");
        if (500..=700).contains(&cam) {
            assert_eq!(nt(&r, 90 % 64, 7), b'@', "column 90 streamed with the change (camera {cam})");
            assert_eq!(nt(&r, 90 % 64, 8), cell(90, 8));
            saw_90 = true;
        }
    }
    assert!(saw_90);
    assert_eq!(r.u16("cam"), 768);
    assert_ne!(nt(&r, 10, 5), b'#', "slot 10 now holds column 74");
    poke16(&mut r, "target", 0);
    r.nes.run_frames(300).unwrap();
    assert_eq!(r.u16("cam"), 0);
    assert_eq!(nt(&r, 10, 5), b'#', "column 10 re-streamed with the change");
    assert_eq!(nt(&r, 10, 4), cell(10, 4));
    assert_eq!(r.nes.bus.render_conflicts, 0);
}

#[test]
fn maps_are_independent_and_reset_map_restores_the_original() {
    let r = run(&format!("{}{} var w: u8; var v: u8; var after: u8;
        fn main() {{
            draw_map(W);
            set_map_tile(W, 2, 3, '#');
            set_map_tile(V, 9, 9, '@');
            w = map_tile(W, 2, 3); v = map_tile(V, 2, 3);
            reset_map(W);
            after = map_tile(W, 2, 3);
            draw_map(W);
        }}", wide_map("W", 32), wide_map("V", 32)));
    assert_eq!((r.u8("w"), r.u8("v"), r.u8("after")), (b'#', cell(2, 3), cell(2, 3)));
    assert_eq!(nt(&r, 2, 3), cell(2, 3), "redrawn from the original");
}

#[test]
fn reset_map_keeps_other_maps_changes() {
    let r = run(&format!("{}{} var v: u8;
        fn main() {{ set_map_tile(W, 1, 1, '#'); set_map_tile(V, 1, 1, '@'); set_map_tile(W, 2, 2, '#'); reset_map(W); v = map_tile(V, 1, 1); }}",
        wide_map("W", 32), wide_map("V", 32)));
    assert_eq!(r.u8("v"), b'@');
}

#[test]
fn capacity_is_64_changes_and_updates_reuse_their_slot() {
    let r = run(&format!("{} var ok: u8; var full: u8; var again: u8; var a: u8; var b: u8;
        fn main() {{
            for i in 0..64 {{ ok += set_map_tile(W, i % 32, i / 32, '#'); }}
            full = set_map_tile(W, 0, 5, '@');
            again = set_map_tile(W, 3, 0, '@');
            a = map_tile(W, 0, 5); b = map_tile(W, 3, 0);
        }}", wide_map("W", 32)));
    assert_eq!((r.u8("ok"), r.u8("full"), r.u8("again")), (64, 0, 1));
    assert_eq!((r.u8("a"), r.u8("b")), (cell(0, 5), b'@'));
}

#[test]
fn out_of_bounds_changes_are_rejected() {
    let r = run(&format!("{} var a: u8; var b: u8;
        fn main() {{ a = set_map_tile(W, 32, 0, '#'); b = set_map_tile(W, 0, 30, '#'); }}", wide_map("W", 32)));
    assert_eq!((r.u8("a"), r.u8("b")), (0, 0));
}

#[test]
fn the_change_list_only_takes_ram_when_used() {
    let without = build("var buf: [u8; 4]; fn main() {}");
    assert_eq!(without.vars["buf"].addr, 0x03A0);
    let with = build(&format!("{} var buf: [u8; 4]; fn main() {{ set_map_tile(W, 0, 0, 1); }}", wide_map("W", 32)));
    assert!(with.vars["buf"].addr >= 0x04BE, "{:#x}", with.vars["buf"].addr);
}

#[test]
fn map_edit_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("const P: [u8] = [1]; fn main() { set_map_tile(P, 0, 0, 1); }").contains("map"));
    assert!(m("fn main() { reset_map(5); }").contains("map"));
}
