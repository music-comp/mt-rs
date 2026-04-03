use crate::interval::IntervalError;
use crate::note::NoteError;
use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaleError {
    InvalidInterval,
    ModeFromRegex,
    InvalidRegex,
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScaleError::InvalidInterval => {
                write!(f, "cannot determine scale intervals")
            }
            ScaleError::ModeFromRegex => write!(f, "cannot determine mode"),
            ScaleError::InvalidRegex => write!(f, "invalid scale regex"),
        }
    }
}

impl error::Error for ScaleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<regex::Error> for ScaleError {
    fn from(_: regex::Error) -> Self {
        ScaleError::ModeFromRegex
    }
}

impl From<NoteError> for ScaleError {
    fn from(_: NoteError) -> Self {
        ScaleError::InvalidRegex
    }
}

impl From<IntervalError> for ScaleError {
    fn from(_: IntervalError) -> Self {
        ScaleError::InvalidInterval
    }
}
