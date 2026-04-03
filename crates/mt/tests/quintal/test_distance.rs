extern crate music_comp_mt as theory;

use theory::quintal::{
    all_distances_from, center, diameter, distance, eccentricity, BaseSpace, PcChord,
};

#[cfg(test)]
mod distance_tests {
    use super::*;
    use std::collections::BTreeMap;

    // --- distance ---

    #[test]
    fn test_distance_self() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(distance(&space, &cgda, &cgda), Some(0));
    }

    #[test]
    fn test_distance_adjacent() {
        let space = BaseSpace::new();
        // [0,2,7,9] and [0,2,6,9] differ only in 7->6 (one semitone), so adjacent.
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert_eq!(distance(&space, &a, &b), Some(1));
    }

    #[test]
    fn test_distance_symmetric() {
        let space = BaseSpace::new();
        let chords = space.chords();
        // Sample several pairs for symmetry.
        for i in (0..chords.len()).step_by(20) {
            for j in (i + 1..chords.len()).step_by(20) {
                let d1 = distance(&space, &chords[i], &chords[j]);
                let d2 = distance(&space, &chords[j], &chords[i]);
                assert_eq!(
                    d1, d2,
                    "Distance not symmetric for {:?} and {:?}",
                    chords[i], chords[j]
                );
            }
        }
    }

    #[test]
    fn test_distance_triangle_inequality() {
        let space = BaseSpace::new();
        let chords = space.chords();
        // Sample triples to verify the triangle inequality.
        for i in (0..chords.len()).step_by(30) {
            for j in (0..chords.len()).step_by(30) {
                for k in (0..chords.len()).step_by(30) {
                    let dij = distance(&space, &chords[i], &chords[j]).unwrap();
                    let djk = distance(&space, &chords[j], &chords[k]).unwrap();
                    let dik = distance(&space, &chords[i], &chords[k]).unwrap();
                    assert!(
                        dik <= dij + djk,
                        "Triangle inequality violated: d({:?},{:?})={} > d({:?},{:?})={} + d({:?},{:?})={}",
                        chords[i], chords[k], dik,
                        chords[i], chords[j], dij,
                        chords[j], chords[k], djk
                    );
                }
            }
        }
    }

    #[test]
    fn test_distance_missing_chord() {
        let space = BaseSpace::new();
        // [0,1,2,3] is not a legal quintal chord, so not in the space.
        let missing = PcChord::new([0, 1, 2, 3]).unwrap();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(distance(&space, &missing, &cgda), None);
        assert_eq!(distance(&space, &cgda, &missing), None);
    }

    // --- all_distances_from ---

    #[test]
    fn test_all_distances_from_covers_all_chords() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);
        assert_eq!(
            dists.len(),
            228,
            "All 228 chords should be reachable (connected graph)"
        );
    }

    #[test]
    fn test_all_distances_from_includes_self() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);
        assert_eq!(dists[&cgda], 0);
    }

    #[test]
    fn test_all_distances_from_distribution() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);

        // Count chords at each distance.
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for &d in dists.values() {
            *counts.entry(d).or_insert(0) += 1;
        }

        // Exact values from paper section 6.
        assert_eq!(counts[&0], 1);
        assert_eq!(counts[&1], 8);
        assert_eq!(counts[&2], 18);
        assert_eq!(counts[&3], 36);
        assert_eq!(counts[&4], 45);
        assert_eq!(counts[&5], 66);
        assert_eq!(counts[&6], 44);
        assert_eq!(counts[&7], 10);
        let total: usize = counts.values().sum();
        assert_eq!(total, 228);
    }

    #[test]
    fn test_distance_antipodal() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        // Ab-Eb-Bb-F = {8,3,10,5} = [3,5,8,10] is at distance 6 from C-G-D-A.
        // The paper's "distance 7" antipodal chords are the 10 chords at max
        // distance from C-G-D-A; [3,5,8,10] happens to be at distance 6.
        let ab_eb_bb_f = PcChord::new([3, 5, 8, 10]).unwrap();
        assert_eq!(distance(&space, &cgda, &ab_eb_bb_f), Some(6));
    }

    #[test]
    fn test_diameter_exact() {
        let space = BaseSpace::new();
        assert_eq!(diameter(&space), 8);
    }

    #[test]
    fn test_eccentricity_range_exact() {
        let space = BaseSpace::new();
        for chord in space.chords() {
            let ecc = eccentricity(&space, chord).unwrap();
            assert!(
                ecc == 7 || ecc == 8,
                "Eccentricity of {:?} is {}, expected 7 or 8",
                chord,
                ecc
            );
        }
    }

    #[test]
    fn test_center_size_exact() {
        let space = BaseSpace::new();
        assert_eq!(center(&space).len(), 54);
    }

    #[test]
    fn test_cgda_in_center() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        assert!(center(&space).contains(&cgda));
    }

    #[test]
    fn test_all_distances_from_missing_chord() {
        let space = BaseSpace::new();
        let missing = PcChord::new([0, 1, 2, 3]).unwrap();
        let dists = all_distances_from(&space, &missing);
        assert!(dists.is_empty());
    }

    // --- eccentricity ---

    #[test]
    fn test_eccentricity_self_distance_zero() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let ecc = eccentricity(&space, &cgda).unwrap();
        // Eccentricity must be at least 1 for a non-trivial graph.
        assert!(ecc >= 1);
    }

    #[test]
    fn test_eccentricity_missing_chord() {
        let space = BaseSpace::new();
        let missing = PcChord::new([0, 1, 2, 3]).unwrap();
        assert_eq!(eccentricity(&space, &missing), None);
    }

    #[test]
    fn test_eccentricity_bounded_by_diameter() {
        let space = BaseSpace::new();
        let d = diameter(&space);
        for chord in space.chords() {
            let ecc = eccentricity(&space, chord).unwrap();
            assert!(
                ecc <= d,
                "Eccentricity {} of {:?} exceeds diameter {}",
                ecc,
                chord,
                d
            );
        }
    }

    // --- diameter ---

    #[test]
    fn test_diameter_positive() {
        let space = BaseSpace::new();
        let d = diameter(&space);
        assert!(d > 0, "Diameter of non-trivial graph must be positive");
    }

    #[test]
    fn test_diameter_is_max_eccentricity() {
        let space = BaseSpace::new();
        let d = diameter(&space);
        let max_ecc = space
            .chords()
            .iter()
            .map(|c| eccentricity(&space, c).unwrap())
            .max()
            .unwrap();
        assert_eq!(d, max_ecc);
    }

    // --- center ---

    #[test]
    fn test_center_nonempty() {
        let space = BaseSpace::new();
        let c = center(&space);
        assert!(!c.is_empty(), "Center must be non-empty");
    }

    #[test]
    fn test_center_eccentricity_is_minimal() {
        let space = BaseSpace::new();
        let c = center(&space);
        let center_ecc = eccentricity(&space, &c[0]).unwrap();

        // All center chords have the same eccentricity.
        for chord in &c {
            assert_eq!(eccentricity(&space, chord), Some(center_ecc));
        }

        // No chord outside the center has a smaller eccentricity.
        for chord in space.chords() {
            let ecc = eccentricity(&space, chord).unwrap();
            assert!(
                ecc >= center_ecc,
                "Chord {:?} has eccentricity {} < center eccentricity {}",
                chord,
                ecc,
                center_ecc
            );
        }
    }

    #[test]
    fn test_center_contains_all_min_ecc_chords() {
        let space = BaseSpace::new();
        let c = center(&space);
        let center_ecc = eccentricity(&space, &c[0]).unwrap();

        // Every chord with minimum eccentricity must be in the center.
        for chord in space.chords() {
            if eccentricity(&space, chord) == Some(center_ecc) {
                assert!(
                    c.contains(chord),
                    "Chord {:?} has min eccentricity but is not in center",
                    chord
                );
            }
        }
    }
}
