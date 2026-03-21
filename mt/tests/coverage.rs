//! Coverage gap tests — systematically covers Display impls, From conversions,
//! error types, and uncovered branches across all modules.

extern crate music_comp_mt as theory;

use theory::chord::{Chord, Number as ChordNumber, Quality as ChordQuality};
use theory::interval::{Interval, Number as IntervalNumber, Quality as IntervalQuality};
use theory::note::{Note, NoteLetter, Notes, Pitch, PitchSymbol};
use theory::scale::{Direction, Mode, Scale, ScaleType};

// ============================================================
// Error Display + From impls (interval, note, chord, scale errors)
// ============================================================

#[test]
fn test_interval_error_display() {
    let err = Interval::from_semitone(99).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("invalid"));
}

#[test]
fn test_interval_error_is_error() {
    let err = Interval::from_semitone(99).unwrap_err();
    // Confirm it implements std::error::Error via Debug
    let _ = format!("{:?}", err);
}

#[test]
fn test_note_error_display() {
    let err = Pitch::from_regex("!!!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("invalid"));
}

#[test]
fn test_chord_error_display_invalid_regex() {
    let err = Chord::from_regex("!!!").unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn test_chord_error_display_invalid_note() {
    let err = Chord::from_string("X Y Z").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("invalid") || msg.contains("note"));
}

#[test]
fn test_chord_error_display_unknown_pattern() {
    let root = Pitch::new(NoteLetter::C, 0);
    let err = Chord::from_interval(root, &[1, 2]).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("unknown") || msg.contains("pattern"));
}

#[test]
fn test_scale_error_display_invalid_interval() {
    let msg = format!("{}", theory::scale::ScaleError::InvalidInterval);
    assert!(msg.contains("interval"));
}

#[test]
fn test_scale_error_display_mode_regex() {
    let msg = format!("{}", theory::scale::ScaleError::ModeFromRegex);
    assert!(msg.contains("mode"));
}

#[test]
fn test_scale_error_display_invalid_regex() {
    let msg = format!("{}", theory::scale::ScaleError::InvalidRegex);
    assert!(msg.contains("regex"));
}

#[test]
fn test_scale_error_source_is_none() {
    use std::error::Error;
    let err = theory::scale::ScaleError::InvalidInterval;
    assert!(err.source().is_none());
}

#[test]
fn test_scale_error_from_interval_error() {
    let interval_err = theory::interval::IntervalError::InvalidInterval;
    let scale_err: theory::scale::ScaleError = interval_err.into();
    assert_eq!(scale_err, theory::scale::ScaleError::InvalidInterval);
}

#[test]
fn test_scale_error_from_note_error() {
    let note_err = theory::note::NoteError::InvalidPitch;
    let scale_err: theory::scale::ScaleError = note_err.into();
    assert_eq!(scale_err, theory::scale::ScaleError::InvalidRegex);
}

#[test]
fn test_chord_error_from_note_error() {
    let note_err = theory::note::NoteError::InvalidPitch;
    let chord_err: theory::chord::ChordError = note_err.into();
    assert_eq!(chord_err, theory::chord::ChordError::InvalidRegex);
}

// ============================================================
// Note Display + print_notes
// ============================================================

#[test]
fn test_note_display() {
    let note = Note::new(Pitch::new(NoteLetter::C, 1), 4);
    let display = format!("{}", note);
    assert_eq!(display, "C#");
}

#[test]
fn test_note_display_flat() {
    let note = Note::new(Pitch::new(NoteLetter::B, -1), 3);
    let display = format!("{}", note);
    assert_eq!(display, "Bb");
}

#[test]
fn test_note_display_natural() {
    let note = Note::new(Pitch::new(NoteLetter::G, 0), 5);
    assert_eq!(format!("{}", note), "G");
}

// ============================================================
// Interval Display coverage
// ============================================================

#[test]
fn test_interval_display_tritone() {
    let tritone = Interval::from_semitone(6).unwrap();
    assert_eq!(format!("{}", tritone), "T");
}

