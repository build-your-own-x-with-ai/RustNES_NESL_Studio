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

/// Compiles MML strings (concatenated in order; state carries across them).
pub fn compile(_parts: &[(String, Pos)]) -> Result<Song, Diagnostic> {
    Err(Diagnostic::new(Pos::default(), "not implemented"))
}

/// NES APU timer period for MIDI-style note `n` (C0 = 0, A4 = 57) on a pulse channel.
pub fn pulse_period(_n: u8) -> u16 {
    0
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
