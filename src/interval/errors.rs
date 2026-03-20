use std::error;
use std::fmt;

/// An error caused while creating an interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalError {
    InvalidInterval,
}

impl fmt::Display for IntervalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid interval")
    }
}

impl error::Error for IntervalError {}
