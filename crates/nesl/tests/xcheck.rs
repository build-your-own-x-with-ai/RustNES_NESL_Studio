//! 007: cross-check against jsnes, an independent emulator. Each scene runs in both the
//! test machine (rendered by `nes_sim::render`) and jsnes for the same number of frames,
//! and the final pictures are compared pixel by pixel. Scenes are static by the end, so
//! the two emulators' different frame alignment cannot cause false mismatches.
//!
//! Needs `node` and `npm ci` in tools/xcheck; otherwise the test is skipped (like nestest).
//!
//! Known jsnes difference: it draws only 32 background tiles per line, so with a fine
//! horizontal scroll f > 0 the rightmost f pixels stay background colour (hardware fetches
//! a 33rd tile). Those pixels are excluded, but jsnes must show background there.
//!
//! Known jsnes bug: screen row 0 comes from its dummy pre-render pass, and the final
//! composite copies opaque row-0 pixels from a background buffer it never filled, leaving
//! them undefined (seen with a fine horizontal scroll). run.js reports those as
//! `UNDRAWN`; they are excluded, and must not appear outside row 0.
mod common;
use common::*;
use nes_sim::render::{self, HEIGHT, WIDTH};
use std::path::{Path, PathBuf};
use std::process::Command;

fn tools() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/xcheck")
}

fn available() -> Result<(), String> {
    if Command::new("node").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        return Err("`node` not found".into());
    }
    if !tools().join("node_modules/jsnes").exists() {
        return Err("jsnes not installed (run `npm ci` in tools/xcheck)".into());
    }
    Ok(())
}

/// (jsnes colour table, frame) after `frames` frames with `buttons` held.
fn jsnes(rom: &[u8], frames: u64, buttons: u8, name: &str) -> (Vec<u32>, Vec<u32>) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("xcheck");
    std::fs::create_dir_all(&dir).unwrap();
    let rom_path = dir.join(format!("{name}.nes"));
    let out_path = dir.join(format!("{name}.txt"));
    std::fs::write(&rom_path, rom).unwrap();
    let out = Command::new("node")
        .current_dir(tools())
        .arg("run.js")
        .arg(&rom_path)
        .arg(frames.to_string())
        .arg(buttons.to_string())
        .arg(&out_path)
        .output()
        .expect("run node");
    assert!(out.status.success(), "jsnes failed: {}", String::from_utf8_lossy(&out.stderr));
    let text = std::fs::read_to_string(&out_path).unwrap();
    let mut lines = text.lines().map(|l| l.split_whitespace().map(|v| v.parse::<u32>().unwrap()).collect::<Vec<_>>());
    let table = lines.next().unwrap();
    let frame = lines.next().unwrap();
    assert_eq!((table.len(), frame.len()), (64, WIDTH * HEIGHT));
    (table, frame)
}

/// Pixel value run.js writes where jsnes left the frame buffer undefined.
const UNDRAWN: u32 = u32::MAX;

