use crate::note::Note;

/// A single voice movement from one note to another.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoiceMovement {
    pub from: Note,
    pub to: Note,
    /// Signed semitone distance (positive=up, negative=down).
    pub semitones: i8,
}

/// The result of a voice-leading calculation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoiceLeading {
    pub movements: Vec<VoiceMovement>,
    /// Sum of absolute semitone movements across all voices.
    pub total_distance: u8,
}

/// Convert a Note to an absolute pitch number for distance calculation.
fn absolute_pitch(note: &Note) -> i16 {
    note.octave as i16 * 12 + note.pitch.as_u8() as i16
}

/// Find the voice assignment that minimizes total semitone movement.
/// Uses brute-force permutation (max 4! = 24 permutations for typical chords).
/// Both inputs should have the same length for meaningful results.
pub fn minimal_movement(from: &[Note], to: &[Note]) -> VoiceLeading {
    if from.is_empty() || to.is_empty() {
        return VoiceLeading {
            movements: vec![],
            total_distance: 0,
        };
    }

    let n = from.len().min(to.len());
    let from = &from[..n];
    let to = &to[..n];

    let mut best_distance = u16::MAX;
    let mut best_perm: Vec<usize> = (0..n).collect();
    let mut indices: Vec<usize> = (0..n).collect();

    permute(
        &mut indices, 0, n, from, to,
        &mut best_distance, &mut best_perm,
    );

    let mut movements = Vec::with_capacity(n);
    let mut total: u16 = 0;
    for (i, &j) in best_perm.iter().enumerate() {
        let dist = absolute_pitch(&to[j]) - absolute_pitch(&from[i]);
        total += dist.unsigned_abs();
        movements.push(VoiceMovement {
            from: from[i].clone(),
            to: to[j].clone(),
            semitones: dist as i8,
        });
    }

    VoiceLeading {
        movements,
        total_distance: total.min(255) as u8,
    }
}

fn permute(
    indices: &mut Vec<usize>,
    start: usize,
    n: usize,
    from: &[Note],
    to: &[Note],
    best_distance: &mut u16,
    best_perm: &mut Vec<usize>,
) {
    if start == n {
        let dist: u16 = indices.iter().enumerate()
            .map(|(i, &j)| {
                let d = absolute_pitch(&to[j]) - absolute_pitch(&from[i]);
                d.unsigned_abs()
            })
            .sum();
        if dist < *best_distance {
            *best_distance = dist;
            *best_perm = indices.clone();
        }
        return;
    }
    for k in start..n {
        indices.swap(start, k);
        permute(indices, start + 1, n, from, to, best_distance, best_perm);
        indices.swap(start, k);
    }
}
