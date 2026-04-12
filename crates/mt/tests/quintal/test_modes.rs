extern crate music_comp_mt as theory;

use theory::quintal::{Orbit, OthMode, StepVocabularyCluster};

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

// ─── OthMode display ────────────────────────────────────────────────────

#[test]
fn test_oth_mode_display() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let s = format!("{}", mode);
    assert!(s.contains("M1"), "Display should include M1 for rotation 0");
}
