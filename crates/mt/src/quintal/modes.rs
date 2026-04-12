//! OTH mode analysis: step sequences, cyclic rotations, modal families,
//! parent-scale identification, and fiber-mode verification for all 14 orbits.

use std::error;
use std::fmt;

use super::{chord_scale, t1, Orbit, PcChord, VoicedChord};
use crate::scale::ScaleType;
use crate::set_class::PitchClassSet;

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

/// Assign a step-vocabulary cluster to an orbit based on its step-size multiset.
///
/// This is a PROVISIONAL classification — see [`StepVocabularyCluster`] docs.
/// Rules (applied in order — first match wins):
/// 1. If step vocabulary includes 6 → ContainsTritoneStep
/// 2. If step vocabulary ⊆ {2, 4} → EvenStepsOnly
/// 3. If step vocabulary includes 1 → ContainsSemitone
/// 4. Otherwise → NoSemitoneNoTritone
pub fn step_vocabulary_cluster(orbit: &Orbit) -> StepVocabularyCluster {
    let steps = orbit_step_sequence(orbit);
    let multiset = step_size_multiset(&steps);
    step_vocabulary_cluster_from_multiset(&multiset)
}

// ─── ParentScale ────────────────────────────────────────────────────────

/// A traditional scale that contains an orbit's PC set.
///
/// Uses structured types instead of stringly-typed fields.
/// Coverage stored as exact integer ratio to avoid f32 imprecision.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParentScale {
    scale_type: ScaleType,
    root: u8,
    cardinality: u8,
    pcs: Vec<u8>,
    coverage_num: u8,
    coverage_den: u8,
}

impl ParentScale {
    /// Create a new parent scale. Coverage is computed as `4 / cardinality`.
    pub fn new(scale_type: ScaleType, root: u8, pcs: Vec<u8>) -> Self {
        let cardinality = pcs.len() as u8;
        Self {
            scale_type,
            root,
            cardinality,
            pcs,
            coverage_num: 4,
            coverage_den: cardinality,
        }
    }

    pub fn scale_type(&self) -> ScaleType {
        self.scale_type
    }

    pub fn root(&self) -> u8 {
        self.root
    }

    pub fn cardinality(&self) -> u8 {
        self.cardinality
    }

    pub fn pcs(&self) -> &[u8] {
        &self.pcs
    }

    /// Coverage as a float ratio (e.g. 4/5 = 0.8). For display only.
    pub fn coverage(&self) -> f32 {
        self.coverage_num as f32 / self.coverage_den as f32
    }

    /// Coverage as exact integer ratio (numerator, denominator).
    pub fn coverage_ratio(&self) -> (u8, u8) {
        (self.coverage_num, self.coverage_den)
    }
}

// ─── OrbitModes ─────────────────────────────────────────────────────────

/// Complete mode data for one orbit.
///
/// Cannot derive `Copy` because of `Vec` fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrbitModes {
    orbit: Orbit,
    step_size_multiset: [u8; 4],
    step_cluster: StepVocabularyCluster,
    modes: Vec<OthMode>,
    parent_scales: Vec<ParentScale>,
}

impl OrbitModes {
    pub fn orbit(&self) -> Orbit {
        self.orbit
    }

    /// Forte number computed on demand via existing set_class infrastructure.
    pub fn forte_number(&self) -> Option<String> {
        let repr = representative_pc_chord(&self.orbit);
        let pcs_set = PitchClassSet::new(&repr.pcs);
        pcs_set.forte_number()
    }

    pub fn step_size_multiset(&self) -> [u8; 4] {
        self.step_size_multiset
    }

    pub fn step_cluster(&self) -> StepVocabularyCluster {
        self.step_cluster
    }

    pub fn modes(&self) -> &[OthMode] {
        &self.modes
    }

    /// Number of distinct modes: 2 for T₆-symmetric orbits, 4 otherwise.
    pub fn distinct_count(&self) -> u8 {
        self.modes.len() as u8
    }