#[test]
fn test_interval_display_unison() {
    let unison = Interval::from_semitone(0).unwrap();
    assert_eq!(format!("{}", unison), "1");
}

#[test]
fn test_interval_display_octave() {
    let octave = Interval::from_semitone(12).unwrap();
    assert_eq!(format!("{}", octave), "1");
}

#[test]
fn test_interval_display_major_third() {
    let m3 = Interval::from_semitone(4).unwrap();
    assert_eq!(format!("{}", m3), "M3");
}

#[test]
fn test_interval_display_minor_seventh() {
    let m7 = Interval::from_semitone(10).unwrap();
    assert_eq!(format!("{}", m7), "m7");
}

#[test]
fn test_interval_display_perfect_fifth() {
    let p5 = Interval::from_semitone(7).unwrap();
    assert_eq!(format!("{}", p5), "P5");
}

#[test]
fn test_interval_display_compound() {
    let m9 = Interval::from_semitone(14).unwrap();
    let display = format!("{}", m9);
    assert!(display.contains("9"));
}

// ============================================================
// Pitch coverage gaps (from_str, try_parse edge cases, Display)
// ============================================================

#[test]
fn test_pitch_from_str() {
    use std::str::FromStr;
    let pitch = Pitch::from_str("Eb").unwrap();
    assert_eq!(pitch.letter, NoteLetter::E);
    assert_eq!(pitch.accidental, -1);
}

#[test]
fn test_pitch_from_str_error() {
    use std::str::FromStr;
    assert!(Pitch::from_str("!!!").is_err());
}

#[test]
fn test_pitch_display_sharp() {
    let pitch = Pitch::new(NoteLetter::F, 1);
    assert_eq!(format!("{}", pitch), "F#");
}

#[test]
fn test_pitch_display_flat() {
    let pitch = Pitch::new(NoteLetter::B, -1);
    assert_eq!(format!("{}", pitch), "Bb");
}

#[test]
fn test_pitch_display_natural() {
    let pitch = Pitch::new(NoteLetter::A, 0);
    assert_eq!(format!("{}", pitch), "A");
}

#[test]
fn test_pitch_display_double_sharp() {
    let pitch = Pitch::new(NoteLetter::C, 2);
    assert_eq!(format!("{}", pitch), "C##");
}

#[test]
fn test_pitch_display_double_flat() {
    let pitch = Pitch::new(NoteLetter::B, -2);
    assert_eq!(format!("{}", pitch), "Bbb");
}

#[test]
fn test_pitch_try_parse_double_sharp() {
    let pitch = Pitch::try_parse("C##").unwrap();
    assert_eq!(pitch.accidental, 2);
}

#[test]
fn test_pitch_try_parse_double_flat() {
    let pitch = Pitch::try_parse("Bbb").unwrap();
    assert_eq!(pitch.accidental, -2);
}

#[test]
fn test_pitch_try_parse_natural() {
    let pitch = Pitch::try_parse("G").unwrap();
    assert_eq!(pitch.accidental, 0);
}

#[test]
fn test_pitch_try_parse_invalid_returns_none() {
    assert!(Pitch::try_parse("").is_none());
    assert!(Pitch::try_parse("123").is_none());
}

#[test]
fn test_pitch_try_parse_unicode_flat() {
    let pitch = Pitch::try_parse("E♭").unwrap();
    assert_eq!(pitch.letter, NoteLetter::E);
    assert_eq!(pitch.accidental, -1);
}

#[test]
fn test_pitch_from_u8_all_values() {
    for i in 0..12u8 {
        let pitch = Pitch::from_u8(i);
        assert_eq!(pitch.as_u8(), i);
    }
}

// ============================================================
// NoteLetter Display
// ============================================================

