extern crate music_comp_mt as theory;
use theory::quintal::{IntervalStructure, PcChord, QuintalError, VoicedChord};

#[cfg(test)]
mod pc_chord_tests {
    use super::*;

    #[test]
    fn test_new_valid_input() {
        let chord = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(chord.pcs(), [0, 2, 7, 9]);
    }

    #[test]
    fn test_new_sorts_input() {
        let chord = PcChord::new([9, 7, 2, 0]).unwrap();
        assert_eq!(chord.pcs(), [0, 2, 7, 9]);
    }

    #[test]
    fn test_new_out_of_range() {
        let result = PcChord::new([0, 2, 7, 12]);
        assert_eq!(result, Err(QuintalError::PitchClassOutOfRange(12)));
    }

    #[test]
    fn test_new_out_of_range_high_value() {
        let result = PcChord::new([0, 2, 7, 255]);
        assert_eq!(result, Err(QuintalError::PitchClassOutOfRange(255)));
    }

    #[test]
    fn test_new_duplicates() {
        let result = PcChord::new([0, 2, 7, 7]);
        assert_eq!(result, Err(QuintalError::DuplicatePitchClasses));
    }

    #[test]
    fn test_from_unsorted_valid() {
        let chord = PcChord::from_unsorted(&[9, 0, 7, 2]).unwrap();
        assert_eq!(chord.pcs(), [0, 2, 7, 9]);
    }

    #[test]
    fn test_from_unsorted_wrong_cardinality_too_few() {
        let result = PcChord::from_unsorted(&[0, 2, 7]);
        assert_eq!(result, Err(QuintalError::WrongCardinality(3)));
    }

    #[test]
    fn test_from_unsorted_wrong_cardinality_too_many() {
        let result = PcChord::from_unsorted(&[0, 2, 7, 9, 11]);
        assert_eq!(result, Err(QuintalError::WrongCardinality(5)));
    }

    #[test]
    fn test_from_unsorted_empty() {
        let result = PcChord::from_unsorted(&[]);
        assert_eq!(result, Err(QuintalError::WrongCardinality(0)));
    }

    #[test]
    fn test_interval_structure_0_2_7_9() {
        let chord = PcChord::new([0, 2, 7, 9]).unwrap();
        assert_eq!(chord.interval_structure(), Some(IntervalStructure(7, 7, 7)));
    }

    #[test]
    fn test_interval_structure_0_2_6_8() {
        let chord = PcChord::new([0, 2, 6, 8]).unwrap();
        assert_eq!(chord.interval_structure(), Some(IntervalStructure(6, 8, 6)));
    }

    #[test]
    fn test_is_legal_true() {
        let chord = PcChord::new([0, 2, 7, 9]).unwrap();
        assert!(chord.is_legal());
    }

    #[test]
    fn test_is_legal_false() {
        let chord = PcChord::new([0, 1, 2, 3]).unwrap();
        assert!(!chord.is_legal());
    }

    #[test]
    fn test_is_legal_chromatic_cluster() {
        let chord = PcChord::new([4, 5, 6, 7]).unwrap();
        assert!(!chord.is_legal());
    }
}

#[cfg(test)]
mod interval_structure_tests {
    use super::*;

    #[test]
    fn test_is_legal_777() {
        assert!(IntervalStructure::new(7, 7, 7).is_legal());
    }

    #[test]
    fn test_is_legal_686() {
        assert!(IntervalStructure::new(6, 8, 6).is_legal());
    }

    #[test]
    fn test_is_legal_false_577() {
        assert!(!IntervalStructure::new(5, 7, 7).is_legal());
    }

    #[test]
    fn test_is_legal_false_970() {
        assert!(!IntervalStructure::new(9, 7, 0).is_legal());
    }

    #[test]
    fn test_intervals_returns_correct_array() {
        let is = IntervalStructure::new(6, 7, 8);
        assert_eq!(is.intervals(), [6, 7, 8]);
    }

    #[test]
    fn test_all_27_legal_structures() {
        for a in 6..=8 {
            for b in 6..=8 {
                for c in 6..=8 {
                    let is = IntervalStructure::new(a, b, c);
                    assert!(
                        is.is_legal(),
                        "IntervalStructure({}, {}, {}) should be legal",
                        a,
                        b,
                        c
                    );
                }
            }
        }
    }

    #[test]
    fn test_outside_range_not_legal() {
        for &val in &[0, 1, 2, 3, 4, 5, 9, 10, 11, 12] {
            let is = IntervalStructure::new(val, 7, 7);
            assert!(
                !is.is_legal(),
                "IntervalStructure({}, 7, 7) should not be legal",
                val
            );
        }
    }
}

#[cfg(test)]
mod voiced_chord_tests {
    use super::*;

    #[test]
    fn test_new_ascending_succeeds() {
        let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
        assert_eq!(vc.pitches, [48, 55, 62, 69]);
    }

    #[test]
    fn test_new_not_ascending_equal() {
        let result = VoicedChord::new([48, 55, 55, 69]);
        assert_eq!(result, Err(QuintalError::NotAscending));
    }

    #[test]
    fn test_new_not_ascending_descending() {
        let result = VoicedChord::new([48, 55, 62, 50]);
        assert_eq!(result, Err(QuintalError::NotAscending));
    }

    #[test]
    fn test_new_not_ascending_first_pair() {
        let result = VoicedChord::new([60, 55, 62, 69]);
        assert_eq!(result, Err(QuintalError::NotAscending));
    }

    #[test]
    fn test_interval_structure_all_sevenths() {
        let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
        assert_eq!(vc.interval_structure(), IntervalStructure(7, 7, 7));
    }

    #[test]
    fn test_interval_structure_mixed() {
        let vc = VoicedChord::new([48, 54, 62, 68]).unwrap();
        assert_eq!(vc.interval_structure(), IntervalStructure(6, 8, 6));
    }

    #[test]
    fn test_to_pc_chord_all_sevenths() {
        let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let pc = vc.to_pc_chord().unwrap();
        assert_eq!(pc.pcs(), [0, 2, 7, 9]);
    }

    #[test]
    fn test_to_pc_chord_different_voicing() {
        let vc = VoicedChord::new([50, 57, 64, 71]).unwrap();
        let pc = vc.to_pc_chord().unwrap();
        assert_eq!(pc.pcs(), [2, 4, 9, 11]);
    }

    #[test]
    fn test_to_pc_chord_duplicate_pitch_classes() {
        // Pitches 48 (C3) and 60 (C4) both reduce to pitch class 0.
        let vc = VoicedChord::new([48, 55, 60, 67]).unwrap();
        let result = vc.to_pc_chord();
        assert_eq!(result, Err(QuintalError::DuplicatePitchClasses));
    }
}

#[cfg(test)]
mod error_display_tests {
    use super::*;

    #[test]
    fn test_pitch_class_out_of_range_display() {
        let err = QuintalError::PitchClassOutOfRange(15);
        assert_eq!(
            err.to_string(),
            "pitch class out of range: 15 (must be 0..=11)"
        );
    }

    #[test]
    fn test_duplicate_pitch_classes_display() {
        let err = QuintalError::DuplicatePitchClasses;
        assert_eq!(err.to_string(), "duplicate pitch classes");
    }

    #[test]
    fn test_wrong_cardinality_display() {
        let err = QuintalError::WrongCardinality(3);
        assert_eq!(err.to_string(), "wrong cardinality: expected 4, got 3");
    }

    #[test]
    fn test_not_ascending_display() {
        let err = QuintalError::NotAscending;
        assert_eq!(
            err.to_string(),
            "pitches must be in strictly ascending order"
        );
    }
}
