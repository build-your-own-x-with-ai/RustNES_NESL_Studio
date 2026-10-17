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

/// The control-register value written by the first note-on (play_music first silences
/// the channel, so the note's own write is the last one in that frame).
fn first_note_ctrl(per: &Frames, ctrl: u16, period_lo: u16) -> u8 {
    let f = writes(per, period_lo)[0].0;
    writes(per, ctrl).into_iter().filter(|(g, _)| *g == f).last().unwrap().1
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
    assert_eq!(first_note_ctrl(&per, 0x4000, 0x4002), 0b1011_1100);
}

#[test]
fn ten_loops_do_not_drift() {
    // eighth notes at T130 are 13.85 frames; the loop is round(4 * 13.85) = 55 frames
    let (_, per) = trace(&song("music M = \"T130 L8 CCCC\";", "play_music(M)"), 560);
    let on: Vec<usize> = writes(&per, 0x4002).into_iter().map(|(f, _)| f).collect();
    // Every loop is exactly round(total) = 55 frames, and inside a loop each note starts at
    // its rounded position, so the k-th loop never shifts relative to the first.
    let eighth = 3600.0 / 130.0 / 2.0;
    for k in 0..40usize {
        let ideal = (k / 4) * 55 + ((k % 4) as f64 * eighth).round() as usize;
        assert_eq!(on[k] - on[0], ideal, "note {k}");
    }
}

#[test]
fn volume_and_duty_commands() {
    let (_, per) = trace(&song("music M = \"V5 @1 C\";", "play_music(M)"), 5);
    assert_eq!(first_note_ctrl(&per, 0x4000, 0x4002), 0b0111_0101);
}