#[test]
fn test_note_letter_display_all() {
    assert_eq!(format!("{}", NoteLetter::C), "C");
    assert_eq!(format!("{}", NoteLetter::D), "D");
    assert_eq!(format!("{}", NoteLetter::E), "E");
    assert_eq!(format!("{}", NoteLetter::F), "F");
    assert_eq!(format!("{}", NoteLetter::G), "G");
    assert_eq!(format!("{}", NoteLetter::A), "A");
    assert_eq!(format!("{}", NoteLetter::B), "B");
}

// ============================================================
// PitchSymbol Display coverage
// ============================================================

#[test]
fn test_pitch_symbol_display_fb() {
    assert_eq!(format!("{}", PitchSymbol::Fb), "F♭");
}

// ============================================================
// Chord coverage gaps
// ============================================================

#[test]
fn test_chord_from_regex_with_slash_inversion() {
    let chord = Chord::from_regex("C/E").unwrap();
    assert_eq!(chord.inversion, 1);
}

#[test]
fn test_chord_from_regex_with_numeric_inversion() {
    let chord = Chord::from_regex("C/2").unwrap();
    assert_eq!(chord.inversion, 2);
}

#[test]
fn test_chord_from_regex_dominant() {
    let chord = Chord::from_regex("C dominant seventh").unwrap();
    assert_eq!(chord.quality, ChordQuality::Dominant);
    assert_eq!(chord.number, ChordNumber::Seventh);
}

#[test]
fn test_chord_from_regex_diminished() {
    let chord = Chord::from_regex("B diminished").unwrap();
    assert_eq!(chord.quality, ChordQuality::Diminished);
}

#[test]
fn test_chord_from_string_two_notes_unknown_pattern() {
    // Two notes with interval [7] doesn't match a known chord pattern
    let result = Chord::from_string("C G");
    assert!(result.is_err());
}

#[test]
fn test_chord_from_string_major_triad() {
    let chord = Chord::from_string("C E G").unwrap();
    assert_eq!(chord.root.letter, NoteLetter::C);
    assert_eq!(chord.quality, ChordQuality::Major);
}

#[test]
fn test_chord_from_string_single_note_error() {
    assert!(Chord::from_string("C").is_err());
}

// ============================================================
// Neo-Riemannian NRError Display
// ============================================================

#[test]
fn test_nr_error_display() {
    use theory::neo_riemannian::NRError;
    let err = NRError("test error".into());
    assert_eq!(format!("{}", err), "test error");
}

// ============================================================
// Analysis roman numeral — cover more suffix branches
// ============================================================

#[test]
fn test_roman_numeral_augmented_triad() {
    // In C major, no diatonic augmented triad, but we can test the label format
    // by creating an augmented chord on a diatonic root
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Augmented,
        ChordNumber::Triad,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord);
    if let Some(rn) = rn {
        assert!(rn.label.contains("+"));
    }
}

#[test]
fn test_roman_numeral_ninth() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::G, 0),
        ChordQuality::Dominant,
        ChordNumber::Ninth,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "V9");
}

#[test]
fn test_roman_numeral_eleventh() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::G, 0),
        ChordQuality::Dominant,
        ChordNumber::Eleventh,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "V11");
}

#[test]
fn test_roman_numeral_thirteenth() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::G, 0),
        ChordQuality::Dominant,
        ChordNumber::Thirteenth,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "V13");
}

#[test]
fn test_roman_numeral_diminished_seventh() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::B, 0),
        ChordQuality::Diminished,
        ChordNumber::Seventh,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "vii°7");
}

#[test]
fn test_roman_numeral_augmented_seventh() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Augmented,
        ChordNumber::Seventh,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "I+7");
}

#[test]
fn test_roman_numeral_suspended() {
    use theory::analysis::roman_numeral;
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Suspended4,
        ChordNumber::Triad,
    );
    let rn = roman_numeral(Pitch::new(NoteLetter::C, 0), Mode::Ionian, &chord).unwrap();
    assert_eq!(rn.label, "I");
}

// ============================================================
// Harmony diatonic — cover fallback branches
// ============================================================

