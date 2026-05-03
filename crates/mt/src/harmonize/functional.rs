//! Functional pathway matching and trajectory classification for harmonizations.
//!
//! Given a [`Harmonization`] produced by [`crate::harmonize::harmonize_melody`],
//! these helpers detect canonical OTH pathways (Cadence, Departure) and
//! classify the progression's degree trajectory (Ascent / Descent / Traverse
//! / Mixed). Useful for re-ranking results by functional interest rather
//! than by least-movement only.

use crate::quintal::{classify_orbit, FunctionalRegion, Pathway};

use super::Harmonization;

/// A canonical pathway found within a progression.
///
/// Returned by [`match_functional_pathways`]. Multiple matches per
/// progression are possible — the same pathway type can appear at
/// different positions, and different pathways can overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchedPathway {
    /// Which canonical pathway matched (e.g. [`Pathway::Cadence`]).
    pub pathway: Pathway,
    /// 0-indexed position in the progression's `chords` array where the
    /// match starts. Inclusive.
    pub start_position: usize,
    /// 0-indexed position where the match ends. Inclusive.
    pub end_position: usize,
}

/// Find all canonical OTH functional pathways that occur as contiguous
/// subsequences in a harmonization's functional-region sequence.
///
/// Each chord in the input is projected to its [`FunctionalRegion`] (via
/// orbit classification and [`crate::quintal::Orbit::functional_region`]).
/// The resulting region sequence is scanned for occurrences of every
/// canonical pathway returned by [`Pathway::all`].
///
/// Matching is contiguous-only. Overlapping matches and multiple matches
/// of the same pathway are returned independently.
pub fn match_functional_pathways(harmonization: &Harmonization) -> Vec<MatchedPathway> {
    let regions: Vec<Option<FunctionalRegion>> = harmonization
        .chords
        .iter()
        .map(|vc| {
            vc.to_pc_chord()
                .ok()
                .and_then(|pc| classify_orbit(&pc))
                .map(|orb| orb.functional_region())
        })
        .collect();

    let mut matches = Vec::new();
    for &pathway in Pathway::all() {
        let pattern = pathway.region_sequence();
        if pattern.len() > regions.len() {
            continue;
        }
        for i in 0..=regions.len() - pattern.len() {
            let window = &regions[i..i + pattern.len()];
            let matched = window
                .iter()
                .zip(pattern.iter())
                .all(|(actual, expected)| *actual == Some(*expected));
            if matched {
                matches.push(MatchedPathway {
                    pathway,
                    start_position: i,
                    end_position: i + pattern.len() - 1,
                });
            }
        }
    }
    matches
}

/// The shape of a progression's degree trajectory across positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Trajectory {
    /// Degrees non-decreasing and not all equal.
    Ascent,
    /// Degrees non-increasing and not all equal.
    Descent,
    /// All degrees equal.
    Traverse,
    /// Neither monotonic nor constant.
    Mixed,
}

impl std::fmt::Display for Trajectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trajectory::Ascent => f.write_str("Ascent"),
            Trajectory::Descent => f.write_str("Descent"),
            Trajectory::Traverse => f.write_str("Traverse"),
            Trajectory::Mixed => f.write_str("Mixed"),
        }
    }
}

