//! Quartal chord types — a dual perspective on the quintal fiber bundle.
//!
//! The quartal module provides quartal-native vocabulary for the same
//! 228-chord base space. Quartal reads intervals top-to-bottom as fourths;
//! quintal reads bottom-to-top as fifths. Both perspectives describe the
//! same mathematical structure.

mod constructors;
mod conversion;
mod error;
mod interval;
mod orbit;
mod types;
mod modes;
mod voicing;

pub use modes::quartal_orbit_modes;
pub use constructors::{
    from_stacked_fourths, from_stacked_fourths_voiced, pure_quartal_stack, quartal_neighbors,
};
pub use conversion::{pc_chord_quartal_intervals, to_quartal, to_quintal};
pub use error::QuartalError;
pub use interval::{
    quartal_to_quintal_interval, quartal_to_quintal_structure, quintal_to_quartal_interval,
    quintal_to_quartal_structure,
};
pub use orbit::QuartalOrbit;
pub use types::{QuartalIntervalStructure, QuartalVoicedChord};
pub use voicing::{quartal_inversion_cycle, quartal_l1_distances, t_quartal, t_quartal_reverse};

// Re-export shared infrastructure from quintal — the base space, fiber
// classes, and graph-theoretic tools are perspective-independent.
pub use crate::quintal::{all_distances_from, center, diameter, distance, eccentricity};
pub use crate::quintal::{betweenness_centrality, crossroads_chords};
pub use crate::quintal::{count_geodesics, geodesics, passing_chords};
pub use crate::quintal::{enumerate_all, is_adjacent};
pub use crate::quintal::{BaseSpace, FiberClass, PcChord};
