use crate::quintal::{IntervalStructure, VoicedChord};

use super::error::QuartalError;
use super::interval::quintal_to_quartal_structure;

/// A quartal interval structure consisting of three intervals measured
/// top-to-bottom through a four-note chord. Each component should be
/// in the set {4, 5, 6} for a legal quartal structure (the inversional
/// complements of the quintal {6, 7, 8}).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuartalIntervalStructure(pub u8, pub u8, pub u8);

impl QuartalIntervalStructure {
    /// Create a new quartal interval structure from three interval values.
    pub fn new(a: u8, b: u8, c: u8) -> Self {
        QuartalIntervalStructure(a, b, c)
    }

    /// Returns `true` if each component is in {4, 5, 6}.
    pub fn is_legal(&self) -> bool {
        (4..=6).contains(&self.0) && (4..=6).contains(&self.1) && (4..=6).contains(&self.2)
    }

    /// Returns the three intervals as an array.
    pub fn intervals(&self) -> [u8; 3] {
        [self.0, self.1, self.2]
    }
}

/// A quartal perspective on a four-note voiced chord.
///
/// This is a newtype wrapper around [`VoicedChord`] that provides
/// quartal-native methods (reading intervals top-to-bottom as fourths)
/// while delegating storage and pitch validation to the inner type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuartalVoicedChord(pub VoicedChord);

impl QuartalVoicedChord {
    /// Create a new quartal voiced chord from four MIDI pitches.
    ///
    /// The pitches must be in strictly ascending order.
    pub fn new(pitches: [u8; 4]) -> Result<Self, QuartalError> {
        let vc = VoicedChord::new(pitches)?;
        Ok(QuartalVoicedChord(vc))
    }

    /// Returns the four MIDI pitches in ascending order.
    pub fn pitches(&self) -> [u8; 4] {
        self.0.pitches
    }

    /// Returns the quartal interval structure: the intervals read
    /// top-to-bottom, complemented to fourths.
    ///
    /// This applies both reversal (bottom-to-top becomes top-to-bottom)
    /// and complementation (fifths become fourths), giving the true
    /// quartal reading of the chord.
    pub fn quartal_interval_structure(&self) -> QuartalIntervalStructure {
        quintal_to_quartal_structure(&self.0.interval_structure())
    }

    /// Returns the quintal interval structure of the inner chord
    /// (bottom-to-top fifths reading).
    pub fn quintal_interval_structure(&self) -> IntervalStructure {
        self.0.interval_structure()
    }

    /// Convert to a pitch-class chord by reducing each pitch mod 12.
    pub fn to_pc_chord(&self) -> Result<crate::quintal::PcChord, QuartalError> {
        self.0.to_pc_chord().map_err(QuartalError::from)
    }

    /// Returns a reference to the inner [`VoicedChord`].
    pub fn inner(&self) -> &VoicedChord {
        &self.0
    }
}
