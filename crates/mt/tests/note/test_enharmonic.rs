extern crate music_comp_mt as theory;
use theory::interval::Interval;
use theory::note::{NoteLetter, Pitch};

#[cfg(test)]
mod enharmonic_tests {
    use super::*;

    #[test]
    fn test_enharmonic_equivalence() {
        let cs = Pitch::new(NoteLetter::C, 1);
        let db = Pitch::new(NoteLetter::D, -1);
        assert!(cs.is_enharmonic_to(&db));
        assert!(db.is_enharmonic_to(&cs));
    }

    #[test]
    fn test_not_enharmonic() {
        let c = Pitch::new(NoteLetter::C, 0);
        let d = Pitch::new(NoteLetter::D, 0);
        assert!(!c.is_enharmonic_to(&d));
    }

    #[test]
    fn test_same_pitch_is_enharmonic() {
        let c = Pitch::new(NoteLetter::C, 0);
        assert!(c.is_enharmonic_to(&c));
    }

    #[test]
    fn test_double_sharp_enharmonic() {
        // C double-sharp = D
        let cxx = Pitch::new(NoteLetter::C, 2);
        let d = Pitch::new(NoteLetter::D, 0);
        assert!(cxx.is_enharmonic_to(&d));
    }

    #[test]
    fn test_letter_distance_ascending() {
        assert_eq!(NoteLetter::C.distance_to(NoteLetter::E), 2);
        assert_eq!(NoteLetter::C.distance_to(NoteLetter::G), 4);
        assert_eq!(NoteLetter::B.distance_to(NoteLetter::D), 2);
        assert_eq!(NoteLetter::F.distance_to(NoteLetter::B), 3);
        assert_eq!(NoteLetter::A.distance_to(NoteLetter::C), 2);
    }

    #[test]
    fn test_letter_distance_unison() {
        assert_eq!(NoteLetter::C.distance_to(NoteLetter::C), 0);
        assert_eq!(NoteLetter::G.distance_to(NoteLetter::G), 0);
    }

    #[test]
    fn test_transpose_up_major_third() {
        let c = Pitch::new(NoteLetter::C, 0);
        let interval = Interval::from_semitone(4).unwrap();
        let result = c.transpose_up(&interval);
        assert_eq!(result.as_u8(), 4); // E
    }

    #[test]
    fn test_transpose_down_perfect_fifth() {
        let g = Pitch::new(NoteLetter::G, 0);
        let interval = Interval::from_semitone(7).unwrap();
        let result = g.transpose_down(&interval);
        assert_eq!(result.as_u8(), 0); // C
    }

    #[test]
    fn test_transpose_up_wraps_around() {
        let b = Pitch::new(NoteLetter::B, 0);
        let interval = Interval::from_semitone(1).unwrap(); // minor second
        let result = b.transpose_up(&interval);
        assert_eq!(result.as_u8(), 0); // C
    }

    #[test]
    fn test_transpose_down_wraps_around() {
        let c = Pitch::new(NoteLetter::C, 0);
        let interval = Interval::from_semitone(1).unwrap(); // minor second
        let result = c.transpose_down(&interval);
        assert_eq!(result.as_u8(), 11); // B
    }

    #[test]
    fn test_transpose_up_tritone() {
        let c = Pitch::new(NoteLetter::C, 0);
        let interval = Interval::from_semitone(6).unwrap();
        let result = c.transpose_up(&interval);
        assert_eq!(result.as_u8(), 6); // F#/Gb
    }
}
