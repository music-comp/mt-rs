extern crate rust_music_theory as theory;
use theory::note::{NoteLetter, Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod identify_tests {
    use super::*;
    use theory::scale::Scale;

    #[test]
    fn test_identify_c_major() {
        let notes: Vec<Pitch> = vec![C, D, E, F, G, A, B]
            .into_iter()
            .map(Pitch::from)
            .collect();
        let matches = Scale::identify(&notes);
        assert!(matches.iter().any(|(p, m)|
            p.letter == NoteLetter::C && p.accidental == 0 && *m == Mode::Ionian
        ));
    }

    #[test]
    fn test_identify_d_dorian() {
        let notes: Vec<Pitch> = vec![D, E, F, G, A, B, C]
            .into_iter()
            .map(Pitch::from)
            .collect();
        let matches = Scale::identify(&notes);
        // D Dorian should be found
        assert!(matches.iter().any(|(p, m)|
            p.letter == NoteLetter::D && p.accidental == 0 && *m == Mode::Dorian
        ));
        // C Ionian has the same notes — should also match
        assert!(matches.iter().any(|(p, m)|
            p.letter == NoteLetter::C && p.accidental == 0 && *m == Mode::Ionian
        ));
    }

    #[test]
    fn test_identify_pentatonic_from_subset() {
        let notes: Vec<Pitch> = vec![G, A, B, D, E]
            .into_iter()
            .map(Pitch::from)
            .collect();
        let matches = Scale::identify(&notes);
        // G major pentatonic = G A B D E (exact match)
        assert!(matches.iter().any(|(p, m)|
            p.letter == NoteLetter::G && *m == Mode::PentatonicMajor
        ));
    }

    #[test]
    fn test_identify_a_minor() {
        let notes: Vec<Pitch> = vec![A, B, C, D, E, F, G]
            .into_iter()
            .map(Pitch::from)
            .collect();
        let matches = Scale::identify(&notes);
        assert!(matches.iter().any(|(p, m)|
            p.letter == NoteLetter::A && *m == Mode::Aeolian
        ));
    }

    #[test]
    fn test_identify_empty_returns_empty() {
        let matches = Scale::identify(&[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_identify_results_sorted_by_size() {
        // G A B D E matches both G pentatonic major (5 notes) and G major (7 notes)
        let notes: Vec<Pitch> = vec![G, A, B, D, E]
            .into_iter()
            .map(Pitch::from)
            .collect();
        let matches = Scale::identify(&notes);
        // Pentatonic (smaller) should come before diatonic (larger)
        let pent_pos = matches.iter().position(|(_, m)| *m == Mode::PentatonicMajor);
        let ionian_pos = matches.iter().position(|(p, m)|
            p.letter == NoteLetter::G && *m == Mode::Ionian
        );
        if let (Some(pp), Some(ip)) = (pent_pos, ionian_pos) {
            assert!(pp < ip, "Pentatonic should sort before diatonic");
        }
    }
}
