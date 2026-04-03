//! Quartal orbit classification — the 14 T/I orbits labeled with quartal intervals.
//!
//! Each quartal orbit corresponds bijectively to a quintal [`Orbit`]. The
//! quartal interval labels are obtained by reversing and complementing
//! (mod 12) the quintal interval structure: quintal `(i1, i2, i3)` becomes
//! quartal `(12−i3, 12−i2, 12−i1)`.

use std::fmt;

use crate::quintal::Orbit;

use super::types::QuartalIntervalStructure;

/// The 14 orbits of legal quartal chords under the T/I group, labeled
/// with quartal interval structures (fourths: 4 = diminished, 5 = perfect,
/// 6 = augmented).
///
/// Each variant is named `Q` followed by the three quartal interval values.
/// For example, `Q555` is the orbit whose chords have a quartal interval
/// structure of (5,5,5) — three stacked perfect fourths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuartalOrbit {
    /// (5,5,5) — size 12, degree 8, "major analogue" (= quintal Q777)
    Q555,
    /// (5,6,5) — size 12, degree 4 (= quintal Q767)
    Q565,
    /// (5,4,5) — size 12, degree 8, "minor analogue" (= quintal Q787)
    Q545,
    /// (6,5,6) — size 6, degree 4, "diminished analogue" (= quintal Q676)
    Q656,
    /// (6,4,6) — size 6, degree 8, "augmented analogue" (= quintal Q686)
    Q646,
    /// (4,5,4) — size 12, degree 4 (= quintal Q878)
    Q454,
    /// (4,6,4) — size 12, degree 4 (= quintal Q868)
    Q464,
    /// (6,5,5) — size 24, degree 5 (= quintal Q776)
    Q655,
    /// (5,5,4) — size 24, degree 6 (= quintal Q877)
    Q554,
    /// (5,6,4) — size 24, degree 4 (= quintal Q867)
    Q564,
    /// (6,5,4) — size 24, degree 5 (= quintal Q876)
    Q654,
    /// (4,4,5) — size 24, degree 4 (= quintal Q788)
    Q445,
    /// (6,4,5) — size 24, degree 6 (= quintal Q786)
    Q645,
    /// (4,4,6) — size 12, degree 6 (= quintal Q688)
    Q446,
}

/// All 14 quartal orbit variants in declaration order.
static ALL_QUARTAL_ORBITS: [QuartalOrbit; 14] = [
    QuartalOrbit::Q555,
    QuartalOrbit::Q565,
    QuartalOrbit::Q545,
    QuartalOrbit::Q656,
    QuartalOrbit::Q646,
    QuartalOrbit::Q454,
    QuartalOrbit::Q464,
    QuartalOrbit::Q655,
    QuartalOrbit::Q554,
    QuartalOrbit::Q564,
    QuartalOrbit::Q654,
    QuartalOrbit::Q445,
    QuartalOrbit::Q645,
    QuartalOrbit::Q446,
];

