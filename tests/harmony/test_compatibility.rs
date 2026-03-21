extern crate mt_rs as theory;
use theory::chord::{Chord, Number, Quality};
use theory::harmony;
use theory::note::{Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn test_c_major_triad_compatible_scales() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let scales = harmony::compatible_scales(&chord);
        assert!(scales
            .iter()
            .any(|(p, m)| p.as_u8() == 0 && *m == Mode::Ionian));
        assert!(scales
            .iter()
            .any(|(p, m)| p.as_u8() == 5 && *m == Mode::Lydian));
    }

    #[test]
    fn test_dm7_compatible_scales() {
        let chord = Chord::new(Pitch::from(D), Quality::Minor, Number::Seventh);
        let scales = harmony::compatible_scales(&chord);
        assert!(scales
            .iter()
            .any(|(p, m)| p.as_u8() == 2 && *m == Mode::Dorian));
    }

    #[test]
    fn test_returns_nonempty() {
        let chord = Chord::new(Pitch::from(G), Quality::Major, Number::Triad);
        let scales = harmony::compatible_scales(&chord);
        assert!(!scales.is_empty());
    }
}
