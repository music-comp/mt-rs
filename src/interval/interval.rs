use crate::interval::errors::IntervalError;
use crate::note::{Note, Pitch};
use std::fmt;
use std::fmt::Display;
use strum_macros::Display;

/// The quality of an interval; major, minor, etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Quality {
    /// A perfect interval; unisons, fourths, fifths, and octaves.
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

impl Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Quality::Perfect => "P",
            Quality::Major => "M",
            Quality::Minor => "m",
            Quality::Augmented => "A",
            Quality::Diminished => "d",
        };
        write!(f, "{}", string)
    }
}

/// The number of an interval.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Number {
    Unison,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Octave,
    Ninth,
    Tenth,
    Eleventh,
    Twelfth,
    Thirteenth,
    Fourteenth,
    Fifteenth,
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Number::Unison => "1",
            Number::Second => "2",
            Number::Third => "3",
            Number::Fourth => "4",
            Number::Fifth => "5",
            Number::Sixth => "6",
            Number::Seventh => "7",
            Number::Octave => "8",
            Number::Ninth => "9",
            Number::Tenth => "10",
            Number::Eleventh => "11",
            Number::Twelfth => "12",
            Number::Thirteenth => "13",
            Number::Fourteenth => "14",
            Number::Fifteenth => "15",
        };
        write!(f, "{}", string)
    }
}

/// A step between notes.
#[derive(Display, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Step {
    /// A semitone step.
    Half,
    /// A tone step.
    Whole,
    /// A tritone step.
    Tritone,
}

/// An interval between two notes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval {
    /// The number of semitones between the notes.
    pub semitone_count: u8,
    /// The quality of the interval.
    pub quality: Quality,
    /// The number of the interval.
    pub number: Number,
    /// The step of the interval.
    pub step: Option<Step>,
}

impl Interval {
    /// Create a new interval.
    pub fn new(semitone_count: u8, quality: Quality, number: Number, step: Option<Step>) -> Self {
        Interval {
            semitone_count,
            quality,
            number,
            step,
        }
    }

    /// Creates multiple intervals each based on the number of semitones from the root.
    ///
    /// # Errors
    ///
    /// Fails if `sc` is greater than 12.
    pub fn from_semitones(semi_tones: &[u8]) -> Result<Vec<Self>, IntervalError> {
        let mut intervals: Vec<Interval> = vec![];

        if semi_tones.is_empty() {
            return Err(IntervalError::InvalidInterval);
        }

        for i in semi_tones {
            let interval = Self::from_semitone(*i)?;
            intervals.push(interval);
        }

        Ok(intervals)
    }

    /// Create an interval based on the number of semitones from the root.
    ///
    /// # Errors
    ///
    /// Fails if `sc` is greater than 12.
    pub fn from_semitone(sc: u8) -> Result<Self, IntervalError> {
        let (number, quality, mut step): (Number, Quality, Option<Step>);
        step = None;

        match sc {
            0 => {
                number = Number::Unison;
                quality = Quality::Perfect;
            }
            1 => {
                number = Number::Second;
                quality = Quality::Minor;
                step = Some(Step::Half);
            }
            2 => {
                number = Number::Second;
                quality = Quality::Major;
                step = Some(Step::Whole);
            }
            3 => {
                number = Number::Third;
                quality = Quality::Minor;
            }
            4 => {
                number = Number::Third;
                quality = Quality::Major;
            }
            5 => {
                number = Number::Fourth;
                quality = Quality::Perfect;
            }
            6 => {
                number = Number::Fifth;
                quality = Quality::Diminished;
                step = Some(Step::Tritone);
            }
            7 => {
                number = Number::Fifth;
                quality = Quality::Perfect;
            }
            8 => {
                number = Number::Sixth;
                quality = Quality::Minor;
            }
            9 => {
                number = Number::Sixth;
                quality = Quality::Major;
            }
            10 => {
                number = Number::Seventh;
                quality = Quality::Minor;
            }
            11 => {
                number = Number::Seventh;
                quality = Quality::Major;
            }
            12 => {
                number = Number::Octave;
                quality = Quality::Perfect;
            }
            13..=24 => {
                // Compound intervals = simple interval + octave
                let simple = Self::from_semitone(sc - 12)?;
                number = match simple.number {
                    Number::Unison => Number::Octave,
                    Number::Second => Number::Ninth,
                    Number::Third => Number::Tenth,
                    Number::Fourth => Number::Eleventh,
                    Number::Fifth => Number::Twelfth,
                    Number::Sixth => Number::Thirteenth,
                    Number::Seventh => Number::Fourteenth,
                    Number::Octave => Number::Fifteenth,
                    _ => return Err(IntervalError::InvalidInterval),
                };
                quality = simple.quality;
            }
            _ => {
                return Err(IntervalError::InvalidInterval);
            }
        };

        Ok(Interval {
            semitone_count: sc,
            number,
            quality,
            step,
        })
    }

    /// Returns true if this interval spans more than one octave.
    pub fn is_compound(&self) -> bool {
        self.semitone_count > 12
    }

    /// Returns the simple (within-octave) equivalent of a compound interval.
    pub fn simple(&self) -> Interval {
        if self.semitone_count <= 12 {
            *self
        } else {
            Interval::from_semitone(self.semitone_count - 12).unwrap_or(*self)
        }
    }

