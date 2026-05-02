//! Voice-led OTH chord progressions for melodies.
//!
//! Given a melody (a sequence of pitches or pitch classes), enumerate the
//! top-K voice-led progressions whose top-voice line traces out that melody,
//! drawing from the full fiber bundle E (all 4 inversions per PcChord) in
//! both quintal and quartal voicings.
//!
//! # Architecture
//!
//! 1. **Canonicalize** the melody to target top MIDI pitches (one per position).
//! 2. **Enumerate candidates** for each position: all VoicedChords whose top
//!    voice matches the target pitch class, shifted to the target octave.
//! 3. **Top-K Viterbi** finds the K lowest-cost paths through the layered DAG
//!    where edge weights are [`min_voiced_chord_l1`](crate::voice_leading::min_voiced_chord_l1).
//!
//! See [`crate::quintal::BaseSpace`] for the 228-chord candidate pool,
//! [`crate::quintal::quintal_root`] / [`crate::quartal::quartal_root`] for
//! canonical voicing construction.

pub(crate) mod canonicalize;
pub(crate) mod candidates;
mod viterbi;

use crate::quintal::VoicedChord;

/// Input melody: either pitch classes (caller-supplied octave-free) or
/// MIDI pitches (caller-supplied with octaves).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum MelodyInput {
    /// Each element is a pitch class in `0..=11`.
    PitchClasses(Vec<u8>),
    /// Each element is a MIDI pitch in `0..=127`.
    Pitches(Vec<u8>),
}

/// Which voicing perspective(s) to draw candidates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DualityScope {
    QuintalOnly,
    QuartalOnly,
    #[default]
    Both,
}

/// Configuration for [`harmonize_melody`].
///
/// Use struct-update syntax to override only the fields you need:
///
/// ```
/// use music_comp_mt::harmonize::{HarmonizeOptions, DualityScope};
///
/// let opts = HarmonizeOptions {
///     k: 5,
///     duality: DualityScope::QuintalOnly,
///     ..Default::default()
/// };
/// assert_eq!(opts.k, 5);
/// assert_eq!(opts.top_voice_offset, -12); // unchanged default
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HarmonizeOptions {
    /// Semitone offset from each melody pitch to that chord's top voice.
    /// Negative pulls the harmony below the melody. Default: -12 (one octave).
    pub top_voice_offset: i8,
    /// Used only with [`MelodyInput::PitchClasses`]: the octave to place the
    /// implied melody in before applying `top_voice_offset`. Default: 5.
    pub melody_octave: i8,
    /// Voicing perspective(s) to draw from. Default: [`DualityScope::Both`].
    pub duality: DualityScope,
    /// Number of distinct progressions to return, ordered by ascending total
    /// movement. Default: 10.
    pub k: usize,
}

impl Default for HarmonizeOptions {
    fn default() -> Self {
        HarmonizeOptions {
            top_voice_offset: -12,
            melody_octave: 5,
            duality: DualityScope::Both,
            k: 10,
        }
    }
}

/// One harmonization result: a sequence of voiced chords plus voice-leading
/// statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Harmonization {
    /// One `VoicedChord` per melody position.
    pub chords: Vec<VoicedChord>,
    /// Per-step minimal voice-leading L1 distances. Length = chords.len() - 1.
    pub per_step_movements: Vec<u32>,
    /// Sum of `per_step_movements`.
    pub total_movement: u32,
}

/// Errors returned by [`harmonize_melody`].
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum HarmonizeError {
    /// The input melody was empty. At least one note is required.
    #[error("melody must contain at least one note")]
    EmptyMelody,

    /// A pitch class in [`MelodyInput::PitchClasses`] was outside `0..=11`.
    #[error("melody pitch class {0} is outside the valid range 0..=11")]
    InvalidPitchClass(u8),

    /// The computed target MIDI pitch (melody note + `top_voice_offset`) fell
    /// outside the playable range `0..=127`. The `target_midi` field contains
    /// the actual computed value (which may be negative or above 127).
    #[error(
        "target MIDI pitch {target_midi} at position {position} is out of \
         the playable range after applying top_voice_offset"
    )]
    TargetMidiOutOfRange { position: usize, target_midi: i16 },

    /// No candidate chord exists for the target pitch class at the given
    /// position. This should be unreachable for any legal pitch class in
    /// `0..=11` — every PC is contained in at least 76 of the 228 base-space
    /// chords.
    #[error(
        "no candidate chord found for melody position {position} \
         (top pitch {top_midi})"
    )]
    NoCandidatesForPosition { position: usize, top_midi: u8 },
}

/// Harmonize a melody with the top-K voice-led OTH chord progressions.
///
/// Each returned [`Harmonization`] has, for every melody position,
/// a [`VoicedChord`] whose top voice (`pitches[3]`) equals the melody pitch
/// at that position plus `options.top_voice_offset`. Progressions are
/// ranked by ascending `total_movement`.
///
/// # Errors
///
/// Returns [`HarmonizeError::EmptyMelody`] if `melody` contains no notes,
/// [`HarmonizeError::InvalidPitchClass`] if a [`MelodyInput::PitchClasses`]
/// entry is outside `0..=11`, or [`HarmonizeError::TargetMidiOutOfRange`]
/// if the computed target pitch falls outside the MIDI range `0..=127`.
///
/// # Examples
///
/// ```
/// use music_comp_mt::harmonize::{harmonize_melody, HarmonizeOptions, MelodyInput};
///
/// let result = harmonize_melody(
///     MelodyInput::PitchClasses(vec![0, 4, 7]),
///     HarmonizeOptions::default(),
/// )?;
/// assert!(!result.is_empty());
/// # Ok::<(), music_comp_mt::harmonize::HarmonizeError>(())
/// ```
pub fn harmonize_melody(
    melody: MelodyInput,
    options: HarmonizeOptions,
) -> Result<Vec<Harmonization>, HarmonizeError> {
    let targets = canonicalize::canonicalize_melody(&melody, &options)?;

    if options.k == 0 {
        return Ok(Vec::new());
    }

    let space = crate::quintal::BaseSpace::new();

    let mut layers: Vec<Vec<VoicedChord>> = Vec::with_capacity(targets.len());
    for (pos, &target) in targets.iter().enumerate() {
        let cands = candidates::candidates_for_top(target, options.duality, &space, pos)?;
        if cands.is_empty() {
            return Err(HarmonizeError::NoCandidatesForPosition {
                position: pos,
                top_midi: target,
            });
        }
        layers.push(cands);
    }

    Ok(viterbi::top_k_viterbi(&layers, options.k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let cases: Vec<(HarmonizeError, &str)> = vec![
            (
                HarmonizeError::EmptyMelody,
                "melody must contain at least one note",
            ),
            (
                HarmonizeError::InvalidPitchClass(13),
                "melody pitch class 13 is outside the valid range 0..=11",
            ),
            (
                HarmonizeError::TargetMidiOutOfRange {
                    position: 2,
                    target_midi: -7,
                },
                "target MIDI pitch -7 at position 2 is out of the playable range after applying top_voice_offset",
            ),
            (
                HarmonizeError::NoCandidatesForPosition {
                    position: 0,
                    top_midi: 60,
                },
                "no candidate chord found for melody position 0 (top pitch 60)",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected, "Display mismatch for {:?}", err);
        }
    }
}