#[test]
fn stop_music_silences_and_stops_advancing() {
    let (_, per) = trace(
        "music M = \"L8 CDEFGAB\"; var t: u8;
         fn main() { play_music(M); loop { wait_vblank(); t += 1; if t == 30 { stop_music(); } } }",
        120,
    );
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

// ---------------------------------------------------------------- US2 voices

#[test]
fn bass_plays_on_the_triangle_an_octave_lower_per_period() {
    let (_, per) = trace(&song("music M = \"O4 A\"; music B = \"O3 A R\";", "play_music(M, B)"), 70);
    // triangle A3 uses the pulse period of A4
    assert_eq!(writes(&per, 0x400A)[0].1, 253);
    assert_eq!(writes(&per, 0x4002)[0].1, 253, "melody A4 on pulse 1");
    let en = writes(&per, 0x4015);
    assert!(en.iter().any(|(_, v)| v & 0x05 == 0x05), "pulse 1 and triangle together: {en:?}");
    // the rest after the bass note turns the triangle off
    let rest_frame = writes(&per, 0x400A)[0].0 + 30;
    assert!(en.iter().any(|(f, v)| *f == rest_frame && v & 0x04 == 0), "{en:?}");
}

#[test]
fn harmony_uses_pulse_2_and_voices_keep_their_own_time() {
    let (_, per) = trace(&song("music M = \"L4 C\"; music B = \"L8 C\"; music H = \"L2 E\";", "play_music(M, B, H)"), 130);
    let mel: Vec<usize> = writes(&per, 0x4002).into_iter().map(|(f, _)| f).collect();
    let bass: Vec<usize> = writes(&per, 0x400A).into_iter().map(|(f, _)| f).collect();
    let harm: Vec<usize> = writes(&per, 0x4006).into_iter().map(|(f, _)| f).collect();
    assert_eq!(mel[1] - mel[0], 30);
    assert_eq!(bass[1] - bass[0], 15);
    assert_eq!(harm[1] - harm[0], 60);
    assert_eq!(writes(&per, 0x4006)[0].1, pulse_period(52) as u8);
}

#[test]
fn unused_voices_stay_silent() {
    let (_, per) = trace(&song("music M = \"C D E F\";", "play_music(M)"), 90);
    assert!(writes(&per, 0x400A).is_empty() && writes(&per, 0x4006).is_empty());
}

#[test]
fn pulse_voices_reject_notes_below_a1_but_bass_accepts_them() {
    let low = "music L = \"O1 G A\";";
    let e = &errors(&format!("{low} fn main() {{ play_music(L); }}"))[0];
    assert!(e.message.contains("below A1"), "{}", e.message);
    assert_eq!(e.col, 15, "points at the G");
    build(&format!("{low} music M = \"C\"; fn main() {{ play_music(M, L); }}"));
    assert!(errors(&format!("{low} music M = \"C\"; fn main() {{ play_music(M, M, L); }}"))[0].message.contains("below A1"));
}

// ---------------------------------------------------------------- US3 custom sounds

#[test]
fn pulse_sound_declaration_drives_pulse_2() {
    let (_, per) = trace(&song("sound S = (pulse, 0x120, 3, 6, 9, 1);", "sfx(S)"), 10);
    assert_eq!(writes(&per, 0x4007).iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![1], "period high byte once");
    let lows: Vec<u8> = writes(&per, 0x4006).into_iter().map(|(_, v)| v).collect();
    assert_eq!(lows, vec![0x23, 0x26, 0x29, 0x2C, 0x2F], "period slides by +3 each frame");
    let ctrl: Vec<u8> = writes(&per, 0x4004).into_iter().map(|(_, v)| v).collect();
    assert!(ctrl.iter().all(|v| v & 0xF0 == 0x70), "duty 1 + constant volume: {ctrl:x?}");
    assert!(ctrl.iter().all(|v| v & 0x0F <= 9), "volume capped at 9: {ctrl:x?}");
    assert_eq!(writes(&per, 0x4015).last().map(|(_, v)| v & 0x02), Some(0), "pulse 2 released at the end");
}

#[test]
fn noise_sound_declaration() {
    let (_, per) = trace(&song("sound N = (noise, 5, 0, 20);", "sfx(N)"), 4);
    assert!(writes(&per, 0x400E).iter().all(|(_, v)| *v == 5));
    assert!(!writes(&per, 0x400C).is_empty());
    assert!(writes(&per, 0x4015).iter().any(|(_, v)| v & 0x08 != 0));
}

#[test]
fn custom_sounds_are_numbered_after_the_builtins() {
    let r = run("sound A = (pulse, 0x80, 0, 4); sound B = (noise, 1, 0, 4); var a: u8; var b: u8;
        fn main() { a = A; b = B; sfx(SFX_JUMP); sfx(B); }");
    assert_eq!((r.u8("a"), r.u8("b")), (5, 6));
}

#[test]
fn sound_errors() {
    let m = |src: &str| errors(src)[0].message.clone();
    assert!(m("sound S = (square, 0x80, 0, 4); fn main() {}").contains("channel"));
    assert!(m("sound S = (pulse, 5, 0, 4); fn main() {}").contains("period"));
    assert!(m("sound S = (noise, 16, 0, 4); fn main() {}").contains("period"));
    assert!(m("sound S = (pulse, 0x80, 200, 4); fn main() {}").contains("delta"));
    assert!(m("sound S = (pulse, 0x80, 0, 0); fn main() {}").contains("frames"));
    assert!(m("sound S = (pulse, 0x80, 0, 4, 16); fn main() {}").contains("volume"));
    assert!(m("sound S = (pulse, 0x80, 0, 4, 9, 4); fn main() {}").contains("duty"));
    let many: String = (0..28).map(|i| format!("sound S{i} = (pulse, 0x80, 0, 4); ")).collect();
    assert!(m(&format!("{many} fn main() {{}}")).contains("at most 32"));
}

// ---------------------------------------------------------------- US4 coexistence

#[test]
fn sound_effects_borrow_pulse_2_from_the_harmony_then_give_it_back() {
    let (_, per) = trace(
        "music M = \"L8 C D E F\"; music B = \"O2 L4 C G\"; music H = \"L16 C\"; var t: u8;
         fn main() { play_music(M, B, H); loop { wait_vblank(); t += 1; if t == 20 { sfx(SFX_JUMP); } } }",
        120,
    );
    // SFX_JUMP writes period high byte 0, the harmony (C4, period $1AB) writes 1.
    let start = writes(&per, 0x4007).into_iter().find(|(_, v)| *v == 0).unwrap().0;
    let end = writes(&per, 0x4015).into_iter().find(|(f, v)| *f > start && v & 0x02 == 0).unwrap().0;
    assert!(end - start >= 14, "effect lasted {} frames", end - start);
    let harmony_on: Vec<usize> = writes(&per, 0x4007).into_iter().filter(|(_, v)| *v == 1).map(|(f, _)| f).collect();
    assert!(harmony_on.iter().any(|f| *f < start), "harmony played before");
    assert!(!harmony_on.iter().any(|f| (start..=end).contains(f)), "harmony yielded: {harmony_on:?} in {start}..={end}");
    assert!(harmony_on.iter().any(|f| *f > end), "harmony came back");
    // melody keeps its 15-frame grid throughout
    let mel: Vec<usize> = writes(&per, 0x4002).into_iter().map(|(f, _)| f).collect();
    assert!(mel.windows(2).all(|w| w[1] - w[0] == 15), "{mel:?}");
}

#[test]
fn stop_sound_also_stops_the_music() {
    let (_, per) = trace(
        "music M = \"L8 CDEFGAB\"; var t: u8;
         fn main() { play_music(M); loop { wait_vblank(); t += 1; if t == 20 { stop_sound(); } } }",
        100,
    );
    assert!(writes(&per, 0x4002).iter().all(|(f, _)| *f < 25));
    assert_eq!(writes(&per, 0x4015).last().unwrap().1, 0);
}
