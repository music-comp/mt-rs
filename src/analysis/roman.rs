use crate::chord::{Chord, Number, Quality};
use crate::harmony;
use crate::note::Pitch;
use crate::scale::Mode;

/// A Roman numeral analysis result.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RomanNumeral {
    /// Scale degree (1-7).
    pub degree: u8,
    /// Roman numeral label (e.g., "I", "vi", "V7", "vii°").
    pub label: String,
    /// The chord quality.
    pub quality: Quality,
    /// The chord number.
    pub number: Number,
}

pub(crate) const ROMAN_UPPER: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];
pub(crate) const ROMAN_LOWER: [&str; 7] = ["i", "ii", "iii", "iv", "v", "vi", "vii"];

/// Analyze a chord in the context of a key, returning its Roman numeral.
/// Returns None if the chord root is not a diatonic scale degree.
pub fn roman_numeral(key_tonic: Pitch, key_mode: Mode, chord: &Chord) -> Option<RomanNumeral> {
    let diatonic = harmony::diatonic_triads(key_tonic, key_mode);

    // Find which scale degree the chord root matches
    let degree_match = diatonic
        .iter()
        .find(|dc| dc.root.as_u8() == chord.root.as_u8())?;

    let degree = degree_match.degree;
    let idx = (degree - 1) as usize;

    let base = match chord.quality {
        Quality::Major | Quality::Dominant | Quality::Augmented => ROMAN_UPPER[idx],
        Quality::Minor | Quality::Diminished | Quality::HalfDiminished => ROMAN_LOWER[idx],
        Quality::Suspended2 | Quality::Suspended4 => ROMAN_UPPER[idx],
    };

    let suffix = match (&chord.quality, &chord.number) {
        (Quality::Diminished, Number::Triad) => "°",
        (Quality::Diminished, Number::Seventh) => "°7",
        (Quality::HalfDiminished, Number::Seventh) => "ø7",
        (Quality::Augmented, Number::Triad) => "+",
        (Quality::Augmented, Number::Seventh) => "+7",
        (_, Number::Seventh) => "7",
        (_, Number::MajorSeventh) => "Δ7",
        (_, Number::Ninth) => "9",
        (_, Number::Eleventh) => "11",
        (_, Number::Thirteenth) => "13",
        _ => "",
    };

    let label = format!("{}{}", base, suffix);

    Some(RomanNumeral {
        degree,
        label,
        quality: chord.quality,
        number: chord.number,
    })
}
