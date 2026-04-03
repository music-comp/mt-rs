use crate::chord::{Chord, Number, Quality};
use crate::note::Pitch;

/// A neo-Riemannian operation on a triad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NROperation {
    /// Parallel: flip major↔minor by moving the third.
    P,
    /// Relative: move to relative major/minor.
    R,
    /// Leading-tone exchange: move the root (major) or fifth (minor) by semitone.
    L,
}

/// Error for invalid neo-Riemannian operations.
#[derive(Debug)]
pub struct NRError(pub String);

impl std::fmt::Display for NRError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Apply a single neo-Riemannian transformation to a major or minor triad.
///
/// - **P** (Parallel): Same root, flip quality (C major → C minor)
/// - **R** (Relative): C major → A minor, A minor → C major
/// - **L** (Leading-tone): C major → E minor, E minor → C major
pub fn transform(chord: &Chord, op: NROperation) -> Result<Chord, NRError> {
    if chord.number != Number::Triad || !matches!(chord.quality, Quality::Major | Quality::Minor) {
        return Err(NRError(
            "Neo-Riemannian operations only apply to major/minor triads".into(),
        ));
    }

    let root_pc = chord.root.as_u8();
    let (new_root_pc, new_quality) = match (chord.quality, op) {
        // P: same root, flip quality
        (Quality::Major, NROperation::P) => (root_pc, Quality::Minor),
        (Quality::Minor, NROperation::P) => (root_pc, Quality::Major),

        // R: major → relative minor (root down minor 3rd = +9)
        //    minor → relative major (root up minor 3rd = +3)
        (Quality::Major, NROperation::R) => ((root_pc + 9) % 12, Quality::Minor),
        (Quality::Minor, NROperation::R) => ((root_pc + 3) % 12, Quality::Major),

        // L: major → move root down semitone → new chord rooted on major 3rd
        //    minor → move 5th up semitone → new chord rooted on (root+8)%12
        (Quality::Major, NROperation::L) => ((root_pc + 4) % 12, Quality::Minor),
        (Quality::Minor, NROperation::L) => ((root_pc + 8) % 12, Quality::Major),

        _ => unreachable!(),
    };

    Ok(Chord::new(
        Pitch::from_u8(new_root_pc),
        new_quality,
        Number::Triad,
    ))
}

/// Apply a chain of neo-Riemannian transformations.
/// Returns each intermediate chord (first element is the input).
pub fn transform_chain(chord: &Chord, ops: &[NROperation]) -> Result<Vec<Chord>, NRError> {
    let mut results = vec![chord.clone()];
    let mut current = chord.clone();
    for &op in ops {
        current = transform(&current, op)?;
        results.push(current.clone());
    }
    Ok(results)
}
