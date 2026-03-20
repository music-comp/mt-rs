extern crate rust_music_theory as theory;
use theory::interval::{Interval, Quality, Number};
use theory::note::{NoteLetter, Pitch};

#[cfg(test)]
mod between_tests {
    use super::*;

    #[test]
    fn test_perfect_unison() {
        let c = Pitch::new(NoteLetter::C, 0);
        let interval = Interval::between(&c, &c).unwrap();
        assert_eq!(interval.quality, Quality::Perfect);
        assert_eq!(interval.number, Number::Unison);
        assert_eq!(interval.semitone_count, 0);
    }

    #[test]
    fn test_major_third() {
        let c = Pitch::new(NoteLetter::C, 0);
        let e = Pitch::new(NoteLetter::E, 0);
        let interval = Interval::between(&c, &e).unwrap();
        assert_eq!(interval.quality, Quality::Major);
        assert_eq!(interval.number, Number::Third);
        assert_eq!(interval.semitone_count, 4);
    }

    #[test]
    fn test_minor_third() {
        let c = Pitch::new(NoteLetter::C, 0);
        let eb = Pitch::new(NoteLetter::E, -1);
        let interval = Interval::between(&c, &eb).unwrap();
        assert_eq!(interval.quality, Quality::Minor);
        assert_eq!(interval.number, Number::Third);
        assert_eq!(interval.semitone_count, 3);
    }

    #[test]
    fn test_augmented_fourth() {
        // F to B = augmented fourth (6 semitones, 3 letter steps = 4th)
        let f = Pitch::new(NoteLetter::F, 0);
        let b = Pitch::new(NoteLetter::B, 0);
        let interval = Interval::between(&f, &b).unwrap();
        assert_eq!(interval.quality, Quality::Augmented);
        assert_eq!(interval.number, Number::Fourth);
        assert_eq!(interval.semitone_count, 6);
    }

    #[test]
    fn test_diminished_fifth() {
        // F to Cb = diminished fifth (6 semitones, 4 letter steps = 5th)
        let f = Pitch::new(NoteLetter::F, 0);
        let cb = Pitch::new(NoteLetter::C, -1);
        let interval = Interval::between(&f, &cb).unwrap();
        assert_eq!(interval.quality, Quality::Diminished);
        assert_eq!(interval.number, Number::Fifth);
        assert_eq!(interval.semitone_count, 6);
    }

    #[test]
    fn test_perfect_fifth() {
        let c = Pitch::new(NoteLetter::C, 0);
        let g = Pitch::new(NoteLetter::G, 0);
        let interval = Interval::between(&c, &g).unwrap();
        assert_eq!(interval.quality, Quality::Perfect);
        assert_eq!(interval.number, Number::Fifth);
        assert_eq!(interval.semitone_count, 7);
    }

    #[test]
    fn test_minor_second() {
        let e = Pitch::new(NoteLetter::E, 0);
        let f = Pitch::new(NoteLetter::F, 0);
        let interval = Interval::between(&e, &f).unwrap();
        assert_eq!(interval.quality, Quality::Minor);
        assert_eq!(interval.number, Number::Second);
        assert_eq!(interval.semitone_count, 1);
    }

    #[test]
    fn test_major_second() {
        let c = Pitch::new(NoteLetter::C, 0);
        let d = Pitch::new(NoteLetter::D, 0);
        let interval = Interval::between(&c, &d).unwrap();
        assert_eq!(interval.quality, Quality::Major);
        assert_eq!(interval.number, Number::Second);
        assert_eq!(interval.semitone_count, 2);
    }

    #[test]
    fn test_diminished_fourth() {
        // F# to Bb = diminished fourth (4 semitones, 3 letter steps = 4th)
        let fs = Pitch::new(NoteLetter::F, 1);
        let bb = Pitch::new(NoteLetter::B, -1);
        let interval = Interval::between(&fs, &bb).unwrap();
        assert_eq!(interval.number, Number::Fourth);
        assert_eq!(interval.semitone_count, 4);
        assert_eq!(interval.quality, Quality::Diminished);
    }

    #[test]
    fn test_augmented_unison() {
        // C to C# = augmented unison
        let c = Pitch::new(NoteLetter::C, 0);
        let cs = Pitch::new(NoteLetter::C, 1);
        let interval = Interval::between(&c, &cs).unwrap();
        assert_eq!(interval.quality, Quality::Augmented);
        assert_eq!(interval.number, Number::Unison);
        assert_eq!(interval.semitone_count, 1);
    }

    #[test]
    fn test_major_seventh() {
        let c = Pitch::new(NoteLetter::C, 0);
        let b = Pitch::new(NoteLetter::B, 0);
        let interval = Interval::between(&c, &b).unwrap();
        assert_eq!(interval.quality, Quality::Major);
        assert_eq!(interval.number, Number::Seventh);
        assert_eq!(interval.semitone_count, 11);
    }

    #[test]
    fn test_minor_seventh() {
        let c = Pitch::new(NoteLetter::C, 0);
        let bb = Pitch::new(NoteLetter::B, -1);
        let interval = Interval::between(&c, &bb).unwrap();
        assert_eq!(interval.quality, Quality::Minor);
        assert_eq!(interval.number, Number::Seventh);
        assert_eq!(interval.semitone_count, 10);
    }

    #[test]
    fn test_perfect_octave_wraps_to_unison() {
        // C to C = unison (same pitch class, between works within one octave)
        let c = Pitch::new(NoteLetter::C, 0);
        let interval = Interval::between(&c, &c).unwrap();
        assert_eq!(interval.number, Number::Unison);
    }
}
