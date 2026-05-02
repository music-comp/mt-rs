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

pub mod canonicalize;
pub mod candidates;
mod viterbi;

use crate::quintal::VoicedChord;

/// Input melody: either pitch classes (caller-supplied octave-free) or
/// MIDI pitches (caller-supplied with octaves).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MelodyInput {
    /// Each element is a pitch class in `0..=11`.
    PitchClasses(Vec<u8>),
    /// Each element is a MIDI pitch in `0..=127`.
    Pitches(Vec<u8>),
}

/// Which voicing perspective(s) to draw candidates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DualityScope {
    QuintalOnly,
    QuartalOnly,
    #[default]
    Both,
}

/// Configuration for [`harmonize_melody`].
#[derive(Debug, Clone)]
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
#[non_exhaustive]
pub enum HarmonizeError {
    #[error("melody must contain at least one note")]
    EmptyMelody,

    #[error("melody pitch class {0} is outside the valid range 0..=11")]
    InvalidPitchClass(u8),

    #[error(
        "target MIDI pitch {target_midi} at position {position} is out of \
         the playable range after applying top_voice_offset"
    )]
    TargetMidiOutOfRange { position: usize, target_midi: u8 },

    #[error(
        "no candidate chord found for melody position {position} \
         (top pitch {top_midi})"
    )]
    NoCandidatesForPosition { position: usize, top_midi: u8 },

    #[error("requested K={requested} but only {available} distinct progressions exist")]
    InsufficientProgressions { requested: usize, available: usize },
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