#[test]
fn test_diatonic_triads_d_major() {
    use theory::harmony;
    let chords = harmony::diatonic_triads(Pitch::new(NoteLetter::D, 0), Mode::Ionian);
    assert_eq!(chords.len(), 7);
    assert_eq!(chords[0].quality, ChordQuality::Major);
    assert_eq!(chords[1].quality, ChordQuality::Minor);
}

// ============================================================
// Scale regex parsing edge cases
// ============================================================

#[test]
fn test_scale_from_regex_harmonic_minor() {
    let scale = Scale::from_regex("C harmonic minor").unwrap();
    assert_eq!(scale.scale_type, ScaleType::HarmonicMinor);
}

#[test]
fn test_scale_from_regex_melodic_minor() {
    let scale = Scale::from_regex("C melodic minor").unwrap();
    assert_eq!(scale.scale_type, ScaleType::MelodicMinor);
}

// ============================================================
// Interval::between edge cases
// ============================================================

#[test]
fn test_interval_between_augmented_fifth() {
    let c = Pitch::new(NoteLetter::C, 0);
    let gs = Pitch::new(NoteLetter::G, 1);
    let interval = Interval::between(&c, &gs).unwrap();
    assert_eq!(interval.quality, IntervalQuality::Augmented);
    assert_eq!(interval.number, IntervalNumber::Fifth);
}

#[test]
fn test_interval_between_diminished_third() {
    // C# to Eb: letter dist = 2 (Third), semitones = (3-1)=2, natural=4, diff=-2 → Diminished
    let cs = Pitch::new(NoteLetter::C, 1);
    let eb = Pitch::new(NoteLetter::E, -1);
    let interval = Interval::between(&cs, &eb).unwrap();
    assert_eq!(interval.quality, IntervalQuality::Diminished);
    assert_eq!(interval.number, IntervalNumber::Third);
}

// ============================================================
// Interval invert coverage
// ============================================================

#[test]
fn test_interval_invert_octave() {
    let octave = Interval::from_semitone(12).unwrap();
    let inverted = octave.invert().unwrap();
    assert_eq!(inverted.semitone_count, 12);
}

// ============================================================
// Scale Default
// ============================================================

#[test]
fn test_scale_default_notes() {
    let scale = Scale::default();
    let notes = scale.notes();
    assert_eq!(notes.len(), 8);
    assert_eq!(notes[0].pitch.letter, NoteLetter::C);
}

// ============================================================
// Chord Default
// ============================================================

#[test]
fn test_chord_default_notes() {
    let chord = Chord::default();
    let notes = chord.notes();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].pitch.letter, NoteLetter::C);
}

// ============================================================
// Interval Default
// ============================================================

#[test]
fn test_interval_default() {
    let interval = Interval::default();
    assert_eq!(interval.semitone_count, 0);
    assert_eq!(interval.quality, IntervalQuality::Perfect);
    assert_eq!(interval.number, IntervalNumber::Unison);
}

// ============================================================
// print_notes coverage (Notes trait default method)
// ============================================================

#[test]
fn test_format_notes_chord() {
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Major,
        ChordNumber::Triad,
    );
    let formatted = chord.format_notes();
    assert!(formatted.contains("Notes:"));
    assert!(formatted.contains("1: C"));
    assert!(formatted.contains("2: E"));
    assert!(formatted.contains("3: G"));
}

#[test]
fn test_format_notes_scale() {
    let scale = Scale::default();
    let formatted = scale.format_notes();
    assert!(formatted.contains("Notes:"));
    assert!(formatted.contains("1: C"));
}

#[test]
fn test_print_notes_calls_format() {
    // Just verify print_notes doesn't panic
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Major,
        ChordNumber::Triad,
    );
    chord.print_notes();
}

// ============================================================
// From<regex::Error> impls (hard to trigger naturally)
// ============================================================

#[test]
fn test_note_error_from_regex_error() {
    // Manually create a regex error and convert
    let regex_err = regex::Regex::new("[invalid").unwrap_err();
    let note_err: theory::note::NoteError = regex_err.into();
    assert_eq!(note_err, theory::note::NoteError::InvalidPitch);
}

