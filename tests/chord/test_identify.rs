extern crate mt_rs as theory;
use theory::chord::{Chord, Quality, Number};
use theory::note::{NoteLetter, Pitch, PitchSymbol::*};

#[cfg(test)]
mod identify_tests {
    use super::*;

    #[test]
    fn test_identify_root_position() {
        let notes: Vec<Pitch> = vec![C, E, G].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(!matches.is_empty());
        let first = &matches[0];
        assert_eq!(first.root.letter, NoteLetter::C);
        assert_eq!(first.quality, Quality::Major);
        assert_eq!(first.number, Number::Triad);
        assert_eq!(first.inversion, 0);
    }

    #[test]
    fn test_identify_first_inversion() {
        // E G C = C major first inversion
        let notes: Vec<Pitch> = vec![E, G, C].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.iter().any(|c|
            c.root.letter == NoteLetter::C
            && c.quality == Quality::Major
            && c.inversion == 1
        ));
    }

    #[test]
    fn test_identify_unordered_input() {
        // C G E (not sorted) should still find C major
        let notes: Vec<Pitch> = vec![C, G, E].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.iter().any(|c|
            c.root.letter == NoteLetter::C
            && c.quality == Quality::Major
        ));
    }

    #[test]
    fn test_identify_minor_chord() {
        let notes: Vec<Pitch> = vec![A, C, E].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.iter().any(|c|
            c.root.letter == NoteLetter::A
            && c.quality == Quality::Minor
        ));
    }

    #[test]
    fn test_identify_seventh_chord() {
        let notes: Vec<Pitch> = vec![G, B, D, F].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.iter().any(|c|
            c.root.letter == NoteLetter::G
            && c.quality == Quality::Dominant
            && c.number == Number::Seventh
        ));
    }

    #[test]
    fn test_identify_second_inversion() {
        // G C E = C major second inversion
        let notes: Vec<Pitch> = vec![G, C, E].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.iter().any(|c|
            c.root.letter == NoteLetter::C
            && c.quality == Quality::Major
            && c.inversion == 2
        ));
    }

    #[test]
    fn test_identify_empty_returns_empty() {
        let matches = Chord::identify(&[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_identify_single_note_returns_empty() {
        let notes: Vec<Pitch> = vec![C].into_iter().map(Pitch::from).collect();
        let matches = Chord::identify(&notes);
        assert!(matches.is_empty());
    }
}
