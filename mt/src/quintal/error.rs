use std::error;
use std::fmt;

/// An error arising from quintal chord construction or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuintalError {
    /// A pitch class value was outside the valid range 0..=11.
    PitchClassOutOfRange(u8),
    /// The pitch class set contains duplicate values (mod 12).
    DuplicatePitchClasses,
    /// The input slice had the wrong number of elements (expected 4).
    WrongCardinality(usize),
    /// MIDI pitches were not in strictly ascending order.
    NotAscending,
}

impl fmt::Display for QuintalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QuintalError::PitchClassOutOfRange(pc) => {
                write!(f, "pitch class out of range: {} (must be 0..=11)", pc)
            }
            QuintalError::DuplicatePitchClasses => {
                write!(f, "duplicate pitch classes")
            }
            QuintalError::WrongCardinality(n) => {
                write!(f, "wrong cardinality: expected 4, got {}", n)
            }
            QuintalError::NotAscending => {
                write!(f, "pitches must be in strictly ascending order")
            }
        }
    }
}

impl error::Error for QuintalError {}
