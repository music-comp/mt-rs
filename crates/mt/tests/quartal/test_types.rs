extern crate music_comp_mt as theory;

use theory::quartal::{
    pc_chord_quartal_intervals, quartal_to_quintal_interval, quartal_to_quintal_structure,
    quintal_to_quartal_interval, quintal_to_quartal_structure, to_quartal, to_quintal,
    QuartalError, QuartalIntervalStructure, QuartalVoicedChord,
};
use theory::quintal::{enumerate_all, IntervalStructure, PcChord, VoicedChord};

// --- interval complement ---

#[test]
fn test_complement_p5_p4() {
    assert_eq!(quintal_to_quartal_interval(7), 5);
}

#[test]
fn test_complement_d5_a4() {
    assert_eq!(quintal_to_quartal_interval(6), 6); // tritone self-dual
}

#[test]
fn test_complement_a5_d4() {
    assert_eq!(quintal_to_quartal_interval(8), 4);
}

#[test]
fn test_complement_involution() {
    for i in 4..=6 {
        assert_eq!(
            quintal_to_quartal_interval(quartal_to_quintal_interval(i)),
            i
        );
    }
    for i in 6..=8 {
        assert_eq!(
            quartal_to_quintal_interval(quintal_to_quartal_interval(i)),
            i
        );
    }
}

// --- structure conversion ---

#[test]
fn test_structure_777_to_555() {
    let q = IntervalStructure(7, 7, 7);
    let qr = quintal_to_quartal_structure(&q);
    assert_eq!(qr, QuartalIntervalStructure(5, 5, 5));
}

#[test]
fn test_structure_686_to_646() {
    let q = IntervalStructure(6, 8, 6);
    let qr = quintal_to_quartal_structure(&q);
    assert_eq!(qr, QuartalIntervalStructure(6, 4, 6));
}

#[test]
fn test_structure_round_trip() {
    let original = IntervalStructure(7, 8, 6);
    let quartal = quintal_to_quartal_structure(&original);
    let back = quartal_to_quintal_structure(&quartal);
    assert_eq!(back, original);
}

// --- QuartalIntervalStructure ---

#[test]
fn test_is_quartal_legal_555() {
    assert!(QuartalIntervalStructure(5, 5, 5).is_legal());
}

#[test]
fn test_is_quartal_legal_456() {
    assert!(QuartalIntervalStructure(4, 5, 6).is_legal());
}

#[test]
fn test_is_quartal_not_legal_3() {
    assert!(!QuartalIntervalStructure(3, 5, 5).is_legal());
}

#[test]
fn test_is_quartal_not_legal_7() {
    assert!(!QuartalIntervalStructure(5, 7, 5).is_legal());
}

#[test]
fn test_quartal_intervals_method() {
    let qis = QuartalIntervalStructure(4, 5, 6);
    assert_eq!(qis.intervals(), [4, 5, 6]);
}

// --- all 228 chords quartal-legal ---

#[test]
fn test_all_228_quartal_legal() {
    let chords = enumerate_all();
    for chord in &chords {
        let qis = pc_chord_quartal_intervals(chord);
        assert!(qis.is_some(), "Chord {:?} has no quartal reading", chord);
        assert!(
            qis.unwrap().is_legal(),
            "Chord {:?} quartal reading not in [4,6]",
            chord
        );
    }
}

// --- QuartalVoicedChord ---

#[test]
fn test_quartal_voiced_cgda_intervals() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    // C-G-D-A: quintal IS = (7,7,7), quartal IS = (5,5,5)
    assert_eq!(
        qvc.quartal_interval_structure(),
        QuartalIntervalStructure(5, 5, 5)
    );
}

#[test]
fn test_to_quartal_round_trip() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    let back = to_quintal(&qvc);
    assert_eq!(back, vc);
}

#[test]
fn test_quartal_voiced_new() {
    let qvc = QuartalVoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(qvc.pitches(), [48, 55, 62, 69]);
}

#[test]
fn test_quartal_voiced_not_ascending() {
    let result = QuartalVoicedChord::new([55, 48, 62, 69]);
    assert!(result.is_err());
}

#[test]
fn test_pc_chord_quartal_intervals() {
    let pc = PcChord::new([0, 2, 7, 9]).unwrap();
    let qis = pc_chord_quartal_intervals(&pc).unwrap();
    assert_eq!(qis, QuartalIntervalStructure(5, 5, 5));
}

#[test]
fn test_quartal_error_display() {
    let err = QuartalError::IllegalInterval(3);
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn test_quartal_error_display_variants() {
    let cases = vec![
        (QuartalError::IllegalInterval(3), "illegal quartal interval"),
        (QuartalError::WrongIntervalCount(2), "wrong interval count"),
        (QuartalError::NotAscending, "ascending"),
        (QuartalError::DuplicatePitchClasses, "duplicate"),
        (QuartalError::PitchClassOutOfRange(13), "out of range"),
    ];
    for (err, expected_substr) in cases {
        let msg = format!("{}", err);
        assert!(
            msg.contains(expected_substr),
            "Expected '{}' in '{}'",
            expected_substr,
            msg
        );
    }
}

#[test]
fn test_quartal_voiced_inner() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    assert_eq!(qvc.inner(), &vc);
}

#[test]
fn test_quartal_voiced_to_pc_chord() {
    let qvc = QuartalVoicedChord::new([48, 55, 62, 69]).unwrap();
    let pc = qvc.to_pc_chord().unwrap();
    assert_eq!(pc.pcs(), [0, 2, 7, 9]);
}

#[test]
fn test_quartal_voiced_quintal_interval_structure() {
    let qvc = QuartalVoicedChord::new([48, 55, 62, 69]).unwrap();
    let qis = qvc.quintal_interval_structure();
    assert_eq!(qis, IntervalStructure(7, 7, 7));
}