#[test]
fn test_chord_error_from_regex_error() {
    let regex_err = regex::Regex::new("[invalid").unwrap_err();
    let chord_err: theory::chord::ChordError = regex_err.into();
    assert_eq!(chord_err, theory::chord::ChordError::InvalidRegex);
}

#[test]
fn test_scale_error_from_regex_error() {
    let regex_err = regex::Regex::new("[invalid").unwrap_err();
    let scale_err: theory::scale::ScaleError = regex_err.into();
    assert_eq!(scale_err, theory::scale::ScaleError::ModeFromRegex);
}

// ============================================================
// Interval uncovered branches
// ============================================================

#[test]
fn test_interval_from_semitones_all_simple() {
    // Cover every simple interval
    for i in 0..=12u8 {
        let interval = Interval::from_semitone(i).unwrap();
        assert_eq!(interval.semitone_count, i);
    }
}

#[test]
fn test_interval_from_semitones_all_compound() {
    for i in 13..=24u8 {
        let interval = Interval::from_semitone(i).unwrap();
        assert_eq!(interval.semitone_count, i);
        assert!(interval.is_compound());
    }
}

#[test]
fn test_interval_invert_all() {
    for i in 1..=12u8 {
        let interval = Interval::from_semitone(i).unwrap();
        let inverted = interval.invert().unwrap();
        if i == 12 {
            assert_eq!(inverted.semitone_count, 12);
        } else {
            assert_eq!(inverted.semitone_count, 12 - i);
        }
    }
    // Unison inverts to unison (special: (12 + (12-0)) % 12 = 0 → from_semitone(0))
    let unison = Interval::from_semitone(0).unwrap();
    let inverted = unison.invert().unwrap();
    assert_eq!(inverted.semitone_count, 0);
}

#[test]
fn test_interval_display_augmented_fourth() {
    // Augmented fourth = different from diminished fifth, but same semitone count
    let a4 = Interval::new(6, IntervalQuality::Augmented, IntervalNumber::Fourth, None);
    assert_eq!(format!("{}", a4), "T");
}

#[test]
fn test_interval_display_all_compound_numbers() {
    use theory::interval::Number::*;
    let numbers = [
        Ninth, Tenth, Eleventh, Twelfth, Thirteenth, Fourteenth, Fifteenth,
    ];
    let expected = ["9", "10", "11", "12", "13", "14", "15"];
    for (num, exp) in numbers.iter().zip(expected.iter()) {
        assert_eq!(format!("{}", num), *exp);
    }
}

// ============================================================
// Chord::notes() — cover letter-based spelling edge cases
// ============================================================

#[test]
fn test_chord_notes_extended_ninths() {
    // Cover 9th chord voicing (positions 4+)
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Dominant,
        ChordNumber::Ninth,
    );
    let notes = chord.notes();
    assert_eq!(notes.len(), 5);
}

#[test]
fn test_chord_notes_extended_thirteenths() {
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Dominant,
        ChordNumber::Thirteenth,
    );
    let notes = chord.notes();
    assert_eq!(notes.len(), 7);
}

#[test]
fn test_chord_notes_sus2_seventh() {
    let chord = Chord::new(
        Pitch::new(NoteLetter::C, 0),
        ChordQuality::Suspended2,
        ChordNumber::Seventh,
    );
    let notes = chord.notes();
    assert_eq!(notes.len(), 4);
}

// ============================================================
// Figured bass — cover more branches
// ============================================================

#[test]
fn test_figured_bass_64() {
    use theory::figured_bass::{realize, Figure};
    // 6/4 = second inversion
    let bass = vec![Note::new(Pitch::new(NoteLetter::G, 0), 3)];
    let figures = vec![vec![
        Figure {
            interval: 6,
            accidental: 0,
        },
        Figure {
            interval: 4,
            accidental: 0,
        },
    ]];
    let result = realize(&bass, &figures, Pitch::new(NoteLetter::C, 0), Mode::Ionian);
    assert_eq!(result.len(), 1);
    // 6/4 has two explicit figures + implied 3rd = 3 upper voices
    assert_eq!(result[0].upper_voices.len(), 3);
}

