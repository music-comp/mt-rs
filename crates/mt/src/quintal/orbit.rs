//! Orbit classification under the T/I group for quintal pitch-class chords.
//!
//! The 228 legal quintal chords partition into exactly 14 orbits under the
//! 24-element T/I group. Each orbit is identified by its representative
//! interval structure (e.g., Q777 for the "major analogue" orbit with
//! interval structure (7,7,7)).

use std::collections::BTreeMap;
use std::fmt;

use super::group;
use super::{IntervalStructure, PcChord};

/// The 14 orbits of legal quintal chords under the T/I group.
///
/// Each variant is named `Q` followed by the three interval values of its
/// representative interval structure. For example, `Q777` represents the
/// orbit whose chords admit an interval structure of (7,7,7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Orbit {
    /// (7,7,7) — size 12, degree 8, "major analogue"
    Q777,
    /// (7,6,7) — size 12, degree 4
    Q767,
    /// (7,8,7) — size 12, degree 8, "minor analogue"
    Q787,
    /// (6,7,6) — size 6, degree 4, "diminished analogue"
    Q676,
    /// (6,8,6) — size 6, degree 8, "augmented analogue"
    Q686,
    /// (8,7,8) — size 12, degree 4
    Q878,
    /// (8,6,8) — size 12, degree 4
    Q868,
    /// (7,7,6) — size 24, degree 5
    Q776,
    /// (8,7,7) — size 24, degree 6
    Q877,
    /// (8,6,7) — size 24, degree 4
    Q867,
    /// (8,7,6) — size 24, degree 5
    Q876,
    /// (7,8,8) — size 24, degree 4
    Q788,
    /// (7,8,6) — size 24, degree 6
    Q786,
    /// (6,8,8) — size 12, degree 6
    Q688,
}

/// All 14 orbit variants in declaration order.
static ALL_ORBITS: [Orbit; 14] = [
    Orbit::Q777,
    Orbit::Q767,
    Orbit::Q787,
    Orbit::Q676,
    Orbit::Q686,
    Orbit::Q878,
    Orbit::Q868,
    Orbit::Q776,
    Orbit::Q877,
    Orbit::Q867,
    Orbit::Q876,
    Orbit::Q788,
    Orbit::Q786,
    Orbit::Q688,
];

impl Orbit {
    /// Returns a reference to the static array of all 14 orbit variants.
    pub fn all() -> &'static [Orbit; 14] {
        &ALL_ORBITS
    }

    /// Returns the representative interval structure for this orbit.
    pub fn representative(&self) -> IntervalStructure {
        match self {
            Orbit::Q777 => IntervalStructure(7, 7, 7),
            Orbit::Q767 => IntervalStructure(7, 6, 7),
            Orbit::Q787 => IntervalStructure(7, 8, 7),
            Orbit::Q676 => IntervalStructure(6, 7, 6),
            Orbit::Q686 => IntervalStructure(6, 8, 6),
            Orbit::Q878 => IntervalStructure(8, 7, 8),
            Orbit::Q868 => IntervalStructure(8, 6, 8),
            Orbit::Q776 => IntervalStructure(7, 7, 6),
            Orbit::Q877 => IntervalStructure(8, 7, 7),
            Orbit::Q867 => IntervalStructure(8, 6, 7),
            Orbit::Q876 => IntervalStructure(8, 7, 6),
            Orbit::Q788 => IntervalStructure(7, 8, 8),
            Orbit::Q786 => IntervalStructure(7, 8, 6),
            Orbit::Q688 => IntervalStructure(6, 8, 8),
        }
    }

    /// Returns the orbit size (number of distinct chords in this orbit).
    ///
    /// Possible values are 6, 12, or 24, determined by the chord's
    /// stabilizer subgroup within the T/I group.
    pub fn size(&self) -> usize {
        match self {
            Orbit::Q676 | Orbit::Q686 => 6,
            Orbit::Q777 | Orbit::Q767 | Orbit::Q787 | Orbit::Q878 | Orbit::Q868 | Orbit::Q688 => 12,
            Orbit::Q776 | Orbit::Q877 | Orbit::Q867 | Orbit::Q876 | Orbit::Q788 | Orbit::Q786 => 24,
        }
    }

    /// Returns the degree of each chord in this orbit within the base space.
    ///
    /// All chords in the same orbit have the same degree because the T/I
    /// group preserves adjacency structure.
    pub fn degree(&self) -> usize {
        match self {
            Orbit::Q767 | Orbit::Q676 | Orbit::Q878 | Orbit::Q868 | Orbit::Q867 | Orbit::Q788 => 4,
            Orbit::Q776 | Orbit::Q876 => 5,
            Orbit::Q877 | Orbit::Q786 | Orbit::Q688 => 6,
            Orbit::Q777 | Orbit::Q787 | Orbit::Q686 => 8,
        }
    }

    /// Returns the tonal analogy for this orbit, if one exists.
    ///
    /// Four of the 14 orbits correspond to familiar tonal chord types:
    /// - Q777: "major" (the stack of three perfect fifths)
    /// - Q787: "minor"
    /// - Q676: "diminished"
    /// - Q686: "augmented"
    pub fn analogy(&self) -> Option<&'static str> {
        match self {
            Orbit::Q777 => Some("major"),
            Orbit::Q787 => Some("minor"),
            Orbit::Q676 => Some("diminished"),
            Orbit::Q686 => Some("augmented"),
            _ => None,
        }
    }
}

impl fmt::Display for Orbit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is = self.representative();
        write!(f, "Q({},{},{})", is.0, is.1, is.2)?;
        if let Some(analogy) = self.analogy() {
            write!(f, " [{}]", analogy)?;
        }
        Ok(())
    }
}

/// Build the representative `PcChord` for an orbit by applying the interval
/// structure forward from pitch class 0.
fn orbit_representative(orb: Orbit) -> PcChord {
    let is = orb.representative();
    let p0: u8 = 0;
    let p1 = (p0 + is.0) % 12;
    let p2 = (p1 + is.1) % 12;
    let p3 = (p2 + is.2) % 12;
    let mut pcs = [p0, p1, p2, p3];
    pcs.sort();
    PcChord { pcs }
}

/// Classify a single chord into its orbit type.
///
/// Computes the full T/I orbit of the given chord, then checks which of
/// the 14 known orbit representatives falls within that orbit. Returns
/// `None` if the chord does not belong to any recognized orbit (which
/// should not happen for legal quintal chords).
pub fn classify_orbit(chord: &PcChord) -> Option<Orbit> {
    let orbit_chords = group::orbit(chord);
    for &orb in Orbit::all() {
        let rep = orbit_representative(orb);
        if orbit_chords.contains(&rep) {
            return Some(orb);
        }
    }
    None
}

/// Partition a slice of chords by orbit.
///
/// Returns a map from each `Orbit` to the vector of chords belonging to it.
/// Chords that cannot be classified (non-legal chords) are silently skipped.
pub fn classify_all(chords: &[PcChord]) -> BTreeMap<Orbit, Vec<PcChord>> {
    let mut map: BTreeMap<Orbit, Vec<PcChord>> = BTreeMap::new();
    for &chord in chords {
        if let Some(orb) = classify_orbit(&chord) {
            map.entry(orb).or_default().push(chord);
        }
    }
    map
}
