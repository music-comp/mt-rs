//! Quartal-perspective functional grammar — symmetric counterpart to
//! [`crate::quintal::functional`].
//!
//! The seven [`FunctionalRegion`] variants and two [`Pathway`] variants are
//! perspective-invariant — they describe the *topographic role* of an
//! orbit (Summit, Saddle, Slope, …), not its interval structure. Both
//! types are re-exported as-is from quintal. This module adds quartal-side
//! ergonomics: [`QuartalOrbit::functional_region`] as an extension method,
//! and two free functions returning quartal-labelled orbit lists.
//!
//! Region-level queries (e.g. "which quartal orbits live on the Slope?")
//! and pathway-level queries (e.g. "what is the quartal stop list for the
//! Cadence pathway?") are the primary use cases the walkthrough documents
//! will exercise.

use super::orbit::QuartalOrbit;

// Re-exports of perspective-invariant types from quintal.

/// The seven OTH functional regions.
///
/// Re-exported from [`crate::quintal::FunctionalRegion`].
/// The regions describe topographic role on the shared base-space graph —
/// they are perspective-invariant. The same orbit, read in either quintal
/// or quartal vocabulary, lives in the same region.
pub use crate::quintal::FunctionalRegion;

/// A canonical OTH functional pathway.
///
/// Re-exported from [`crate::quintal::Pathway`].
/// Pathway region sequences are perspective-invariant.
pub use crate::quintal::Pathway;

impl QuartalOrbit {
    /// Return the [`FunctionalRegion`] this orbit belongs to.
    ///
    /// Symmetric counterpart to [`crate::quintal::Orbit::functional_region`].
    /// Delegates via [`QuartalOrbit::to_quintal`] to maintain a single source
    /// of truth for the orbit-to-region mapping (decision D-quartal-T2-001).
    ///
    /// # Examples
    ///
    /// ```
    /// use music_comp_mt::quartal::{FunctionalRegion, QuartalOrbit};
    ///
    /// // Palindromic, single-orbit region:
    /// assert_eq!(QuartalOrbit::Q555.functional_region(), FunctionalRegion::Summit);
    /// // Palindromic, single-orbit region:
    /// assert_eq!(QuartalOrbit::Q646.functional_region(), FunctionalRegion::Saddle);
    /// // Asymmetric orbit maps correctly too:
    /// assert_eq!(QuartalOrbit::Q554.functional_region(), FunctionalRegion::Plateau);
    /// ```
    #[must_use]
    pub fn functional_region(&self) -> FunctionalRegion {
        self.to_quintal().functional_region()
    }
}

/// All quartal orbits that belong to a given functional region.
///
/// Returns the orbits in [`QuartalOrbit::all`] declaration order. The
/// returned `Vec` length matches the canonical region sizes:
/// Summit (1), Plateau (2), Slope (4), Valley (3), Saddle (1),
/// Precipice (2), Narrows (1) — totalling 14.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_orbits_in, FunctionalRegion, QuartalOrbit};
///
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Summit), vec![QuartalOrbit::Q555]);
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Saddle), vec![QuartalOrbit::Q646]);
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Slope).len(), 4);
/// ```
#[must_use]
pub fn quartal_orbits_in(region: FunctionalRegion) -> Vec<QuartalOrbit> {
    QuartalOrbit::all()
        .iter()
        .filter(|orbit| orbit.functional_region() == region)
        .copied()
        .collect()
}

/// The quartal-labelled stop list for a canonical [`Pathway`].
///
/// Returns one entry per region in [`Pathway::region_sequence`], pairing
/// the region with all quartal orbits in that region (in
/// [`QuartalOrbit::all`] declaration order). Used directly by the
/// walkthrough documents to render pathway tables in quartal vocabulary.
///
/// For [`Pathway::Cadence`] the returned `Vec` has length 3 (Saddle →
/// Slope → Summit). For [`Pathway::Departure`] the length is 4
/// (Summit → Plateau → Slope → Saddle).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_pathway_stops, FunctionalRegion, Pathway, QuartalOrbit};
///
/// let stops = quartal_pathway_stops(Pathway::Cadence);
/// assert_eq!(stops.len(), 3);
/// assert_eq!(stops[0].0, FunctionalRegion::Saddle);
/// assert_eq!(stops[0].1, vec![QuartalOrbit::Q646]);
/// assert_eq!(stops[2].0, FunctionalRegion::Summit);
/// assert_eq!(stops[2].1, vec![QuartalOrbit::Q555]);
/// ```
#[must_use]
pub fn quartal_pathway_stops(pathway: Pathway) -> Vec<(FunctionalRegion, Vec<QuartalOrbit>)> {
    pathway
        .region_sequence()
        .iter()
        .map(|&region| (region, quartal_orbits_in(region)))
        .collect()
}
