//! Quartal chord types — a dual perspective on the quintal fiber bundle.
//!
//! The quartal module provides quartal-native vocabulary for the same
//! 228-chord base space. Quartal reads intervals top-to-bottom as fourths;
//! quintal reads bottom-to-top as fifths. Both perspectives describe the
//! same mathematical structure.

mod conversion;
mod error;
mod interval;
mod types;

pub use conversion::{pc_chord_quartal_intervals, to_quartal, to_quintal};
pub use error::QuartalError;
pub use interval::{
    quartal_to_quintal_interval, quartal_to_quintal_structure, quintal_to_quartal_interval,
    quintal_to_quartal_structure,
};
pub use types::{QuartalIntervalStructure, QuartalVoicedChord};
