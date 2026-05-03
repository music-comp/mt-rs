//! OTH functional regions and canonical pathways.
//!
//! The 14 T/I orbits map to seven functional regions named after natural
//! landforms. Two canonical pathways (Cadence and Departure) define
//! recognized harmonic motions through this terrain.

use super::Orbit;

/// The seven OTH functional regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FunctionalRegion {
    /// Q777. Maximum stability.
    Summit,
    /// Q787, Q877. Stable but not the apex.
    Plateau,
    /// Q786, Q776, Q876, Q867. Transitional terrain.
    Slope,
    /// Q767, Q868, Q878. Maximum tension, minimum connectivity.
    Valley,
    /// Q686. Structural dominant, T6-symmetric pivot.
    Saddle,
    /// Q688, Q788. Edge of stability.
    Precipice,
    /// Q676. Most constrained orbit.
    Narrows,
}

impl std::fmt::Display for FunctionalRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FunctionalRegion::Summit => "Summit",
            FunctionalRegion::Plateau => "Plateau",
            FunctionalRegion::Slope => "Slope",
            FunctionalRegion::Valley => "Valley",
            FunctionalRegion::Saddle => "Saddle",
            FunctionalRegion::Precipice => "Precipice",
            FunctionalRegion::Narrows => "Narrows",
        };
        f.write_str(name)
    }
}

impl Orbit {
    /// Return the functional region this orbit belongs to.
    pub fn functional_region(&self) -> FunctionalRegion {
        match self {
            Orbit::Q777 => FunctionalRegion::Summit,
            Orbit::Q787 | Orbit::Q877 => FunctionalRegion::Plateau,
            Orbit::Q786 | Orbit::Q776 | Orbit::Q876 | Orbit::Q867 => FunctionalRegion::Slope,
            Orbit::Q767 | Orbit::Q868 | Orbit::Q878 => FunctionalRegion::Valley,
            Orbit::Q686 => FunctionalRegion::Saddle,
            Orbit::Q688 | Orbit::Q788 => FunctionalRegion::Precipice,
            Orbit::Q676 => FunctionalRegion::Narrows,
        }
    }
}

/// A canonical OTH functional pathway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Pathway {
    /// Saddle → Slope → Summit.
    Cadence,
    /// Summit → Plateau → Slope → Saddle.
    Departure,
}

impl Pathway {
    /// The functional-region sequence this pathway represents.
    pub fn region_sequence(&self) -> &'static [FunctionalRegion] {
        match self {
            Pathway::Cadence => &[
                FunctionalRegion::Saddle,
                FunctionalRegion::Slope,
                FunctionalRegion::Summit,
            ],
            Pathway::Departure => &[
                FunctionalRegion::Summit,
                FunctionalRegion::Plateau,
                FunctionalRegion::Slope,
                FunctionalRegion::Saddle,
            ],
        }
    }

    /// All canonical pathways.
    pub fn all() -> &'static [Pathway] {
        &[Pathway::Cadence, Pathway::Departure]
    }
}

impl std::fmt::Display for Pathway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pathway::Cadence => f.write_str("Cadence"),
            Pathway::Departure => f.write_str("Departure"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_14_orbits_map_to_regions() {
        use FunctionalRegion::*;
        let expected: Vec<(Orbit, FunctionalRegion)> = vec![
            (Orbit::Q777, Summit),
            (Orbit::Q787, Plateau),
            (Orbit::Q877, Plateau),
            (Orbit::Q786, Slope),
            (Orbit::Q776, Slope),
            (Orbit::Q876, Slope),
            (Orbit::Q867, Slope),
            (Orbit::Q767, Valley),
            (Orbit::Q868, Valley),
            (Orbit::Q878, Valley),
            (Orbit::Q686, Saddle),
            (Orbit::Q688, Precipice),
            (Orbit::Q788, Precipice),
            (Orbit::Q676, Narrows),
        ];
        assert_eq!(expected.len(), 14);
        for (orbit, region) in expected {
            assert_eq!(
                orbit.functional_region(),
                region,
                "{:?} should map to {:?}",
                orbit,
                region
            );
        }
    }

    #[test]
    fn test_region_counts() {
        let mut counts = std::collections::HashMap::new();
        for orbit in Orbit::all() {
            *counts.entry(orbit.functional_region()).or_insert(0) += 1;
        }
        assert_eq!(counts[&FunctionalRegion::Summit], 1);
        assert_eq!(counts[&FunctionalRegion::Plateau], 2);
        assert_eq!(counts[&FunctionalRegion::Slope], 4);
        assert_eq!(counts[&FunctionalRegion::Valley], 3);
        assert_eq!(counts[&FunctionalRegion::Saddle], 1);
        assert_eq!(counts[&FunctionalRegion::Precipice], 2);
        assert_eq!(counts[&FunctionalRegion::Narrows], 1);
    }

    #[test]
    fn test_pathway_region_sequences() {
        use FunctionalRegion::*;
        assert_eq!(
            Pathway::Cadence.region_sequence(),
            &[Saddle, Slope, Summit]
        );
        assert_eq!(
            Pathway::Departure.region_sequence(),
            &[Summit, Plateau, Slope, Saddle]
        );
    }

    #[test]
    fn test_pathway_all() {
        let all = Pathway::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], Pathway::Cadence);
        assert_eq!(all[1], Pathway::Departure);
    }
}
