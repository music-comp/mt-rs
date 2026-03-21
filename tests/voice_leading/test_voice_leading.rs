extern crate rust_music_theory as theory;
use theory::note::{Note, NoteLetter, Pitch};
use theory::voice_leading;

#[cfg(test)]
mod voice_leading_tests {
    use super::*;

    fn note(letter: NoteLetter, acc: i8, octave: u8) -> Note {
        Note::new(Pitch::new(letter, acc), octave)
    }

    #[test]
    fn test_zero_movement() {
        let chord = vec![
            note(NoteLetter::C, 0, 4),
            note(NoteLetter::E, 0, 4),
            note(NoteLetter::G, 0, 4),
        ];
        let vl = voice_leading::minimal_movement(&chord, &chord);
        assert_eq!(vl.total_distance, 0);
        assert_eq!(vl.movements.len(), 3);
    }

    #[test]
    fn test_c_major_to_f_major() {
        // C4-E4-G4 to C4-F4-A4
        // Optimal: C→C(0), E→F(1), G→A(2) = 3 total
        let from = vec![
            note(NoteLetter::C, 0, 4),
            note(NoteLetter::E, 0, 4),
            note(NoteLetter::G, 0, 4),
        ];
        let to = vec![
            note(NoteLetter::C, 0, 4),
            note(NoteLetter::F, 0, 4),
            note(NoteLetter::A, 0, 4),
        ];
        let vl = voice_leading::minimal_movement(&from, &to);
        assert_eq!(vl.total_distance, 3);
    }

    #[test]
    fn test_c_major_to_e_minor() {
        // C4-E4-G4 to B3-E4-G4
        // Optimal: C→B(-1), E→E(0), G→G(0) = 1 total
        let from = vec![
            note(NoteLetter::C, 0, 4),
            note(NoteLetter::E, 0, 4),
            note(NoteLetter::G, 0, 4),
        ];
        let to = vec![
            note(NoteLetter::B, 0, 3),
            note(NoteLetter::E, 0, 4),
            note(NoteLetter::G, 0, 4),
        ];
        let vl = voice_leading::minimal_movement(&from, &to);
        assert_eq!(vl.total_distance, 1);
    }

    #[test]
    fn test_movement_direction() {
        let from = vec![note(NoteLetter::C, 0, 4)];
        let to = vec![note(NoteLetter::E, 0, 4)];
        let vl = voice_leading::minimal_movement(&from, &to);
        assert_eq!(vl.movements[0].semitones, 4); // up
    }

    #[test]
    fn test_downward_movement() {
        let from = vec![note(NoteLetter::E, 0, 4)];
        let to = vec![note(NoteLetter::C, 0, 4)];
        let vl = voice_leading::minimal_movement(&from, &to);
        assert_eq!(vl.movements[0].semitones, -4); // down
    }

    #[test]
    fn test_empty_returns_zero() {
        let vl = voice_leading::minimal_movement(&[], &[]);
        assert_eq!(vl.total_distance, 0);
        assert!(vl.movements.is_empty());
    }

    #[test]
    fn test_finds_optimal_permutation() {
        // If we naively pair by index, we'd get:
        // G4→C4(-7), C4→E4(+4), E4→G4(+3) = 14
        // But optimal is: G4→G4(0), C4→C4(0), E4→E4(0)... wait, those aren't the same notes.
        // Better example: from [C4, G4] to [G4, C5]
        // Naive: C→G(7), G→C(5) = 12
        // Optimal: C→C5(12)? No... let's do from [E4, C4] to [C4, E4]
        // Naive: E→C(-4), C→E(4) = 8
        // Optimal: E→E(0), C→C(0) = 0
        let from = vec![
            note(NoteLetter::E, 0, 4),
            note(NoteLetter::C, 0, 4),
        ];
        let to = vec![
            note(NoteLetter::C, 0, 4),
            note(NoteLetter::E, 0, 4),
        ];
        let vl = voice_leading::minimal_movement(&from, &to);
        assert_eq!(vl.total_distance, 0); // optimal swaps the pairing
    }
}
