extern crate rust_music_theory as theory;

use theory::chord::{Chord, Number::*, Quality::*};
use theory::note::{NoteLetter, Notes, Pitch, PitchSymbol::*};

fn assert_chord_notes(expected: &[theory::note::PitchSymbol], chord: Chord) {
    let notes = chord.notes();
    assert_eq!(notes.len(), expected.len(), 
        "Chord {} should have {} notes, got {}", 
        format!("{:?}", chord), expected.len(), notes.len());
    
    for (i, &expected_pitch) in expected.iter().enumerate() {
        assert_eq!(
            Pitch::from(expected_pitch), 
            notes[i].pitch,
            "Note {} of chord {:?}: expected {:?}, got {:?}",
            i + 1, chord, expected_pitch, notes[i].pitch
        );
    }
}

#[cfg(test)]
mod chord_enharmonic_tests {
    use super::*;

    #[test]
    fn test_major_triads_flat_keys() {
        // Test all major triads in flat keys use consistent flat spelling
        let test_cases = vec![
            (Pitch::new(NoteLetter::F, 0), vec![F, A, C]),           // F major
            (Pitch::new(NoteLetter::B, -1), vec![Bb, D, F]),         // Bb major  
            (Pitch::new(NoteLetter::E, -1), vec![Eb, G, Bb]),        // Eb major
            (Pitch::new(NoteLetter::A, -1), vec![Ab, C, Eb]),        // Ab major
            (Pitch::new(NoteLetter::D, -1), vec![Db, F, Ab]),        // Db major
            (Pitch::new(NoteLetter::G, -1), vec![Gb, Bb, Db]),       // Gb major
        ];

        for (root, expected) in test_cases {
            let chord = Chord::new(root, Major, Triad);
            assert_chord_notes(&expected, chord);
        }
    }

    #[test]
    fn test_major_triads_sharp_keys() {
        // Test all major triads in sharp keys use consistent sharp spelling
        let test_cases = vec![
            (Pitch::new(NoteLetter::G, 0), vec![G, B, D]),           // G major
            (Pitch::new(NoteLetter::D, 0), vec![D, Fs, A]),          // D major
            (Pitch::new(NoteLetter::A, 0), vec![A, Cs, E]),          // A major
            (Pitch::new(NoteLetter::E, 0), vec![E, Gs, B]),          // E major
            (Pitch::new(NoteLetter::B, 0), vec![B, Ds, Fs]),         // B major
            (Pitch::new(NoteLetter::F, 1), vec![Fs, As, Cs]),        // F# major
            (Pitch::new(NoteLetter::C, 1), vec![Cs, Es, Gs]),        // C# major
        ];

        for (root, expected) in test_cases {
            let chord = Chord::new(root, Major, Triad);
            assert_chord_notes(&expected, chord);
        }
    }

    #[test]
    fn test_minor_triads_enharmonic_spelling() {
        let test_cases = vec![
            // Minor chords use the minor key signature (relative major context)
            // F minor: relative major = Ab (4 flats)
            (Pitch::new(NoteLetter::F, 0), vec![F, Ab, C]),
            // Bb minor: relative major = Db (5 flats)
            (Pitch::new(NoteLetter::B, -1), vec![Bb, Db, F]),
            // Eb minor: relative major = Gb (6 flats)
            (Pitch::new(NoteLetter::E, -1), vec![Eb, Gb, Bb]),
            // Ab minor: letter-based spelling gives Cb and Eb (correct thirds)
            (Pitch::new(NoteLetter::A, -1), vec![Ab, Cb, Eb]),
            // Db minor: letter-based gives Fb and Ab (correct thirds)
            (Pitch::new(NoteLetter::D, -1), vec![Db, Fb, Ab]),
            // Gb minor: Bbb (double flat) falls back to A; Db is single flat
            (Pitch::new(NoteLetter::G, -1), vec![Gb, A, Db]),

            // Sharp/natural key minors
            // G minor: relative major = Bb (2 flats)
            (Pitch::new(NoteLetter::G, 0), vec![G, Bb, D]),
            // D minor: relative major = F (1 flat)
            (Pitch::new(NoteLetter::D, 0), vec![D, F, A]),
            // A minor: relative major = C (no sharps/flats, prefer sharps)
            (Pitch::new(NoteLetter::A, 0), vec![A, C, E]),
            // E minor: relative major = G (1 sharp)
            (Pitch::new(NoteLetter::E, 0), vec![E, G, B]),
            // B minor: relative major = D (2 sharps)
            (Pitch::new(NoteLetter::B, 0), vec![B, D, Fs]),
            // F# minor: relative major = A (3 sharps)
            (Pitch::new(NoteLetter::F, 1), vec![Fs, A, Cs]),
            // C# minor: relative major = E (4 sharps)
            (Pitch::new(NoteLetter::C, 1), vec![Cs, E, Gs]),
        ];

        for (root, expected) in test_cases {
            let chord = Chord::new(root, Minor, Triad);
            assert_chord_notes(&expected, chord);
        }
    }

