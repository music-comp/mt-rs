//! Melody harmonization with voice-led OTH chord progressions.
//!
//! Given a melody (a sequence of pitches or pitch classes), this module
//! finds the top-K chord progressions whose top voices trace out that
//! melody, ranked by least total semitone movement across all voice parts.
//! Candidate chords are drawn from the full 228-chord Open Tone Harmony
//! base space in both quintal and quartal voicings.
//!
//! # Conceptual model
//!
//! Each melody note pins the **top voice** of a four-note voiced chord.
//! For every pitch class in the melody, all chords in
//! [`crate::quintal::BaseSpace`] containing that pitch class contribute
//! exactly one inversion with that PC on top — from both the quintal
//! cycle (intervals in {6, 7, 8} semitones) and the quartal cycle
//! (intervals in {4, 5, 6} semitones). This yields 76 candidates per
//! pitch class per perspective, or 152 with both perspectives enabled.
//!
//! The algorithm then finds the K lowest-cost paths through the resulting
//! layered graph, where edge cost is the assignment-optimal L1 voice-leading
//! distance ([`crate::voice_leading::min_voiced_chord_l1`]).
//!
//! # Quick example
//!
//! ```
//! use music_comp_mt::harmonize::{harmonize_melody, HarmonizeOptions, MelodyInput};
//!
//! // Harmonize C–E–G (as pitch classes) with default options.
//! let results = harmonize_melody(
//!     MelodyInput::PitchClasses(vec![0, 4, 7]),
//!     HarmonizeOptions::default(),
//! )?;
//!
//! // Default K=10: up to 10 progressions, sorted by ascending movement.
//! assert!(results.len() <= 10);
//! assert!(results[0].total_movement <= results.last().unwrap().total_movement);
//!
//! // Every chord's top voice matches the melody target.
//! for chord in &results[0].chords {
//!     assert!(chord.pitches[3] <= 127);
//! }
//! # Ok::<(), music_comp_mt::harmonize::HarmonizeError>(())
//! ```
//!
//! # How it works
//!
//! Internally, [`harmonize_melody`] runs three stages:
//!
//! 1. **Canonicalize** — convert the input melody to target MIDI pitches
//!    by applying [`HarmonizeOptions::top_voice_offset`] (and
//!    [`HarmonizeOptions::melody_octave`] for pitch-class input).
//! 2. **Enumerate candidates** — for each target, find all voiced chords
//!    in the OTH base space whose top voice matches, drawn from quintal
//!    and/or quartal inversion cycles per [`DualityScope`].
//! 3. **Top-K Viterbi** — a k-best dynamic programming pass over the
//!    layered candidate graph, minimizing total L1 voice-leading cost.
//!
//! # When to use this
//!
//! - **Composers** exploring voice-led harmonizations of a melodic line
//!   within the OTH sound world.
//! - **Music theorists** investigating minimal-movement paths through the
//!   228-chord quintal/quartal space.
//! - **ML and algorithmic composition** pipelines that need a ranked list
//!   of harmonization candidates as training data or generation seeds.
//!
//! # See also
//!
//! - [`crate::quintal::BaseSpace`] — the 228-chord candidate pool.
//! - [`crate::quintal::Orbit`] — the 14 T/I orbit classification.
//! - [`crate::voice_leading::min_voiced_chord_l1`] — the edge-weight function.
//! - [`crate::quintal::quintal_root`] / [`crate::quartal::quartal_root`] —
//!   canonical voicing constructors used internally.

pub(crate) mod canonicalize;
pub(crate) mod candidates;
pub mod functional;
mod viterbi;

pub use functional::{classify_trajectory, match_functional_pathways, MatchedPathway, Trajectory};

use crate::quintal::VoicedChord;

/// Input melody: either pitch classes (octave-free) or absolute MIDI pitches.
///
/// Use [`MelodyInput::PitchClasses`] when you have scale degrees or pitch
/// classes without a specific octave — the octave is supplied via
/// [`HarmonizeOptions::melody_octave`]. Use [`MelodyInput::Pitches`] when
/// you already have concrete MIDI pitch numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum MelodyInput {
    /// Pitch classes in `0..=11` (C=0, C#=1, ..., B=11).
    ///
    /// The actual octave placement is determined by
    /// [`HarmonizeOptions::melody_octave`].
    PitchClasses(Vec<u8>),

    /// Absolute MIDI pitches in `0..=127` (Middle C = 60, A4 = 69).
    Pitches(Vec<u8>),
}

/// Which voicing perspective(s) to draw candidates from.
///
/// The OTH base space admits two disjoint inversion cycles per chord:
/// one quintal (intervals in {6, 7, 8}) and one quartal (intervals in
/// {4, 5, 6}). This enum controls which cycles contribute candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DualityScope {
    /// Draw candidates only from quintal inversion cycles (76 per position).
    QuintalOnly,
    /// Draw candidates only from quartal inversion cycles (76 per position).
    QuartalOnly,
    /// Draw from both quintal and quartal cycles (152 per position).
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

/// One harmonization result: a chord progression with voice-leading statistics.
///
/// Each [`Harmonization`] represents a complete assignment of one
/// [`VoicedChord`] per melody position,
/// together with the per-step and total voice-leading costs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Harmonization {
    /// One [`VoicedChord`] per melody position.
    /// `chords.len()` always equals the input melody length.
    pub chords: Vec<VoicedChord>,

    /// Per-step minimal voice-leading L1 distances.
    /// `per_step_movements[i]` is the cost of moving from `chords[i]` to
    /// `chords[i+1]`. Length is always `chords.len() - 1`.
    pub per_step_movements: Vec<u32>,

    /// Total voice-leading cost: the sum of [`per_step_movements`](Self::per_step_movements).
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
/// a [`VoicedChord`] whose top voice
/// (`pitches[3]`) equals the melody pitch at that position plus
/// [`HarmonizeOptions::top_voice_offset`]. Progressions are ranked by
/// ascending [`Harmonization::total_movement`].
///
/// # Errors
///
/// - [`HarmonizeError::EmptyMelody`] — the input melody contains no notes.
/// - [`HarmonizeError::InvalidPitchClass`] — a [`MelodyInput::PitchClasses`]
///   entry is outside `0..=11`.
/// - [`HarmonizeError::TargetMidiOutOfRange`] — the computed target MIDI pitch
///   (melody note + `top_voice_offset`) fell outside `0..=127`.
/// - [`HarmonizeError::NoCandidatesForPosition`] — no chord in the base space
///   has the target pitch class on top after shifting. Structurally unreachable
///   for any PC in `0..=11`.
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
///
/// # OTH framing
///
/// The returned progressions draw exclusively from the 228-chord OTH base
/// space. Each chord is a four-note quintal or quartal voicing; the
/// progression minimizes total semitone movement across all four voices.
///
/// ```
/// use music_comp_mt::harmonize::{
///     harmonize_melody, DualityScope, HarmonizeOptions, MelodyInput,
/// };
///
/// // Two-note melody: C then E (as MIDI, octave 5).
/// let results = harmonize_melody(
///     MelodyInput::Pitches(vec![72, 76]),
///     HarmonizeOptions { k: 3, ..Default::default() },
/// )?;
///
/// assert_eq!(results.len(), 3);
/// // Each result has exactly 2 chords (one per melody note).
/// for r in &results {
///     assert_eq!(r.chords.len(), 2);
///     assert_eq!(r.per_step_movements.len(), 1);
///     assert_eq!(r.total_movement, r.per_step_movements[0]);
/// }
/// // Results are sorted by ascending movement.
/// assert!(results[0].total_movement <= results[2].total_movement);
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
