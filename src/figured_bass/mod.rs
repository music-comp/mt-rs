use crate::note::{Note, Notes, Pitch};
use crate::scale::{Direction, Mode, Scale, ScaleType};

/// A single figured bass figure (interval above bass with optional accidental).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Figure {
    /// Interval number above the bass (3, 5, 6, 7, etc.)
    pub interval: u8,
    /// Accidental modification: 0=diatonic, 1=sharp, -1=flat
    pub accidental: i8,
}

/// A realized figured bass chord.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Realization {
    pub bass: Note,
    pub upper_voices: Vec<Note>,
}

/// Realize a figured bass line into chord voicings.
///
/// Each bass note is paired with a set of figures. Empty figures imply
/// root position (5/3). The key signature determines default accidentals.
pub fn realize(
    bass_notes: &[Note],
    figures: &[Vec<Figure>],
    key_tonic: Pitch,
    key_mode: Mode,
) -> Vec<Realization> {
    let n = bass_notes.len().min(figures.len());
    let scale_pcs = get_scale_pitch_classes(key_tonic, key_mode);

    let mut results = Vec::with_capacity(n);

    for i in 0..n {
        let bass = &bass_notes[i];
        let figs = &figures[i];

        let intervals = resolve_figures(figs);

        let bass_pc = bass.pitch.as_u8();
        let bass_octave = bass.octave;

        let mut upper_voices = Vec::new();
        for (interval_num, accidental) in &intervals {
            let degree_offset = (*interval_num - 1) as usize;
            let target_pc = diatonic_interval_above(bass_pc, degree_offset, &scale_pcs);
            let target_pc = ((target_pc as i8 + accidental + 12) % 12) as u8;

            let target_octave = if target_pc > bass_pc {
                bass_octave
            } else {
                bass_octave + 1
            };

            upper_voices.push(Note::new(Pitch::from_u8(target_pc), target_octave));
        }

        results.push(Realization {
            bass: bass.clone(),
            upper_voices,
        });
    }

    results
}

/// Resolve figure shorthand into explicit interval+accidental pairs.
fn resolve_figures(figs: &[Figure]) -> Vec<(u8, i8)> {
    if figs.is_empty() {
        // Root position triad: 3rd and 5th
        return vec![(3, 0), (5, 0)];
    }

    if figs.len() == 1 {
        match figs[0].interval {
            6 => return vec![(3, 0), (6, figs[0].accidental)],
            7 => return vec![(3, 0), (5, 0), (7, figs[0].accidental)],
            _ => {}
        }
    }

    // Custom figures: include implied intervals
    let mut intervals: Vec<(u8, i8)> = figs.iter().map(|f| (f.interval, f.accidental)).collect();
    if !intervals.iter().any(|(i, _)| *i == 3) {
        intervals.push((3, 0));
    }
    if !intervals.iter().any(|(i, _)| *i == 5 || *i == 6) {
        intervals.push((5, 0));
    }
    intervals.sort_by_key(|(i, _)| *i);
    intervals
}

/// Get the scale degree pitch classes for a key.
fn get_scale_pitch_classes(tonic: Pitch, mode: Mode) -> Vec<u8> {
    let scale_type = ScaleType::from_mode(mode);
    if let Ok(scale) = Scale::new(scale_type, tonic, 4, Some(mode), Direction::Ascending) {
        scale.notes().iter().map(|n| n.pitch.as_u8()).collect()
    } else {
        (0..12).collect()
    }
}

/// Find the pitch class that is `degree_offset` diatonic steps above `bass_pc`.
fn diatonic_interval_above(bass_pc: u8, degree_offset: usize, scale_pcs: &[u8]) -> u8 {
    let scale_len = scale_pcs.len().saturating_sub(1).max(1);
    if let Some(bass_idx) = scale_pcs.iter().position(|&pc| pc == bass_pc) {
        let target_idx = (bass_idx + degree_offset) % scale_len;
        scale_pcs[target_idx]
    } else {
        // Bass not in scale — approximate with chromatic interval
        (bass_pc + degree_offset as u8 * 2) % 12
    }
}
