extern crate music_comp_mt as theory;

use theory::quintal::{all_distances_from, classify_orbit, BaseSpace, Orbit, PcChord};

/// Mirror of the helper in `examples/distance_matrix.rs`.
fn pcs_to_str(pcs: &[u8; 4]) -> String {
    format!("{}-{}-{}-{}", pcs[0], pcs[1], pcs[2], pcs[3])
}

// ---------------------------------------------------------------------------
// pcs_to_str formatting
// ---------------------------------------------------------------------------

#[test]
fn test_pcs_to_str_basic() {
    assert_eq!(pcs_to_str(&[0, 2, 7, 9]), "0-2-7-9");
}

#[test]
fn test_pcs_to_str_high_values() {
    assert_eq!(pcs_to_str(&[3, 5, 8, 10]), "3-5-8-10");
}

#[test]
fn test_pcs_to_str_zeros() {
    assert_eq!(pcs_to_str(&[0, 0, 0, 0]), "0-0-0-0");
}

#[test]
fn test_pcs_to_str_boundary() {
    assert_eq!(pcs_to_str(&[0, 1, 11, 11]), "0-1-11-11");
}

#[test]
fn test_pcs_to_str_all_base_space_chords() {
    let space = BaseSpace::new();
    for chord in space.chords() {
        let s = pcs_to_str(&chord.pcs);
        let parts: Vec<&str> = s.split('-').collect();
        assert_eq!(
            parts.len(),
            4,
            "pcs_to_str should produce 4 dash-separated fields"
        );
        for part in &parts {
            let val: u8 = part.parse().expect("each field should be a valid u8");
            assert!(val < 12, "pitch class {} out of range", val);
        }
    }
}

// ---------------------------------------------------------------------------
// Orbit classification completeness
// ---------------------------------------------------------------------------

#[test]
fn test_every_base_space_chord_has_orbit() {
    let space = BaseSpace::new();
    for chord in space.chords() {
        assert!(
            classify_orbit(chord).is_some(),
            "chord {:?} has no orbit classification",
            chord.pcs
        );
    }
}

#[test]
fn test_all_14_orbits_represented() {
    let space = BaseSpace::new();
    let mut seen: std::collections::BTreeSet<Orbit> = std::collections::BTreeSet::new();
    for chord in space.chords() {
        if let Some(orbit) = classify_orbit(chord) {
            seen.insert(orbit);
        }
    }
    assert_eq!(
        seen.len(),
        14,
        "all 14 orbits should appear in the base space"
    );
    for orbit in Orbit::all() {
        assert!(
            seen.contains(orbit),
            "orbit {:?} missing from base space",
            orbit
        );
    }
}

// ---------------------------------------------------------------------------
// Distance matrix dimensions & structure
// ---------------------------------------------------------------------------

#[test]
fn test_base_space_has_228_chords() {
    let space = BaseSpace::new();
    assert_eq!(space.chords().len(), 228);
}

#[test]
fn test_single_source_row_count() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let distances = all_distances_from(&space, source);
    let non_self_count = distances.keys().filter(|t| *t != source).count();
    assert_eq!(
        non_self_count, 227,
        "each source should reach 227 other chords"
    );
}

#[test]
fn test_full_matrix_row_count() {
    let space = BaseSpace::new();
    let chords = space.chords();
    let mut total_rows = 0usize;
    for source in chords {
        let distances = all_distances_from(&space, source);
        total_rows += distances.keys().filter(|t| *t != source).count();
    }
    assert_eq!(
        total_rows,
        228 * 227,
        "total distance matrix rows: 228 sources × 227 targets"
    );
}

// ---------------------------------------------------------------------------
// Distance value properties
// ---------------------------------------------------------------------------

#[test]
fn test_all_non_self_distances_positive() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let distances = all_distances_from(&space, source);
    for (target, &d) in &distances {
        if target != source {
            assert!(
                d > 0,
                "non-self distance must be > 0, got 0 for {:?}",
                target.pcs
            );
        }
    }
}

#[test]
fn test_self_distance_zero() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let distances = all_distances_from(&space, source);
    assert_eq!(
        distances[source], 0,
        "distance from a chord to itself must be 0"
    );
}

#[test]
fn test_distances_bounded_by_diameter() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let distances = all_distances_from(&space, source);
    for &d in distances.values() {
        assert!(d <= 8, "distance {} exceeds known diameter 8", d);
    }
}

// ---------------------------------------------------------------------------
// Symmetry
// ---------------------------------------------------------------------------

#[test]
fn test_distance_matrix_symmetry_sampled() {
    let space = BaseSpace::new();
    let chords = space.chords();
    for i in (0..chords.len()).step_by(15) {
        let dists_i = all_distances_from(&space, &chords[i]);
        for j in (i + 1..chords.len()).step_by(15) {
            let dists_j = all_distances_from(&space, &chords[j]);
            assert_eq!(
                dists_i[&chords[j]], dists_j[&chords[i]],
                "distance asymmetry between {:?} and {:?}",
                chords[i].pcs, chords[j].pcs
            );
        }
    }
}

// ---------------------------------------------------------------------------
// CSV output simulation (mirrors the example's main loop)
// ---------------------------------------------------------------------------

