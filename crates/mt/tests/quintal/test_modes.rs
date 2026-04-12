extern crate music_comp_mt as theory;

use theory::quintal::{orbit_step_sequence, step_size_multiset, Orbit, OthMode, StepVocabularyCluster};

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

// ─── OthMode display ────────────────────────────────────────────────────

#[test]
fn test_oth_mode_display() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let s = format!("{}", mode);
    assert!(s.contains("M1"), "Display should include M1 for rotation 0");
}
