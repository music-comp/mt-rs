//! OTH mode analysis: step sequences, cyclic rotations, modal families,
//! parent-scale identification, and fiber-mode verification for all 14 orbits.

use std::error;
use std::fmt;

use super::{Orbit, PcChord};

/// An error arising from OTH mode computation or verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeError {
    /// The fiber-mode connection failed for a specific orbit and rotation.
    FiberModeMismatch {
        orbit: Orbit,
        rotation: u8,
        expected: [u8; 4],
        actual: [u8; 4],
    },
    /// Two distinct orbits share the same step-size multiset.
    MultisetCollision {
        a: Orbit,
        b: Orbit,
        multiset: [u8; 4],
    },
    /// Step sequence does not sum to 12.
    InvalidStepSum {
        orbit: Orbit,
        steps: [u8; 4],
        sum: u8,
    },
}

impl fmt::Display for ModeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModeError::FiberModeMismatch {
                orbit,
                rotation,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "fiber-mode mismatch for {} rotation {}: expected {:?}, got {:?}",
                    orbit, rotation, expected, actual
                )
            }
            ModeError::MultisetCollision { a, b, multiset } => {
                write!(
                    f,
                    "multiset collision between {} and {}: {:?}",
                    a, b, multiset
                )
            }
            ModeError::InvalidStepSum { orbit, steps, sum } => {
                write!(
                    f,
                    "step sequence {:?} for {} sums to {} (expected 12)",
                    steps, orbit, sum
                )
            }
        }
    }
}

impl error::Error for ModeError {}

// ─── StepVocabularyCluster ───────────────────────────────────────────────

/// Provisional step-vocabulary cluster for an orbit.
///
/// IMPORTANT: These clusters are descriptive groupings based on which step sizes
/// appear in the orbit's step-size multiset. They are NOT proven theoretical
/// categories. The only established scalar relationship is:
///   - Summit \[7,7,7\] → pentatonic major
///
/// All other cluster assignments are observations awaiting formal justification.
/// The classification rules are purely mechanical (based on step-size membership)
/// and do not account for harmonic connections between orbits in B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum StepVocabularyCluster {
    /// Steps from {2, 3, 4, 5}, no semitone (1) or tritone step (6).
    NoSemitoneNoTritone,
    /// Step vocabulary includes 1 (semitone) but not 6 (tritone step).
    ContainsSemitone,
    /// Step vocabulary ⊆ {2, 4} — all steps are even, no semitones.
    EvenStepsOnly,
    /// Step vocabulary includes 6 (tritone step).
    ContainsTritoneStep,
}

impl fmt::Display for StepVocabularyCluster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StepVocabularyCluster::NoSemitoneNoTritone => write!(f, "No Semitone, No Tritone"),
            StepVocabularyCluster::ContainsSemitone => write!(f, "Contains Semitone"),
            StepVocabularyCluster::EvenStepsOnly => write!(f, "Even Steps Only"),
            StepVocabularyCluster::ContainsTritoneStep => write!(f, "Contains Tritone Step"),
        }
    }
}

// ─── OthMode ────────────────────────────────────────────────────────────

/// A mode of an OTH orbit: a cyclic rotation of the chord-scale step sequence.
///
/// `rotation` is 0-3, indicating which PC of the representative chord-scale
/// is treated as the starting note. The opening interval is derived from
/// `steps[0]` and is not independently settable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OthMode {
    orbit: Orbit,
    rotation: u8,
    steps: [u8; 4],
    pcs_from_c: [u8; 4],
}

impl OthMode {
    /// Create a new OTH mode.
    pub fn new(orbit: Orbit, rotation: u8, steps: [u8; 4], pcs_from_c: [u8; 4]) -> Self {
        Self {
            orbit,
            rotation,
            steps,
            pcs_from_c,
        }
    }

    pub fn orbit(&self) -> Orbit {
        self.orbit
    }

    pub fn rotation(&self) -> u8 {
        self.rotation
    }

    pub fn steps(&self) -> [u8; 4] {
        self.steps
    }

    pub fn pcs_from_c(&self) -> [u8; 4] {
        self.pcs_from_c
    }

    /// Opening interval (first step) — derived, not stored.
    pub fn opening_interval(&self) -> u8 {
        self.steps[0]
    }
}

impl fmt::Display for OthMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} M{} {:?}", self.orbit, self.rotation + 1, self.steps)
    }
}

// ─── Core computation functions ─────────────────────────────────────────

/// Build the representative `PcChord` for an orbit by stacking its interval
/// structure forward from pitch class 0.
fn representative_pc_chord(orbit: &Orbit) -> PcChord {
    let is = orbit.representative();
    let intervals = is.intervals();
    let p0: u8 = 0;
    let p1 = (p0 + intervals[0]) % 12;
    let p2 = (p1 + intervals[1]) % 12;
    let p3 = (p2 + intervals[2]) % 12;
    PcChord::new([p0, p1, p2, p3]).expect("orbit representative is always valid")
}

/// Compute the chord-scale step sequence for any PcChord (4 PCs in ascending order).
pub fn pc_chord_step_sequence(chord: &PcChord) -> [u8; 4] {
    let pcs = chord.pcs;
    let mut steps = [0u8; 4];
    for i in 0..3 {
        steps[i] = pcs[i + 1] - pcs[i];
    }
    steps[3] = pcs[0] + 12 - pcs[3];
    steps
}

/// Compute the chord-scale step sequence for an orbit from its representative PcChord.
/// Works entirely in pitch-class space — no VoicedChord needed.
pub fn orbit_step_sequence(orbit: &Orbit) -> [u8; 4] {
    let repr = representative_pc_chord(orbit);
    pc_chord_step_sequence(&repr)
}

/// Compute the step-size multiset for a step sequence (sorted, rotation-invariant).
/// Returns a fixed-size array — OTH chords are always 4-note.
pub fn step_size_multiset(steps: &[u8; 4]) -> [u8; 4] {
    let mut sorted = *steps;
    sorted.sort();
    sorted
}

/// Rotate a step sequence by `n` positions to the left.
fn rotate_steps(steps: &[u8; 4], n: usize) -> [u8; 4] {
    let n = n % 4;
    let mut result = [0u8; 4];
    for i in 0..4 {
        result[i] = steps[(i + n) % 4];
    }
    result
}

/// Classify a sorted step multiset into a StepVocabularyCluster.
fn step_vocabulary_cluster_from_multiset(multiset: &[u8; 4]) -> StepVocabularyCluster {
    let has_tritone_step = multiset.contains(&6);
    let has_semitone = multiset.contains(&1);
    let all_even = multiset.iter().all(|&s| s % 2 == 0);

    if has_tritone_step {
        StepVocabularyCluster::ContainsTritoneStep
    } else if all_even {
        StepVocabularyCluster::EvenStepsOnly
    } else if has_semitone {
        StepVocabularyCluster::ContainsSemitone
    } else {
        StepVocabularyCluster::NoSemitoneNoTritone
    }
}
