//! Verification routines for L1 distance patterns, fiber classification,
//! and the Universal L1 Law in the quintal fiber bundle.
//!
//! The Universal L1 Law states that for every legal quintal voiced chord,
//! the L1 distances between consecutive inversions in the T1 cycle are
//! always `[12, 12, 12, 36]`, where the final entry measures the closing
//! distance from the third inversion back to the *original* root position.

use std::collections::BTreeMap;

use super::base_space::BaseSpace;
use super::fiber::{inversion_cycle, l1_distance};
use super::orbit::{classify_orbit, Orbit};
use super::types::{FiberClass, PcChord, VoicedChord};

/// Construct a `VoicedChord` from a `PcChord` by finding a legal quintal
/// ordering and placing it in a default register starting at MIDI 48 (C3).
///
/// Returns `None` if the chord has no legal quintal interval structure.
fn pc_chord_to_voiced(chord: &PcChord) -> Option<VoicedChord> {
    let is = chord.interval_structure()?;

    // Try each pitch class as the bottom note of the voicing.
    for &start_pc in &chord.pcs {
        let base = 48 + start_pc;
        let pitches = [
            base,
            base + is.0,
            base + is.0 + is.1,
            base + is.0 + is.1 + is.2,
        ];

        if let Ok(vc) = VoicedChord::new(pitches) {
            // Verify the voicing projects back to the same pitch-class chord.
            if let Ok(pc) = vc.to_pc_chord() {
                if pc == *chord {
                    return Some(vc);
                }
            }
        }
    }
    None
}

/// Which inversions (indices 0..3) of the voiced chord have legal quintal
/// interval structures (all components in {6, 7, 8})?
///
/// For Class A chords this returns a single index; for Class B chords it
/// returns two indices.
pub fn inversions_in_base(chord: &VoicedChord) -> Vec<usize> {
    let cycle = inversion_cycle(chord);
    cycle
        .iter()
        .enumerate()
        .filter(|(_, inv)| inv.interval_structure().is_legal())
        .map(|(i, _)| i)
        .collect()
}

/// L1 distances between consecutive inversions in the T1 cycle, closing
/// back to the original root position.
///
/// Returns `[d(inv0, inv1), d(inv1, inv2), d(inv2, inv3), d(inv3, inv0)]`.
///
/// The Universal L1 Law asserts this is always `[12, 12, 12, 36]` for
/// every legal quintal chord. The closing distance is 36 because all four
/// voices have each ascended by 12 semitones over the three preceding
/// T1 steps, placing them one octave above their starting positions.
pub fn inversion_l1_distances(chord: &VoicedChord) -> [u32; 4] {
    let cycle = inversion_cycle(chord);
    [
        l1_distance(&cycle[0], &cycle[1]),
        l1_distance(&cycle[1], &cycle[2]),
        l1_distance(&cycle[2], &cycle[3]),
        l1_distance(&cycle[3], &cycle[0]),
    ]
}

/// Classify a `PcChord`'s fiber as Class A (1 inversion with a legal
/// interval structure) or Class B (2 inversions with legal structures).
///
/// Returns `None` if the chord is not a legal quintal chord or if the
/// count of legal inversions is unexpected.
pub fn fiber_class(chord: &PcChord) -> Option<FiberClass> {
    let vc = pc_chord_to_voiced(chord)?;
    let count = inversions_in_base(&vc).len();
    match count {
        1 => Some(FiberClass::ClassA),
        2 => Some(FiberClass::ClassB),
        _ => None,
    }
}

/// Verify the Universal L1 Law for all 228 chords in the base space.
///
/// Returns `Ok(())` if every chord's inversion cycle has the distance
/// pattern `[12, 12, 12, 36]`. Returns `Err` with the list of chords
/// that violate the law if any are found.
pub fn verify_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>> {
    let mut failures = Vec::new();
    for &chord in space.chords() {
        if let Some(vc) = pc_chord_to_voiced(&chord) {
            let dists = inversion_l1_distances(&vc);
            if dists != [12, 12, 12, 36] {
                failures.push(chord);
            }
        } else {
            failures.push(chord);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Map each of the 14 orbits to its fiber class by finding a representative
/// chord for each orbit and classifying it.
///
/// The expected result is 11 Class A orbits and 3 Class B orbits (Q676,
/// Q686, Q688).
pub fn verify_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass> {
    let mut result = BTreeMap::new();
    for &orb in Orbit::all() {
        for &chord in space.chords() {
            if classify_orbit(&chord) == Some(orb) {
                if let Some(fc) = fiber_class(&chord) {
                    result.insert(orb, fc);
                    break;
                }
            }
        }
    }
    result
}
