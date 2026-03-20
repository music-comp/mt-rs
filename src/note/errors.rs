use std::error;
use std::fmt;

/// An error caused when parsing a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteError {
    InvalidPitch,
}

impl fmt::Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid pitch class")
    }
}

impl error::Error for NoteError {}

impl From<regex::Error> for NoteError {
    fn from(_: regex::Error) -> Self {
        NoteError::InvalidPitch
    }
}
