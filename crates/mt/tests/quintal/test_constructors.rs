extern crate music_comp_mt as theory;

use theory::quintal::{quintal_root, BaseSpace, PcChord};

#[test]
fn test_quintal_root_cgda() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let voiced = quintal_root(&chord, 4).unwrap();
    assert_eq!(voiced.pitches, [48, 55, 62, 69]);
}

#[test]
fn test_quintal_root_interval_structure_matches_all_228() {
    let space = BaseSpace::new();
    for pc_chord in space.chords() {
        let voiced = quintal_root(pc_chord, 3).unwrap();
        let is = voiced.interval_structure();
        assert_eq!(
            Some(is),
            pc_chord.interval_structure(),
            "PcChord {:?}: interval structure mismatch",
            pc_chord.pcs
        );
    }
}

#[test]
fn test_quintal_root_octave_shift() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let v3 = quintal_root(&chord, 3).unwrap();
    let v5 = quintal_root(&chord, 5).unwrap();
    for i in 0..4 {
        assert_eq!(v5.pitches[i] - v3.pitches[i], 24);
    }
}

#[test]
fn test_quintal_root_round_trip_pc_chord_all_228() {
    let space = BaseSpace::new();
    for pc_chord in space.chords() {
        let voiced = quintal_root(pc_chord, 3).unwrap();
        let recovered = voiced.to_pc_chord().unwrap();
        assert_eq!(
            recovered, *pc_chord,
            "round-trip failed for {:?}",
            pc_chord.pcs
        );
    }
}

#[test]
fn test_quintal_root_different_chords() {
    // {0,1,6,9} — non-palindromic (6,7,8)
    let chord = PcChord::new([0, 1, 6, 9]).unwrap();
    let voiced = quintal_root(&chord, 3).unwrap();
    let is = voiced.interval_structure();
    assert!(is.is_legal());
    assert_eq!(voiced.to_pc_chord().unwrap(), chord);
}
