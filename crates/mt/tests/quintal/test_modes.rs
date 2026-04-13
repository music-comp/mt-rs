extern crate music_comp_mt as theory;

use theory::quintal::{
    all_modes, modes_by_opening_interval, modes_in_cluster, orbit_modes, orbit_step_sequence,
    parent_scales, step_size_multiset, step_vocabulary_cluster, verify_fiber_mode_connection,
    verify_multiset_uniqueness, Orbit, OthMode, StepVocabularyCluster,
};
use theory::scale::ScaleType;

// ─── OthMode construction and accessors ─────────────────────────────────

#[test]
fn test_oth_mode_new_and_accessors() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    assert_eq!(mode.orbit(), Orbit::Q777);
    assert_eq!(mode.rotation(), 0);
    assert_eq!(mode.steps(), [2, 5, 2, 3]);
    assert_eq!(mode.pcs_from_c(), [0, 2, 7, 9]);
    assert_eq!(
        mode.opening_interval(),
        2,
        "opening_interval should be steps[0]"
    );
}

#[test]
fn test_oth_mode_opening_interval_derived_from_steps() {
    let mode = OthMode::new(Orbit::Q686, 1, [4, 2, 4, 2], [0, 4, 6, 10]);
    assert_eq!(mode.opening_interval(), 4);
}

#[test]
fn test_oth_mode_is_copy() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let copy = mode;
    assert_eq!(mode, copy);
}

// ─── StepVocabularyCluster ───────────────────────────────────────────────

#[test]
fn test_step_vocabulary_cluster_display() {
    assert_eq!(
        format!("{}", StepVocabularyCluster::NoSemitoneNoTritone),
        "No Semitone, No Tritone"
    );
    assert_eq!(
        format!("{}", StepVocabularyCluster::ContainsSemitone),
        "Contains Semitone"
    );
    assert_eq!(
        format!("{}", StepVocabularyCluster::EvenStepsOnly),
        "Even Steps Only"
    );
    assert_eq!(
        format!("{}", StepVocabularyCluster::ContainsTritoneStep),
        "Contains Tritone Step"
    );
}

#[test]
fn test_step_vocabulary_cluster_is_copy_and_ord() {
    let a = StepVocabularyCluster::NoSemitoneNoTritone;
    let b = a;
    assert_eq!(a, b);
    let mut v = vec![
        StepVocabularyCluster::ContainsTritoneStep,
        StepVocabularyCluster::NoSemitoneNoTritone,
    ];
    v.sort();
    assert_eq!(v[0], StepVocabularyCluster::NoSemitoneNoTritone);
}

// ─── orbit_step_sequence ─────────────────────────────────────────────────

#[test]
fn test_orbit_step_sequence_summit() {
    assert_eq!(orbit_step_sequence(&Orbit::Q777), [2, 5, 2, 3]);
}

#[test]
fn test_orbit_step_sequence_crossroads() {
    assert_eq!(orbit_step_sequence(&Orbit::Q686), [2, 4, 2, 4]);
}

#[test]
fn test_orbit_step_sequence_narrows() {
    assert_eq!(orbit_step_sequence(&Orbit::Q676), [1, 5, 1, 5]);
}

#[test]
fn test_all_orbit_step_sequences_sum_to_12() {
    for orbit in Orbit::all() {
        let steps = orbit_step_sequence(orbit);
        let sum: u8 = steps.iter().sum();
        assert_eq!(
            sum, 12,
            "steps {:?} for {} sum to {} instead of 12",
            steps, orbit, sum
        );
    }
}

// ─── step_size_multiset ─────────────────────────────────────────────────

#[test]
fn test_step_size_multiset_sorts() {
    assert_eq!(step_size_multiset(&[2, 5, 2, 3]), [2, 2, 3, 5]);
}

#[test]
fn test_step_size_multiset_symmetric() {
    assert_eq!(step_size_multiset(&[2, 4, 2, 4]), [2, 2, 4, 4]);
}

// ─── orbit_modes ────────────────────────────────────────────────────────

#[test]
fn test_orbit_modes_summit_has_4_modes() {
    let om = orbit_modes(&Orbit::Q777);
    assert_eq!(om.distinct_count(), 4);
    assert_eq!(om.modes().len(), 4);
}

