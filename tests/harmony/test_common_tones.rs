extern crate rust_music_theory as theory;
use theory::harmony;
use theory::note::{NoteLetter, Pitch, PitchSymbol::*};

#[cfg(test)]
mod common_tone_tests {
    use super::*;

    #[test]
    fn test_c_major_and_a_minor_share_two_tones() {
        let c_major: Vec<Pitch> = vec![C, E, G].into_iter().map(Pitch::from).collect();
        let a_minor: Vec<Pitch> = vec![A, C, E].into_iter().map(Pitch::from).collect();
        let common = harmony::common_tones(&c_major, &a_minor);
        assert_eq!(common.len(), 2);
        assert!(common.iter().any(|p| p.as_u8() == 0)); // C
        assert!(common.iter().any(|p| p.as_u8() == 4)); // E
    }

    #[test]
    fn test_no_common_tones() {
        let c_major: Vec<Pitch> = vec![C, E, G].into_iter().map(Pitch::from).collect();
        let fs_major: Vec<Pitch> = vec![Fs, As, Cs].into_iter().map(Pitch::from).collect();
        let common = harmony::common_tones(&c_major, &fs_major);
        assert!(common.is_empty());
    }

    #[test]
    fn test_enharmonic_equivalence() {
        let a: Vec<Pitch> = vec![Cs].into_iter().map(Pitch::from).collect();
        let b: Vec<Pitch> = vec![Db].into_iter().map(Pitch::from).collect();
        let common = harmony::common_tones(&a, &b);
        assert_eq!(common.len(), 1);
        assert_eq!(common[0].letter, NoteLetter::C); // spelling from first arg
        assert_eq!(common[0].accidental, 1);
    }

    #[test]
    fn test_empty_inputs() {
        let empty: Vec<Pitch> = vec![];
        let c: Vec<Pitch> = vec![C].into_iter().map(Pitch::from).collect();
        assert!(harmony::common_tones(&empty, &c).is_empty());
        assert!(harmony::common_tones(&c, &empty).is_empty());
    }

    #[test]
    fn test_preserves_first_argument_order() {
        let a: Vec<Pitch> = vec![G, E, C].into_iter().map(Pitch::from).collect();
        let b: Vec<Pitch> = vec![C, E, G, B].into_iter().map(Pitch::from).collect();
        let common = harmony::common_tones(&a, &b);
        assert_eq!(common.len(), 3);
        assert_eq!(common[0].as_u8(), 7); // G first
        assert_eq!(common[1].as_u8(), 4); // E second
        assert_eq!(common[2].as_u8(), 0); // C third
    }
}
