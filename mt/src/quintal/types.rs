use super::QuintalError;

/// All 24 permutations of indices [0, 1, 2, 3].
const PERMUTATIONS_4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// An interval structure consisting of three intervals that partition
/// the octave within a quintal chord voicing. Each component should
/// be in the set {6, 7, 8} for a legal quintal structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntervalStructure(pub u8, pub u8, pub u8);

impl IntervalStructure {
    /// Create a new interval structure from three interval values.
    pub fn new(a: u8, b: u8, c: u8) -> Self {
        IntervalStructure(a, b, c)
    }

    /// Returns `true` if each component is in {6, 7, 8}.
    pub fn is_legal(&self) -> bool {
        (6..=8).contains(&self.0) && (6..=8).contains(&self.1) && (6..=8).contains(&self.2)
    }

    /// Returns the three intervals as an array.
    pub fn intervals(&self) -> [u8; 3] {
        [self.0, self.1, self.2]
    }
}

/// A pitch-class chord of exactly 4 distinct pitch classes (values 0..=11),
/// stored in sorted ascending order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PcChord {
    /// The four pitch classes in sorted ascending order.
    pub pcs: [u8; 4],
}

impl PcChord {
    /// Create a new `PcChord` from four pitch classes.
    ///
    /// The input is sorted ascending internally. All values must be in 0..=11
    /// and all four must be distinct.
    pub fn new(mut pcs: [u8; 4]) -> Result<Self, QuintalError> {
        for &pc in &pcs {
            if pc > 11 {
                return Err(QuintalError::PitchClassOutOfRange(pc));
            }
        }
        pcs.sort();
        // Check for duplicates (easy after sorting).
        for i in 0..3 {
            if pcs[i] == pcs[i + 1] {
                return Err(QuintalError::DuplicatePitchClasses);
            }
        }
        Ok(PcChord { pcs })
    }

    /// Create a `PcChord` from an unsorted slice of pitch classes.
    ///
    /// The slice must contain exactly 4 elements.
    pub fn from_unsorted(pcs: &[u8]) -> Result<Self, QuintalError> {
        if pcs.len() != 4 {
            return Err(QuintalError::WrongCardinality(pcs.len()));
        }
        let arr: [u8; 4] = [pcs[0], pcs[1], pcs[2], pcs[3]];
        Self::new(arr)
    }

    /// Search all 24 permutations of the four pitch classes for one whose
    /// three forward intervals (mod 12) are each in {6, 7, 8}.
    ///
    /// Returns `Some(IntervalStructure)` for the first valid permutation
    /// found, or `None` if no valid quintal ordering exists.
    pub fn interval_structure(&self) -> Option<IntervalStructure> {
        for perm in &PERMUTATIONS_4 {
            let a = (self.pcs[perm[1]] as u16 + 12 - self.pcs[perm[0]] as u16) % 12;
            let b = (self.pcs[perm[2]] as u16 + 12 - self.pcs[perm[1]] as u16) % 12;
            let c = (self.pcs[perm[3]] as u16 + 12 - self.pcs[perm[2]] as u16) % 12;
            let is = IntervalStructure(a as u8, b as u8, c as u8);
            if is.is_legal() {
                return Some(is);
            }
        }
        None
    }

    /// Returns `true` if this pitch-class chord admits a legal quintal
    /// interval structure (all forward intervals in {6, 7, 8}).
    pub fn is_legal(&self) -> bool {
        self.interval_structure().is_some()
    }

    /// Returns the four pitch classes as an array.
    pub fn pcs(&self) -> [u8; 4] {
        self.pcs
    }
}

/// A voiced chord of exactly 4 MIDI pitches in strictly ascending order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoicedChord {
    /// The four MIDI pitches in strictly ascending order.
    pub pitches: [u8; 4],
}

impl VoicedChord {
    /// Create a new `VoicedChord` from four MIDI pitches.
    ///
    /// The pitches must be in strictly ascending order.
    pub fn new(pitches: [u8; 4]) -> Result<Self, QuintalError> {
        if pitches[0] >= pitches[1] || pitches[1] >= pitches[2] || pitches[2] >= pitches[3] {
            return Err(QuintalError::NotAscending);
        }
        Ok(VoicedChord { pitches })
    }

    /// Returns the interval structure by direct subtraction of adjacent pitches.
    pub fn interval_structure(&self) -> IntervalStructure {
        IntervalStructure(
            self.pitches[1] - self.pitches[0],
            self.pitches[2] - self.pitches[1],
            self.pitches[3] - self.pitches[2],
        )
    }

    /// Convert to a `PcChord` by reducing each pitch mod 12.
    ///
    /// Returns an error if the reduction produces duplicate pitch classes
    /// (e.g., pitches 48 and 60 both reduce to pitch class 0).
    pub fn to_pc_chord(&self) -> Result<PcChord, QuintalError> {
        let pcs = [
            self.pitches[0] % 12,
            self.pitches[1] % 12,
            self.pitches[2] % 12,
            self.pitches[3] % 12,
        ];
        PcChord::new(pcs)
    }
}

/// A stub classification for quintal fiber bundles.
///
/// This will be expanded in later milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FiberClass {
    ClassA,
    ClassB,
}
