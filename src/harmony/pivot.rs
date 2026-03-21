use crate::analysis::{roman_numeral, RomanNumeral};
use crate::chord::Chord;
use crate::harmony::diatonic_triads;
use crate::note::{Notes, Pitch};
use crate::scale::Mode;
use std::collections::HashSet;

/// A chord that is diatonic to two different keys.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PivotChord {
    pub chord: Chord,
    pub roman_in_a: RomanNumeral,
    pub roman_in_b: RomanNumeral,
}

/// Find chords that are diatonic to both keys.
/// Matches by pitch-class set content.
pub fn pivot_chords(
    key_a_tonic: Pitch,
    key_a_mode: Mode,
    key_b_tonic: Pitch,
    key_b_mode: Mode,
) -> Vec<PivotChord> {
    let chords_a = diatonic_triads(key_a_tonic, key_a_mode);
    let chords_b = diatonic_triads(key_b_tonic, key_b_mode);

    let mut results = Vec::new();

    for dc_a in &chords_a {
        let pcs_a: HashSet<u8> = dc_a.chord.notes().iter()
            .map(|n| n.pitch.as_u8())
            .collect();

        for dc_b in &chords_b {
            let pcs_b: HashSet<u8> = dc_b.chord.notes().iter()
                .map(|n| n.pitch.as_u8())
                .collect();

            if pcs_a == pcs_b {
                if let (Some(rn_a), Some(rn_b)) = (
                    roman_numeral(key_a_tonic, key_a_mode, &dc_a.chord),
                    roman_numeral(key_b_tonic, key_b_mode, &dc_b.chord),
                ) {
                    results.push(PivotChord {
                        chord: dc_a.chord.clone(),
                        roman_in_a: rn_a,
                        roman_in_b: rn_b,
                    });
                }
            }
        }
    }

    results
}
