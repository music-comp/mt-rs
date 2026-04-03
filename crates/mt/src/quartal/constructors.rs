//! Quartal-native constructors for building chords from stacked fourths.
//!
//! These functions let you think in quartal terms (root + fourths upward)
//! rather than converting from quintal voicings after the fact.

use crate::quintal::{BaseSpace, PcChord};

use super::conversion::pc_chord_quartal_intervals;
use super::error::QuartalError;
use super::types::{QuartalIntervalStructure, QuartalVoicedChord};

/// Build a [`PcChord`] by stacking fourths upward from a root pitch class.
///
/// Each interval must be in {4, 5, 6} (diminished, perfect, or augmented
/// fourth). Exactly 3 intervals are required to produce a 4-note chord.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::from_stacked_fourths;
///
/// // A-D-G-C = {9,2,7,0} = sorted [0,2,7,9]
/// let chord = from_stacked_fourths(9, &[5, 5, 5]).unwrap();
/// assert_eq!(chord.pcs, [0, 2, 7, 9]);
/// ```
///
/// # Errors
///
/// Returns [`QuartalError::PitchClassOutOfRange`] if `root_pc > 11`,
/// [`QuartalError::WrongIntervalCount`] if the slice length is not 3,
/// [`QuartalError::IllegalInterval`] if any interval is outside {4, 5, 6},
/// or [`QuartalError::DuplicatePitchClasses`] if stacking produces
/// duplicate pitch classes mod 12.
pub fn from_stacked_fourths(root_pc: u8, intervals: &[u8]) -> Result<PcChord, QuartalError> {
    if root_pc > 11 {
        return Err(QuartalError::PitchClassOutOfRange(root_pc));
    }
    if intervals.len() != 3 {
        return Err(QuartalError::WrongIntervalCount(intervals.len()));
    }
    for &iv in intervals {
        if !(4..=6).contains(&iv) {
            return Err(QuartalError::IllegalInterval(iv));
        }
    }

    let cumulative_1 = u16::from(root_pc) + u16::from(intervals[0]);
    let cumulative_2 = cumulative_1 + u16::from(intervals[1]);
    let cumulative_3 = cumulative_2 + u16::from(intervals[2]);

    let pcs = [
        root_pc,
        (cumulative_1 % 12) as u8,
        (cumulative_2 % 12) as u8,
        (cumulative_3 % 12) as u8,
    ];

    PcChord::new(pcs).map_err(QuartalError::from)
}

/// Build a [`QuartalVoicedChord`] by stacking fourths upward from a root
/// MIDI pitch.
///
/// Each interval must be in {4, 5, 6} (diminished, perfect, or augmented
/// fourth). Exactly 3 intervals are required.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::from_stacked_fourths_voiced;
///
/// // A3-D4-G4-C5 = [57, 62, 67, 72]
/// let chord = from_stacked_fourths_voiced(57, &[5, 5, 5]).unwrap();
/// assert_eq!(chord.pitches(), [57, 62, 67, 72]);
/// ```
///
/// # Errors
///
/// Returns [`QuartalError::WrongIntervalCount`] if the slice length is not 3,
/// or [`QuartalError::IllegalInterval`] if any interval is outside {4, 5, 6}.
pub fn from_stacked_fourths_voiced(
    root_pitch: u8,
    intervals: &[u8],
) -> Result<QuartalVoicedChord, QuartalError> {
    if intervals.len() != 3 {
        return Err(QuartalError::WrongIntervalCount(intervals.len()));
    }
    for &iv in intervals {
        if !(4..=6).contains(&iv) {
            return Err(QuartalError::IllegalInterval(iv));
        }
    }

    let pitches = [
        root_pitch,
        root_pitch + intervals[0],
        root_pitch + intervals[0] + intervals[1],
        root_pitch + intervals[0] + intervals[1] + intervals[2],
    ];
    QuartalVoicedChord::new(pitches)
}

/// Build a [`PcChord`] from three stacked perfect fourths (5, 5, 5).
///
/// This is a convenience wrapper around [`from_stacked_fourths`] for the
/// most common quartal voicing.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::pure_quartal_stack;
///
/// // C-F-Bb-Eb = {0,5,10,3} = sorted [0,3,5,10]
/// let chord = pure_quartal_stack(0);
/// assert_eq!(chord.pcs, [0, 3, 5, 10]);
/// ```
///
/// # Panics
///
/// Panics if `root_pc > 11`. Use [`from_stacked_fourths`] for fallible
/// construction.
pub fn pure_quartal_stack(root_pc: u8) -> PcChord {
    from_stacked_fourths(root_pc, &[5, 5, 5])
        .expect("pure_quartal_stack requires root_pc in 0..=11")
}

/// Return the neighbors of a chord in the base space, paired with their
/// quartal interval structures.
///
/// Each neighbor is returned as `(PcChord, Option<QuartalIntervalStructure>)`.
/// The quartal interval structure is `Some` for all legal quartal chords
/// (which all chords in the base space are).
///
/// Returns an empty `Vec` if the chord is not in the base space.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{pure_quartal_stack, quartal_neighbors, BaseSpace};
///
/// let space = BaseSpace::new();
/// let chord = pure_quartal_stack(0);
/// let neighbors = quartal_neighbors(&chord, &space);
/// assert!(!neighbors.is_empty());
/// ```
pub fn quartal_neighbors(
    chord: &PcChord,
    space: &BaseSpace,
) -> Vec<(PcChord, Option<QuartalIntervalStructure>)> {
    let mut result = Vec::new();
    if let Some(neighbor_indices) = space.neighbors(chord) {
        for &idx in neighbor_indices {
            let neighbor = space.chords()[idx];
            let qis = pc_chord_quartal_intervals(&neighbor);
            result.push((neighbor, qis));
        }
    }
    result
}
