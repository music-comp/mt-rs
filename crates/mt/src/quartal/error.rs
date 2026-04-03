use std::error;
use std::fmt;

use crate::quintal::QuintalError;

/// An error arising from quartal chord construction or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuartalError {
    /// An interval value was outside the legal quartal range {4, 5, 6}.
    IllegalInterval(u8),
    /// The wrong number of intervals was provided (expected 3).
    WrongIntervalCount(usize),
    /// MIDI pitches were not in strictly ascending order.
    NotAscending,
    /// The pitch class set contains duplicate values (mod 12).
    DuplicatePitchClasses,
    /// A pitch class value was outside the valid range 0..=11.
    PitchClassOutOfRange(u8),
}

impl fmt::Display for QuartalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QuartalError::IllegalInterval(i) => {
                write!(f, "illegal quartal interval: {} (must be 4, 5, or 6)", i)
            }
            QuartalError::WrongIntervalCount(n) => {
                write!(f, "wrong interval count: expected 3, got {}", n)
            }
            QuartalError::NotAscending => {
                write!(f, "pitches must be in strictly ascending order")
            }
            QuartalError::DuplicatePitchClasses => {
                write!(f, "duplicate pitch classes")
            }
            QuartalError::PitchClassOutOfRange(pc) => {
                write!(f, "pitch class out of range: {} (must be 0..=11)", pc)
            }
        }
    }
}

impl error::Error for QuartalError {}

impl From<QuintalError> for QuartalError {
    fn from(e: QuintalError) -> Self {
        match e {
            QuintalError::PitchClassOutOfRange(pc) => QuartalError::PitchClassOutOfRange(pc),
            QuintalError::DuplicatePitchClasses => QuartalError::DuplicatePitchClasses,
            QuintalError::WrongCardinality(_) => QuartalError::WrongIntervalCount(0),
            QuintalError::NotAscending => QuartalError::NotAscending,
        }
    }
}
