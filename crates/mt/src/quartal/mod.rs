//! Quartal chord types — a dual perspective on the quintal fiber bundle.
//!
//! The quartal module provides quartal-native vocabulary for the same
//! 228-chord base space. Quartal reads intervals top-to-bottom as fourths;
//! quintal reads bottom-to-top as fifths. Both perspectives describe the
//! same mathematical structure.

mod centrality;
mod constructors;
mod conversion;
mod display;
mod duality;
mod error;
mod functional;
mod interval;
mod modes;
mod orbit;
mod types;
mod verification;
mod voicing;

pub use centrality::{quartal_orbits_by_betweenness, quartal_saddle_chords};
pub use constructors::{
    from_stacked_fourths, from_stacked_fourths_voiced, pure_quartal_stack, quartal_neighbors,
    quartal_root,
};
pub use conversion::{pc_chord_quartal_intervals, to_quartal, to_quintal};
pub use display::{
    pc_to_note_name, render_pcset_dashed, render_quartal_chord_dashed, render_quartal_is,
};
pub use duality::{
    orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t_quartal_reversal_equivalence, verify_all_orbits_self_dual,
};
pub use error::QuartalError;
pub use interval::{
    quartal_to_quintal_interval, quartal_to_quintal_structure, quintal_to_quartal_interval,
    quintal_to_quartal_structure,
};
pub use modes::quartal_orbit_modes;
pub use orbit::QuartalOrbit;
pub use types::{QuartalIntervalStructure, QuartalVoicedChord};
pub use verification::{verify_quartal_fiber_classes, verify_quartal_universal_l1_law};
pub use voicing::{quartal_inversion_cycle, quartal_l1_distances, t_quartal, t_quartal_reverse};

/// Create a new [`BaseSpace`] — the shared 228-chord base space.
///
/// This is the same base space used by the quintal perspective. The 228
/// PcChords and their adjacency structure are perspective-independent;
/// only the voicing interpretation (quintal = fifths bottom-up, quartal =
/// fourths bottom-up) differs.
pub fn base_space() -> BaseSpace {
    BaseSpace::new()
}

// Re-export shared infrastructure from quintal — the base space, fiber
// classes, and graph-theoretic tools are perspective-independent.
#[allow(deprecated)]
pub use crate::quintal::crossroads_chords;
pub use crate::quintal::{all_distances_from, center, diameter, distance, eccentricity};
pub use crate::quintal::{betweenness_centrality, saddle_chords};
pub use crate::quintal::{count_geodesics, geodesics, passing_chords};
pub use crate::quintal::{enumerate_all, is_adjacent};
pub use crate::quintal::{BaseSpace, FiberClass, PcChord};
pub use crate::voice_leading::min_voiced_chord_l1;
