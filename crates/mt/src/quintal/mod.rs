//! Quintal chord types for the fiber bundle framework.

mod base_space;
mod centrality;
mod constructors;
mod display;
mod distance;
mod duality;
mod error;
mod fiber;
pub mod functional;
mod geodesics;
mod group;
pub(crate) mod modes;
mod orbit;
mod types;
mod verification;

pub use crate::voice_leading::min_voiced_chord_l1;
pub use base_space::{enumerate_all, is_adjacent, BaseSpace};
#[allow(deprecated)]
pub use centrality::crossroads_chords;
pub use centrality::{betweenness_centrality, saddle_chords};
pub use constructors::quintal_root;
pub use display::{pc_to_note_name, render_chord_dashed, render_pcset_dashed};
pub use distance::{all_distances_from, center, diameter, distance, eccentricity};
pub use duality::{
    orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t1_reversal_equivalence, verify_all_orbits_self_dual,
};
pub use error::QuintalError;
pub use fiber::{chord_scale, inversion_cycle, l1_distance, project, t1, t_minus1, ChordScale};
pub use functional::{FunctionalRegion, Pathway};
pub use geodesics::{
    count_geodesics, distances_and_geodesic_counts, geodesic_distribution, geodesics,
    passing_chords, DistAndGeodesicCounts, GeodesicBucket, GeodesicDistribution,
    GeodesicProfileEntry,
};
pub use group::{invert, invert_transpose, orbit, transpose};
pub use modes::{
    all_modes, all_parent_scales, modes_by_opening_interval, modes_in_cluster, orbit_modes,
    orbit_step_sequence, parent_scales, step_size_multiset, step_vocabulary_cluster,
    verify_fiber_mode_connection, verify_multiset_uniqueness, ModeError, OrbitModes, OthMode,
    ParentScale, StepVocabularyCluster,
};
pub use orbit::{classify_all, classify_orbit, Orbit};
pub use types::{FiberClass, IntervalStructure, PcChord, VoicedChord};
pub use verification::{
    fiber_class, inversion_l1_distances, inversions_in_base, verify_fiber_classes,
    verify_universal_l1_law,
};
