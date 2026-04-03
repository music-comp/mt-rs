extern crate music_comp_mt as theory;

use theory::quartal::{
    from_stacked_fourths, from_stacked_fourths_voiced, pure_quartal_stack, quartal_neighbors,
    BaseSpace, QuartalError,
};

#[test]
fn test_pure_quartal_stack_c() {
    // C with P4+P4+P4 = C-F-Bb-Eb = {0,5,10,3} = sorted [0,3,5,10]
    let chord = pure_quartal_stack(0);
    assert_eq!(chord.pcs, [0, 3, 5, 10]);
}

#[test]
fn test_pure_quartal_stack_a() {
    // A with P4+P4+P4 = A-D-G-C = {9,2,7,0} = sorted [0,2,7,9]
    // Same as quintal C-G-D-A!
    let chord = pure_quartal_stack(9);
    assert_eq!(chord.pcs, [0, 2, 7, 9]);
}

#[test]
fn test_from_stacked_fourths_valid() {
    let chord = from_stacked_fourths(9, &[5, 5, 5]).unwrap();
    assert_eq!(chord.pcs, [0, 2, 7, 9]);
}

#[test]
fn test_from_stacked_fourths_mixed() {
    // P4+A4+P4 = (5,6,5)
    let chord = from_stacked_fourths(0, &[5, 6, 5]);
    assert!(chord.is_ok());
}

#[test]
fn test_from_stacked_fourths_bad_interval_3() {
    let result = from_stacked_fourths(0, &[3, 5, 5]);
    assert_eq!(result, Err(QuartalError::IllegalInterval(3)));
}

#[test]
fn test_from_stacked_fourths_bad_interval_7() {
    let result = from_stacked_fourths(0, &[5, 5, 7]);
    assert_eq!(result, Err(QuartalError::IllegalInterval(7)));
}

#[test]
fn test_from_stacked_fourths_wrong_count_2() {
    let result = from_stacked_fourths(0, &[5, 5]);
    assert_eq!(result, Err(QuartalError::WrongIntervalCount(2)));
}

#[test]
fn test_from_stacked_fourths_wrong_count_4() {
    let result = from_stacked_fourths(0, &[5, 5, 5, 5]);
    assert_eq!(result, Err(QuartalError::WrongIntervalCount(4)));
}

#[test]
fn test_from_stacked_fourths_bad_root() {
    let result = from_stacked_fourths(13, &[5, 5, 5]);
    assert_eq!(result, Err(QuartalError::PitchClassOutOfRange(13)));
}

#[test]
fn test_voiced_stacked_fourths() {
    // A3-D4-G4-C5 = [57,62,67,72]
    let qvc = from_stacked_fourths_voiced(57, &[5, 5, 5]).unwrap();
    assert_eq!(qvc.pitches(), [57, 62, 67, 72]);
}

#[test]
fn test_all_pure_stacks_in_base_space() {
    let space = BaseSpace::new();
    for root in 0..12u8 {
        let chord = pure_quartal_stack(root);
        assert!(
            space.chords().contains(&chord),
            "pure_quartal_stack({}) = {:?} not in BaseSpace",
            root,
            chord
        );
    }
}

#[test]
fn test_quartal_neighbors_nonempty() {
    let space = BaseSpace::new();
    let chord = pure_quartal_stack(0);
    let neighbors = quartal_neighbors(&chord, &space);
    assert!(!neighbors.is_empty());
}

#[test]
fn test_quartal_neighbors_have_quartal_intervals() {
    let space = BaseSpace::new();
    let chord = pure_quartal_stack(0);
    let neighbors = quartal_neighbors(&chord, &space);
    for (_, qis) in &neighbors {
        assert!(
            qis.is_some(),
            "Neighbor should have quartal interval structure"
        );
        assert!(
            qis.unwrap().is_legal(),
            "Neighbor quartal IS should be in [4,6]"
        );
    }
}
