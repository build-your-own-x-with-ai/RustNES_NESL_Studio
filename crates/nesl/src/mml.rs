//! MML (Music Macro Language) → 2-byte event stream played by the runtime music engine.
//!
//! | first byte  | second byte | meaning                                   |
//! |-------------|-------------|-------------------------------------------|
//! | `0x00-0x5F` | frames      | note `octave * 12 + semitone`, retriggers |
//! | `0x80-0x8F` | 0           | set volume                                |
//! | `0x90-0x93` | 0           | set duty cycle                            |
//! | `0xFD`      | frames      | continue the current note                 |
//! | `0xFE`      | frames      | rest                                      |
//! | `0xFF`      | 0           | end of song (loop)                        |

use crate::ast::Pos;
use crate::Diagnostic;

pub const EV_CONT: u8 = 0xFD;
pub const EV_REST: u8 = 0xFE;
pub const EV_END: u8 = 0xFF;
pub const MAX_NOTE: u8 = 95; // B7

/// A compiled song plus its lowest note (for per-voice range checks).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub events: Vec<u8>,
    pub lowest: Option<(u8, Pos)>,
}

struct State {
    octave: i32,
    length: u32,
    tempo: u32,
    /// Accumulated time in frames, so per-note rounding never drifts.
    time: f64,
    events: Vec<u8>,
    total: u32,
    /// Pitch of a note ending in `&` that the next note may continue.
    tied: Option<u8>,
    lowest: Option<(u8, Pos)>,
}

impl State {
    /// Frames for a note of `len` (1-64) with `dots`, advancing the clock.
    fn frames(&mut self, len: u32, dots: u32) -> u32 {
        let quarter = 3600.0 / self.tempo as f64;
        let mut d = quarter * 4.0 / len as f64;
        let mut add = d / 2.0;
        for _ in 0..dots {
            d += add;
            add /= 2.0;
        }
        let start = self.time.round();
        self.time += d;
        let f = (self.time.round() - start) as u32;
        self.total += f;
        f
    }

    fn push_split(&mut self, first: u8, mut frames: u32, rest: u8) {
        let mut code = first;
        while frames > 0 {
            let n = frames.min(255);
            self.events.extend_from_slice(&[code, n as u8]);
            frames -= n;
            code = rest;
        }
    }
}

