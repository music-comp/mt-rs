//! Constructors that build voiced chords from abstract PcChord + octave.

use super::error::QuintalError;
use super::types::{PcChord, VoicedChord};

/// Build the canonical quintal-root [`VoicedChord`] for a [`PcChord`].
///
/// The bottom voice is placed at MIDI pitch `12 * base_octave + bottom_pc`,
/// where `bottom_pc` is the first pitch class in the chord's legal quintal
/// stacking. Intervals are stacked upward from there.
///
/// # Errors
///
/// Returns [`QuintalError::NoLegalStacking`] if the PcChord has no legal
/// quintal interval structure, or [`QuintalError::NotAscending`] if the
/// resulting pitches overflow or are not strictly ascending (only possible
/// for extreme `base_octave` values).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{quintal_root, PcChord};
///
/// let chord = PcChord::new([0, 2, 7, 9]).unwrap();
/// let voiced = quintal_root(&chord, 4).unwrap();
/// assert_eq!(voiced.pitches, [48, 55, 62, 69]);
/// ```
pub fn quintal_root(pc_chord: &PcChord, base_octave: u8) -> Result<VoicedChord, QuintalError> {
    let (ordered, is) = pc_chord
        .legal_quintal_stacking()
        .ok_or(QuintalError::NoLegalStacking)?;

    let bottom_midi = 12u16 * base_octave as u16 + ordered[0] as u16;
    let pitches: [u8; 4] = [
        bottom_midi as u8,
        (bottom_midi + is.0 as u16) as u8,
        (bottom_midi + is.0 as u16 + is.1 as u16) as u8,
        (bottom_midi + is.0 as u16 + is.1 as u16 + is.2 as u16) as u8,
    ];

    VoicedChord::new(pitches)
}
