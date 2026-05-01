extern crate music_comp_mt as theory;

use theory::quintal::{enumerate_all, is_adjacent, BaseSpace, PcChord};

#[cfg(test)]
mod base_space_tests {
    use super::*;
    use std::collections::BTreeMap;

    // --- enumerate_all ---

    #[test]
    fn test_enumerate_all_count() {
        assert_eq!(enumerate_all().len(), 228);
    }

    #[test]
    fn test_enumerate_all_unique() {
        let chords = enumerate_all();
        let mut seen = std::collections::HashSet::new();
        for c in &chords {
            assert!(seen.insert(c), "Duplicate chord: {:?}", c);
        }
    }

    #[test]
    fn test_enumerate_all_all_legal() {
        for c in enumerate_all() {
            assert!(c.is_legal(), "Chord {:?} is not legal", c);
        }
    }

    #[test]
    fn test_enumerate_all_sorted() {
        let chords = enumerate_all();
        for i in 1..chords.len() {
            assert!(
                chords[i - 1] < chords[i],
                "Chords not sorted at index {}: {:?} >= {:?}",
                i,
                chords[i - 1],
                chords[i]
            );
        }
    }

    #[test]
    fn test_known_chord_present() {
        let chords = enumerate_all();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        assert!(chords.contains(&cgda));
    }

    // --- is_adjacent ---

    #[test]
    fn test_adjacency_single_voice() {
        // C-G-D-A [0,2,7,9] -> C-F#-D-A [0,2,6,9] (G=7 -> F#=6)
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert!(is_adjacent(&a, &b));
    }

    #[test]
    fn test_adjacency_symmetric() {
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert_eq!(is_adjacent(&a, &b), is_adjacent(&b, &a));
    }

    #[test]
    fn test_not_adjacent_same() {
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        assert!(!is_adjacent(&a, &a));
    }

    #[test]
    fn test_not_adjacent_two_voices() {
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 3, 8, 9]).unwrap();
        // Two voices differ: 2->3 and 7->8.
        // Even though each differs by 1, TWO voices changed.
        if b.is_legal() {
            assert!(!is_adjacent(&a, &b));
        }
    }

    #[test]
    fn test_not_adjacent_disjoint() {
        let a = PcChord::new([0, 1, 6, 7]).unwrap();
        let b = PcChord::new([2, 3, 8, 9]).unwrap();
        if a.is_legal() && b.is_legal() {
            assert!(!is_adjacent(&a, &b));
        }
    }

    #[test]
    fn test_adjacency_mod12_wrap() {
        // Test that adjacency wraps around the 0/11 boundary.
        let chords = enumerate_all();
        let has_wrap = chords.iter().any(|a| {
            a.pcs.contains(&0)
                && chords
                    .iter()
                    .any(|b| b.pcs.contains(&11) && is_adjacent(a, b))
        });
        assert!(
            has_wrap,
            "Expected at least one adjacency across the 0/11 boundary"
        );
    }

    #[test]
    fn test_known_adjacency() {
        // From the paper: C-G-D-A is adjacent to C-F#-D-A.
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert!(is_adjacent(&a, &b));
    }

    // --- BaseSpace ---

    #[test]
    fn test_base_space_size() {
        let space = BaseSpace::new();
        assert_eq!(space.len(), 228);
    }

    #[test]
    fn test_base_space_not_empty() {
        let space = BaseSpace::new();
        assert!(!space.is_empty());
    }

    #[test]
    fn test_chord_index_round_trip() {
        let space = BaseSpace::new();
        for (i, chord) in space.chords().iter().enumerate() {
            assert_eq!(space.chord_index(chord), Some(i));
        }
    }

    #[test]
    fn test_chord_index_missing() {
        let space = BaseSpace::new();
        // This chord is illegal so it should not be in the space.
        let illegal = PcChord::new([0, 1, 2, 3]).unwrap();
        assert_eq!(space.chord_index(&illegal), None);
    }

    #[test]
    fn test_neighbors_returns_none_for_missing() {
        let space = BaseSpace::new();
        let illegal = PcChord::new([0, 1, 2, 3]).unwrap();
        assert!(space.neighbors(&illegal).is_none());
    }

    #[test]
    fn test_degree_distribution() {
        let space = BaseSpace::new();
        let dist = space.degree_distribution();
        let mut expected = BTreeMap::new();
        expected.insert(4, 90);
        expected.insert(5, 48);
        expected.insert(6, 60);
        expected.insert(8, 30);
        assert_eq!(dist, expected);
    }

    #[test]
    fn test_is_connected() {
        let space = BaseSpace::new();
        assert!(space.is_connected());
    }

    #[test]
    fn test_cgda_degree() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(space.degree(&cgda), Some(8));
    }

    #[test]
    fn test_saddle_degree() {
        let space = BaseSpace::new();
        let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
        assert_eq!(space.degree(&saddle), Some(8));
    }

    #[test]
    fn test_total_degree_sum_even() {
        let space = BaseSpace::new();
        let total: usize = space
            .chords()
            .iter()
            .map(|c| space.degree(c).unwrap())
            .sum();
        assert_eq!(total % 2, 0);
        // 2 * edges = total degree sum
        assert_eq!(total, 1200);
    }

    #[test]
    fn test_degree_returns_none_for_missing() {
        let space = BaseSpace::new();
        let illegal = PcChord::new([0, 1, 2, 3]).unwrap();
        assert!(space.degree(&illegal).is_none());
    }

    #[test]
    fn test_adjacency_consistency() {
        // If i is in adjacency[j], then j must be in adjacency[i].
        let space = BaseSpace::new();
        let chords = space.chords();
        for (i, chord) in chords.iter().enumerate() {
            if let Some(neighbors) = space.neighbors(chord) {
                for &j in neighbors {
                    let j_neighbors = space.neighbors(&chords[j]).unwrap();
                    assert!(
                        j_neighbors.contains(&i),
                        "Asymmetric adjacency: {} -> {} but not {} -> {}",
                        i,
                        j,
                        j,
                        i
                    );
                }
            }
        }
    }
}
