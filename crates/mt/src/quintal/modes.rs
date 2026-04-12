//! OTH mode analysis: step sequences, cyclic rotations, modal families,
//! parent-scale identification, and fiber-mode verification for all 14 orbits.

use std::error;
use std::fmt;

use super::Orbit;

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
