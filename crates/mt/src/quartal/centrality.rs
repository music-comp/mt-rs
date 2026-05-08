//! Quartal-labelled centrality views — symmetric counterpart to
//! [`crate::quintal::centrality`].
//!
//! The mathematical content (Brandes' algorithm, betweenness centrality
//! values) lives in quintal. This module surfaces those values keyed by
//! [`QuartalOrbit`] and [`QuartalIntervalStructure`] so quartal-minded
//! callers can read centrality in their native vocabulary without
//! quintal-side conversion at every call site.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::quintal::{betweenness_centrality, classify_orbit, saddle_chords, BaseSpace, PcChord};

use super::conversion::pc_chord_quartal_intervals;
use super::orbit::QuartalOrbit;
use super::types::QuartalIntervalStructure;

/// Saddle chords paired with their quartal interval structures.
///
/// Identical PcChord set to [`crate::quintal::saddle_chords`] (the top six
/// chords by betweenness centrality — all members of the Saddle orbit
/// `Q686` / `Q646`), paired with each chord's quartal IS for direct
/// quartal-perspective inspection.
///
/// All six pairs return `QuartalIntervalStructure(6, 4, 6)` — the canonical
/// `Q646` quartal IS — because the Saddle orbit is palindromic.
///
/// # Panics
///
/// Panics if any saddle chord lacks a legal quintal interval structure (and
/// therefore no quartal IS). Saddle chords are always members of `Q686`,
/// which has a legal IS, so this should never happen on the canonical base
/// space; the `expect` exists to surface a regression elsewhere in the
/// codebase rather than silently mis-pair the IS.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{base_space, quartal_saddle_chords};
///
/// let space = base_space();
/// let saddle = quartal_saddle_chords(&space);
/// assert_eq!(saddle.len(), 6);
/// ```
#[must_use]
pub fn quartal_saddle_chords(space: &BaseSpace) -> Vec<(PcChord, QuartalIntervalStructure)> {
    saddle_chords(space)
        .into_iter()
        .map(|chord| {
            let is = pc_chord_quartal_intervals(&chord)
                .expect("saddle chord must have a legal quartal interval structure");
            (chord, is)
        })
        .collect()
}

/// All 14 quartal orbits ranked by maximum betweenness centrality of any
/// member chord (descending).
///
/// Ties are broken by the natural ordering of [`QuartalOrbit`] (its
/// variant declaration order).
///
/// # Algorithm
///
/// 1. Compute [`crate::quintal::betweenness_centrality`] once — `O(V·E)`.
/// 2. For each `PcChord` in the result, look up its quintal `Orbit` via
///    [`crate::quintal::classify_orbit`] and convert to [`QuartalOrbit`]
///    via [`QuartalOrbit::from_quintal`].
/// 3. Aggregate per `QuartalOrbit`: keep the maximum betweenness across
///    all members.
/// 4. Sort by descending betweenness, with ties broken by the natural
///    `QuartalOrbit` ordering.
///
/// `f64` ordering uses `partial_cmp(...).unwrap_or(Ordering::Equal)`;
/// `betweenness_centrality` produces non-negative finite values
/// (Brandes' algorithm on a non-empty connected graph) so `NaN` never
/// arises.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{base_space, quartal_orbits_by_betweenness, QuartalOrbit};
///
/// let space = base_space();
/// let ranked = quartal_orbits_by_betweenness(&space);
/// assert_eq!(ranked.len(), 14);
/// // The Saddle (Q646) is at the top.
/// assert_eq!(ranked[0].0, QuartalOrbit::Q646);
/// ```
#[must_use]
pub fn quartal_orbits_by_betweenness(space: &BaseSpace) -> Vec<(QuartalOrbit, f64)> {
    let bc = betweenness_centrality(space);

    let mut max_per_orbit: HashMap<QuartalOrbit, f64> = HashMap::new();
    for (chord, value) in &bc {
        if let Some(orb) = classify_orbit(chord) {
            let q_orb = QuartalOrbit::from_quintal(&orb);
            max_per_orbit
                .entry(q_orb)
                .and_modify(|cur| {
                    if *value > *cur {
                        *cur = *value;
                    }
                })
                .or_insert(*value);
        }
    }

    let mut ranked: Vec<(QuartalOrbit, f64)> = max_per_orbit.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
}
