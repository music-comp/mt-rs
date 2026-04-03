use crate::quintal::{PcChord, VoicedChord};

use super::interval::quintal_to_quartal_structure;
use super::types::{QuartalIntervalStructure, QuartalVoicedChord};

/// Reinterpret a quintal [`VoicedChord`] as a quartal chord (thin wrapper).
pub fn to_quartal(chord: &VoicedChord) -> QuartalVoicedChord {
    QuartalVoicedChord(*chord)
}

/// Extract the quintal [`VoicedChord`] from a quartal wrapper.
pub fn to_quintal(chord: &QuartalVoicedChord) -> VoicedChord {
    chord.0
}

/// Read a [`PcChord`]'s intervals as quartal (top-to-bottom, complemented).
///
/// Returns `None` if the chord has no legal quintal interval structure.
pub fn pc_chord_quartal_intervals(chord: &PcChord) -> Option<QuartalIntervalStructure> {
    let quintal_is = chord.interval_structure()?;
    Some(quintal_to_quartal_structure(&quintal_is))
}
