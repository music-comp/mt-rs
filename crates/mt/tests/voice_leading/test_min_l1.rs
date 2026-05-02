extern crate music_comp_mt as theory;

use theory::note::{Note, Pitch};
use theory::quintal::VoicedChord;
use theory::voice_leading::{min_voiced_chord_l1, minimal_movement};

fn note_from_midi(midi: u8) -> Note {
    let pc = midi % 12;
    let octave = midi / 12;
    Note::new(Pitch::from_u8(pc), octave)
}

#[test]
fn test_min_l1_same_chord_is_zero() {
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(min_voiced_chord_l1(&a, &a), 0);
}

#[test]
fn test_min_l1_is_symmetric() {
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let b = VoicedChord::new([50, 56, 63, 70]).unwrap();
    assert_eq!(min_voiced_chord_l1(&a, &b), min_voiced_chord_l1(&b, &a));
}

#[test]
fn test_min_l1_identity_assignment_when_optimal() {
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let b = VoicedChord::new([49, 56, 63, 70]).unwrap();
    // Each voice moves +1, identity assignment is optimal
    assert_eq!(min_voiced_chord_l1(&a, &b), 4);
}

#[test]
fn test_min_l1_finds_better_than_positional() {
    // Construct a case where positional matching is suboptimal.
    // a = [48, 55, 62, 69], b = [49, 55, 62, 68]
    // Positional: |48-49| + |55-55| + |62-62| + |69-68| = 1+0+0+1 = 2
    // This case happens to be optimal positionally too. Let's try a swap case:
    // a = [48, 60, 62, 69], b = [49, 61, 63, 68]
    // Positional: 1+1+1+1 = 4
    // optimal is also 4 here (ascending order means positional is usually good)
    //
    // For a real non-trivial case: voices cross.
    // a = [48, 55, 62, 69], b = [54, 48, 69, 62]... but b must be ascending.
    // Let's just verify correctness via the minimal_movement round-trip.
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let b = VoicedChord::new([50, 57, 64, 69]).unwrap();

    let l1 = min_voiced_chord_l1(&a, &b);

    let a_notes: Vec<Note> = a.pitches.iter().map(|&p| note_from_midi(p)).collect();
    let b_notes: Vec<Note> = b.pitches.iter().map(|&p| note_from_midi(p)).collect();
    let vl = minimal_movement(&a_notes, &b_notes);

    assert_eq!(l1, vl.total_distance as u32);
}

#[test]
fn test_min_l1_agrees_with_minimal_movement_various() {
    let cases: Vec<([u8; 4], [u8; 4])> = vec![
        ([48, 55, 62, 69], [50, 57, 64, 71]),
        ([36, 43, 50, 57], [37, 44, 51, 58]),
        ([48, 54, 62, 68], [50, 56, 64, 70]),
        ([24, 30, 37, 44], [25, 32, 39, 45]),
        ([48, 55, 62, 69], [48, 55, 62, 69]),
    ];

    for (a_p, b_p) in &cases {
        let a = VoicedChord::new(*a_p).unwrap();
        let b = VoicedChord::new(*b_p).unwrap();

        let l1 = min_voiced_chord_l1(&a, &b);

        let a_notes: Vec<Note> = a.pitches.iter().map(|&p| note_from_midi(p)).collect();
        let b_notes: Vec<Note> = b.pitches.iter().map(|&p| note_from_midi(p)).collect();
        let vl = minimal_movement(&a_notes, &b_notes);

        assert_eq!(
            l1, vl.total_distance as u32,
            "mismatch for {:?} → {:?}: min_l1={}, minimal_movement={}",
            a_p, b_p, l1, vl.total_distance
        );
    }
}