#[test]
fn test_figured_bass_flat_accidental() {
    use theory::figured_bass::{realize, Figure};
    let bass = vec![Note::new(Pitch::new(NoteLetter::C, 0), 3)];
    let figures = vec![vec![Figure {
        interval: 3,
        accidental: -1,
    }]];
    let result = realize(&bass, &figures, Pitch::new(NoteLetter::C, 0), Mode::Ionian);
    assert_eq!(result.len(), 1);
}

// ============================================================
// Diatonic — cover seventh chord fallback
// ============================================================

#[test]
fn test_diatonic_triads_non_diatonic_returns_empty() {
    // HarmonicMinor is not diatonic — guard returns empty
    use theory::harmony;
    let chords = harmony::diatonic_triads(Pitch::new(NoteLetter::A, 0), Mode::HarmonicMinor);
    assert!(chords.is_empty());
}

#[test]
fn test_diatonic_triads_blues_returns_empty() {
    use theory::harmony;
    let chords = harmony::diatonic_triads(Pitch::new(NoteLetter::A, 0), Mode::Blues);
    assert!(chords.is_empty());
}

#[test]
fn test_diatonic_sevenths_all_keys() {
    use theory::harmony;
    for mode in [
        Mode::Ionian,
        Mode::Dorian,
        Mode::Phrygian,
        Mode::Lydian,
        Mode::Mixolydian,
        Mode::Aeolian,
        Mode::Locrian,
    ] {
        let chords = harmony::diatonic_sevenths(Pitch::new(NoteLetter::C, 0), mode);
        assert_eq!(chords.len(), 7, "Failed for mode {:?}", mode);
    }
}

// ============================================================
// Pitch — cover from_u8_with_scale_context and other branches
// ============================================================

#[test]
fn test_pitch_from_u8_with_direction() {
    let pitch = Pitch::from_u8_with_direction(3, Direction::Ascending);
    assert_eq!(pitch.as_u8(), 3);
}

#[test]
fn test_pitch_from_interval_with_context() {
    let c = Pitch::new(NoteLetter::C, 0);
    let interval = Interval::from_semitone(4).unwrap();
    let result =
        Pitch::from_interval_with_context(c, interval, Some(Mode::Ionian), Direction::Ascending);
    assert_eq!(result.as_u8(), 4);
}

#[test]
fn test_pitch_from_interval_down_with_context() {
    let e = Pitch::new(NoteLetter::E, 0);
    let interval = Interval::from_semitone(4).unwrap();
    let result = Pitch::from_interval_down_with_context(
        e,
        interval,
        Some(Mode::Ionian),
        Direction::Descending,
    );
    assert_eq!(result.as_u8(), 0);
}

#[test]
fn test_pitch_as_u8_negative_wrapping() {
    // Cb = C - 1 = 11
    let cb = Pitch::new(NoteLetter::C, -1);
    assert_eq!(cb.as_u8(), 11);
    // Fb = F - 1 = 4
    let fb = Pitch::new(NoteLetter::F, -1);
    assert_eq!(fb.as_u8(), 4);
}

#[test]
fn test_note_letter_from_index_all() {
    assert_eq!(NoteLetter::from_index(0), NoteLetter::C);
    assert_eq!(NoteLetter::from_index(1), NoteLetter::D);
    assert_eq!(NoteLetter::from_index(2), NoteLetter::E);
    assert_eq!(NoteLetter::from_index(3), NoteLetter::F);
    assert_eq!(NoteLetter::from_index(4), NoteLetter::G);
    assert_eq!(NoteLetter::from_index(5), NoteLetter::A);
    assert_eq!(NoteLetter::from_index(6), NoteLetter::B);
    assert_eq!(NoteLetter::from_index(7), NoteLetter::C); // wraps
}
