extern crate mt_rs as theory;
use theory::harmony;
use theory::note::{Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod pivot_tests {
    use super::*;

    #[test]
    fn test_c_major_to_g_major() {
        let pivots =
            harmony::pivot_chords(Pitch::from(C), Mode::Ionian, Pitch::from(G), Mode::Ionian);
        // G major is V in C and I in G
        assert!(pivots
            .iter()
            .any(|p| p.roman_in_a.degree == 5 && p.roman_in_b.degree == 1));
        // C major is I in C and IV in G
        assert!(pivots
            .iter()
            .any(|p| p.roman_in_a.degree == 1 && p.roman_in_b.degree == 4));
    }

    #[test]
    fn test_c_major_to_a_minor() {
        let pivots =
            harmony::pivot_chords(Pitch::from(C), Mode::Ionian, Pitch::from(A), Mode::Aeolian);
        // Relative major/minor share many chords
        assert!(pivots.len() >= 3);
    }

    #[test]
    fn test_distant_keys_fewer_pivots() {
        let close =
            harmony::pivot_chords(Pitch::from(C), Mode::Ionian, Pitch::from(G), Mode::Ionian);
        let distant =
            harmony::pivot_chords(Pitch::from(C), Mode::Ionian, Pitch::from(Fs), Mode::Ionian);
        assert!(close.len() > distant.len());
    }

    #[test]
    fn test_same_key_all_chords_are_pivots() {
        let pivots =
            harmony::pivot_chords(Pitch::from(C), Mode::Ionian, Pitch::from(C), Mode::Ionian);
        assert_eq!(pivots.len(), 7);
    }
}
