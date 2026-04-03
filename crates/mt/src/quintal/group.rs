//! T/I group operations on quintal pitch-class chords.
//!
//! The T/I group consists of 24 operations: 12 transpositions T_0 through T_11
//! and 12 inversion-transpositions I_0 through I_11. These operations act on
//! pitch-class sets and preserve set cardinality.

use super::PcChord;

/// Transpose all pitch classes by `n` semitones mod 12.
///
/// The resulting chord is sorted and guaranteed to be valid since
/// mod-12 arithmetic on valid pitch classes produces valid pitch classes.
pub fn transpose(chord: &PcChord, n: u8) -> PcChord {
    let mut pcs = [0u8; 4];
    for (i, &pc) in chord.pcs.iter().enumerate() {
        pcs[i] = (pc + n) % 12;
    }
    pcs.sort();
    PcChord { pcs }
}

/// Invert each pitch class: x -> (12 - x) mod 12.
///
/// This is equivalent to reflection about pitch class 0 in the pitch-class
/// circle. Inversion is an involution: `invert(invert(chord)) == chord`.
pub fn invert(chord: &PcChord) -> PcChord {
    let mut pcs = [0u8; 4];
    for (i, &pc) in chord.pcs.iter().enumerate() {
        pcs[i] = (12 - pc) % 12;
    }
    pcs.sort();
    PcChord { pcs }
}

/// Apply inversion followed by transposition by `n`: T_n . I.
///
/// This computes `transpose(invert(chord), n)`, which maps each pitch class
/// `x` to `(12 - x + n) mod 12`.
pub fn invert_transpose(chord: &PcChord, n: u8) -> PcChord {
    transpose(&invert(chord), n)
}

/// Compute the full T/I orbit of a chord.
///
/// The orbit contains all distinct chords reachable by applying any of the
/// 24 T/I operations. The result is deduplicated and sorted in ascending
/// order. Orbit sizes are always 6, 12, or 24 depending on the chord's
/// stabilizer subgroup.
pub fn orbit(chord: &PcChord) -> Vec<PcChord> {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    for n in 0..12u8 {
        set.insert(transpose(chord, n));
        set.insert(invert_transpose(chord, n));
    }
    set.into_iter().collect()
}
