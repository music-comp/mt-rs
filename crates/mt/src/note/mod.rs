//! Individual notes.

mod errors;
mod key_signature;
mod pitch;
mod pitch_symbol;

pub use errors::NoteError;
pub use key_signature::KeySignature;
pub use pitch::{NoteLetter, Pitch};
pub use pitch_symbol::PitchSymbol;

use std::fmt;
use std::fmt::Formatter;

/// A note.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note {
    /// The pitch of the note (A, B, C#, etc).
    pub pitch: Pitch,
    /// The octave of the note in standard notation.
    pub octave: u8,
}

impl Note {
    /// Create a new note.
    pub fn new(pitch: Pitch, octave: u8) -> Self {
        Note { pitch, octave }
    }

    /// Convert to MIDI pitch number (0-127).
    ///
    /// Middle C (C4) = 60, A4 (440Hz) = 69.
    /// Uses standard MIDI octave convention where octave -1 starts at 0.
    #[cfg(feature = "midi")]
    pub fn midi_pitch(&self) -> u8 {
        let semitone = self.pitch.as_u8();
        let midi_value = (self.octave as u16 + 1) * 12 + semitone as u16;
        midi_value.min(127) as u8
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.pitch)
    }
}

/// Parse a pitch string like "C4", "F#3", "Bb-1" into a MIDI pitch number (0–127).
///
/// Accepts ASCII accidentals (`#`, `b`) and Unicode (`♯`, `♭`).
/// Octave convention: C-1 = 0, C0 = 12, C4 = 60 (Middle C).
///
/// # Errors
///
/// Returns [`NoteError::InvalidPitch`] if the string is malformed or the
/// resulting pitch is outside MIDI range 0–127.
///
/// # Examples
///
/// ```
/// use music_comp_mt::note::parse_midi_pitch;
///
/// assert_eq!(parse_midi_pitch("C4").unwrap(), 60);
/// assert_eq!(parse_midi_pitch("A4").unwrap(), 69);
/// assert_eq!(parse_midi_pitch("F#3").unwrap(), 54);
/// assert_eq!(parse_midi_pitch("Bb3").unwrap(), 58);
/// ```
pub fn parse_midi_pitch(s: &str) -> Result<u8, NoteError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(NoteError::InvalidPitch);
    }

    // Find where the octave number starts: scan from the end for digits and optional leading '-'
    let mut octave_start = s.len();
    for (i, ch) in s.char_indices().rev() {
        if ch.is_ascii_digit() || (ch == '-' && i > 0) {
            octave_start = i;
        } else {
            break;
        }
    }

    if octave_start == 0 || octave_start == s.len() {
        return Err(NoteError::InvalidPitch);
    }

    let pitch_part = &s[..octave_start];
    let octave_part = &s[octave_start..];

    let pitch = Pitch::try_parse(pitch_part).ok_or(NoteError::InvalidPitch)?;
    let octave: i16 = octave_part.parse().map_err(|_| NoteError::InvalidPitch)?;

    let midi = (octave + 1) as i32 * 12 + pitch.as_u8() as i32;
    if !(0..=127).contains(&midi) {
        return Err(NoteError::InvalidPitch);
    }
    Ok(midi as u8)
}

/// A type that can produce a sequence of notes.
pub trait Notes {
    /// Get the sequence of notes.
    fn notes(&self) -> Vec<Note>;

    /// Format the notes as a numbered list string.
    ///
    /// Returns a string like:
    /// ```text
    /// Notes:
    ///   1: C
    ///   2: E
    ///   3: G
    /// ```
    fn format_notes(&self) -> String {
        let notes = self.notes();
        let mut output = String::from("Notes:\n");
        for (i, note) in notes.iter().enumerate() {
            output.push_str(&format!("  {}: {}\n", i + 1, note.pitch));
        }
        output
    }

    /// Print the sequence of notes to stdout.
    fn print_notes(&self) {
        print!("{}", self.format_notes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "midi")]
    use crate::note::PitchSymbol::*;

    #[test]
    fn test_parse_midi_pitch_basic() {
        assert_eq!(parse_midi_pitch("C4").unwrap(), 60);
        assert_eq!(parse_midi_pitch("A4").unwrap(), 69);
        assert_eq!(parse_midi_pitch("C0").unwrap(), 12);
        assert_eq!(parse_midi_pitch("C-1").unwrap(), 0);
        assert_eq!(parse_midi_pitch("C9").unwrap(), 120);
        assert_eq!(parse_midi_pitch("G9").unwrap(), 127);
    }

    #[test]
    fn test_parse_midi_pitch_sharps() {
        assert_eq!(parse_midi_pitch("C#3").unwrap(), 49);
        assert_eq!(parse_midi_pitch("F#4").unwrap(), 66);
        assert_eq!(parse_midi_pitch("F#3").unwrap(), 54);
    }

    #[test]
    fn test_parse_midi_pitch_flats_ascii() {
        assert_eq!(parse_midi_pitch("Db3").unwrap(), 49);
        assert_eq!(parse_midi_pitch("Bb3").unwrap(), 58);
    }

    #[test]
    fn test_parse_midi_pitch_flats_unicode() {
        assert_eq!(parse_midi_pitch("B\u{266d}3").unwrap(), 58);
        assert_eq!(parse_midi_pitch("D\u{266d}3").unwrap(), 49);
    }

    #[test]
    fn test_parse_midi_pitch_sharps_unicode() {
        assert_eq!(parse_midi_pitch("C\u{266f}4").unwrap(), 61);
    }

    #[test]
    fn test_parse_midi_pitch_out_of_range() {
        assert!(parse_midi_pitch("C10").is_err());
        assert!(parse_midi_pitch("G#9").is_err());
    }

    #[test]
    fn test_parse_midi_pitch_malformed() {
        assert!(parse_midi_pitch("").is_err());
        assert!(parse_midi_pitch("X3").is_err());
        assert!(parse_midi_pitch("C").is_err());
        assert!(parse_midi_pitch("3C").is_err());
    }

    #[test]
    #[cfg(feature = "midi")]
    fn midi_pitch_middle_c() {
        let note = Note::new(Pitch::from(C), 4);
        assert_eq!(note.midi_pitch(), 60);
    }

    #[test]
    #[cfg(feature = "midi")]
    fn midi_pitch_a440() {
        let note = Note::new(Pitch::from(A), 4);
        assert_eq!(note.midi_pitch(), 69);
    }

    #[test]
    #[cfg(feature = "midi")]
    fn midi_pitch_octaves() {
        assert_eq!(Note::new(Pitch::from(C), 0).midi_pitch(), 12);
        assert_eq!(Note::new(Pitch::from(C), 1).midi_pitch(), 24);
        assert_eq!(Note::new(Pitch::from(C), 2).midi_pitch(), 36);
        assert_eq!(Note::new(Pitch::from(C), 3).midi_pitch(), 48);
        assert_eq!(Note::new(Pitch::from(C), 5).midi_pitch(), 72);
    }

    #[test]
    #[cfg(feature = "midi")]
    fn midi_pitch_accidentals() {
        assert_eq!(Note::new(Pitch::from(Cs), 4).midi_pitch(), 61);
        assert_eq!(Note::new(Pitch::from(Db), 4).midi_pitch(), 61);
        assert_eq!(Note::new(Pitch::from(Fs), 4).midi_pitch(), 66);
    }

    #[test]
    #[cfg(feature = "midi")]
    fn midi_pitch_clamps_to_127() {
        // Very high octave should clamp
        let note = Note::new(Pitch::from(G), 10);
        assert!(note.midi_pitch() <= 127);
    }
}