/// Compiles MML strings (concatenated in order; state carries across them).
pub fn compile(parts: &[(String, Pos)]) -> Result<Song, Diagnostic> {
    let mut st = State { octave: 4, length: 4, tempo: 120, time: 0.0, events: Vec::new(), total: 0, tied: None, lowest: None };
    for (text, start) in parts {
        let chars: Vec<char> = text.chars().collect();
        // Columns count the opening quote of the string literal.
        let at = |i: usize| Pos { line: start.line, col: start.col + 1 + i as u32 };
        let mut i = 0;
        let number = |i: &mut usize| -> Option<u32> {
            let s = *i;
            while *i < chars.len() && chars[*i].is_ascii_digit() {
                *i += 1;
            }
            chars[s..*i].iter().collect::<String>().parse().ok()
        };
        while i < chars.len() {
            let c = chars[i].to_ascii_uppercase();
            let pos = at(i);
            i += 1;
            let need = |v: Option<u32>, what: &str| v.ok_or_else(|| Diagnostic::new(pos, format!("`{c}` expects a number ({what})")));
            let check = |v: u32, lo: u32, hi: u32, what: &str| {
                if (lo..=hi).contains(&v) {
                    Ok(v)
                } else {
                    Err(Diagnostic::new(pos, format!("{what} {v} is out of range ({lo}-{hi})")))
                }
            };
            match c {
                ' ' | '\t' | '\n' | '\r' | '|' => {}
                'A'..='G' | 'R' => {
                    let mut semi = match c {
                        'C' => 0, 'D' => 2, 'E' => 4, 'F' => 5, 'G' => 7, 'A' => 9, 'B' => 11, _ => 0,
                    };
                    while i < chars.len() && matches!(chars[i], '+' | '#' | '-') {
                        semi += if chars[i] == '-' { -1 } else { 1 };
                        i += 1;
                    }
                    let len = match number(&mut i) {
                        Some(n) => check(n, 1, 64, "note length")?,
                        None => st.length,
                    };
                    let mut dots = 0;
                    while i < chars.len() && chars[i] == '.' {
                        dots += 1;
                        i += 1;
                    }
                    let tie = i < chars.len() && chars[i] == '&';
                    if tie {
                        i += 1;
                    }
                    let frames = st.frames(len, dots);
                    if c == 'R' {
                        st.tied = None;
                        st.push_split(EV_REST, frames, EV_REST);
                        continue;
                    }
                    let n = st.octave * 12 + semi;
                    if !(0..=MAX_NOTE as i32).contains(&n) {
                        return Err(Diagnostic::new(pos, format!("note `{c}` in octave {} is out of range (C0-B7)", st.octave)));
                    }
                    let n = n as u8;
                    if st.lowest.is_none_or(|(l, _)| n < l) {
                        st.lowest = Some((n, pos));
                    }
                    let continues = st.tied == Some(n);
                    st.tied = if tie { Some(n) } else { None };
                    if frames == 0 {
                        continue;
                    }
                    // Without a tie the last frame is silent, so repeated notes are distinct.
                    let (sound, gap) = if tie || frames < 2 { (frames, 0) } else { (frames - 1, 1) };
                    st.push_split(if continues { EV_CONT } else { n }, sound, EV_CONT);
                    if gap > 0 {
                        st.events.extend_from_slice(&[EV_REST, 1]);
                    }
                }
                'L' => st.length = check(need(number(&mut i), "1-64")?, 1, 64, "length")?,
                'O' => st.octave = check(need(number(&mut i), "0-7")?, 0, 7, "octave")? as i32,
                'T' => st.tempo = check(need(number(&mut i), "30-300")?, 30, 300, "tempo")?,
                'V' => {
                    let v = check(need(number(&mut i), "0-15")?, 0, 15, "volume")?;
                    st.events.extend_from_slice(&[0x80 | v as u8, 0]);
                }
                '@' => {
                    let v = check(need(number(&mut i), "0-3")?, 0, 3, "duty")?;
                    st.events.extend_from_slice(&[0x90 | v as u8, 0]);
                }
                '>' => st.octave += 1,
                '<' => st.octave -= 1,
                other => return Err(Diagnostic::new(pos, format!("unknown MML command `{other}`"))),
            }
        }
    }
    // A song must take time, otherwise the player would loop forever within one frame.
    // A score of only rests is fine (silence of a given length).
    if st.total == 0 {
        let pos = parts.first().map(|(_, p)| *p).unwrap_or_default();
        return Err(Diagnostic::new(pos, "music has no notes (its total length is zero)"));
    }
    st.events.extend_from_slice(&[EV_END, 0]);
    Ok(Song { events: st.events, lowest: st.lowest })
}

