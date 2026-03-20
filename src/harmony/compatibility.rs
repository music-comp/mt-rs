use crate::chord::Chord;
use crate::note::{Notes, Pitch};
use crate::scale::{Mode, Scale};

/// Find all scales whose pitch-class set contains all chord tones.
/// Results sorted by scale size (smallest first).
pub fn compatible_scales(chord: &Chord) -> Vec<(Pitch, Mode)> {
    let chord_pitches: Vec<Pitch> = chord.notes().iter().map(|n| n.pitch).collect();
    Scale::identify(&chord_pitches)
}
