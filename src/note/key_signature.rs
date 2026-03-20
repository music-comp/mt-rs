use crate::note::{NoteLetter, Pitch, PitchSymbol};
use crate::scale::Mode;
use std::collections::HashMap;
use std::sync::LazyLock;

static KEY_SIGNATURE_SPELLINGS: LazyLock<HashMap<(NoteLetter, i8), Vec<PitchSymbol>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
        m.insert((NoteLetter::C, 0), vec![PitchSymbol::C, PitchSymbol::D, PitchSymbol::E, PitchSymbol::F, PitchSymbol::G, PitchSymbol::A, PitchSymbol::B]);
        m.insert((NoteLetter::G, 0), vec![PitchSymbol::G, PitchSymbol::A, PitchSymbol::B, PitchSymbol::C, PitchSymbol::D, PitchSymbol::E, PitchSymbol::Fs]);
        m.insert((NoteLetter::D, 0), vec![PitchSymbol::D, PitchSymbol::E, PitchSymbol::Fs, PitchSymbol::G, PitchSymbol::A, PitchSymbol::B, PitchSymbol::Cs]);
        m.insert((NoteLetter::A, 0), vec![PitchSymbol::A, PitchSymbol::B, PitchSymbol::Cs, PitchSymbol::D, PitchSymbol::E, PitchSymbol::Fs, PitchSymbol::Gs]);
        m.insert((NoteLetter::E, 0), vec![PitchSymbol::E, PitchSymbol::Fs, PitchSymbol::Gs, PitchSymbol::A, PitchSymbol::B, PitchSymbol::Cs, PitchSymbol::Ds]);
        m.insert((NoteLetter::B, 0), vec![PitchSymbol::B, PitchSymbol::Cs, PitchSymbol::Ds, PitchSymbol::E, PitchSymbol::Fs, PitchSymbol::Gs, PitchSymbol::As]);
        m.insert((NoteLetter::F, 0), vec![PitchSymbol::F, PitchSymbol::G, PitchSymbol::A, PitchSymbol::Bb, PitchSymbol::C, PitchSymbol::D, PitchSymbol::E]);
        m.insert((NoteLetter::B, -1), vec![PitchSymbol::Bb, PitchSymbol::C, PitchSymbol::D, PitchSymbol::Eb, PitchSymbol::F, PitchSymbol::G, PitchSymbol::A]);
        m.insert((NoteLetter::E, -1), vec![PitchSymbol::Eb, PitchSymbol::F, PitchSymbol::G, PitchSymbol::Ab, PitchSymbol::Bb, PitchSymbol::C, PitchSymbol::D]);
        m.insert((NoteLetter::A, -1), vec![PitchSymbol::Ab, PitchSymbol::Bb, PitchSymbol::C, PitchSymbol::Db, PitchSymbol::Eb, PitchSymbol::F, PitchSymbol::G]);
        m.insert((NoteLetter::D, -1), vec![PitchSymbol::Db, PitchSymbol::Eb, PitchSymbol::F, PitchSymbol::Gb, PitchSymbol::Ab, PitchSymbol::Bb, PitchSymbol::C]);
        m.insert((NoteLetter::G, -1), vec![PitchSymbol::Gb, PitchSymbol::Ab, PitchSymbol::Bb, PitchSymbol::Cb, PitchSymbol::Db, PitchSymbol::Eb, PitchSymbol::F]);
        m.insert((NoteLetter::F, 1), vec![PitchSymbol::Fs, PitchSymbol::Gs, PitchSymbol::As, PitchSymbol::B, PitchSymbol::Cs, PitchSymbol::Ds, PitchSymbol::Es]);
        m.insert((NoteLetter::C, 1), vec![PitchSymbol::Cs, PitchSymbol::Ds, PitchSymbol::Es, PitchSymbol::Fs, PitchSymbol::Gs, PitchSymbol::As, PitchSymbol::Bs]);
    m
});

/// A key signature.
#[derive(Debug, Clone, PartialEq)]
pub struct KeySignature {
    /// The tonic of the key signature.
    pub tonic: Pitch,
    /// The mode of the key signature.
    pub mode: Option<Mode>,
}

impl KeySignature {
    /// Create a new key signature.
    pub fn new(tonic: Pitch) -> Self {
        KeySignature {
            tonic,
            mode: None,
        }
    }

    /// Create a new key signature with a mode.
    pub fn new_with_mode(tonic: Pitch, mode: Option<Mode>) -> Self {
        KeySignature { tonic, mode }
    }

    /// Create a key signature appropriate for a chord of the given quality.
    /// Minor/diminished chords use the minor (Aeolian) key signature of their root.
    pub fn for_chord(root: Pitch, quality: crate::chord::Quality) -> Self {
        use crate::chord::Quality;
        let mode = match quality {
            Quality::Minor | Quality::Diminished | Quality::HalfDiminished => {
                Some(Mode::Aeolian)
            },
            _ => None,  // Major, Dominant, Augmented, Suspended use major key sig
        };
        KeySignature { tonic: root, mode }
    }

