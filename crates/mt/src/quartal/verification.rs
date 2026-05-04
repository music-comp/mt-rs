//! Quartal-perspective verification of structural laws — symmetric
//! counterpart to [`crate::quintal::verification`].
//!
//! The Universal L1 Law and fiber-class assignment are perspective-independent
//! statements about the shared base space, but having a quartal-direction
//! verifier is useful both as documentation and as a check that the quartal
//! traversal ([`super::voicing::t_quartal`], [`super::voicing::quartal_inversion_cycle`],
//! [`super::voicing::quartal_l1_distances`]) produces the expected results.

use std::collections::BTreeMap;

use super::conversion::to_quartal;
use super::types::QuartalVoicedChord;
use super::voicing::{quartal_inversion_cycle, quartal_l1_distances};
use crate::quintal::{classify_orbit, BaseSpace, FiberClass, Orbit, PcChord, VoicedChord};

/// Verify the Universal L1 Law in the quartal direction.
///
/// For every legal [`QuartalVoicedChord`] derived from the 228 PcChords,
/// [`super::voicing::quartal_l1_distances`] must return `[12, 12, 12, 36]`.
///
/// # Errors
///
/// Returns `Err` containing the list of [`PcChord`] violators if any chord's
/// quartal L1 pattern deviates from `[12, 12, 12, 36]` or if a chord has no
/// legal quintal stacking from which to derive a [`QuartalVoicedChord`].
///
/// Mirrors the signature of [`crate::quintal::verify_universal_l1_law`].
/// See plan note D-T0-001 for the rationale on this richer signature.
pub fn verify_quartal_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>> {
    let mut failures = Vec::new();
    for &chord in space.chords() {
        match pc_chord_to_quartal_voiced(&chord) {
            Some(qvc) => {
                if quartal_l1_distances(&qvc) != [12, 12, 12, 36] {
                    failures.push(chord);
                }
            }
            None => failures.push(chord),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Verify all 14 fiber classes via quartal traversal.
///
/// Symmetric counterpart to [`crate::quintal::verify_fiber_classes`]. Returns
/// the orbit-to-class map computed by classifying a representative chord of
/// each orbit through the quartal inversion cycle. Should agree pointwise
/// with the quintal verifier (modulo identical Class A / Class B
/// assignments).
///
/// The cross-check that [`super::voicing::quartal_inversion_cycle`] and
/// [`crate::quintal::inversion_cycle`] visit the same pitch-class set per
/// chord is the *added* value of this quartal-side verifier; it is asserted
/// in `tests/quartal/test_quartal_quintal_identity.rs::test_fibers_same_chords`.
pub fn verify_quartal_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass> {
    let mut result = BTreeMap::new();
    for &orb in Orbit::all() {
        for &chord in space.chords() {
            if classify_orbit(&chord) == Some(orb) {
                if let Some(qvc) = pc_chord_to_quartal_voiced(&chord) {
                    let cycle = quartal_inversion_cycle(&qvc);
                    let count = cycle
                        .iter()
                        .filter(|qinv| qinv.0.interval_structure().is_legal())
                        .count();
                    let fc = match count {
                        1 => FiberClass::ClassA,
                        2 => FiberClass::ClassB,
                        _ => continue,
                    };
                    result.insert(orb, fc);
                    break;
                }
            }
        }
    }
    result
}

/// Construct a [`QuartalVoicedChord`] from a [`PcChord`] in a default
/// register starting at MIDI 48 (C3), via the chord's legal quintal
/// stacking.
///
/// Returns `None` if the chord has no legal quintal interval structure.
///
/// This duplicates ~10 lines of logic from
/// `crate::quintal::verification::pc_chord_to_voiced` (which is private)
/// rather than widening quintal's API surface for a single internal use
/// (decision D-quartal-T0-001 — see plan §2).
fn pc_chord_to_quartal_voiced(chord: &PcChord) -> Option<QuartalVoicedChord> {
    let is = chord.interval_structure()?;

    for &start_pc in &chord.pcs {
        let base = 48 + start_pc;
        let pitches = [
            base,
            base + is.0,
            base + is.0 + is.1,
            base + is.0 + is.1 + is.2,
        ];

        if let Ok(vc) = VoicedChord::new(pitches) {
            if let Ok(pc) = vc.to_pc_chord() {
                if pc == *chord {
                    return Some(to_quartal(&vc));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_round_trips_q777() {
        // pcs {0, 2, 7, 9} is C-D-G-A (= the Q777 Summit pitch-class set).
        let pc = PcChord::new([0, 2, 7, 9]).unwrap();
        let qvc = pc_chord_to_quartal_voiced(&pc).unwrap();
        // Round-trip preserves the chord through the quartal wrap.
        assert_eq!(qvc.0.to_pc_chord().unwrap(), pc);
        // And the chord lands in the expected orbit.
        assert_eq!(classify_orbit(&pc), Some(Orbit::Q777));
    }
}
