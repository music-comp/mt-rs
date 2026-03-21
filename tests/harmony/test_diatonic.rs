extern crate mt_rs as theory;
use theory::chord::{Quality, Number};
use theory::harmony;
use theory::note::{NoteLetter, Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod diatonic_tests {
    use super::*;

    #[test]
    fn test_c_major_diatonic_triads() {
        let tonic = Pitch::from(C);
        let chords = harmony::diatonic_triads(tonic, Mode::Ionian);

        assert_eq!(chords.len(), 7);
        // I = C major
        assert_eq!(chords[0].degree, 1);
        assert_eq!(chords[0].quality, Quality::Major);
        assert_eq!(chords[0].root.letter, NoteLetter::C);
        // ii = D minor
        assert_eq!(chords[1].degree, 2);
        assert_eq!(chords[1].quality, Quality::Minor);
        assert_eq!(chords[1].root.letter, NoteLetter::D);
        // iii = E minor
        assert_eq!(chords[2].degree, 3);
        assert_eq!(chords[2].quality, Quality::Minor);
        // IV = F major
        assert_eq!(chords[3].degree, 4);
        assert_eq!(chords[3].quality, Quality::Major);
        // V = G major
        assert_eq!(chords[4].degree, 5);
        assert_eq!(chords[4].quality, Quality::Major);
        // vi = A minor
        assert_eq!(chords[5].degree, 6);
        assert_eq!(chords[5].quality, Quality::Minor);
        // vii° = B diminished
        assert_eq!(chords[6].degree, 7);
        assert_eq!(chords[6].quality, Quality::Diminished);
    }

    #[test]
    fn test_g_major_diatonic_triads() {
        let tonic = Pitch::from(G);
        let chords = harmony::diatonic_triads(tonic, Mode::Ionian);

        assert_eq!(chords.len(), 7);
        assert_eq!(chords[0].root.letter, NoteLetter::G);
        assert_eq!(chords[0].quality, Quality::Major);
        // V = D major
        assert_eq!(chords[4].root.letter, NoteLetter::D);
        assert_eq!(chords[4].quality, Quality::Major);
    }

    #[test]
    fn test_a_minor_diatonic_triads() {
        let tonic = Pitch::from(A);
        let chords = harmony::diatonic_triads(tonic, Mode::Aeolian);

        assert_eq!(chords.len(), 7);
        // i = A minor
        assert_eq!(chords[0].quality, Quality::Minor);
        // ii° = B diminished
        assert_eq!(chords[1].quality, Quality::Diminished);
        // III = C major
        assert_eq!(chords[2].quality, Quality::Major);
        assert_eq!(chords[2].root.letter, NoteLetter::C);
        // iv = D minor
        assert_eq!(chords[3].quality, Quality::Minor);
        // v = E minor
        assert_eq!(chords[4].quality, Quality::Minor);
        // VI = F major
        assert_eq!(chords[5].quality, Quality::Major);
        // VII = G major
        assert_eq!(chords[6].quality, Quality::Major);
    }

    #[test]
    fn test_c_major_diatonic_sevenths() {
        let tonic = Pitch::from(C);
        let chords = harmony::diatonic_sevenths(tonic, Mode::Ionian);

        assert_eq!(chords.len(), 7);
        // Imaj7 = C major seventh
        assert_eq!(chords[0].quality, Quality::Major);
        assert_eq!(chords[0].number, Number::MajorSeventh);
        // ii7 = D minor seventh
        assert_eq!(chords[1].quality, Quality::Minor);
        assert_eq!(chords[1].number, Number::Seventh);
        // V7 = G dominant seventh
        assert_eq!(chords[4].quality, Quality::Dominant);
        assert_eq!(chords[4].number, Number::Seventh);
        // viiø7 = B half-diminished seventh
        assert_eq!(chords[6].quality, Quality::HalfDiminished);
        assert_eq!(chords[6].number, Number::Seventh);
    }

    #[test]
    fn test_non_diatonic_mode_returns_empty() {
        let tonic = Pitch::from(C);
        let chords = harmony::diatonic_triads(tonic, Mode::PentatonicMajor);
        assert!(chords.is_empty());
    }
}
