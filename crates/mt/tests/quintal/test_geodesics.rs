extern crate music_comp_mt as theory;

use theory::quintal::{
    all_distances_from, count_geodesics, distance, geodesics, is_adjacent, passing_chords,
    BaseSpace, PcChord,
};

#[cfg(test)]
mod geodesics_tests {
    use super::*;

    // --- geodesics ---

    #[test]
    fn test_geodesics_same_chord() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let paths = geodesics(&space, &a, &a);
        assert_eq!(paths, vec![vec![a]]);
    }

    #[test]
    fn test_geodesics_adjacent() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert_eq!(distance(&space, &a, &b), Some(1));
        let paths = geodesics(&space, &a, &b);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 2);
        assert_eq!(paths[0][0], a);
        assert_eq!(paths[0][1], b);
    }

    #[test]
    fn test_geodesics_distance_1_all() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);
        let neighbors: Vec<_> = dists
            .iter()
            .filter(|(_, &d)| d == 1)
            .map(|(c, _)| *c)
            .collect();
        assert_eq!(neighbors.len(), 8);
        for nb in &neighbors {
            let paths = geodesics(&space, &cgda, nb);
            assert_eq!(
                paths.len(),
                1,
                "Expected exactly 1 geodesic to neighbor {:?}, got {}",
                nb,
                paths.len()
            );
            assert_eq!(paths[0].len(), 2);
        }
    }

    #[test]
    fn test_geodesics_path_lengths() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let chords = space.chords();
        for &b in chords.iter().step_by(25) {
            let d = distance(&space, &a, &b).unwrap();
            let paths = geodesics(&space, &a, &b);
            for path in &paths {
                assert_eq!(
                    path.len() as u8,
                    d + 1,
                    "Path length {} != d+1={} for {:?} -> {:?}",
                    path.len(),
                    d + 1,
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn test_geodesics_path_adjacency() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let chords = space.chords();
        for &b in chords.iter().step_by(40) {
            let paths = geodesics(&space, &a, &b);
            for path in &paths {
                for w in path.windows(2) {
                    assert!(
                        is_adjacent(&w[0], &w[1]),
                        "Consecutive chords {:?} and {:?} are not adjacent",
                        w[0],
                        w[1]
                    );
                }
            }
        }
    }

    // --- count_geodesics ---

    #[test]
    fn test_count_geodesics_same() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(count_geodesics(&space, &a, &a), 1);
    }

    #[test]
    fn test_count_matches_len() {
        let space = BaseSpace::new();
        let chords = space.chords();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        for &b in chords.iter().step_by(25) {
            let count = count_geodesics(&space, &a, &b);
            let paths = geodesics(&space, &a, &b);
            assert_eq!(
                count,
                paths.len(),
                "count_geodesics={} != geodesics().len()={} for {:?} -> {:?}",
                count,
                paths.len(),
                a,
                b
            );
        }
    }

    #[test]
    fn test_geodesic_max_at_distance_7() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);
        let max_count = dists
            .iter()
            .filter(|(_, &d)| d == 7)
            .map(|(c, _)| count_geodesics(&space, &cgda, c))
            .max()
            .unwrap();
        assert_eq!(
            max_count, 298,
            "Max geodesic count at distance 7 from C-G-D-A should be 298"
        );
    }

    #[test]
    fn test_geodesic_max_at_distance_6() {
        let space = BaseSpace::new();
        let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
        let dists = all_distances_from(&space, &cgda);
        let max_count = dists
            .iter()
            .filter(|(_, &d)| d == 6)
            .map(|(c, _)| count_geodesics(&space, &cgda, c))
            .max()
            .unwrap();
        assert_eq!(
            max_count, 176,
            "Max geodesic count at distance 6 from C-G-D-A should be 176"
        );
    }

    // --- passing_chords ---

    #[test]
    fn test_passing_chords_adjacent() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let b = PcChord::new([0, 2, 6, 9]).unwrap();
        assert_eq!(distance(&space, &a, &b), Some(1));
        assert!(passing_chords(&space, &a, &b).is_empty());
    }

    #[test]
    fn test_passing_chords_metric() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let chords = space.chords();
        for &b in chords.iter().step_by(30) {
            let d_ab = distance(&space, &a, &b).unwrap();
            let passing = passing_chords(&space, &a, &b);
            for z in &passing {
                let da = distance(&space, &a, z).unwrap();
                let db = distance(&space, z, &b).unwrap();
                assert_eq!(
                    da + db,
                    d_ab,
                    "Passing chord {:?}: d(a,z)={} + d(z,b)={} != d(a,b)={}",
                    z,
                    da,
                    db,
                    d_ab
                );
            }
        }
    }

    #[test]
    fn test_passing_chords_exclude_endpoints() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        let chords = space.chords();
        for &b in chords.iter().step_by(30) {
            let passing = passing_chords(&space, &a, &b);
            assert!(
                !passing.contains(&a),
                "Passing chords should not contain endpoint a"
            );
            assert!(
                !passing.contains(&b),
                "Passing chords should not contain endpoint b"
            );
        }
    }

    #[test]
    fn test_passing_chords_same() {
        let space = BaseSpace::new();
        let a = PcChord::new([0, 2, 7, 9]).unwrap();
        assert!(passing_chords(&space, &a, &a).is_empty());
    }
}
