use crate::chord::errors::ChordError;
use regex::{Match, Regex};
use std::sync::LazyLock;
use strum_macros::Display;

static NUMBER_REGEXES: LazyLock<Vec<(Regex, Number)>> = LazyLock::new(|| {
    use Number::*;
    vec![
        (Regex::new("(?i)(triad)").unwrap(), Triad),
        (Regex::new("(?i)(seventh)").unwrap(), Seventh),
        (Regex::new(r"(?i)(major\s*seventh)").unwrap(), MajorSeventh),
        (Regex::new("(?i)(ninth)").unwrap(), Ninth),
        (Regex::new("(?i)(eleventh)").unwrap(), Eleventh),
        (Regex::new("(?i)(thirteenth)").unwrap(), Thirteenth),
    ]
});

/// The superscript number after a chord.
#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Triad,
    Seventh,
    MajorSeventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

impl Number {
    /// Parse the number using a regex.
    pub fn from_regex(string: &str) -> Result<(Self, Option<Match<'_>>), ChordError> {
        for (regex, number_enum) in &*NUMBER_REGEXES {
            let mode = regex.find(string);

            if let Some(number_match) = mode {
                return Ok((*number_enum, Some(number_match)));
            };
        }

        Err(ChordError::InvalidRegex)
    }
}