/// Classify a harmonization's degree trajectory.
///
/// Computes the per-position degree by classifying each chord's orbit,
/// then categorizes the sequence per the [`Trajectory`] variants.
pub fn classify_trajectory(harmonization: &Harmonization) -> Trajectory {
    let degrees: Vec<usize> = harmonization
        .chords
        .iter()
        .map(|vc| {
            let pc = vc.to_pc_chord().expect(
                "invariant: harmonize_melody output is in BaseSpace (PcChord projection cannot fail)",
            );
            let orb = classify_orbit(&pc).expect(
                "invariant: harmonize_melody output is in BaseSpace (orbit classification cannot fail)",
            );
            orb.degree()
        })
        .collect();

    if degrees.len() <= 1 {
        return Trajectory::Traverse;
    }

    let all_equal = degrees.windows(2).all(|w| w[0] == w[1]);
    if all_equal {
        return Trajectory::Traverse;
    }

    let non_decreasing = degrees.windows(2).all(|w| w[0] <= w[1]);
    if non_decreasing {
        return Trajectory::Ascent;
    }

    let non_increasing = degrees.windows(2).all(|w| w[0] >= w[1]);
    if non_increasing {
        return Trajectory::Descent;
    }

    Trajectory::Mixed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quintal::VoicedChord;

    fn make_harmonization(pitches_list: &[[u8; 4]]) -> Harmonization {
        Harmonization {
            chords: pitches_list
                .iter()
                .map(|p| VoicedChord { pitches: *p })
                .collect(),
            per_step_movements: vec![0; pitches_list.len().saturating_sub(1)],
            total_movement: 0,
        }
    }

    #[test]
    fn test_trajectory_ascent() {
        // Degrees: 4, 5, 6, 8
        // Q867=4, Q776=5, Q877=6, Q777=8
        // Need chords from each orbit. Use quintal_root at octave 4.
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let find_chord_in_orbit = |target_orbit: Orbit| -> [u8; 4] {
            for pc in space.chords() {
                if classify_orbit(pc) == Some(target_orbit) {
                    let vc = quintal_root(pc, 4).unwrap();
                    return vc.pitches;
                }
            }
            panic!("no chord found in orbit {:?}", target_orbit);
        };

        let h = make_harmonization(&[
            find_chord_in_orbit(Orbit::Q867), // degree 4
            find_chord_in_orbit(Orbit::Q776), // degree 5
            find_chord_in_orbit(Orbit::Q877), // degree 6
            find_chord_in_orbit(Orbit::Q777), // degree 8
        ]);
        assert_eq!(classify_trajectory(&h), Trajectory::Ascent);
    }

    #[test]
    fn test_trajectory_descent() {
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let find_chord_in_orbit = |target_orbit: Orbit| -> [u8; 4] {
            for pc in space.chords() {
                if classify_orbit(pc) == Some(target_orbit) {
                    return quintal_root(pc, 4).unwrap().pitches;
                }
            }
            panic!("no chord found in orbit {:?}", target_orbit);
        };

        let h = make_harmonization(&[
            find_chord_in_orbit(Orbit::Q777), // degree 8
            find_chord_in_orbit(Orbit::Q877), // degree 6
            find_chord_in_orbit(Orbit::Q776), // degree 5
            find_chord_in_orbit(Orbit::Q867), // degree 4
        ]);
        assert_eq!(classify_trajectory(&h), Trajectory::Descent);
    }

    #[test]
    fn test_trajectory_traverse() {
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let mut degree_6_chords: Vec<[u8; 4]> = Vec::new();
        for pc in space.chords() {
            if classify_orbit(pc) == Some(Orbit::Q877) {
                degree_6_chords.push(quintal_root(pc, 4).unwrap().pitches);
                if degree_6_chords.len() == 3 {
                    break;
                }
            }
        }

        let h = make_harmonization(&degree_6_chords);
        assert_eq!(classify_trajectory(&h), Trajectory::Traverse);
    }

    #[test]
    fn test_trajectory_mixed() {
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let find_chord_in_orbit = |target_orbit: Orbit| -> [u8; 4] {
            for pc in space.chords() {
                if classify_orbit(pc) == Some(target_orbit) {
                    return quintal_root(pc, 4).unwrap().pitches;
                }
            }
            panic!("no chord found in orbit {:?}", target_orbit);
        };

        // Degrees: 6, 4, 8 — not monotonic
        let h = make_harmonization(&[
            find_chord_in_orbit(Orbit::Q877), // degree 6
            find_chord_in_orbit(Orbit::Q867), // degree 4
            find_chord_in_orbit(Orbit::Q777), // degree 8
        ]);
        assert_eq!(classify_trajectory(&h), Trajectory::Mixed);
    }

    #[test]
    fn test_match_cadence() {
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let find_chord_in_orbit = |target_orbit: Orbit| -> [u8; 4] {
            for pc in space.chords() {
                if classify_orbit(pc) == Some(target_orbit) {
                    return quintal_root(pc, 4).unwrap().pitches;
                }
            }
            panic!("no chord found in orbit {:?}", target_orbit);
        };

        // Saddle (Q686) → Slope (Q786) → Summit (Q777)
        let h = make_harmonization(&[
            find_chord_in_orbit(Orbit::Q686), // Saddle
            find_chord_in_orbit(Orbit::Q786), // Slope
            find_chord_in_orbit(Orbit::Q777), // Summit
        ]);

        let matches = match_functional_pathways(&h);
        eprintln!("cadence matches: {:?}", matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pathway, Pathway::Cadence);
        assert_eq!(matches[0].start_position, 0);
        assert_eq!(matches[0].end_position, 2);
    }

    #[test]
    fn test_match_no_pathway() {
        use crate::quintal::{quintal_root, BaseSpace, classify_orbit, Orbit};
        let space = BaseSpace::new();

        let find_chord_in_orbit = |target_orbit: Orbit| -> [u8; 4] {
            for pc in space.chords() {
                if classify_orbit(pc) == Some(target_orbit) {
                    return quintal_root(pc, 4).unwrap().pitches;
                }
            }
            panic!("no chord found in orbit {:?}", target_orbit);
        };

        // Valley → Valley → Valley — no pathway match
        let h = make_harmonization(&[
            find_chord_in_orbit(Orbit::Q767),
            find_chord_in_orbit(Orbit::Q868),
            find_chord_in_orbit(Orbit::Q878),
        ]);

        let matches = match_functional_pathways(&h);
        assert!(matches.is_empty());
    }
}