    pub fn parent_scales(&self) -> &[ParentScale] {
        &self.parent_scales
    }

}

// ─── orbit_modes ────────────────────────────────────────────────────────

/// Compute all distinct modes for an orbit.
///
/// Generates all 4 cyclic rotations of the step sequence, deduplicates
/// (T₆-symmetric orbits produce only 2 distinct modes), and computes
/// pitch classes transposed to start on C (pc 0).
pub fn orbit_modes(orbit: &Orbit) -> OrbitModes {
    let base_steps = orbit_step_sequence(orbit);
    let multiset = step_size_multiset(&base_steps);
    let cluster = step_vocabulary_cluster_from_multiset(&multiset);

    let mut modes = Vec::with_capacity(4);
    let mut seen_steps: Vec<[u8; 4]> = Vec::with_capacity(4);

    for rot in 0..4u8 {
        let steps = rotate_steps(&base_steps, rot as usize);
        if seen_steps.contains(&steps) {
            continue;
        }
        seen_steps.push(steps);

        // Compute pcs_from_c by cumulative sum from 0
        let mut pcs = [0u8; 4];
        for i in 1..4 {
            pcs[i] = pcs[i - 1] + steps[i - 1];
        }

        modes.push(OthMode::new(*orbit, rot, steps, pcs));
    }

    OrbitModes {
        orbit: *orbit,
        step_size_multiset: multiset,
        step_cluster: cluster,
        modes,
        parent_scales: Vec::new(),
    }
}

// ─── Collection / filter functions ──────────────────────────────────────

/// Get all modes across all 14 orbits.
pub fn all_modes() -> Vec<OrbitModes> {
    Orbit::all().iter().map(|o| orbit_modes(o)).collect()
}

/// Get all modes with a given opening interval size.
/// Returns owned OthMode values (OthMode is Copy).
pub fn modes_by_opening_interval(interval: u8) -> Vec<OthMode> {
    let mut result = Vec::new();
    for orbit in Orbit::all() {
        let om = orbit_modes(orbit);
        for mode in om.modes() {
            if mode.opening_interval() == interval {
                result.push(*mode);
            }
        }
    }
    result
}

// ─── Verification functions ──────────────────────────────────────────────

/// Verify that no two distinct orbits share the same step-size multiset.
pub fn verify_multiset_uniqueness() -> Result<(), ModeError> {
    let orbits = Orbit::all();
    for i in 0..orbits.len() {
        for j in (i + 1)..orbits.len() {
            let si = step_size_multiset(&orbit_step_sequence(&orbits[i]));
            let sj = step_size_multiset(&orbit_step_sequence(&orbits[j]));
            if si == sj {
                return Err(ModeError::MultisetCollision {
                    a: orbits[i],
                    b: orbits[j],
                    multiset: si,
                });
            }
        }
    }
    Ok(())
}

/// Verify that mode rotation is the PC-level projection of the t₁ fiber action.
///
/// For each orbit: voice the representative in a reference register, apply t₁ k
/// times, and verify that the step sequence *starting from the lowest voiced pitch*
/// equals the k-th rotation of the root step sequence.
///
/// The key insight: `chord_scale()` always sorts PCs ascending, which erases the
/// rotation. But the voicing order tracks which PC is the "starting point" — the
/// lowest MIDI pitch identifies which rotation we're in.
pub fn verify_fiber_mode_connection() -> Result<(), ModeError> {
    for orbit in Orbit::all() {
        let base_steps = orbit_step_sequence(orbit);
        let repr = representative_pc_chord(orbit);
        let pcs = repr.pcs;

        let pitches: [u8; 4] = [48 + pcs[0], 48 + pcs[1], 48 + pcs[2], 48 + pcs[3]];
        let voiced = VoicedChord::new(pitches).expect("orbit representative should be valid");

        let mut current = voiced;
        for rot in 0..4u8 {
            // Get the chord scale (PCs sorted ascending)
            let cs = chord_scale(&current);
            // Find which rotation matches: the lowest MIDI pitch's PC tells us
            // which position in the sorted PC set is the "root" of this voicing.
            let lowest_pc = current.pitches[0] % 12;
            let root_idx = cs.pcs.iter().position(|&p| p == lowest_pc).unwrap();

            // The step sequence starting from root_idx should be the rotation
            let actual_steps = rotate_steps(&cs.steps, root_idx);
            let expected = rotate_steps(&base_steps, rot as usize);

            if actual_steps != expected {
                return Err(ModeError::FiberModeMismatch {
                    orbit: *orbit,
                    rotation: rot,
                    expected,
                    actual: actual_steps,
                });
            }
            current = t1(&current);
        }
    }
    Ok(())
}

