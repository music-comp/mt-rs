//! Quartal traversal operations — inversion cycle and L1 distances.
//!
//! In the quartal perspective, the "forward" inversion step is `t_minus1`
//! from the quintal world (dropping the top note by an octave), and the
//! "reverse" step is `t1`. Starting from the *same* VoicedChord, the
//! quartal inversion cycle traverses the same four voiced chords as the
//! quintal cycle but in the opposite direction (because t_quartal is
//! the inverse of t1).
//!
//! Note: the quintal-rooted and quartal-rooted cycles of a given PcChord
//! are *disjoint* — they share no voiced chords. Together they account
//! for all 8 distinct voicings of that PcChord modulo octave. See
//! [`crate::quartal::quartal_root`] for constructing the quartal-rooted
//! cycle.

use crate::quintal;

use super::types::QuartalVoicedChord;

/// One step in the quartal direction (= `t_minus1` in quintal terms).
///
/// This drops the highest pitch by an octave, producing the next quartal
/// inversion of the chord.
pub fn t_quartal(chord: &QuartalVoicedChord) -> QuartalVoicedChord {
    QuartalVoicedChord(quintal::t_minus1(&chord.0))
}

/// One step in the reverse-quartal direction (= `t1` in quintal terms).
///
/// This raises the lowest pitch by an octave, producing the previous
/// quartal inversion.
pub fn t_quartal_reverse(chord: &QuartalVoicedChord) -> QuartalVoicedChord {
    QuartalVoicedChord(quintal::t1(&chord.0))
}

/// Quartal inversion cycle: traversed via [`t_quartal`] (= `t_minus1`).
///
/// Returns `[chord, t_q(chord), t_q^2(chord), t_q^3(chord)]`.
///
/// When starting from the same VoicedChord, the quartal cycle visits the
/// same 4 chords as the quintal cycle but in reverse order. However, when
/// comparing the quintal-rooted cycle and the quartal-rooted cycle of the
/// same PcChord (via [`quintal_root`](crate::quintal::quintal_root) vs
/// [`quartal_root`](crate::quartal::quartal_root)), the two cycles are
/// disjoint — 8 distinct voicings total.
pub fn quartal_inversion_cycle(chord: &QuartalVoicedChord) -> [QuartalVoicedChord; 4] {
    let inv0 = *chord;
    let inv1 = t_quartal(&inv0);
    let inv2 = t_quartal(&inv1);
    let inv3 = t_quartal(&inv2);
    [inv0, inv1, inv2, inv3]
}

/// L1 distances between consecutive quartal inversions, closing back to root.
///
/// Returns `[d(qinv0,qinv1), d(qinv1,qinv2), d(qinv2,qinv3), d(qinv3,qinv0)]`.
///
/// By the Universal L1 Law, these must be `[12, 12, 12, 36]` for every
/// legal quartal chord — the same pattern holds in both quintal and quartal
/// directions.
pub fn quartal_l1_distances(chord: &QuartalVoicedChord) -> [u32; 4] {
    let cycle = quartal_inversion_cycle(chord);
    [
        quintal::l1_distance(&cycle[0].0, &cycle[1].0),
        quintal::l1_distance(&cycle[1].0, &cycle[2].0),
        quintal::l1_distance(&cycle[2].0, &cycle[3].0),
        quintal::l1_distance(&cycle[3].0, &cycle[0].0),
    ]
}
