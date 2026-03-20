use crate::chord::errors::ChordError;
use regex::{Match, Regex};
use std::sync::LazyLock;
use strum_macros::Display;

static QUALITY_REGEXES: LazyLock<Vec<(Regex, Quality)>> = LazyLock::new(|| {
    use Quality::*;

    vec![
        (
            Regex::new(r"^(M\s+|M$|(?i)maj|Maj|Major|major)").unwrap(),
            Major,
        ),
        (
            Regex::new(r"^(m\s+|m$|(?i)min|Min|Minor|minor)").unwrap(),
            Minor,
        ),
        (Regex::new(r"(?i)^(diminished)").unwrap(), Diminished),
        (Regex::new(r"(?i)^(augmented)").unwrap(), Augmented),
        (
            Regex::new(r"(?i)^(half\s*diminished|halfdiminished)").unwrap(),
            HalfDiminished,
        ),
        (Regex::new(r"(?i)^(dominant)").unwrap(), Dominant),
        (
            Regex::new(r"(?i)^(sus2\s+|sus2$|suspended2)").unwrap(),
            Suspended2,
        ),
        (
            Regex::new(r"(?i)^(sus4\s+|sus4$|suspended4)").unwrap(),
            Suspended4,
        ),
    ]
});

/// The quality of a chord.
#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Quality {
    Major,
    Minor,
    Diminished,
    Augmented,
    HalfDiminished,
    Dominant,
    Suspended2,
    Suspended4,
}

impl Quality {
    /// Parse a quality from a regex.
    pub fn from_regex(string: &str) -> Result<(Self, Option<Match<'_>>), ChordError> {
        use Quality::*;

        for (regex, quality_enum) in &*QUALITY_REGEXES {
            let mode = regex.find(string.trim());

            if let Some(quality_match) = mode {
                return Ok((*quality_enum, Some(quality_match)));
            };
        }

        Ok((Major, None))
    }
}
