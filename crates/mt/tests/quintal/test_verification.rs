extern crate music_comp_mt as theory;

use theory::quintal::{
    fiber_class, inversion_l1_distances, inversions_in_base, verify_fiber_classes,
    verify_universal_l1_law, BaseSpace, FiberClass, Orbit, PcChord, VoicedChord,
};

// --- inversions_in_base ---

#[test]
fn test_inversions_in_base_cgda() {
    // Q777: only the root position has a legal IS.
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(inversions_in_base(&chord), vec![0]);
}

#[test]
fn test_inversions_in_base_crossroads() {
    // Q686: two inversions land in [6,8], at indices 0 and 2.
    let chord = VoicedChord::new([48, 54, 62, 68]).unwrap();
    assert_eq!(inversions_in_base(&chord), vec![0, 2]);
}

// --- inversion_l1_distances ---

#[test]
fn test_l1_distances_cgda() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(inversion_l1_distances(&chord), [12, 12, 12, 36]);
}

#[test]
fn test_l1_distances_crossroads() {
    let chord = VoicedChord::new([48, 54, 62, 68]).unwrap();
    assert_eq!(inversion_l1_distances(&chord), [12, 12, 12, 36]);
}

#[test]
fn test_l1_distances_sum_72() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let dists = inversion_l1_distances(&chord);
    assert_eq!(dists.iter().sum::<u32>(), 72);
}

// --- universal L1 law ---

#[test]
fn test_universal_l1_law() {
    let space = BaseSpace::new();
    assert!(verify_universal_l1_law(&space).is_ok());
}

// --- fiber_class ---

#[test]
fn test_fiber_class_q777() {
    // Q777 is Class A (only 1 inversion in [6,8]).
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(fiber_class(&chord), Some(FiberClass::ClassA));
}

#[test]
fn test_fiber_class_q686() {
    // Q686 is Class B (2 inversions in [6,8]).
    let chord = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(fiber_class(&chord), Some(FiberClass::ClassB));
}

#[test]
fn test_fiber_class_q676() {
    // Q676 is Class B.
    let chord = PcChord::new([0, 1, 6, 7]).unwrap();
    assert_eq!(fiber_class(&chord), Some(FiberClass::ClassB));
}

#[test]
fn test_fiber_class_q688() {
    // Q688 is Class B.
    let chord = PcChord::new([0, 2, 6, 10]).unwrap();
    assert_eq!(fiber_class(&chord), Some(FiberClass::ClassB));
}

// --- verify_fiber_classes ---

#[test]
fn test_verify_fiber_classes_count() {
    let space = BaseSpace::new();
    let classes = verify_fiber_classes(&space);
    assert_eq!(classes.len(), 14);
    let class_a_count = classes
        .values()
        .filter(|&&fc| fc == FiberClass::ClassA)
        .count();
    let class_b_count = classes
        .values()
        .filter(|&&fc| fc == FiberClass::ClassB)
        .count();
    assert_eq!(class_a_count, 11);
    assert_eq!(class_b_count, 3);
}

#[test]
fn test_class_b_orbits() {
    let space = BaseSpace::new();
    let classes = verify_fiber_classes(&space);
    let class_b: Vec<Orbit> = classes
        .iter()
        .filter(|(_, &fc)| fc == FiberClass::ClassB)
        .map(|(&orb, _)| orb)
        .collect();
    assert!(class_b.contains(&Orbit::Q676));
    assert!(class_b.contains(&Orbit::Q686));
    assert!(class_b.contains(&Orbit::Q688));
}
