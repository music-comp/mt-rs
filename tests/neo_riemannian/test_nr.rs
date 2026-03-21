extern crate mt_rs as theory;
use theory::chord::{Chord, Number, Quality};
use theory::neo_riemannian::{transform, transform_chain, NROperation};
use theory::note::{Pitch, PitchSymbol::*};

#[cfg(test)]
mod nr_tests {
    use super::*;

    #[test]
    fn test_p_major_to_minor() {
        let c_major = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result = transform(&c_major, NROperation::P).unwrap();
        assert_eq!(result.quality, Quality::Minor);
        assert_eq!(result.root.as_u8(), 0);
    }

    #[test]
    fn test_p_minor_to_major() {
        let c_minor = Chord::new(Pitch::from(C), Quality::Minor, Number::Triad);
        let result = transform(&c_minor, NROperation::P).unwrap();
        assert_eq!(result.quality, Quality::Major);
        assert_eq!(result.root.as_u8(), 0);
    }

    #[test]
    fn test_r_c_major_to_a_minor() {
        let c_major = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result = transform(&c_major, NROperation::R).unwrap();
        assert_eq!(result.quality, Quality::Minor);
        assert_eq!(result.root.as_u8(), 9); // A
    }

    #[test]
    fn test_r_a_minor_to_c_major() {
        let a_minor = Chord::new(Pitch::from(A), Quality::Minor, Number::Triad);
        let result = transform(&a_minor, NROperation::R).unwrap();
        assert_eq!(result.quality, Quality::Major);
        assert_eq!(result.root.as_u8(), 0); // C
    }

    #[test]
    fn test_l_c_major_to_e_minor() {
        let c_major = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result = transform(&c_major, NROperation::L).unwrap();
        assert_eq!(result.quality, Quality::Minor);
        assert_eq!(result.root.as_u8(), 4); // E
    }

    #[test]
    fn test_l_e_minor_to_c_major() {
        let e_minor = Chord::new(Pitch::from(E), Quality::Minor, Number::Triad);
        let result = transform(&e_minor, NROperation::L).unwrap();
        assert_eq!(result.quality, Quality::Major);
        assert_eq!(result.root.as_u8(), 0); // C
    }

    #[test]
    fn test_p_is_involution() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result =
            transform(&transform(&chord, NROperation::P).unwrap(), NROperation::P).unwrap();
        assert_eq!(result.root.as_u8(), chord.root.as_u8());
        assert_eq!(result.quality, chord.quality);
    }

    #[test]
    fn test_r_is_involution() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result =
            transform(&transform(&chord, NROperation::R).unwrap(), NROperation::R).unwrap();
        assert_eq!(result.root.as_u8(), chord.root.as_u8());
        assert_eq!(result.quality, chord.quality);
    }

    #[test]
    fn test_l_is_involution() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let result =
            transform(&transform(&chord, NROperation::L).unwrap(), NROperation::L).unwrap();
        assert_eq!(result.root.as_u8(), chord.root.as_u8());
        assert_eq!(result.quality, chord.quality);
    }

    #[test]
    fn test_chain_returns_intermediates() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let chain =
            transform_chain(&chord, &[NROperation::P, NROperation::R, NROperation::L]).unwrap();
        assert_eq!(chain.len(), 4); // original + 3 transforms
        assert_eq!(chain[0].root.as_u8(), 0); // C major
        assert_eq!(chain[1].quality, Quality::Minor); // C minor (after P)
    }

    #[test]
    fn test_non_triad_returns_error() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Seventh);
        assert!(transform(&chord, NROperation::P).is_err());
    }

    #[test]
    fn test_diminished_returns_error() {
        let chord = Chord::new(Pitch::from(C), Quality::Diminished, Number::Triad);
        assert!(transform(&chord, NROperation::P).is_err());
    }
}