    /// Calculate the interval between two pitches using letter names to
    /// determine the interval number and semitone distance for quality.
    ///
    /// This distinguishes augmented 4th (F to B) from diminished 5th (F to Cb)
    /// even though both span 6 semitones. Only handles standard qualities
    /// (Perfect, Major, Minor, Augmented, Diminished); returns Err for
    /// doubly-augmented/diminished intervals.
    pub fn between(from: &Pitch, to: &Pitch) -> Result<Self, IntervalError> {
        let letter_dist = from.letter.distance_to(to.letter);
        let semitone_dist = (to.as_u8() + 12 - from.as_u8()) % 12;

        let number = match letter_dist {
            0 => Number::Unison,
            1 => Number::Second,
            2 => Number::Third,
            3 => Number::Fourth,
            4 => Number::Fifth,
            5 => Number::Sixth,
            6 => Number::Seventh,
            _ => return Err(IntervalError::InvalidInterval),
        };

        // Expected semitone count for the "natural" version of each interval
        let natural_semitones: u8 = match number {
            Number::Unison => 0,
            Number::Second => 2,
            Number::Third => 4,
            Number::Fourth => 5,
            Number::Fifth => 7,
            Number::Sixth => 9,
            Number::Seventh => 11,
            Number::Octave => 12,
            // between() only produces simple intervals (letter_dist 0-6)
            _ => return Err(IntervalError::InvalidInterval),
        };

        let is_perfect_kind = matches!(
            number,
            Number::Unison | Number::Fourth | Number::Fifth | Number::Octave
        );

        // diff: how many semitones above (+) or below (-) the natural interval
        let diff = semitone_dist as i8 - natural_semitones as i8;

        let quality = if is_perfect_kind {
            match diff {
                0 => Quality::Perfect,
                1 | -11 => Quality::Augmented,
                -1 | 11 => Quality::Diminished,
                _ => return Err(IntervalError::InvalidInterval),
            }
        } else {
            match diff {
                0 => Quality::Major,
                -1 | 11 => Quality::Minor,
                1 | -11 => Quality::Augmented,
                -2 | 10 => Quality::Diminished,
                _ => return Err(IntervalError::InvalidInterval),
            }
        };

        let step = match semitone_dist {
            1 => Some(Step::Half),
            2 => Some(Step::Whole),
            6 => Some(Step::Tritone),
            _ => None,
        };

        Ok(Interval {
            semitone_count: semitone_dist,
            quality,
            number,
            step,
        })
    }

    /// Creates an interval by inverting the given interval
    /// e.g. Perfect fifth (C to G) becomes a perfect fourth (G to C)
    pub fn invert(self) -> Result<Self, IntervalError> {
        if self.semitone_count == 12 {
            Self::from_semitone(12)
        } else {
            let adjusted = (12 + (12i16 - self.semitone_count as i16)) % 12;
            Self::from_semitone(adjusted as u8)
        }
    }

    /// Move the given note up by this interval.
    pub fn second_note_from(self, first_note: Note) -> Note {
        let pitch = Pitch::from_interval(first_note.pitch, self);
        let octave = first_note.octave;
        let excess_octave = (first_note.pitch.as_u8() + self.semitone_count) / 12;

        Note {
            octave: octave + excess_octave,
            pitch,
        }
    }

    /// Move the given note down by this interval.
    pub fn second_note_down_from(self, first_note: Note) -> Note {
        let pitch = Pitch::from_interval_down(first_note.pitch, self);
        let octave = first_note.octave;
        let raw_diff = first_note.pitch.as_u8() as i16 - self.semitone_count as i16;
        let excess_octave = (raw_diff / -12) + if raw_diff < 0 { 1 } else { 0 };

        Note {
            octave: octave - excess_octave as u8,
            pitch,
        }
    }

    /// Produce the list of notes that have had each interval applied in order.
    pub fn to_notes(root: Note, intervals: impl IntoIterator<Item = Interval>) -> Vec<Note> {
        let mut notes = vec![root];

        for interval in intervals {
            let last_note = notes.last().unwrap();
            let interval_first_note = Note::new(last_note.pitch, last_note.octave);
            let interval_second_note = interval.second_note_from(interval_first_note);
            notes.push(interval_second_note);
        }

        notes
    }

    /// Produce the list of notes that have had each interval applied in order.
    pub fn to_notes_reverse(
        root: Note,
        intervals: impl IntoIterator<Item = Interval>,
    ) -> Vec<Note> {
        let mut notes = vec![root];

        let reversed = intervals
            .into_iter()
            .collect::<Vec<Interval>>()
            .into_iter()
            .rev();
        for interval in reversed {
            let last_note = notes.last().unwrap();
            let interval_first_note = Note::new(last_note.pitch, last_note.octave);
            let interval_second_note = interval.second_note_down_from(interval_first_note);
            notes.push(interval_second_note);
        }

        notes
    }
}

impl Default for Interval {
    fn default() -> Self {
        Interval {
            semitone_count: 0,
            quality: Quality::Perfect,
            number: Number::Unison,
            step: None,
        }
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Interval {
                semitone_count: _,
                quality: Quality::Diminished,
                number: Number::Fifth,
                step: _,
            } => write!(f, "T"),
            Interval {
                semitone_count: _,
                quality: Quality::Augmented,
                number: Number::Fourth,
                step: _,
            } => write!(f, "T"),
            Interval {
                semitone_count: _,
                quality: _,
                number: Number::Unison,
                step: _,
            } => write!(f, "1"),
            Interval {
                semitone_count: _,
                quality: _,
                number: Number::Octave,
                step: _,
            } => write!(f, "1"),
            _ => write!(f, "{}{}", self.quality, self.number),
        }
    }
}
