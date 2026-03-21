extern crate mt_rs as theory;
use theory::analysis;
use theory::chord::{Chord, Quality, Number};
use theory::note::{Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod secondary_tests {
    use super::*;

    #[test]
    fn test_d_major_is_v_of_v_in_c() {
        let chord = Chord::new(Pitch::from(D), Quality::Major, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        let sd = result.unwrap();
        assert_eq!(sd.label, "V/V");
        assert_eq!(sd.target_degree, 5);
    }

    #[test]
    fn test_e_major_is_v_of_vi_in_c() {
        let chord = Chord::new(Pitch::from(E), Quality::Major, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        let sd = result.unwrap();
        assert_eq!(sd.label, "V/vi");
        assert_eq!(sd.target_degree, 6);
    }

    #[test]
    fn test_d7_is_v7_of_v_in_c() {
        let chord = Chord::new(Pitch::from(D), Quality::Dominant, Number::Seventh);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        let sd = result.unwrap();
        assert_eq!(sd.label, "V7/V");
    }

    #[test]
    fn test_a_major_is_v_of_ii_in_c() {
        let chord = Chord::new(Pitch::from(A), Quality::Major, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        let sd = result.unwrap();
        assert_eq!(sd.label, "V/ii");
        assert_eq!(sd.target_degree, 2);
    }

    #[test]
    fn test_g_major_is_not_secondary_dominant() {
        // G major is V in C major — not a secondary dominant (V/I excluded)
        let chord = Chord::new(Pitch::from(G), Quality::Major, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_non_dominant_quality_returns_none() {
        let chord = Chord::new(Pitch::from(D), Quality::Minor, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_b_major_is_v_of_iii_in_c() {
        let chord = Chord::new(Pitch::from(B), Quality::Major, Number::Triad);
        let result = analysis::secondary_dominant(
            Pitch::from(C), Mode::Ionian, &chord,
        );
        let sd = result.unwrap();
        assert_eq!(sd.label, "V/iii");
        assert_eq!(sd.target_degree, 3);
    }
}
