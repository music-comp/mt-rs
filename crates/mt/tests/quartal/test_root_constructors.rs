extern crate music_comp_mt as theory;

use theory::quartal::{quartal_root, BaseSpace, PcChord, QuartalIntervalStructure};

#[test]
fn test_quartal_root_cgda() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let voiced = quartal_root(&chord, 4).unwrap();
    assert_eq!(voiced.pitches(), [57, 62, 67, 72]);
}

#[test]
fn test_quartal_root_raw_intervals_are_in_quartal_range_all_228() {
    let space = BaseSpace::new();
    for pc_chord in space.chords() {
        let voiced = quartal_root(pc_chord, 3).unwrap();
        let p = voiced.pitches();
        let i1 = p[1] - p[0];
        let i2 = p[2] - p[1];
        let i3 = p[3] - p[2];
        assert!(
            (4..=6).contains(&i1) && (4..=6).contains(&i2) && (4..=6).contains(&i3),
            "PcChord {:?}: raw quartal intervals ({},{},{}) not all in {{4,5,6}}",
            pc_chord.pcs,
            i1,
            i2,
            i3
        );
    }
}

#[test]
fn test_quartal_root_intervals_are_reverse_complement_of_quintal_all_228() {
    let space = BaseSpace::new();
    for pc_chord in space.chords() {
        let quintal_is = pc_chord.interval_structure().unwrap();
        let voiced = quartal_root(pc_chord, 3).unwrap();
        let p = voiced.pitches();
        let qi1 = p[1] - p[0];
        let qi2 = p[2] - p[1];
        let qi3 = p[3] - p[2];
        let expected = QuartalIntervalStructure(
            (12 - quintal_is.2) % 12,
            (12 - quintal_is.1) % 12,
            (12 - quintal_is.0) % 12,
        );
        let actual = QuartalIntervalStructure(qi1, qi2, qi3);
        assert_eq!(
            actual, expected,
            "PcChord {:?}: quartal raw intervals mismatch",
            pc_chord.pcs
        );
    }
}

#[test]
fn test_quartal_root_round_trip_pc_chord_all_228() {
    let space = BaseSpace::new();
    for pc_chord in space.chords() {
        let voiced = quartal_root(pc_chord, 3).unwrap();
        let recovered = voiced.to_pc_chord().unwrap();
        assert_eq!(
            recovered, *pc_chord,
            "round-trip failed for {:?}",
            pc_chord.pcs
        );
    }
}

#[test]
fn test_quartal_root_octave_shift() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let v3 = quartal_root(&chord, 3).unwrap();
    let v5 = quartal_root(&chord, 5).unwrap();
    for i in 0..4 {
        assert_eq!(v5.pitches()[i] - v3.pitches()[i], 24);
    }
}

#[test]
fn test_quartal_root_different_chords() {
    let chord = PcChord::new([0, 1, 6, 9]).unwrap();
    let voiced = quartal_root(&chord, 3).unwrap();
    let p = voiced.pitches();
    let i1 = p[1] - p[0];
    let i2 = p[2] - p[1];
    let i3 = p[3] - p[2];
    assert!(
        (4..=6).contains(&i1) && (4..=6).contains(&i2) && (4..=6).contains(&i3),
        "raw quartal intervals not in {{4,5,6}}: ({},{},{})",
        i1,
        i2,
        i3
    );
    assert_eq!(voiced.to_pc_chord().unwrap(), chord);
}