#[test]
fn test_csv_header() {
    let header = "source_pcs,source_orbit,target_pcs,target_orbit,distance";
    let fields: Vec<&str> = header.split(',').collect();
    assert_eq!(
        fields,
        vec![
            "source_pcs",
            "source_orbit",
            "target_pcs",
            "target_orbit",
            "distance"
        ]
    );
}

#[test]
fn test_csv_row_format() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let source_orbit = classify_orbit(source).unwrap();
    let distances = all_distances_from(&space, source);

    for target in space.chords() {
        if target == source {
            continue;
        }
        let d = distances[target];
        let target_orbit = classify_orbit(target).unwrap();
        let row = format!(
            "{},{:?},{},{:?},{}",
            pcs_to_str(&source.pcs),
            source_orbit,
            pcs_to_str(&target.pcs),
            target_orbit,
            d
        );
        let fields: Vec<&str> = row.split(',').collect();
        assert_eq!(fields.len(), 5, "CSV row should have 5 fields: {}", row);

        // source_pcs format
        assert!(
            fields[0].split('-').count() == 4,
            "source_pcs should have 4 dash-separated parts"
        );
        // source_orbit starts with Q
        assert!(fields[1].starts_with('Q'), "orbit should start with Q");
        // target_pcs format
        assert!(
            fields[2].split('-').count() == 4,
            "target_pcs should have 4 dash-separated parts"
        );
        // target_orbit starts with Q
        assert!(fields[3].starts_with('Q'), "orbit should start with Q");
        // distance is parseable
        let _: u8 = fields[4].parse().expect("distance should be a valid u8");
    }
}

#[test]
fn test_orbit_debug_format_has_no_commas() {
    for orbit in Orbit::all() {
        let debug_str = format!("{:?}", orbit);
        assert!(
            !debug_str.contains(','),
            "orbit Debug format must not contain commas (CSV safety): {:?}",
            debug_str
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn test_output_deterministic() {
    let space = BaseSpace::new();
    let chords = space.chords();

    let generate_rows = || -> Vec<String> {
        let mut rows = Vec::new();
        for source in chords {
            let source_orbit = classify_orbit(source).unwrap();
            let distances = all_distances_from(&space, source);
            for target in chords {
                if target == source {
                    continue;
                }
                let d = distances[target];
                let target_orbit = classify_orbit(target).unwrap();
                rows.push(format!(
                    "{},{:?},{},{:?},{}",
                    pcs_to_str(&source.pcs),
                    source_orbit,
                    pcs_to_str(&target.pcs),
                    target_orbit,
                    d
                ));
            }
        }
        rows
    };

    let run1 = generate_rows();
    let run2 = generate_rows();
    assert_eq!(run1.len(), run2.len());
    for (i, (a, b)) in run1.iter().zip(run2.iter()).enumerate() {
        assert_eq!(a, b, "row {} differs between runs", i);
    }
}

// ---------------------------------------------------------------------------
// Spot-check known values
// ---------------------------------------------------------------------------

#[test]
fn test_known_distance_cgda_to_adjacent() {
    let space = BaseSpace::new();
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    let adjacent = PcChord::new([0, 2, 6, 9]).unwrap();
    let distances = all_distances_from(&space, &cgda);
    assert_eq!(
        distances[&adjacent], 1,
        "C-G-D-A to [0,2,6,9] should be distance 1"
    );
}

#[test]
fn test_known_orbit_cgda() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(classify_orbit(&cgda), Some(Orbit::Q777));
}

#[test]
fn test_known_orbit_debug_format() {
    assert_eq!(format!("{:?}", Orbit::Q777), "Q777");
    assert_eq!(format!("{:?}", Orbit::Q686), "Q686");
    assert_eq!(format!("{:?}", Orbit::Q867), "Q867");
}

#[test]
fn test_pcs_sorted_ascending_in_base_space() {
    let space = BaseSpace::new();
    for chord in space.chords() {
        for i in 0..3 {
            assert!(
                chord.pcs[i] <= chord.pcs[i + 1],
                "PcChord pcs should be sorted ascending: {:?}",
                chord.pcs
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Self-exclusion (the `if target == source { continue }` branch)
// ---------------------------------------------------------------------------

#[test]
fn test_self_pairs_excluded() {
    let space = BaseSpace::new();
    let chords = space.chords();
    for source in chords {
        let distances = all_distances_from(&space, source);
        for target in chords {
            if target == source {
                assert_eq!(
                    distances[target], 0,
                    "self-distance should be 0; the example skips this row"
                );
            } else {
                assert!(distances[target] > 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Distance distribution per source (sanity check)
// ---------------------------------------------------------------------------

#[test]
fn test_distance_distribution_nonzero_counts() {
    let space = BaseSpace::new();
    let source = &space.chords()[0];
    let distances = all_distances_from(&space, source);
    let mut by_distance: std::collections::BTreeMap<u8, usize> = std::collections::BTreeMap::new();
    for (&chord, &d) in &distances {
        if chord != *source {
            *by_distance.entry(d).or_insert(0) += 1;
        }
    }
    let total: usize = by_distance.values().sum();
    assert_eq!(total, 227, "should have exactly 227 non-self entries");
    for (&d, &count) in &by_distance {
        assert!((1..=8).contains(&d), "distance {} out of expected range", d);
        assert!(count > 0, "distance {} bucket should be non-empty", d);
    }
}