    /// Get the relative major key for a given semitone value.
    /// For ambiguous semitones (1=Db/C#, 6=Gb/F#), uses the tonic's
    /// accidental context to pick the appropriate enharmonic spelling.
    fn get_relative_major_key(&self, semitones: u8) -> (NoteLetter, i8) {
        use NoteLetter::*;
        let prefer_sharps = self.tonic.accidental > 0
            || (self.tonic.accidental >= 0 && matches!(
                self.tonic.letter,
                NoteLetter::G | NoteLetter::D | NoteLetter::A | NoteLetter::E | NoteLetter::B
            ));

        match semitones {
            0 => (C, 0),
            1 => if prefer_sharps { (C, 1) } else { (D, -1) },
            2 => (D, 0),
            3 => (E, -1),  // Eb major (D# major is theoretical)
            4 => (E, 0),
            5 => (F, 0),
            6 => if prefer_sharps { (F, 1) } else { (G, -1) },
            7 => (G, 0),
            8 => (A, -1),  // Ab major (G# major is theoretical)
            9 => (A, 0),
            10 => (B, -1), // Bb major (A# major is theoretical)
            11 => (B, 0),
            _ => unreachable!(),
        }
    }

    /// Compute the relative major key letter and accidental for the current tonic and mode.
    fn resolve_key(&self) -> (NoteLetter, i8) {
        match self.mode {
            Some(Mode::Aeolian) | Some(Mode::HarmonicMinor) | Some(Mode::MelodicMinor) => {
                // All minor-family modes share the same key signature as natural minor (Aeolian).
                // Aeolian is the 6th degree: relative major is 3 semitones up.
                let relative_major_semitones = (self.tonic.as_u8() + 3) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Dorian) => {
                let relative_major_semitones = (self.tonic.as_u8() + 12 - 2) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Phrygian) => {
                let relative_major_semitones = (self.tonic.as_u8() + 12 - 4) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Lydian) => {
                let relative_major_semitones = (self.tonic.as_u8() + 12 - 5) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Mixolydian) => {
                let relative_major_semitones = (self.tonic.as_u8() + 12 - 7) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Locrian) => {
                let relative_major_semitones = (self.tonic.as_u8() + 12 - 11) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            Some(Mode::Blues) | Some(Mode::PentatonicMinor) => {
                // Blues and minor pentatonic use the same key signature as Aeolian
                let relative_major_semitones = (self.tonic.as_u8() + 3) % 12;
                self.get_relative_major_key(relative_major_semitones)
            },
            _ => {
                // Ionian, PentatonicMajor, Chromatic, WholeTone, or no mode: use tonic as major
                (self.tonic.letter, self.tonic.accidental)
            }
        }
    }

    /// Determine if a key (identified by its relative major) prefers sharps or flats
    /// for chromatic notes not in the key signature.
    fn is_sharp_context(key_letter: NoteLetter, key_accidental: i8) -> bool {
        // C major (no sharps/flats) and sharp keys prefer sharps for chromatic notes
        matches!(
            (key_letter, key_accidental),
            (NoteLetter::C, 0) | (NoteLetter::G, 0) | (NoteLetter::D, 0) |
            (NoteLetter::A, 0) | (NoteLetter::E, 0) | (NoteLetter::B, 0) |
            (NoteLetter::F, 1) | (NoteLetter::C, 1)
        )
    }

    pub fn get_preferred_spelling(&self, pitch: Pitch) -> PitchSymbol {
        use PitchSymbol::*;

        let (key_letter, key_accidental) = self.resolve_key();

        // First, check if this pitch appears in the key signature's diatonic spellings
        if let Some(key_spellings) = KEY_SIGNATURE_SPELLINGS.get(&(key_letter, key_accidental)) {
            for &spelling in key_spellings {
                if Pitch::from(spelling).as_u8() == pitch.as_u8() {
                    return spelling;
                }
            }
        }

        // For chromatic notes not in the key, use sharp/flat preference based on key context
        let is_sharp = Self::is_sharp_context(key_letter, key_accidental);

        match pitch.as_u8() {
            0 => C,
            1 => if is_sharp { Cs } else { Db },
            2 => D,
            3 => if is_sharp { Ds } else { Eb },
            4 => E,
            5 => F,
            6 => if is_sharp { Fs } else { Gb },
            7 => G,
            8 => if is_sharp { Gs } else { Ab },
            9 => A,
            10 => if is_sharp { As } else { Bb },
            11 => B,
            _ => unreachable!(),
        }
    }
}
