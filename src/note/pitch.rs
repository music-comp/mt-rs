use crate::interval::Interval;
use crate::note::errors::NoteError;
use crate::scale::{Direction, Mode};
use regex::{Match, Regex};
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;
use strum_macros::EnumIter;

static REGEX_PITCH: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[ABCDEFGabcdefg][b♭#s𝄪x]*").unwrap());

/// A note letter without an accidental.
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum NoteLetter {
    C = 0,
    D = 1,
    E = 2,
    F = 3,
    G = 4,
    A = 5,
    B = 6,
}

impl NoteLetter {
    /// Number of letter steps from self to other (ascending, 0-6).
    /// C.distance_to(E) = 2, B.distance_to(D) = 2 (wrapping).
    pub fn distance_to(self, other: NoteLetter) -> u8 {
        let from = self as u8;
        let to = other as u8;
        (to + 7 - from) % 7
    }
}

impl fmt::Display for NoteLetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            NoteLetter::C => "C",
            NoteLetter::D => "D",
            NoteLetter::E => "E",
            NoteLetter::F => "F",
            NoteLetter::G => "G",
            NoteLetter::A => "A",
            NoteLetter::B => "B",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitch {
    pub letter: NoteLetter,
    pub accidental: i8,
}

impl Pitch {
    /// Create a pitch with a given note letter and accidental
    pub fn new(letter: NoteLetter, accidental: i8) -> Self {
        Self { letter, accidental }
    }

    /// Returns true if two pitches sound the same (same semitone value).
    pub fn is_enharmonic_to(&self, other: &Pitch) -> bool {
        self.as_u8() == other.as_u8()
    }

    /// Create a pitch from an integer, where 0 is C and everything climbs up from there,
    /// looping back around once it reaches 12.
    pub fn from_u8(val: u8) -> Self {
        use NoteLetter::*;
        match val % 12 {
            0 => Pitch::new(C, 0),
            1 => Pitch::new(C, 1),
            2 => Pitch::new(D, 0),
            3 => Pitch::new(D, 1),
            4 => Pitch::new(E, 0),
            5 => Pitch::new(F, 0),
            6 => Pitch::new(F, 1),
            7 => Pitch::new(G, 0),
            8 => Pitch::new(G, 1),
            9 => Pitch::new(A, 0),
            10 => Pitch::new(A, 1),
            11 => Pitch::new(B, 0),
            _ => unreachable!("val % 12 should always be 0-11"),
        }
    }

    /// Create a pitch from an integer with a preferred spelling based on mode and scale type
    pub fn from_u8_with_scale_context(val: u8, mode: Option<Mode>, direction: Direction) -> Self {
        use super::PitchSymbol;
        use PitchSymbol::*;
        use Mode::*;

        let pitch_number = val % 12;

        // Determine spelling based on mode and pitch number
        let use_flats = match (mode, pitch_number) {
            // Dorian: flat 3rd and 7th
            (Some(Dorian), 3 | 10) => true,
            // Phrygian: flat 2nd, 3rd, 6th, 7th
            (Some(Phrygian), 1 | 3 | 8 | 10) => true,
            // Lydian: all sharps
            (Some(Lydian), _) => false,
            // Mixolydian: flat 7th
            (Some(Mixolydian), 10) => true,
            // Aeolian: flat 3rd, 6th, 7th
            (Some(Aeolian), 3 | 8 | 10) => true,
            // Locrian: flat 2nd, 3rd, 5th, 6th, 7th
            (Some(Locrian), 1 | 3 | 6 | 8 | 10) => true,
            // For melodic minor and other modes, use the direction
            (None, _) => matches!(direction, Direction::Descending),
            // For other modes, use sharps by default
            _ => false,
        };

        if use_flats {
            match pitch_number {
                0 => Pitch::from(C),
                1 => Pitch::from(Db),
                2 => Pitch::from(D),
                3 => Pitch::from(Eb),
                4 => Pitch::from(E),
                5 => Pitch::from(F),
                6 => Pitch::from(Gb),
                7 => Pitch::from(G),
                8 => Pitch::from(Ab),
                9 => Pitch::from(A),
                10 => Pitch::from(Bb),
                11 => Pitch::from(B),
                _ => unreachable!(),
            }
        } else {
            match pitch_number {
                0 => Pitch::from(C),
                1 => Pitch::from(Cs),
                2 => Pitch::from(D),
                3 => Pitch::from(Ds),
                4 => Pitch::from(E),
                5 => Pitch::from(F),
                6 => Pitch::from(Fs),
                7 => Pitch::from(G),
                8 => Pitch::from(Gs),
                9 => Pitch::from(A),
                10 => Pitch::from(As),
                11 => Pitch::from(B),
                _ => unreachable!(),
            }
        }
    }

    /// Create a pitch from an integer with a preferred spelling based on direction
    pub fn from_u8_with_direction(val: u8, direction: Direction) -> Self {
        // Default to no mode and diatonic scale when only direction is provided
        Self::from_u8_with_scale_context(val, None, direction)
    }

    /// Convert the pitch into its corresponding integer, where 0 is C and 11 is B.
    pub fn as_u8(&self) -> u8 {
        use NoteLetter::*;
        let base = match self.letter {
            C => 0,
            D => 2,
            E => 4,
            F => 5,
            G => 7,
            A => 9,
            B => 11,
        };
        
        // Handle negative accidentals correctly by adding 12 first
        
        ((base as i8 + self.accidental + 12) % 12) as u8
    }

    /// Create a pitch by moving up the given pitch by an interval.
    pub fn from_interval(pitch: Self, interval: Interval) -> Self {
        let current_pitch = pitch.as_u8();
        let new_pitch = current_pitch + interval.semitone_count;

        Self::from_u8(new_pitch)
    }

    /// Create a pitch by moving down the given pitch by an interval.
    pub fn from_interval_down(pitch: Self, interval: Interval) -> Self {
        let current_pitch = pitch.as_u8();
        let new_pitch = (12 + (current_pitch as i16 - interval.semitone_count as i16)) % 12;

        Self::from_u8(new_pitch as u8)
    }

    /// Create a pitch by moving up the given pitch by an interval with scale context.
    pub fn from_interval_with_context(pitch: Self, interval: Interval, mode: Option<Mode>, direction: Direction) -> Self {
        let current_pitch = pitch.as_u8();
        let new_pitch = current_pitch + interval.semitone_count;
        Self::from_u8_with_scale_context(new_pitch, mode, direction)
    }

    /// Create a pitch by moving down the given pitch by an interval with scale context.
    pub fn from_interval_down_with_context(pitch: Self, interval: Interval, mode: Option<Mode>, direction: Direction) -> Self {
        let current_pitch = pitch.as_u8();
        let new_pitch = (12 + (current_pitch as i16 - interval.semitone_count as i16)) % 12;
        Self::from_u8_with_scale_context(new_pitch as u8, mode, direction)
    }

    /// Attempt to parse a pitch from a string. It should contain the name of the note in either
    /// uppercase or lowercase, followed by `#`, `s`, `S`, or `♯` for sharps and `b` or `♭` for
    /// flats.
    pub fn try_parse(string: &str) -> Option<Self> {
        use NoteLetter::*;
        let mut characters = string.chars();

        let first_char = characters.next()?;
        let letter = match first_char {
            'C' | 'c' => C,
            'D' | 'd' => D,
            'E' | 'e' => E,
            'F' | 'f' => F,
            'G' | 'g' => G,
            'A' | 'a' => A,
            'B' | 'b' => B,
            _ => return None,
        };

        let mut accidental: i8 = 0;
        let mut direction: Option<i8> = None; // None = undecided, Some(1) = sharps, Some(-1) = flats
        for ch in characters {
            match ch {
                '#' | 's' | 'S' | '♯' => {
                    if direction == Some(-1) { return None; }
                    direction = Some(1);
                    accidental += 1;
                }
                '𝄪' | 'x' => {
                    if direction == Some(-1) { return None; }
                    direction = Some(1);
                    accidental += 2;
                }
                'b' | '♭' => {
                    if direction == Some(1) { return None; }
                    direction = Some(-1);
                    accidental -= 1;
                }
                _ => return None,
            }
        }

        Some(Pitch { letter, accidental })
    }

    /// Parse the pitch using a regex, with the same algorithm as described in `from_str`.
    pub fn from_regex(string: &str) -> Result<(Self, Match<'_>), NoteError> {
        let pitch_match = REGEX_PITCH.find(string).ok_or(NoteError::InvalidPitch)?;

        let pitch = Self::try_parse(&string[pitch_match.start()..pitch_match.end()])
            .ok_or(NoteError::InvalidPitch)?;

        Ok((pitch, pitch_match))
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.letter)?;
        let acc_char = if self.accidental < 0 { 'b' } else { '#' };
        for _ in 0..self.accidental.abs() {
            write!(fmt, "{}", acc_char)?;
        }
        Ok(())
    }
}

impl FromStr for Pitch {
    type Err = NoteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s).ok_or(NoteError::InvalidPitch)
    }
}