// ─── Parent-scale analysis ───────────────────────────────────────────────

/// Scale patterns as intervals from root (used for subset checking).
/// Each entry: (ScaleType, interval pattern from root).
fn scale_library() -> Vec<(ScaleType, Vec<u8>)> {
    use ScaleType::*;
    vec![
        // Pentatonic
        (PentatonicMajor, vec![0, 2, 4, 7, 9]),
        (PentatonicMinor, vec![0, 3, 5, 7, 10]),
        // Diatonic (Ionian mode; other modes covered by transposition)
        (Diatonic, vec![0, 2, 4, 5, 7, 9, 11]),
        // Melodic minor (ascending)
        (MelodicMinor, vec![0, 2, 3, 5, 7, 9, 11]),
        // Harmonic minor
        (HarmonicMinor, vec![0, 2, 3, 5, 7, 8, 11]),
        // Whole-tone
        (WholeTone, vec![0, 2, 4, 6, 8, 10]),
        // Blues
        (Blues, vec![0, 3, 5, 6, 7, 10]),
        // Octatonic (half-whole)
        (Octatonic, vec![0, 1, 3, 4, 6, 7, 9, 10]),
        // Octatonic (whole-half)
        (Octatonic, vec![0, 2, 3, 5, 6, 8, 9, 11]),
    ]
}

/// Identify all traditional scales that contain the given 4-PC set as a subset.
///
/// Generates all 12 transpositions of each scale type, checks subset containment,
/// and returns results sorted by coverage (descending), then by scale_type + root.
pub fn parent_scales(pcs: &[u8; 4]) -> Vec<ParentScale> {
    let target: std::collections::BTreeSet<u8> = pcs.iter().copied().collect();
    let mut results = Vec::new();

    for (scale_type, base_intervals) in &scale_library() {
        for root in 0..12u8 {
            let scale_pcs: std::collections::BTreeSet<u8> = base_intervals
                .iter()
                .map(|&iv| (root + iv) % 12)
                .collect();

            if target.is_subset(&scale_pcs) {
                let pcs_vec: Vec<u8> = scale_pcs.into_iter().collect();
                results.push(ParentScale::new(*scale_type, root, pcs_vec));
            }
        }
    }

    // Sort by coverage descending, then by scale_type + root for stability
    results.sort_by(|a, b| {
        b.coverage()
            .partial_cmp(&a.coverage())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.scale_type().cmp(&b.scale_type()))
            .then_with(|| a.root().cmp(&b.root()))
    });

    results
}

/// Compute parent scales for all 14 orbits.
pub fn all_parent_scales() -> Vec<(Orbit, Vec<ParentScale>)> {
    Orbit::all()
        .iter()
        .map(|orbit| {
            let repr = representative_pc_chord(orbit);
            (*orbit, parent_scales(&repr.pcs))
        })
        .collect()
}

/// Get all orbits in a given step-vocabulary cluster.
pub fn modes_in_cluster(cluster: StepVocabularyCluster) -> Vec<OrbitModes> {
    all_modes()
        .into_iter()
        .filter(|om| om.step_cluster() == cluster)
        .collect()
}