impl QuartalOrbit {
    /// Returns a reference to the static array of all 14 quartal orbit variants.
    pub fn all() -> &'static [QuartalOrbit; 14] {
        &ALL_QUARTAL_ORBITS
    }

    /// Returns the representative quartal interval structure for this orbit.
    pub fn representative(&self) -> QuartalIntervalStructure {
        match self {
            QuartalOrbit::Q555 => QuartalIntervalStructure(5, 5, 5),
            QuartalOrbit::Q565 => QuartalIntervalStructure(5, 6, 5),
            QuartalOrbit::Q545 => QuartalIntervalStructure(5, 4, 5),
            QuartalOrbit::Q656 => QuartalIntervalStructure(6, 5, 6),
            QuartalOrbit::Q646 => QuartalIntervalStructure(6, 4, 6),
            QuartalOrbit::Q454 => QuartalIntervalStructure(4, 5, 4),
            QuartalOrbit::Q464 => QuartalIntervalStructure(4, 6, 4),
            QuartalOrbit::Q655 => QuartalIntervalStructure(6, 5, 5),
            QuartalOrbit::Q554 => QuartalIntervalStructure(5, 5, 4),
            QuartalOrbit::Q564 => QuartalIntervalStructure(5, 6, 4),
            QuartalOrbit::Q654 => QuartalIntervalStructure(6, 5, 4),
            QuartalOrbit::Q445 => QuartalIntervalStructure(4, 4, 5),
            QuartalOrbit::Q645 => QuartalIntervalStructure(6, 4, 5),
            QuartalOrbit::Q446 => QuartalIntervalStructure(4, 4, 6),
        }
    }

    /// Returns the orbit size (number of distinct chords in this orbit).
    ///
    /// This is the same as the corresponding quintal orbit's size, since the
    /// T/I group acts identically in both perspectives.
    pub fn size(&self) -> usize {
        self.to_quintal().size()
    }

    /// Returns the degree of each chord in this orbit within the base space.
    ///
    /// This is the same as the corresponding quintal orbit's degree.
    pub fn degree(&self) -> usize {
        self.to_quintal().degree()
    }

    /// Returns the tonal analogy for this orbit, if one exists.
    ///
    /// Four of the 14 orbits correspond to familiar tonal chord types:
    /// - Q555: "major" (three stacked perfect fourths)
    /// - Q545: "minor"
    /// - Q656: "diminished"
    /// - Q646: "augmented"
    pub fn analogy(&self) -> Option<&'static str> {
        match self {
            QuartalOrbit::Q555 => Some("major"),
            QuartalOrbit::Q545 => Some("minor"),
            QuartalOrbit::Q656 => Some("diminished"),
            QuartalOrbit::Q646 => Some("augmented"),
            _ => None,
        }
    }

    /// Convert a quintal [`Orbit`] to its quartal counterpart.
    pub fn from_quintal(orbit: &Orbit) -> QuartalOrbit {
        match orbit {
            Orbit::Q777 => QuartalOrbit::Q555,
            Orbit::Q767 => QuartalOrbit::Q565,
            Orbit::Q787 => QuartalOrbit::Q545,
            Orbit::Q676 => QuartalOrbit::Q656,
            Orbit::Q686 => QuartalOrbit::Q646,
            Orbit::Q878 => QuartalOrbit::Q454,
            Orbit::Q868 => QuartalOrbit::Q464,
            Orbit::Q776 => QuartalOrbit::Q655,
            Orbit::Q877 => QuartalOrbit::Q554,
            Orbit::Q867 => QuartalOrbit::Q564,
            Orbit::Q876 => QuartalOrbit::Q654,
            Orbit::Q788 => QuartalOrbit::Q445,
            Orbit::Q786 => QuartalOrbit::Q645,
            Orbit::Q688 => QuartalOrbit::Q446,
        }
    }

    /// Convert this quartal orbit to its quintal counterpart.
    pub fn to_quintal(&self) -> Orbit {
        match self {
            QuartalOrbit::Q555 => Orbit::Q777,
            QuartalOrbit::Q565 => Orbit::Q767,
            QuartalOrbit::Q545 => Orbit::Q787,
            QuartalOrbit::Q656 => Orbit::Q676,
            QuartalOrbit::Q646 => Orbit::Q686,
            QuartalOrbit::Q454 => Orbit::Q878,
            QuartalOrbit::Q464 => Orbit::Q868,
            QuartalOrbit::Q655 => Orbit::Q776,
            QuartalOrbit::Q554 => Orbit::Q877,
            QuartalOrbit::Q564 => Orbit::Q867,
            QuartalOrbit::Q654 => Orbit::Q876,
            QuartalOrbit::Q445 => Orbit::Q788,
            QuartalOrbit::Q645 => Orbit::Q786,
            QuartalOrbit::Q446 => Orbit::Q688,
        }
    }
}

impl fmt::Display for QuartalOrbit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is = self.representative();
        let name = |semitones: u8| -> &'static str {
            match semitones {
                4 => "d4",
                5 => "P4",
                6 => "A4",
                _ => "??",
            }
        };
        write!(f, "[{},{},{}]", name(is.0), name(is.1), name(is.2))?;
        if let Some(analogy) = self.analogy() {
            write!(f, " [{}]", analogy)?;
        }
        Ok(())
    }
}
