//! Chord-scale and Tymoczko inversion operators for the quintal fiber bundle.
//!
//! This module implements the chord-scale construction and the T1/T-1
//! inversion operators from Tymoczko's voice-leading geometry, specialized
//! to four-note quintal chord voicings.

use std::collections::BTreeSet;

use super::types::{PcChord, VoicedChord};
use super::QuintalError;

/// A chord scale derived from a voiced chord: the sorted pitch-class set
/// together with the cyclic step intervals between adjacent pitch classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChordScale {
    /// The four pitch classes in sorted ascending order.
    pub pcs: [u8; 4],
    /// The four cyclic step intervals between adjacent pitch classes.
    /// `steps[i]` is the interval from `pcs[i]` to `pcs[(i+1) % 4]` (mod 12).
    /// The steps always sum to 12.
    pub steps: [u8; 4],
}

/// Compute the chord scale for a voiced chord.
///
/// Extracts the pitch classes, sorts and deduplicates them, then computes
/// the cyclic step intervals. The steps always sum to 12.
///
/// # Panics
///
/// Panics if the voiced chord contains duplicate pitch classes (mod 12).
/// This should not occur for valid quintal voiced chords.
pub fn chord_scale(chord: &VoicedChord) -> ChordScale {
    let pc_set: BTreeSet<u8> = chord.pitches.iter().map(|&p| p % 12).collect();
    assert!(
        pc_set.len() == 4,
        "chord_scale requires 4 distinct pitch classes, got {}",
        pc_set.len()
    );

    let pcs_vec: Vec<u8> = pc_set.into_iter().collect();
    let pcs: [u8; 4] = [pcs_vec[0], pcs_vec[1], pcs_vec[2], pcs_vec[3]];

    let steps: [u8; 4] = [
        (pcs[1] + 12 - pcs[0]) % 12,
        (pcs[2] + 12 - pcs[1]) % 12,
        (pcs[3] + 12 - pcs[2]) % 12,
        (pcs[0] + 12 - pcs[3]) % 12,
    ];

    ChordScale { pcs, steps }
}

/// Apply the Tymoczko T1 inversion operator.
///
/// Each voice moves up by the chord-scale step corresponding to its current
/// pitch class. The resulting pitches are sorted ascending to produce a new
/// voiced chord. Applying T1 four times yields a transposition up by one
/// octave (T12).
pub fn t1(chord: &VoicedChord) -> VoicedChord {
    let cs = chord_scale(chord);
    let mut new_pitches: [u8; 4] = [0; 4];

    for (i, &pitch) in chord.pitches.iter().enumerate() {
        let pc = pitch % 12;
        let j = cs
            .pcs
            .iter()
            .position(|&p| p == pc)
            .expect("pitch class must be in chord scale");
        new_pitches[i] = pitch + cs.steps[j];
    }

    new_pitches.sort();
    VoicedChord {
        pitches: new_pitches,
    }
}

/// Apply the inverse Tymoczko T-1 inversion operator.
///
/// Each voice moves down by the chord-scale step that *leads to* its current
/// pitch class. This is the inverse of [`t1`]: applying T-1 after T1
/// preserves the pitch-class set (though the voicing may differ by octave
/// placement).
pub fn t_minus1(chord: &VoicedChord) -> VoicedChord {
    let cs = chord_scale(chord);
    let mut new_pitches: [i16; 4] = [0; 4];

    for (i, &pitch) in chord.pitches.iter().enumerate() {
        let pc = pitch % 12;
        let j = cs
            .pcs
            .iter()
            .position(|&p| p == pc)
            .expect("pitch class must be in chord scale");
        let prev = (j + 3) % 4;
        new_pitches[i] = pitch as i16 - cs.steps[prev] as i16;
    }

    new_pitches.sort();
    // Convert back to u8; the values should be non-negative for any
    // reasonable MIDI pitch input.
    let pitches: [u8; 4] = [
        new_pitches[0] as u8,
        new_pitches[1] as u8,
        new_pitches[2] as u8,
        new_pitches[3] as u8,
    ];
    VoicedChord { pitches }
}

/// Compute the full inversion cycle of length 4.
///
/// Returns `[chord, t1(chord), t1^2(chord), t1^3(chord)]`. The next
/// application of T1 would yield the original chord transposed up one octave.
pub fn inversion_cycle(chord: &VoicedChord) -> [VoicedChord; 4] {
    let inv0 = *chord;
    let inv1 = t1(&inv0);
    let inv2 = t1(&inv1);
    let inv3 = t1(&inv2);
    [inv0, inv1, inv2, inv3]
}

/// Project a voiced chord down to its pitch-class chord.
///
/// This is a convenience wrapper around [`VoicedChord::to_pc_chord`].
pub fn project(chord: &VoicedChord) -> Result<PcChord, QuintalError> {
    chord.to_pc_chord()
}

/// Compute the L1 (taxicab / Manhattan) voice-leading distance between
/// two voiced chords.
///
/// This measures the total number of semitone steps across all four voices.
pub fn l1_distance(a: &VoicedChord, b: &VoicedChord) -> u32 {
    a.pitches
        .iter()
        .zip(b.pitches.iter())
        .map(|(&x, &y)| (x as i32 - y as i32).unsigned_abs())
        .sum()
}