/// jsnes stores colours as 0xBBGGRR.
fn rgb(c: u32) -> [u8; 3] {
    [c as u8, (c >> 8) as u8, (c >> 16) as u8]
}

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// Minimal PNG writer (stored deflate blocks), enough to look at a mismatch.
fn png(path: &Path, pixels: &[[u8; 3]]) {
    let mut raw = Vec::new();
    for row in pixels.chunks(WIDTH) {
        raw.push(0);
        row.iter().for_each(|p| raw.extend_from_slice(p));
    }
    let mut z = vec![0x78, 0x01];
    let chunks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, c) in chunks.iter().enumerate() {
        z.push((i == chunks.len() - 1) as u8);
        z.extend_from_slice(&(c.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        z.extend_from_slice(c);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&(b << 16 | a).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(WIDTH as u32).to_be_bytes());
    ihdr.extend_from_slice(&(HEIGHT as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    std::fs::write(path, out).unwrap();
}

/// Runs `src` in both emulators and compares the final pictures.
fn check(name: &str, src: &str, frames: u64, buttons: u8) {
    if let Err(why) = available() {
        eprintln!("xcheck `{name}` skipped: {why}");
        return;
    }
    let out = build(src);
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.set_buttons(0, buttons);
    nes.run_frames(frames).unwrap();
    let ours = render::frame(&nes.bus, nes.frames - 1);
    let (table, theirs) = jsnes(&out.rom, frames, buttons, name);
    let ours_rgb: Vec<u32> = ours.iter().map(|&i| table[i as usize]).collect();
    let fine = render::line_fine_x(&nes.bus, nes.frames - 1);
    let backdrop = table[(nes.bus.vram[0x3F00] & 0x3F) as usize];
    let undrawn: Vec<usize> = (0..theirs.len()).filter(|&i| theirs[i] == UNDRAWN).collect();
    assert!(undrawn.iter().all(|&i| i < WIDTH), "`{name}`: jsnes left pixels undefined outside row 0: {:?}", &undrawn[..undrawn.len().min(5)]);
    if !undrawn.is_empty() {
        eprintln!("xcheck `{name}`: {} undefined jsnes pixels in row 0 excluded", undrawn.len());
    }
    let tile_gap = |i: usize| i % WIDTH >= WIDTH - fine[i / WIDTH] as usize && theirs[i] != UNDRAWN;
    let jsnes_gap = |i: usize| tile_gap(i) || theirs[i] == UNDRAWN;
    let skipped = (0..ours.len()).filter(|&i| tile_gap(i)).count();
    let gap_wrong: Vec<usize> = (0..ours.len()).filter(|&i| tile_gap(i) && theirs[i] != backdrop).collect();
    assert!(gap_wrong.is_empty(), "`{name}`: jsnes drew something in its 33rd-tile gap at {:?}", &gap_wrong[..gap_wrong.len().min(5)]);
    if skipped > 0 {
        eprintln!("xcheck `{name}`: {skipped} pixels in jsnes' 33rd-tile gap excluded");
    }
    let bad: Vec<usize> = (0..ours.len()).filter(|&i| !jsnes_gap(i) && ours_rgb[i] != theirs[i]).collect();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/xcheck");
    std::fs::create_dir_all(&dir).unwrap();
    png(&dir.join(format!("{name}-ours.png")), &ours_rgb.iter().map(|&c| rgb(c)).collect::<Vec<_>>());
    png(&dir.join(format!("{name}-jsnes.png")), &theirs.iter().map(|&c| rgb(if c == UNDRAWN { 0 } else { c })).collect::<Vec<_>>());
    let diff_path = dir.join(format!("{name}-diff.png"));
    let _ = std::fs::remove_file(&diff_path); // a stale diff from an earlier failure would mislead
    if !bad.is_empty() {
        let diff: Vec<[u8; 3]> = (0..ours.len())
            .map(|i| if !jsnes_gap(i) && ours_rgb[i] != theirs[i] { [255, 0, 0] } else { rgb(theirs[i]).map(|v| v / 3) })
            .collect();
        png(&diff_path, &diff);
        let mut rows: Vec<usize> = bad.iter().map(|i| i / WIDTH).collect();
        rows.dedup();
        let first = bad[0];
        panic!(
            "`{name}`: {} of {} pixels differ (first at x={}, y={}; ours {:06x}, jsnes {:06x}); rows {:?}; see target/xcheck/{name}-*.png",
            bad.len(),
            ours.len(),
            first % WIDTH,
            first / WIDTH,
            ours_rgb[first],
            theirs[first],
            &rows[..rows.len().min(20)]
        );
    }
}

fn example(name: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../examples/{name}.nesl"))).unwrap()
}

#[test]
fn hello_matches_jsnes() {
    check("hello", &example("hello"), 30, 0);
}

#[test]
fn sprite_matches_jsnes() {
    check("sprite", &example("sprite"), 30, 0);
}

#[test]
fn runner_with_status_bar_matches_jsnes() {
    check("runner", &example("runner"), 60, 0);
}

#[test]
fn quest_title_matches_jsnes() {
    check("quest", &example("quest"), 40, 0);
}

#[test]
fn scrolled_level_with_status_bar_matches_jsnes() {
    let src = format!(
        "{}
        const PAL: [u8] = [0x0F, 0x30, 0x16, 0x27, 0x0F, 0x21, 0x31, 0x11, 0x0F, 0x29, 0x19, 0x09, 0x0F, 0x26, 0x16, 0x06,
                           0x0F, 0x16, 0x27, 0x30, 0x0F, 0x30, 0x10, 0x00, 0x0F, 0x30, 0x10, 0x00, 0x0F, 0x30, 0x10, 0x00];
        palette_zone W = [\"0000111122223333000011112222333300001111222233330000111122223333\", \"\", \"\", \"1\", \"2\", \"3\"];
        var hero: u16; var cam: u16;
        fn main() {{
            load_palette(PAL);
            draw_map(W);
            status_bar(4);
            print(1, 1, \"SCORE 123\");
            print(1, 3, \"LEVEL 1-1 WORLD MAP TEST\"); // last bar row: a mistimed split shows here
            set_map_tile(W, 90, 12, '#');
            ppu_on();
            loop {{
                wait_vblank();
                if hero < 700 {{ hero += 3; }}
                camera_follow(hero, W);
                cam = camera_x();
                sprite(1, (hero - cam) as u8, 120, 'A', 0);
            }}
        }}",
        wide_map("W", 128)
    );
    check("scroll", &src, 330, 0);
}

/// 010: a scrolled UNROM (mapper 2, CHR-RAM) level, so mirroring and banking are
/// compared against jsnes too, not only static NROM screens.
#[test]
fn scrolled_unrom_level_matches_jsnes() {
    let mut maps = String::from("const MAPS: [map] = [");
    (0..9).for_each(|i| maps.push_str(&format!("M{i},")));
    maps.push_str("];\n");
    (0..9).for_each(|i| maps.push_str(&wide_map(&format!("M{i}"), 128)));
    let src = format!(
        "{maps}
        var hero: u16; var cam: u16;
        fn main() {{
            draw_map(MAPS[5]);
            ppu_on();
            loop {{
                wait_vblank();
                if hero < 700 {{ hero += 3; }}
                camera_follow(hero, MAPS[5]);
                cam = camera_x();
            }}
        }}"
    );
    assert_eq!(build(&src).mapper, 2, "scene must be UNROM");
    check("scroll-unrom", &src, 330, 0);
}
