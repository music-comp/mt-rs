//! Optimal-assignment voice-leading distance for VoicedChords.

use crate::quintal::VoicedChord;

const PERMS: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// Minimum voice-leading L1 distance between two 4-voice chords.
///
/// Finds the voice assignment (permutation) that minimizes the sum of
/// absolute semitone movements. This is the *assignment-optimal* distance,
/// not the positional distance used by [`crate::quintal::l1_distance`].
///
/// # Difference from `quintal::l1_distance`
///
/// [`crate::quintal::l1_distance`] matches voices by array position
/// (voice 0 to voice 0, etc.). This is correct for measuring T1-cycle
/// distances within a fiber but **not** for general voice-leading between
/// unrelated chords. `min_voiced_chord_l1` tries all 24 assignments and
/// picks the cheapest.
pub fn min_voiced_chord_l1(a: &VoicedChord, b: &VoicedChord) -> u32 {
    let mut best = u32::MAX;
    for perm in &PERMS {
        let dist: u32 = (0..4)
            .map(|i| a.pitches[i].abs_diff(b.pitches[perm[i]]) as u32)
            .sum();
        if dist < best {
            best = dist;
        }
    }
    best
}
