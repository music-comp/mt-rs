extern crate rust_music_theory as theory;
use theory::interval::{Interval, Quality, Number};

#[cfg(test)]
mod compound_tests {
    use super::*;

    #[test]
    fn test_minor_ninth() {
        let interval = Interval::from_semitone(13).unwrap();
        assert_eq!(interval.quality, Quality::Minor);
        assert_eq!(interval.number, Number::Ninth);
        assert_eq!(interval.semitone_count, 13);
    }

    #[test]
    fn test_major_ninth() {
        let interval = Interval::from_semitone(14).unwrap();
        assert_eq!(interval.quality, Quality::Major);
        assert_eq!(interval.number, Number::Ninth);
    }

    #[test]
    fn test_minor_tenth() {
        let interval = Interval::from_semitone(15).unwrap();
        assert_eq!(interval.quality, Quality::Minor);
        assert_eq!(interval.number, Number::Tenth);
    }

    #[test]
    fn test_perfect_eleventh() {
        let interval = Interval::from_semitone(17).unwrap();
        assert_eq!(interval.quality, Quality::Perfect);
        assert_eq!(interval.number, Number::Eleventh);
    }

    #[test]
    fn test_diminished_twelfth() {
        // 18 = 12 + 6; from_semitone(6) = d5, compound = d12
        let interval = Interval::from_semitone(18).unwrap();
        assert_eq!(interval.quality, Quality::Diminished);
        assert_eq!(interval.number, Number::Twelfth);
    }

    #[test]
    fn test_perfect_twelfth() {
        let interval = Interval::from_semitone(19).unwrap();
        assert_eq!(interval.quality, Quality::Perfect);
        assert_eq!(interval.number, Number::Twelfth);
    }

    #[test]
    fn test_major_thirteenth() {
        let interval = Interval::from_semitone(21).unwrap();
        assert_eq!(interval.quality, Quality::Major);
        assert_eq!(interval.number, Number::Thirteenth);
    }

    #[test]
    fn test_perfect_fifteenth() {
        let interval = Interval::from_semitone(24).unwrap();
        assert_eq!(interval.quality, Quality::Perfect);
        assert_eq!(interval.number, Number::Fifteenth);
    }

    #[test]
    fn test_is_compound() {
        let simple = Interval::from_semitone(7).unwrap();
        assert!(!simple.is_compound());
        let compound = Interval::from_semitone(14).unwrap();
        assert!(compound.is_compound());
        let octave = Interval::from_semitone(12).unwrap();
        assert!(!octave.is_compound());
    }

    #[test]
    fn test_simple_equivalent() {
        let compound = Interval::from_semitone(14).unwrap(); // Major 9th
        let simple = compound.simple();
        assert_eq!(simple.quality, Quality::Major);
        assert_eq!(simple.number, Number::Second);
        assert_eq!(simple.semitone_count, 2);
    }

    #[test]
    fn test_simple_of_simple_is_identity() {
        let interval = Interval::from_semitone(7).unwrap(); // P5
        let simple = interval.simple();
        assert_eq!(simple, interval);
    }

    #[test]
    fn test_beyond_24_errors() {
        assert!(Interval::from_semitone(25).is_err());
    }
}