#[test]
fn test_orbit_modes_crossroads_has_2_modes() {
    let om = orbit_modes(&Orbit::Q686);
    assert_eq!(om.distinct_count(), 2);
}

#[test]
fn test_orbit_modes_narrows_has_2_modes() {
    let om = orbit_modes(&Orbit::Q676);
    assert_eq!(om.distinct_count(), 2);
}

#[test]
fn test_orbit_modes_summit_first_mode_steps() {
    let om = orbit_modes(&Orbit::Q777);
    let m1 = &om.modes()[0];
    assert_eq!(m1.steps(), [2, 5, 2, 3]);
    assert_eq!(m1.pcs_from_c(), [0, 2, 7, 9]);
    assert_eq!(m1.rotation(), 0);
}

#[test]
fn test_all_modes_pcs_start_with_zero() {
    for orbit in Orbit::all() {
        let om = orbit_modes(orbit);
        for mode in om.modes() {
            assert_eq!(
                mode.pcs_from_c()[0],
                0,
                "{} rotation {} pcs_from_c should start with 0",
                orbit,
                mode.rotation()
            );
        }
    }
}

#[test]
fn test_all_mode_steps_sum_to_12() {
    for orbit in Orbit::all() {
        let om = orbit_modes(orbit);
        for mode in om.modes() {
            let sum: u8 = mode.steps().iter().sum();
            assert_eq!(
                sum,
                12,
                "{} mode {} steps {:?} sum to {}",
                orbit,
                mode.rotation(),
                mode.steps(),
                sum
            );
        }
    }
}

#[test]
fn test_total_distinct_modes() {
    let mut total: u32 = 0;
    for o in Orbit::all() {
        let om = orbit_modes(o);
        let n = om.distinct_count() as u32;
        if n != 4 {
            eprintln!("{}: {} modes, steps={:?}", o, n, orbit_step_sequence(o));
        }
        total += n;
    }
    // 2 T₆-symmetric orbits (Q686, Q676) have 2 modes each.
    // All others have 4 modes. 12×4 + 2×2 = 52.
    assert_eq!(total, 52);
}

#[test]
fn test_orbit_modes_forte_number_summit() {
    let om = orbit_modes(&Orbit::Q777);
    // Summit {0,2,7,9} has Forte number 4-23
    assert_eq!(om.forte_number(), Some("4-23".to_string()));
}

// ─── step_vocabulary_cluster ─────────────────────────────────────────────

#[test]
fn test_cluster_crossroads_even_steps() {
    assert_eq!(
        step_vocabulary_cluster(&Orbit::Q686),
        StepVocabularyCluster::EvenStepsOnly
    );
}

#[test]
fn test_cluster_narrows_contains_semitone() {
    // Q676 (Narrows): step sequence [1,5,1,5], multiset {1,1,5,5}.
    // Contains semitone (1), no tritone step (6) in the *step* sequence.
    // NOTE: The stacking interval contains 6 (diminished 5th), but the
    // cluster classification is based on step-size vocabulary, not stacking intervals.
    assert_eq!(
        step_vocabulary_cluster(&Orbit::Q676),
        StepVocabularyCluster::ContainsSemitone
    );
}

#[test]
fn test_cluster_summit_no_semitone_no_tritone() {
    assert_eq!(
        step_vocabulary_cluster(&Orbit::Q777),
        StepVocabularyCluster::NoSemitoneNoTritone
    );
}

#[test]
fn test_cluster_q767_contains_tritone_step() {
    // Q767: step sequence has a 6 in it (from the stacking interval pattern)
    let steps = orbit_step_sequence(&Orbit::Q767);
    let multiset = step_size_multiset(&steps);
    assert!(
        multiset.contains(&6),
        "Q767 step multiset {:?} should contain 6",
        multiset
    );
    assert_eq!(
        step_vocabulary_cluster(&Orbit::Q767),
        StepVocabularyCluster::ContainsTritoneStep
    );
}

