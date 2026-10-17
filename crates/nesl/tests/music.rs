//! 005: the music engine, verified frame by frame against the APU write log.
mod common;
use common::*;
use nesl::mml::pulse_period;

type Frames = Vec<Vec<(u16, u8)>>;

/// Runs `frames` frames, returning the APU writes made during each one.
fn trace(src: &str, frames: usize) -> (Run, Frames) {
    let mut r = run_frames(src, 0, |_| {});
    let mut per = Vec::new();
    for _ in 0..frames {
        let before = r.nes.bus.apu_writes.len();
        r.nes.run_frames(1).unwrap();
        per.push(r.nes.bus.apu_writes[before..].to_vec());
    }
    (r, per)
}

/// (frame, value) of every write to `reg`.
fn writes(per: &Frames, reg: u16) -> Vec<(usize, u8)> {
    per.iter()
        .enumerate()
        .flat_map(|(f, w)| w.iter().filter(move |(a, _)| *a == reg).map(move |(_, v)| (f, *v)))
        .collect()
}

fn song(music: &str, play: &str) -> String {
    format!("{music} var t: u8; fn main() {{ {play}; loop {{ wait_vblank(); t += 1; }} }}")
}

// ---------------------------------------------------------------- US1 melody

#[test]
fn melody_notes_start_on_the_expected_frames_and_loop() {
    let (_, per) = trace(&song("music M = \"T150 L4 C D E\";", "play_music(M)"), 240);
    let on = writes(&per, 0x4002);
    let f0 = on[0].0;
    let expected: Vec<(usize, u8)> = (0..9)
        .map(|k| (f0 + 24 * k, pulse_period([48, 50, 52][k % 3]) as u8))
        .collect();
    assert_eq!(&on[..9], &expected[..]);
    // the last frame of each note is silent (staccato gap)
    let vol: Vec<(usize, u8)> = writes(&per, 0x4000).into_iter().filter(|(f, _)| *f == f0 + 23).collect();
    assert_eq!(vol.iter().map(|(_, v)| v & 0x0F).collect::<Vec<_>>(), vec![0]);
}

#[test]
fn a4_is_440_hz() {
    let (_, per) = trace(&song("music M = \"O4 A\";", "play_music(M)"), 5);
    assert_eq!(writes(&per, 0x4002)[0].1, 253);
    assert_eq!(writes(&per, 0x4003)[0].1, 0);
    // default V12 @2: duty bits 10, length halt + constant volume, volume 12
    assert_eq!(writes(&per, 0x4000)[0].1, 0b1011_1100);
}

#[test]
fn ten_loops_do_not_drift() {
    // eighth notes at T130 are 13.85 frames; the loop is round(4 * 13.85) = 55 frames
    let (_, per) = trace(&song("music M = \"T130 L8 CCCC\";", "play_music(M)"), 560);
    let on: Vec<usize> = writes(&per, 0x4002).into_iter().map(|(f, _)| f).collect();
    let eighth = 3600.0 / 130.0 / 2.0;
    for k in 0..40usize {
        let ideal = (k as f64 * eighth).round() as usize;
        assert_eq!(on[k] - on[0], ideal, "note {k}");
    }
}

#[test]
fn volume_and_duty_commands() {
    let (_, per) = trace(&song("music M = \"V5 @1 C\";", "play_music(M)"), 5);
    assert_eq!(writes(&per, 0x4000)[0].1, 0b0111_0101);
}

#[test]
fn stop_music_silences_and_stops_advancing() {
    let (_, per) = trace(&song("music M = \"L8 CDEFGAB\";", "play_music(M); if t == 99 { stop_music(); }")
        .replace("t += 1; }", "t += 1; if t == 30 { stop_music(); } }"), 120);
    let on = writes(&per, 0x4002);
    let last_on = on.last().unwrap().0;
    assert!(last_on < 35, "no notes after stop_music (last at frame {last_on})");
    assert_eq!(writes(&per, 0x4000).last().unwrap().1 & 0x0F, 0, "pulse 1 silenced");
}

#[test]
fn music_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("music M = \"C X\"; fn main() {}").contains("unknown MML command"));
    assert!(m("const P: [u8] = [1]; fn main() { play_music(P); }").contains("music"));
    assert!(m("fn main() { play_music(); }").contains("1 to 3"));
}
