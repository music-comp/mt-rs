//! Quartal-perspective duality — symmetric counterpart to [`crate::quintal::duality`].
//!
//! The mathematical content of the quartal/quintal duality lives in
//! [`crate::quintal::duality`]. This module provides a quartal-native interface
//! for the same operations: functions taking [`QuartalVoicedChord`] and
//! returning [`QuartalIntervalStructure`] where natural, plus a quartal-direction
//! fiber-reversal verifier.
//!
//! See [`crate::quintal::duality`] for the underlying theorems and proofs.

use super::types::{QuartalIntervalStructure, QuartalVoicedChord};
use crate::quintal;

/// Read a quartal voicing's intervals top-to-bottom, complemented to fourths —
/// the natural quartal reading.
///
/// Delegates to [`QuartalVoicedChord::quartal_interval_structure`], which
/// composes `chord.0.interval_structure()` (bottom-up quintal IS) with
/// [`crate::quartal::quintal_to_quartal_structure`] (reverse-and-complement).
///
/// For a chord with bottom-up quintal IS `(i1, i2, i3)`, this returns
/// `QuartalIntervalStructure((12 - i3) % 12, (12 - i2) % 12, (12 - i1) % 12)` —
/// the canonical quartal label for that orbit.
///
/// Note: a naive composition of [`crate::quintal::quartal_reading`] (top-down)
/// with [`crate::quartal::quintal_to_quartal_structure`] (bottom-up input)
/// silently inverts the result for asymmetric IS. This function avoids that
/// pitfall by delegating to the existing method on [`QuartalVoicedChord`].
pub fn quartal_reading(chord: &QuartalVoicedChord) -> QuartalIntervalStructure {
    chord.quartal_interval_structure()
}

/// Read a quartal voicing's intervals bottom-to-top — the dual (quintal) reading.
///
/// Returns the underlying [`crate::quintal::IntervalStructure`]. For a legal
/// quintal voicing the components are in `{6, 7, 8}`. This is the quintal
/// interpretation of the same pitch collection.
pub fn quintal_reading(chord: &QuartalVoicedChord) -> quintal::IntervalStructure {
    quintal::quintal_reading(&chord.0)
}

/// Verify that the quartal traversal `t_quartal` traverses the same fiber
/// as `t1` in reversed order.
///
/// Symmetric counterpart to [`crate::quintal::t1_reversal_equivalence`].
/// Because `t_quartal == t_minus1` (the inverse of `t1`), the quartal
/// inversion cycle visits the four members of the fiber in the reverse
/// pitch-class order.
///
/// Returns `true` if the reversal equivalence holds for the given chord.
pub fn t_quartal_reversal_equivalence(chord: &QuartalVoicedChord) -> bool {
    quintal::t1_reversal_equivalence(&chord.0)
}

// --- Symmetric re-exports from the quintal-side duality module ---
// These functions operate on quintal types but are conceptually shared;
// re-exporting here lets quartal-minded readers find them without
// reaching into the quintal namespace.

/// Reverse an [`crate::quintal::IntervalStructure`]: `(a, b, c) → (c, b, a)`.
///
/// Re-exported from [`crate::quintal::reverse_interval_structure`].
pub use crate::quintal::reverse_interval_structure;

/// Check whether a single [`crate::quintal::Orbit`] is self-dual under
/// interval-structure reversal.
///
/// Re-exported from [`crate::quintal::orbit_self_duality`].
pub use crate::quintal::orbit_self_duality;

/// Verify that all 14 orbits are self-dual.
///
/// Re-exported from [`crate::quintal::verify_all_orbits_self_dual`].
pub use crate::quintal::verify_all_orbits_self_dual;
