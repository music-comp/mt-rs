extern crate rust_music_theory as theory;
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
}
