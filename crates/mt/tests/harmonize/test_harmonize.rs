extern crate music_comp_mt as theory;

use std::collections::BTreeSet;
use theory::harmonize::{
    harmonize_melody, DualityScope, HarmonizeError, HarmonizeOptions, MelodyInput,
};

#[test]
fn test_harmonize_with_pitches_default_offset() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![72, 76, 79]),
        HarmonizeOptions { k: 1, ..Default::default() },
    )
    .unwrap();
    assert_eq!(result[0].chords[0].pitches[3], 60);
    assert_eq!(result[0].chords[1].pitches[3], 64);
    assert_eq!(result[0].chords[2].pitches[3], 67);
}

#[test]
fn test_harmonize_with_pcs_default_offset() {
    let result = harmonize_melody(
        MelodyInput::PitchClasses(vec![0, 4, 7]),
        HarmonizeOptions { k: 1, ..Default::default() },
    )
    .unwrap();
    assert_eq!(result[0].chords[0].pitches[3], 60);
    assert_eq!(result[0].chords[1].pitches[3], 64);
    assert_eq!(result[0].chords[2].pitches[3], 67);
}

#[test]
fn test_candidates_for_c4_with_quintal_only() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![72]),
        HarmonizeOptions {
            k: 200,
            duality: DualityScope::QuintalOnly,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result.len(), 76);
    for h in &result {
        assert_eq!(h.chords[0].pitches[3], 60);
    }
}

#[test]
fn test_candidates_for_c4_with_both_dualities() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![72]),
        HarmonizeOptions {
            k: 200,
            duality: DualityScope::Both,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result.len(), 152);
    let unique: BTreeSet<_> = result.iter().map(|h| h.chords[0]).collect();
    assert_eq!(unique.len(), 152);
}

#[test]
fn test_top_voice_pinned_for_short_melody() {
    let result = harmonize_melody(
        MelodyInput::PitchClasses(vec![0, 4, 7]),
        HarmonizeOptions::default(),
    )
    .unwrap();
    let expected_tops: [u8; 3] = [60, 64, 67];
    for harmonization in &result {
        for (i, chord) in harmonization.chords.iter().enumerate() {
            assert_eq!(chord.pitches[3], expected_tops[i]);
        }
    }
}

#[test]
fn test_total_movement_is_monotonic_in_returned_order() {
    let result = harmonize_melody(
        MelodyInput::PitchClasses(vec![0, 4, 7, 2]),
        HarmonizeOptions { k: 10, ..Default::default() },
    )
    .unwrap();
    for window in result.windows(2) {
        assert!(window[0].total_movement <= window[1].total_movement);
    }
}

#[test]
fn test_repeated_notes_admit_zero_movement_step() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![72, 72]),
        HarmonizeOptions { k: 1, ..Default::default() },
    )
    .unwrap();
    assert_eq!(result[0].total_movement, 0);
    assert_eq!(result[0].chords[0], result[0].chords[1]);
}

#[test]
fn test_quartal_only_excludes_quintal_voicings() {
    let quartal_result = harmonize_melody(
        MelodyInput::Pitches(vec![72]),
        HarmonizeOptions {
            k: 200,
            duality: DualityScope::QuartalOnly,
            ..Default::default()
        },
    )
    .unwrap();
    let quintal_result = harmonize_melody(
        MelodyInput::Pitches(vec![72]),
        HarmonizeOptions {
            k: 200,
            duality: DualityScope::QuintalOnly,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(quartal_result.len(), 76);
    assert_eq!(quintal_result.len(), 76);

    let quartal_set: BTreeSet<_> = quartal_result.iter().map(|h| h.chords[0]).collect();
    let quintal_set: BTreeSet<_> = quintal_result.iter().map(|h| h.chords[0]).collect();

    let overlap: Vec<_> = quartal_set.intersection(&quintal_set).collect();
    assert!(
        overlap.is_empty(),
        "quartal and quintal candidate sets should be disjoint, found {} overlaps",
        overlap.len()
    );
}

#[test]
fn test_empty_melody_errors() {
    let result = harmonize_melody(MelodyInput::PitchClasses(vec![]), HarmonizeOptions::default());
    assert!(matches!(result, Err(HarmonizeError::EmptyMelody)));
}

#[test]
fn test_invalid_pitch_class_errors() {
    let result =
        harmonize_melody(MelodyInput::PitchClasses(vec![13]), HarmonizeOptions::default());
    assert!(matches!(result, Err(HarmonizeError::InvalidPitchClass(13))));
}

#[test]
fn test_default_options_match_documented_defaults() {
    let opts = HarmonizeOptions::default();
    assert_eq!(opts.top_voice_offset, -12);
    assert_eq!(opts.melody_octave, 5);
    assert_eq!(opts.duality, DualityScope::Both);
    assert_eq!(opts.k, 10);
}

#[test]
fn test_empty_melody_with_k_zero_still_errors() {
    let result = harmonize_melody(
        MelodyInput::PitchClasses(vec![]),
        HarmonizeOptions { k: 0, ..Default::default() },
    );
    assert!(matches!(result, Err(HarmonizeError::EmptyMelody)));
}

#[test]
fn test_k_zero_returns_empty() {
    let result = harmonize_melody(
        MelodyInput::PitchClasses(vec![0, 4, 7]),
        HarmonizeOptions { k: 0, ..Default::default() },
    )
    .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_target_midi_underflow_errors() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![5]),
        HarmonizeOptions {
            top_voice_offset: -12,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(HarmonizeError::TargetMidiOutOfRange { .. })));
}

#[test]
fn test_target_midi_overflow_errors() {
    let result = harmonize_melody(
        MelodyInput::Pitches(vec![127]),
        HarmonizeOptions {
            top_voice_offset: 12,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(HarmonizeError::TargetMidiOutOfRange { .. })));
}