#[test]
fn test_cluster_counts_per_category() {
    use std::collections::HashMap;
    let mut counts: HashMap<StepVocabularyCluster, usize> = HashMap::new();
    for orbit in Orbit::all() {
        *counts.entry(step_vocabulary_cluster(orbit)).or_insert(0) += 1;
    }
    // Verify all 14 orbits are classified (no panics, no None)
    let total: usize = counts.values().sum();
    assert_eq!(total, 14);
    // Each category should have at least 1 orbit
    assert!(counts.len() >= 2, "Should have multiple categories");
}

#[test]
fn test_cluster_matches_orbit_modes_step_cluster() {
    for orbit in Orbit::all() {
        let cluster = step_vocabulary_cluster(orbit);
        let om = orbit_modes(orbit);
        assert_eq!(
            cluster,
            om.step_cluster(),
            "step_vocabulary_cluster and orbit_modes.step_cluster() should agree for {}",
            orbit
        );
    }
}

// ─── all_modes, modes_by_opening_interval, modes_in_cluster ─────────────

#[test]
fn test_all_modes_returns_14_orbits() {
    assert_eq!(all_modes().len(), 14);
}

#[test]
fn test_all_modes_52_total() {
    let total: usize = all_modes().iter().map(|om| om.modes().len()).sum();
    assert_eq!(total, 52);
}

#[test]
fn test_modes_by_opening_interval_all_match() {
    let semitone_modes = modes_by_opening_interval(1);
    for mode in &semitone_modes {
        assert_eq!(mode.opening_interval(), 1);
    }
    assert!(
        !semitone_modes.is_empty(),
        "should have modes with opening interval 1"
    );
}

#[test]
fn test_modes_in_cluster_even_steps_only() {
    let even = modes_in_cluster(StepVocabularyCluster::EvenStepsOnly);
    for om in &even {
        assert_eq!(om.step_cluster(), StepVocabularyCluster::EvenStepsOnly);
    }
    assert!(!even.is_empty());
}

// ─── verification functions ──────────────────────────────────────────────

#[test]
fn test_multiset_collisions_exist() {
    // Some orbits share step-size multisets. This is a mathematical fact.
    // Q777 and Q877 both have multiset [2,2,3,5].
    let q777_ms = step_size_multiset(&orbit_step_sequence(&Orbit::Q777));
    let q877_ms = step_size_multiset(&orbit_step_sequence(&Orbit::Q877));
    assert_eq!(q777_ms, q877_ms, "Q777 and Q877 share step multiset");
    // verify_multiset_uniqueness correctly detects this
    assert!(verify_multiset_uniqueness().is_err());
}

#[test]
fn test_verify_fiber_mode_connection_passes() {
    match verify_fiber_mode_connection() {
        Ok(()) => {}
        Err(e) => panic!("fiber-mode connection failed: {}", e),
    }
}

// ─── parent_scales ──────────────────────────────────────────────────────

#[test]
fn test_parent_scales_summit_includes_pentatonic() {
    let scales = parent_scales(&[0, 2, 7, 9]);
    let pentatonic = scales
        .iter()
        .find(|s| s.scale_type() == ScaleType::PentatonicMajor && s.root() == 0);
    assert!(
        pentatonic.is_some(),
        "Summit should be subset of C major pentatonic"
    );
    assert_eq!(pentatonic.unwrap().coverage_ratio(), (4, 5));
}

#[test]
fn test_parent_scales_crossroads_includes_whole_tone() {
    let scales = parent_scales(&[0, 2, 6, 8]);
    let wt = scales
        .iter()
        .find(|s| s.scale_type() == ScaleType::WholeTone);
    assert!(
        wt.is_some(),
        "Crossroads should be subset of whole-tone scale"
    );
    assert_eq!(wt.unwrap().coverage_ratio(), (4, 6));
}

#[test]
fn test_parent_scales_sorted_by_coverage_desc() {
    let scales = parent_scales(&[0, 2, 7, 9]);
    for pair in scales.windows(2) {
        assert!(
            pair[0].coverage() >= pair[1].coverage(),
            "parent scales should be sorted by coverage descending"
        );
    }
}

// ─── OthMode display ────────────────────────────────────────────────────

#[test]
fn test_oth_mode_display() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let s = format!("{}", mode);
    assert!(s.contains("M1"), "Display should include M1 for rotation 0");
}
