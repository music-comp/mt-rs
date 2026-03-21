extern crate mt_rs as theory;
use theory::figured_bass::{Figure, realize};
use theory::note::{Note, NoteLetter, Pitch, PitchSymbol::*};
use theory::scale::Mode;

#[cfg(test)]
mod figured_bass_tests {
    use super::*;

    fn n(letter: NoteLetter, acc: i8, oct: u8) -> Note {
        Note::new(Pitch::new(letter, acc), oct)
    }

    #[test]
    fn test_root_position_triad() {
        // C bass, no figures (implied 5/3) in C major = C E G
        let bass = vec![n(NoteLetter::C, 0, 3)];
        let figures: Vec<Vec<Figure>> = vec![vec![]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].bass.pitch.as_u8(), 0);
        assert_eq!(result[0].upper_voices.len(), 2);
        let upper_pcs: Vec<u8> = result[0].upper_voices.iter().map(|n| n.pitch.as_u8()).collect();
        assert!(upper_pcs.contains(&4)); // E
        assert!(upper_pcs.contains(&7)); // G
    }

    #[test]
    fn test_first_inversion_with_6() {
        // E bass with figure "6" in C major = E with G and C above
        let bass = vec![n(NoteLetter::E, 0, 3)];
        let figures = vec![vec![Figure { interval: 6, accidental: 0 }]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        assert_eq!(result.len(), 1);
        let upper_pcs: Vec<u8> = result[0].upper_voices.iter().map(|n| n.pitch.as_u8()).collect();
        assert!(upper_pcs.contains(&7)); // G
        assert!(upper_pcs.contains(&0)); // C
    }

    #[test]
    fn test_seventh_chord() {
        // G bass with "7" in C major = G B D F
        let bass = vec![n(NoteLetter::G, 0, 3)];
        let figures = vec![vec![Figure { interval: 7, accidental: 0 }]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        let upper_pcs: Vec<u8> = result[0].upper_voices.iter().map(|n| n.pitch.as_u8()).collect();
        assert!(upper_pcs.contains(&11)); // B
        assert!(upper_pcs.contains(&2));  // D
        assert!(upper_pcs.contains(&5));  // F
    }

    #[test]
    fn test_sharp_accidental() {
        // C bass with #3 in C major = raised third (E→E#=F enharmonic)
        let bass = vec![n(NoteLetter::C, 0, 3)];
        let figures = vec![vec![Figure { interval: 3, accidental: 1 }]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        let upper_pcs: Vec<u8> = result[0].upper_voices.iter().map(|n| n.pitch.as_u8()).collect();
        assert!(upper_pcs.contains(&5)); // E# = semitone 5
        assert!(upper_pcs.contains(&7)); // G (implied 5th)
    }

    #[test]
    fn test_multiple_bass_notes() {
        let bass = vec![n(NoteLetter::C, 0, 3), n(NoteLetter::F, 0, 3)];
        let figures = vec![vec![], vec![]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_empty_input() {
        let result = realize(&[], &[], Pitch::from(C), Mode::Ionian);
        assert!(result.is_empty());
    }

    #[test]
    fn test_upper_voices_above_bass() {
        // All upper voices should be at or above the bass octave
        let bass = vec![n(NoteLetter::C, 0, 3)];
        let figures: Vec<Vec<Figure>> = vec![vec![]];
        let result = realize(&bass, &figures, Pitch::from(C), Mode::Ionian);
        for voice in &result[0].upper_voices {
            assert!(voice.octave >= 3, "Upper voice should be at or above bass octave");
        }
    }

    #[test]
    fn test_f_major_context() {
        // F bass, no figures, in F major = F A C
        let bass = vec![n(NoteLetter::F, 0, 3)];
        let figures: Vec<Vec<Figure>> = vec![vec![]];
        let result = realize(&bass, &figures, Pitch::from(F), Mode::Ionian);
        let upper_pcs: Vec<u8> = result[0].upper_voices.iter().map(|n| n.pitch.as_u8()).collect();
        assert!(upper_pcs.contains(&9)); // A
        assert!(upper_pcs.contains(&0)); // C
    }
}
