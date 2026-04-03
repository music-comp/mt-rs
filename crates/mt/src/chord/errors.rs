use crate::note::NoteError;
use std::error;
use std::fmt;

/// An error while parsing a chord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordError {
    InvalidRegex,
    InvalidNote(String),
    UnknownIntervalPattern(Vec<u8>),
}

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChordError::InvalidRegex => write!(f, "invalid regex"),
            ChordError::InvalidNote(note) => write!(f, "invalid note: {}", note),
            ChordError::UnknownIntervalPattern(intervals) => {
                write!(f, "unknown chord interval pattern: {:?}", intervals)
            }
        }
    }
}

impl error::Error for ChordError {}

impl From<NoteError> for ChordError {
    fn from(_: NoteError) -> Self {
        ChordError::InvalidRegex
    }
}

impl From<regex::Error> for ChordError {
    fn from(_: regex::Error) -> Self {
        ChordError::InvalidRegex
    }
}
