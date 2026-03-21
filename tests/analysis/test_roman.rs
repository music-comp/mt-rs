extern crate mt_rs as theory;
use theory::analysis::roman_numeral;
use theory::chord::{Chord, Quality, Number};
use theory::note::{NoteLetter, Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod roman_tests {
    use super::*;

    #[test]
    fn test_tonic_in_major() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.degree, 1);
        assert_eq!(rn.label, "I");
    }

    #[test]
    fn test_v_chord() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(G), Quality::Major, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.degree, 5);
        assert_eq!(rn.label, "V");
    }

    #[test]
    fn test_minor_chord_lowercase() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(A), Quality::Minor, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.degree, 6);
        assert_eq!(rn.label, "vi");
    }

    #[test]
    fn test_diminished_chord() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(B), Quality::Diminished, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.degree, 7);
        assert_eq!(rn.label, "vii°");
    }

    #[test]
    fn test_dominant_seventh() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(G), Quality::Dominant, Number::Seventh);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.label, "V7");
    }

    #[test]
    fn test_half_diminished_seventh() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(B), Quality::HalfDiminished, Number::Seventh);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.label, "viiø7");
    }

    #[test]
    fn test_major_seventh() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::MajorSeventh);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.label, "IΔ7");
    }

    #[test]
    fn test_non_diatonic_returns_none() {
        let key_tonic = Pitch::from(C);
        let chord = Chord::new(Pitch::new(NoteLetter::F, 1), Quality::Major, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord);
        assert!(rn.is_none());
    }

    #[test]
    fn test_g_major_key() {
        let key_tonic = Pitch::from(G);
        // IV in G major = C major
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let rn = roman_numeral(key_tonic, Mode::Ionian, &chord).unwrap();
        assert_eq!(rn.degree, 4);
        assert_eq!(rn.label, "IV");
    }
}
