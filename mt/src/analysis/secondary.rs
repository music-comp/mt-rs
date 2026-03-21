use crate::analysis::roman::{ROMAN_LOWER, ROMAN_UPPER};
use crate::chord::{Chord, Number, Quality};
use crate::harmony::{diatonic_triads, DiatonicChord};
use crate::note::Pitch;
use crate::scale::Mode;

/// A secondary dominant analysis result.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SecondaryDominant {
    /// Label like "V/V", "V7/vi"
    pub label: String,
    /// The scale degree being tonicized
    pub target_degree: u8,
    /// The diatonic chord being tonicized
    pub target_chord: DiatonicChord,
}

/// Check if a chord functions as a secondary dominant (V/x) in the given key.
/// Returns None if it doesn't function as a secondary dominant.
///
/// A chord is V/x if:
/// 1. It has Major or Dominant quality
/// 2. Its root is a perfect 5th (7 semitones) above a diatonic chord root
/// 3. The target is not degree 1 (V/I is just V, not secondary)
pub fn secondary_dominant(
    key_tonic: Pitch,
    key_mode: Mode,
    chord: &Chord,
) -> Option<SecondaryDominant> {
    if !matches!(chord.quality, Quality::Major | Quality::Dominant) {
        return None;
    }

    let diatonic = diatonic_triads(key_tonic, key_mode);
    let chord_root_pc = chord.root.as_u8();

    for dc in &diatonic {
        let target_pc = dc.root.as_u8();
        let dominant_of_target = (target_pc + 7) % 12;

        if chord_root_pc == dominant_of_target && dc.degree != 1 {
            let seventh_suffix = match chord.number {
                Number::Seventh => "7",
                Number::Ninth => "9",
                _ => "",
            };

            let target_label = match dc.quality {
                Quality::Major | Quality::Dominant | Quality::Augmented => {
                    ROMAN_UPPER[(dc.degree - 1) as usize]
                }
                _ => ROMAN_LOWER[(dc.degree - 1) as usize],
            };

            let label = format!("V{}/{}", seventh_suffix, target_label);

            return Some(SecondaryDominant {
                label,
                target_degree: dc.degree,
                target_chord: dc.clone(),
            });
        }
    }

    None
}
