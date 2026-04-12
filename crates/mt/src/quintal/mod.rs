//! Quintal chord types for the fiber bundle framework.

mod base_space;
mod centrality;
mod distance;
mod duality;
mod error;
mod fiber;
mod modes;
mod geodesics;
mod group;
mod orbit;
mod types;
mod verification;

pub use base_space::{enumerate_all, is_adjacent, BaseSpace};
pub use centrality::{betweenness_centrality, crossroads_chords};
pub use distance::{all_distances_from, center, diameter, distance, eccentricity};
pub use duality::{
    orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t1_reversal_equivalence, verify_all_orbits_self_dual,
};
pub use error::QuintalError;
pub use modes::{ModeError, OthMode};
pub use fiber::{chord_scale, inversion_cycle, l1_distance, project, t1, t_minus1, ChordScale};
pub use geodesics::{count_geodesics, geodesics, passing_chords};
pub use group::{invert, invert_transpose, orbit, transpose};
pub use orbit::{classify_all, classify_orbit, Orbit};
pub use types::{FiberClass, IntervalStructure, PcChord, VoicedChord};
pub use verification::{
    fiber_class, inversion_l1_distances, inversions_in_base, verify_fiber_classes,
    verify_universal_l1_law,
};
