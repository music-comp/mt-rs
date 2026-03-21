extern crate mt_rs as theory;
use theory::counterpoint::check_first_species;
use theory::note::{Note, NoteLetter, Pitch};

#[cfg(test)]
mod counterpoint_tests {
    use super::*;

    fn n(letter: NoteLetter, acc: i8, oct: u8) -> Note {
        Note::new(Pitch::new(letter, acc), oct)
    }

    #[test]
    fn test_valid_counterpoint() {
        // CF: C4 D4 E4 F4, CP: C5 B4 G4 C5
        // Intervals: P8, M6, m3, P5 — all consonant
        // No parallels, starts/ends on perfect consonance
        let cf = vec![
            n(NoteLetter::C, 0, 4),
            n(NoteLetter::D, 0, 4),
            n(NoteLetter::E, 0, 4),
            n(NoteLetter::F, 0, 4),
        ];
        let cp = vec![
            n(NoteLetter::C, 0, 5),
            n(NoteLetter::B, 0, 4),
            n(NoteLetter::G, 0, 4),
            n(NoteLetter::C, 0, 5),
        ];
        let result = check_first_species(&cf, &cp);
        assert!(result.valid, "Expected valid, got: {:?}", result.violations);
    }

    #[test]
    fn test_parallel_fifths() {
        // C→D in bass, G→A in upper = parallel 5ths
        let cf = vec![n(NoteLetter::C, 0, 4), n(NoteLetter::D, 0, 4)];
        let cp = vec![n(NoteLetter::G, 0, 4), n(NoteLetter::A, 0, 4)];
        let result = check_first_species(&cf, &cp);
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.rule == "parallel_perfect"));
    }

    #[test]
    fn test_parallel_octaves() {
        let cf = vec![n(NoteLetter::C, 0, 4), n(NoteLetter::D, 0, 4)];
        let cp = vec![n(NoteLetter::C, 0, 5), n(NoteLetter::D, 0, 5)];
        let result = check_first_species(&cf, &cp);
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.rule == "parallel_perfect"));
    }

    #[test]
    fn test_dissonant_interval() {
        // C and Db = minor 2nd
        let cf = vec![n(NoteLetter::C, 0, 4)];
        let cp = vec![n(NoteLetter::D, -1, 4)];
        let result = check_first_species(&cf, &cp);
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.rule == "dissonance"));
    }

    #[test]
    fn test_must_start_on_perfect_consonance() {
        // Starting on 3rd (C-E)
        let cf = vec![n(NoteLetter::C, 0, 4), n(NoteLetter::D, 0, 4)];
        let cp = vec![n(NoteLetter::E, 0, 4), n(NoteLetter::D, 0, 5)];
        let result = check_first_species(&cf, &cp);
        assert!(result.violations.iter().any(|v| v.rule == "opening"));
    }

    #[test]
    fn test_must_end_on_perfect_consonance() {
        let cf = vec![n(NoteLetter::C, 0, 4), n(NoteLetter::D, 0, 4)];
        let cp = vec![n(NoteLetter::C, 0, 5), n(NoteLetter::F, 0, 4)];
        let result = check_first_species(&cf, &cp);
        assert!(result.violations.iter().any(|v| v.rule == "closing"));
    }

    #[test]
    fn test_voice_crossing() {
        let cf = vec![n(NoteLetter::E, 0, 4), n(NoteLetter::E, 0, 4)];
        let cp = vec![n(NoteLetter::G, 0, 4), n(NoteLetter::D, 0, 4)];
        let result = check_first_species(&cf, &cp);
        assert!(result.violations.iter().any(|v| v.rule == "voice_crossing"));
    }

    #[test]
    fn test_empty_is_valid() {
        let result = check_first_species(&[], &[]);
        assert!(result.valid);
    }

    #[test]
    fn test_tritone_is_dissonant() {
        // C and F# = tritone (ic 6) = dissonant
        let cf = vec![n(NoteLetter::C, 0, 4)];
        let cp = vec![n(NoteLetter::F, 1, 4)];
        let result = check_first_species(&cf, &cp);
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.rule == "dissonance"));
    }

    #[test]
    fn test_multiple_violations_all_reported() {
        // Dissonant AND wrong opening
        let cf = vec![n(NoteLetter::C, 0, 4)];
        let cp = vec![n(NoteLetter::D, -1, 4)]; // m2: dissonant + not perfect opening
        let result = check_first_species(&cf, &cp);
        assert!(result.violations.len() >= 2);
    }
}
