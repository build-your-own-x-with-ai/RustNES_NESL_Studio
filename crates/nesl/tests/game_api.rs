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
