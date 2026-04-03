//! Quartal/quintal duality for the fiber bundle framework.
//!
//! This module explores the duality between quartal (fourths-based) and
//! quintal (fifths-based) readings of the same chord voicings. Every
//! four-note quintal chord can be read bottom-to-top as a stack of fifths
//! or top-to-bottom as a stack of fourths, producing reversed interval
//! structures. This duality is reflected at every level of the framework:
//! interval structures, T/I orbits, and inversion cycles.

use super::base_space::BaseSpace;
use super::fiber::{inversion_cycle, t_minus1};
use super::orbit::{classify_orbit, Orbit};
use super::types::{IntervalStructure, VoicedChord};

/// Read intervals bottom-to-top (quintal/fifths perspective).
///
/// This is identical to [`VoicedChord::interval_structure`]: the three
/// intervals `(p1-p0, p2-p1, p3-p2)` measured upward from the bass.
pub fn quintal_reading(chord: &VoicedChord) -> IntervalStructure {
    chord.interval_structure()
}

/// Read intervals top-to-bottom (quartal/fourths perspective).
///
/// Returns `(p3-p2, p2-p1, p1-p0)` — the reversal of the quintal reading.
/// When a quintal chord is read downward, the perfect fifths become
/// perfect fourths (their inversional complements), yielding a quartal
/// interpretation of the same pitch collection.
pub fn quartal_reading(chord: &VoicedChord) -> IntervalStructure {
    let p = chord.pitches;
    IntervalStructure(p[3] - p[2], p[2] - p[1], p[1] - p[0])
}

/// Reverse an interval structure: `(a, b, c)` becomes `(c, b, a)`.
///
/// This is an involution: applying it twice returns the original structure.
/// Reversal connects quintal and quartal readings of the same chord.
pub fn reverse_interval_structure(is: &IntervalStructure) -> IntervalStructure {
    IntervalStructure(is.2, is.1, is.0)
}

/// Verify that `t_minus1` traverses the same fiber as `t1` in reverse order.
///
/// The `t1` cycle is `[inv0, inv1, inv2, inv3]`. The `t_minus1` cycle
/// starting from the same chord should visit the same pitch-class sets
/// in the order `[inv0, inv3, inv2, inv1]`.
///
/// Returns `true` if the reversal equivalence holds for the given chord.
pub fn t1_reversal_equivalence(chord: &VoicedChord) -> bool {
    let t1_cycle = inversion_cycle(chord);

    // Compute t_minus1 cycle from the same starting chord.
    let tm1 = t_minus1(chord);
    let tm2 = t_minus1(&tm1);
    let tm3 = t_minus1(&tm2);

    // Get pc sets for t1 cycle.
    let t1_pcs: Vec<_> = t1_cycle
        .iter()
        .map(|inv| inv.to_pc_chord().unwrap())
        .collect();

    // Get pc sets for t_minus1 cycle.
    let tm_pcs = [
        chord.to_pc_chord().unwrap(),
        tm1.to_pc_chord().unwrap(),
        tm2.to_pc_chord().unwrap(),
        tm3.to_pc_chord().unwrap(),
    ];

    // t_minus1 cycle should be [inv0, inv3, inv2, inv1] in pc terms.
    tm_pcs[0] == t1_pcs[0]
        && tm_pcs[1] == t1_pcs[3]
        && tm_pcs[2] == t1_pcs[2]
        && tm_pcs[3] == t1_pcs[1]
}

/// Check if an orbit is self-dual: its interval structure reversed
/// belongs to the same T/I orbit.
///
/// An orbit is self-dual when reading its chords in quartal order
/// (top-to-bottom) produces chords that remain in the same T/I orbit.
/// This means the quartal and quintal perspectives are equivalent at
/// the orbit level.
pub fn orbit_self_duality(orb: &Orbit, space: &BaseSpace) -> bool {
    let rep_is = orb.representative();
    let reversed_is = reverse_interval_structure(&rep_is);

    // If the interval structure is palindromic, it is trivially self-dual.
    if rep_is == reversed_is {
        return true;
    }

    // Find a chord with the reversed interval structure among legal chords.
    for &chord in space.chords() {
        if let Some(is) = chord.interval_structure() {
            if is == reversed_is {
                // Check if this chord is in the same T/I orbit.
                if classify_orbit(&chord) == Some(*orb) {
                    return true;
                }
            }
        }
    }

    false
}

/// Verify that all 14 orbits are self-dual.
///
/// This is a fundamental property of the quintal system: the quartal/quintal
/// duality is fully symmetric, so every orbit maps to itself under reversal
/// of interval structures.
pub fn verify_all_orbits_self_dual(space: &BaseSpace) -> bool {
    Orbit::all()
        .iter()
        .all(|orb| orbit_self_duality(orb, space))
}
