//! Quintal chord types for the fiber bundle framework.

mod base_space;
mod distance;
mod error;
mod geodesics;
mod group;
mod orbit;
mod types;

pub use base_space::{enumerate_all, is_adjacent, BaseSpace};
pub use distance::{all_distances_from, center, diameter, distance, eccentricity};
pub use error::QuintalError;
pub use geodesics::{count_geodesics, geodesics, passing_chords};
pub use group::{invert, invert_transpose, orbit, transpose};
pub use orbit::{classify_all, classify_orbit, Orbit};
pub use types::{FiberClass, IntervalStructure, PcChord, VoicedChord};
