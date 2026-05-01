extern crate music_comp_mt as theory;

use theory::quintal::{
    all_distances_from, classify_orbit, count_geodesics, distance, geodesic_distribution,
    geodesics, is_adjacent, passing_chords, saddle_chords, transpose, BaseSpace, Orbit, PcChord,
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

    // ───────────────────────── geodesic_distribution ─────────────────────────

    /// §6.1 — bucket sizes sum to |B| − 1 = 227 for the connected base space.
    #[test]
    fn geodesic_distribution_sums_to_b_minus_1() {
        let space = BaseSpace::new();
        let source = PcChord::new([0, 2, 7, 9]).unwrap();
        let dist = geodesic_distribution(&space, &source).unwrap();
        let total: usize = dist.buckets.iter().map(|b| b.chords_at_d).sum();
        assert_eq!(total, 227);
        assert_eq!(dist.reachable_chords, 227);
    }

    /// §6.2 — the §6 paper table's "chords_at_d" column reproduces from the
    /// Summit (C-G-D-A). Pins the row at (8, 18, 36, 45, 66, 44, 10).
    #[test]
    fn cgda_chords_at_d_matches_section_6_table() {
        let space = BaseSpace::new();
        let source = PcChord::new([0, 2, 7, 9]).unwrap();
        let dist = geodesic_distribution(&space, &source).unwrap();
        let counts: Vec<usize> = dist.buckets.iter().map(|b| b.chords_at_d).collect();
        assert_eq!(counts, vec![8, 18, 36, 45, 66, 44, 10]);
    }

    /// §6.3 — σ counts on the per_chord vector match the slow per-pair
    /// `count_geodesics` for an arbitrary subset. Also pins the
    /// "source excluded from per_chord" invariant.
    #[test]
    fn distribution_geodesic_counts_match_pairwise() {
        let space = BaseSpace::new();
        let source = PcChord::new([0, 2, 7, 9]).unwrap();
        let dist = geodesic_distribution(&space, &source).unwrap();
        assert_eq!(
            dist.per_chord.len(),
            227,
            "per_chord should exclude the source"
        );
        assert!(
            dist.per_chord.iter().all(|e| e.distance >= 1),
            "no per_chord entry should have distance 0"
        );
        for entry in dist.per_chord.iter().take(20) {
            let pairwise = count_geodesics(&space, &source, &entry.chord) as u64;
            assert_eq!(pairwise, entry.geodesic_count, "chord {:?}", entry.chord);
        }
    }

    /// §3a — the corrected paper claim: max-σ at d=7 is exactly two chords,
    /// both in Q867, σ = 298. (The previously-claimed identity
    /// "A♭–E♭–B♭–F" was wrong on orbit AND identity; this test pins the
    /// real answer under regression protection.)
    #[test]
    fn cgda_max_sigma_at_d_7_lives_in_q867() {
        let space = BaseSpace::new();
        let source = PcChord::new([0, 2, 7, 9]).unwrap();
        let dist = geodesic_distribution(&space, &source).unwrap();
        let d7 = dist
            .buckets
            .iter()
            .find(|b| b.distance == 7)
            .expect("d=7 bucket exists");
        assert_eq!(d7.max_geodesics, 298);
        assert_eq!(
            d7.max_chords.len(),
            2,
            "expected exactly two max-σ chords at d=7"
        );
        for (_, orbit) in &d7.max_chords {
            assert_eq!(
                *orbit,
                Orbit::Q867,
                "max-σ chords at d=7 should both be in Q867"
            );
        }
    }

    /// §6.4 — distribution is invariant under transposition (Tk is an
    /// isometry of B). Compares aggregate (distance, chords_at_d, max σ)
    /// triples; chord identities translate, so we don't compare those.
    #[test]
    fn distribution_invariant_under_transposition() {
        let space = BaseSpace::new();
        let source = PcChord::new([0, 2, 7, 9]).unwrap();
        let baseline = geodesic_distribution(&space, &source).unwrap();
        for k in 1u8..12 {
            let shifted = transpose(&source, k);
            let candidate = geodesic_distribution(&space, &shifted).unwrap();
            let baseline_buckets: Vec<_> = baseline
                .buckets
                .iter()
                .map(|b| (b.distance, b.chords_at_d, b.max_geodesics))
                .collect();
            let candidate_buckets: Vec<_> = candidate
                .buckets
                .iter()
                .map(|b| (b.distance, b.chords_at_d, b.max_geodesics))
                .collect();
            assert_eq!(baseline_buckets, candidate_buckets, "T{} broke isometry", k);
        }
    }

    /// §6.5 — Saddle profile differs from Summit. We pull the Saddle
    /// representative from `saddle_chords` rather than hard-coding [0,2,6,8],
    /// and assert the picked chord is in Q686 so a future change to
    /// `saddle_chords` surfaces here, not as a silent test-strength regression.
    #[test]
    fn saddle_distribution_differs_from_summit() {
        let space = BaseSpace::new();
        let summit = PcChord::new([0, 2, 7, 9]).unwrap();
        let saddle = *saddle_chords(&space)
            .first()
            .expect("saddle_chords always returns 6 members");
        assert_eq!(
            classify_orbit(&saddle),
            Some(Orbit::Q686),
            "saddle_chords should pick from the Saddle (Q686) orbit",
        );
        let s = geodesic_distribution(&space, &summit).unwrap();
        let q = geodesic_distribution(&space, &saddle).unwrap();
        let s_counts: Vec<usize> = s.buckets.iter().map(|b| b.chords_at_d).collect();
        let q_counts: Vec<usize> = q.buckets.iter().map(|b| b.chords_at_d).collect();
        assert_ne!(s_counts, q_counts);
    }

    /// §6.6 — source not in BaseSpace returns None. PcChord::new always
    /// succeeds for distinct values 0..=11, but [0,1,2,3] has no [6,8]-legal
    /// interval ordering, so it is NOT in the base space.
    #[test]
    fn distribution_returns_none_for_chord_not_in_base_space() {
        let space = BaseSpace::new();
        let bogus = PcChord::new([0, 1, 2, 3]).expect("range/unique checks pass");
        assert!(space.chord_index(&bogus).is_none());
        assert!(geodesic_distribution(&space, &bogus).is_none());
    }
}
