extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, quartal_l1_distances, to_quartal, verify_quartal_fiber_classes,
    verify_quartal_universal_l1_law,
};
use theory::quintal::{self, FiberClass, Orbit, VoicedChord};

#[test]
fn test_quartal_universal_l1_law() {
    let space = base_space();
    assert!(verify_quartal_universal_l1_law(&space).is_ok());
}

#[test]
fn test_quartal_fiber_classes_count() {
    // Mirrors quintal::test_verify_fiber_classes_count: 14 orbits,
    // 11 ClassA + 3 ClassB (Q676, Q686, Q688).
    let space = base_space();
    let classes = verify_quartal_fiber_classes(&space);
    assert_eq!(classes.len(), 14);
    let class_a = classes
        .values()
        .filter(|&&fc| fc == FiberClass::ClassA)
        .count();
    let class_b = classes
        .values()
        .filter(|&&fc| fc == FiberClass::ClassB)
        .count();
    assert_eq!(class_a, 11);
    assert_eq!(class_b, 3);
    let class_b_orbits: Vec<Orbit> = classes
        .iter()
        .filter(|(_, &fc)| fc == FiberClass::ClassB)
        .map(|(&orb, _)| orb)
        .collect();
    assert!(class_b_orbits.contains(&Orbit::Q676));
    assert!(class_b_orbits.contains(&Orbit::Q686));
    assert!(class_b_orbits.contains(&Orbit::Q688));
}

#[test]
fn test_quartal_l1_pattern_summit() {
    // Q555 Summit voiced quintally as Eb-Bb-F-C ([51,58,65,72]).
    let qvc = to_quartal(&VoicedChord::new([51, 58, 65, 72]).unwrap());
    assert_eq!(quartal_l1_distances(&qvc), [12, 12, 12, 36]);
}

#[test]
fn test_quartal_l1_pattern_saddle() {
    // Q686 saddle voiced quintally as C-F#-D-G# ([48,54,62,68]).
    let qvc = to_quartal(&VoicedChord::new([48, 54, 62, 68]).unwrap());
    assert_eq!(quartal_l1_distances(&qvc), [12, 12, 12, 36]);
}

#[test]
fn test_quartal_verifiers_agree_with_quintal() {
    let space = base_space();

    // Universal L1 law: both directions must succeed (and we want the
    // expect message to surface the failing chord list if not). Equality of
    // .is_ok() alone would pass even if the two perspectives flagged
    // different chords as failures.
    verify_quartal_universal_l1_law(&space)
        .expect("quartal Universal L1 Law must hold on the canonical base space");
    quintal::verify_universal_l1_law(&space)
        .expect("quintal Universal L1 Law must hold on the canonical base space");

    // Fiber classes: identical maps. PartialEq on BTreeMap<Orbit, FiberClass>
    // gives us a strict pointwise comparison.
    assert_eq!(
        verify_quartal_fiber_classes(&space),
        quintal::verify_fiber_classes(&space)
    );
}