/// NES APU timer period for MIDI-style note `n` (C0 = 0, A4 = 57) on a pulse channel.
pub fn pulse_period(n: u8) -> u16 {
    let freq = 440.0 * 2f64.powf((n as f64 - 57.0) / 12.0);
    let p = (1_789_773.0 / (16.0 * freq)).round() - 1.0;
    p.clamp(8.0, 2047.0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(src: &str) -> Vec<u8> {
        compile(&[(src.to_string(), Pos { line: 1, col: 1 })]).unwrap_or_else(|e| panic!("{src}: {e}")).events
    }

    fn err(src: &str) -> Diagnostic {
        compile(&[(src.to_string(), Pos { line: 3, col: 11 })]).unwrap_err()
    }

    #[test]
    fn quarter_notes_at_150_bpm_are_24_frames_with_a_one_frame_gap() {
        assert_eq!(ev("T150 L4 C D E"), vec![48, 23, EV_REST, 1, 50, 23, EV_REST, 1, 52, 23, EV_REST, 1, EV_END, 0]);
    }

    #[test]
    fn default_tempo_octave_and_length() {
        // T120 O4 L4: a quarter note is 30 frames
        assert_eq!(ev("A"), vec![57, 29, EV_REST, 1, EV_END, 0]);
    }

    #[test]
    fn lengths_dots_and_rests() {
        assert_eq!(ev("C4."), vec![48, 44, EV_REST, 1, EV_END, 0]);
        assert_eq!(ev("C8 R4"), vec![48, 14, EV_REST, 1, EV_REST, 30, EV_END, 0]);
        assert_eq!(ev("L16 C"), vec![48, 7, EV_REST, 1, EV_END, 0]); // 7.5 frames rounds to 8
        assert_eq!(ev("C2.."), vec![48, 104, EV_REST, 1, EV_END, 0]);
    }

    #[test]
    fn accidentals_and_octave_changes() {
        let notes: Vec<u8> = ev("L64 C+ D# E- O4 B > C < C O2 A").chunks(2).map(|c| c[0]).filter(|n| *n <= MAX_NOTE).collect();
        assert_eq!(notes, vec![49, 51, 51, 59, 60, 48, 33]);
    }

    #[test]
    fn ties_continue_without_retriggering_or_gaps() {
        assert_eq!(ev("C4&C8"), vec![48, 30, EV_CONT, 14, EV_REST, 1, EV_END, 0]);
        // a tie to a different pitch is legato: new note, but no gap before it
        assert_eq!(ev("C4&D4"), vec![48, 30, 50, 29, EV_REST, 1, EV_END, 0]);
    }

    #[test]
    fn long_notes_are_split_into_255_frame_chunks() {
        // T30: quarter = 120 frames, whole = 480
        assert_eq!(ev("T30 C1"), vec![48, 255, EV_CONT, 224, EV_REST, 1, EV_END, 0]);
        assert_eq!(ev("T30 R1"), vec![EV_REST, 255, EV_REST, 225, EV_END, 0]);
    }

    #[test]
    fn timing_is_accumulated_so_loops_do_not_drift() {
        // T130 eighth = 13.846 frames; four of them must total round(55.38) = 55
        let e = ev("T130 L8 CCCC");
        let total: u32 = e.chunks(2).filter(|c| c[0] != EV_END).map(|c| c[1] as u32).sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn volume_duty_bars_case_and_multiple_parts() {
        assert_eq!(ev("v5 @1 | c"), vec![0x85, 0, 0x91, 0, 48, 29, EV_REST, 1, EV_END, 0]);
        let parts = [("O5 L8".to_string(), Pos::default()), ("C".to_string(), Pos::default())];
        assert_eq!(compile(&parts).unwrap().events, vec![60, 14, EV_REST, 1, EV_END, 0]);
    }

    #[test]
    fn reports_the_lowest_note_and_its_position() {
        let s = compile(&[("O3 C O1 A O5 C".to_string(), Pos { line: 2, col: 11 })]).unwrap();
        assert_eq!(s.lowest, Some((21, Pos { line: 2, col: 20 }))); // `A` is index 8
    }

    #[test]
    fn errors_point_into_the_string() {
        let e = err("C X");
        assert!(e.message.contains("unknown MML command `X`"), "{}", e.message);
        assert_eq!((e.line, e.col), (3, 14)); // quote at col 11, so `X` (index 2) is at 11 + 1 + 2
        assert!(err("O0 C-").message.contains("out of range"));
        assert!(err("O7 B+").message.contains("out of range"));
        assert!(err("T10 C").message.contains("tempo"));
        assert!(err("L65 C").message.contains("length"));
        assert!(err("V16 C").message.contains("volume"));
        assert!(err("@4 C").message.contains("duty"));
        assert!(err("O C").message.contains("expects a number"));
        assert!(err("O4 V3").message.contains("no notes"));
    }

    #[test]
    fn pulse_periods_match_the_ntsc_apu() {
        assert_eq!(pulse_period(57), 253); // A4 = 440 Hz
        assert_eq!(pulse_period(45), 507); // A3
        assert_eq!(pulse_period(21), 2033); // A1, the lowest a pulse channel can play
        assert_eq!(pulse_period(95), 27); // B7
        assert_eq!(pulse_period(0), 0x7FF, "clamped");
    }
}
