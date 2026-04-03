use crate::chord::{Chord, Number, Quality};
use crate::note::{Notes, Pitch};
use crate::scale::{Direction, Mode, Scale, ScaleType};

/// A chord built on a scale degree with its diatonic context.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiatonicChord {
    /// Scale degree (1-7).
    pub degree: u8,
    /// Root pitch of the chord.
    pub root: Pitch,
    /// Chord quality (determined by stacking thirds from the scale).
    pub quality: Quality,
    /// Chord number (Triad or Seventh).
    pub number: Number,
    /// The chord itself.
    pub chord: Chord,
}

/// Generate diatonic triads for each degree of the given scale.
/// Only works with diatonic (7-note) modes; returns empty for others.
pub fn diatonic_triads(tonic: Pitch, mode: Mode) -> Vec<DiatonicChord> {
    build_diatonic_chords(tonic, mode, false)
}

/// Generate diatonic seventh chords for each degree of the given scale.
/// Only works with diatonic (7-note) modes; returns empty for others.
pub fn diatonic_sevenths(tonic: Pitch, mode: Mode) -> Vec<DiatonicChord> {
    build_diatonic_chords(tonic, mode, true)
}

fn build_diatonic_chords(tonic: Pitch, mode: Mode, sevenths: bool) -> Vec<DiatonicChord> {
    if !mode.is_diatonic() {
        return vec![];
    }

    let scale_type = ScaleType::from_mode(mode);
    let scale = match Scale::new(scale_type, tonic, 4, Some(mode), Direction::Ascending) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let scale_notes = scale.notes();
    // scale_notes includes octave duplicate at end; use only degrees 0..6
    let degree_count = scale_notes.len() - 1;
    let pitch_classes: Vec<u8> = scale_notes
        .iter()
        .take(degree_count)
        .map(|n| n.pitch.as_u8())
        .collect();

    let mut chords = Vec::new();

    for degree_idx in 0..degree_count {
        let root = scale_notes[degree_idx].pitch;

        // Stack thirds: root, 3rd (skip one), 5th (skip two), [7th (skip three)]
        let third_pc = pitch_classes[(degree_idx + 2) % degree_count];
        let fifth_pc = pitch_classes[(degree_idx + 4) % degree_count];

        let third_interval = (third_pc + 12 - root.as_u8()) % 12;
        let fifth_interval = (fifth_pc + 12 - root.as_u8()) % 12;

        let (quality, number) = if sevenths {
            let seventh_pc = pitch_classes[(degree_idx + 6) % degree_count];
            let seventh_interval = (seventh_pc + 12 - root.as_u8()) % 12;
            classify_seventh(third_interval, fifth_interval, seventh_interval)
        } else {
            (
                classify_triad(third_interval, fifth_interval),
                Number::Triad,
            )
        };

        let chord = Chord::new(root, quality, number);
        chords.push(DiatonicChord {
            degree: (degree_idx + 1) as u8,
            root,
            quality,
            number,
            chord,
        });
    }

    chords
}

fn classify_triad(third: u8, fifth: u8) -> Quality {
    match (third, fifth) {
        (4, 7) => Quality::Major,
        (3, 7) => Quality::Minor,
        (3, 6) => Quality::Diminished,
        (4, 8) => Quality::Augmented,
        _ => Quality::Major, // fallback
    }
}

fn classify_seventh(third: u8, fifth: u8, seventh: u8) -> (Quality, Number) {
    match (third, fifth, seventh) {
        (4, 7, 11) => (Quality::Major, Number::MajorSeventh),
        (3, 7, 10) => (Quality::Minor, Number::Seventh),
        (4, 7, 10) => (Quality::Dominant, Number::Seventh),
        (3, 6, 10) => (Quality::HalfDiminished, Number::Seventh),
        (3, 6, 9) => (Quality::Diminished, Number::Seventh),
        (4, 8, 11) => (Quality::Augmented, Number::MajorSeventh),
        (3, 7, 11) => (Quality::Minor, Number::MajorSeventh),
        _ => (Quality::Major, Number::Seventh), // fallback
    }
}