    #[test]
    fn test_seventh_chords_enharmonic_spelling() {
        let test_cases = vec![
            // Major 7th chords in flat keys
            (Pitch::new(NoteLetter::D, -1), vec![Db, F, Ab, C]),     // Db maj7
            (Pitch::new(NoteLetter::G, -1), vec![Gb, Bb, Db, F]),    // Gb maj7
            (Pitch::new(NoteLetter::A, -1), vec![Ab, C, Eb, G]),     // Ab maj7
            
            // Major 7th chords in sharp keys  
            (Pitch::new(NoteLetter::F, 1), vec![Fs, As, Cs, Es]),    // F# maj7
            (Pitch::new(NoteLetter::C, 1), vec![Cs, Es, Gs, Bs]),    // C# maj7
            (Pitch::new(NoteLetter::B, 0), vec![B, Ds, Fs, As]),     // B maj7
            
            // Dominant 7th chords
            (Pitch::new(NoteLetter::G, -1), vec![Gb, Bb, Db, Fb]),   // Gb7 (Fb is the correct b7)
            (Pitch::new(NoteLetter::F, 1), vec![Fs, As, Cs, E]),     // F#7
        ];

        for (root, expected) in test_cases {
            let chord_maj7 = Chord::new(root, Major, MajorSeventh);
            if expected == vec![Gb, Bb, Db, Fb] || expected == vec![Fs, As, Cs, E] {
                // These are dominant 7ths, not major 7ths
                let chord_dom7 = Chord::new(root, Dominant, Seventh);
                assert_chord_notes(&expected, chord_dom7);
            } else {
                assert_chord_notes(&expected, chord_maj7);
            }
        }
    }

    #[test]
    fn test_diminished_and_augmented_triads() {
        let test_cases = vec![
            // Diminished triads
            (Pitch::new(NoteLetter::G, -1), Diminished, vec![Gb, A, C]),      // Gb dim (A=Bbb, C=Dbb enharmonic)
            (Pitch::new(NoteLetter::F, 1), Diminished, vec![Fs, A, C]),       // F# dim
            (Pitch::new(NoteLetter::A, -1), Diminished, vec![Ab, Cb, D]),     // Ab dim (Cb is correct 3rd; Ebb→D enharmonic for 5th)
            
            // Augmented triads  
            (Pitch::new(NoteLetter::G, -1), Augmented, vec![Gb, Bb, D]),      // Gb aug
            (Pitch::new(NoteLetter::F, 1), Augmented, vec![Fs, As, D]),       // F# aug (D natural, not in F# major scale)
            (Pitch::new(NoteLetter::D, -1), Augmented, vec![Db, F, A]),       // Db aug
        ];

        for (root, quality, expected) in test_cases {
            let chord = Chord::new(root, quality, Triad);
            assert_chord_notes(&expected, chord);
        }
    }

    #[test]
    fn test_chord_inversions_preserve_spelling() {
        // Test that inversions maintain the same accidental spelling
        let root = Pitch::new(NoteLetter::G, -1); // Gb major
        
        // Root position: Gb Bb Db
        let root_pos = Chord::with_inversion(root, Major, Triad, 0);
        assert_chord_notes(&vec![Gb, Bb, Db], root_pos);
        
        // First inversion: Bb Db Gb  
        let first_inv = Chord::with_inversion(root, Major, Triad, 1);
        let first_inv_notes = first_inv.notes();
        assert_eq!(Pitch::from(Bb), first_inv_notes[0].pitch);
        assert_eq!(Pitch::from(Db), first_inv_notes[1].pitch);
        assert_eq!(Pitch::from(Gb), first_inv_notes[2].pitch);
        
        // Second inversion: Db Gb Bb
        let second_inv = Chord::with_inversion(root, Major, Triad, 2);
        let second_inv_notes = second_inv.notes();
        assert_eq!(Pitch::from(Db), second_inv_notes[0].pitch);
        assert_eq!(Pitch::from(Gb), second_inv_notes[1].pitch);
        assert_eq!(Pitch::from(Bb), second_inv_notes[2].pitch);
    }

    #[test]
    fn test_enharmonic_chord_equivalence() {
        // Test that enharmonically equivalent chords have different spellings
        // but same semitone content
        
        // F# major vs Gb major
        let fs_major = Chord::new(Pitch::new(NoteLetter::F, 1), Major, Triad);
        let gb_major = Chord::new(Pitch::new(NoteLetter::G, -1), Major, Triad);
        
        let fs_notes = fs_major.notes();
        let gb_notes = gb_major.notes();
        
        // Same semitone content
        let fs_semitones: Vec<u8> = fs_notes.iter().map(|n| n.pitch.as_u8()).collect();
        let gb_semitones: Vec<u8> = gb_notes.iter().map(|n| n.pitch.as_u8()).collect();
        assert_eq!(fs_semitones, gb_semitones);
        
        // Different spelling
        assert_chord_notes(&vec![Fs, As, Cs], fs_major);
        assert_chord_notes(&vec![Gb, Bb, Db], gb_major);
    }

    #[test]
    fn test_complex_chord_extensions() {
        // Test that extended chords maintain consistent spelling
        let test_cases = vec![
            // 9th chords
            (Pitch::new(NoteLetter::D, -1), Major, Ninth, vec![Db, F, Ab, C, Eb]),
            (Pitch::new(NoteLetter::F, 1), Major, Ninth, vec![Fs, As, Cs, Es, Gs]),
            
            // 11th chords
            (Pitch::new(NoteLetter::A, -1), Major, Eleventh, vec![Ab, C, Eb, G, Bb, Db]),
            (Pitch::new(NoteLetter::B, 0), Major, Eleventh, vec![B, Ds, Fs, As, Cs, E]),
        ];

        for (root, quality, number, expected) in test_cases {
            let chord = Chord::new(root, quality, number);
            assert_chord_notes(&expected, chord);
        }
    }
}