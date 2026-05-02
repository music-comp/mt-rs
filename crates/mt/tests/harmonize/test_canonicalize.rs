extern crate music_comp_mt as theory;

use theory::harmonize::canonicalize::canonicalize_melody;
use theory::harmonize::{HarmonizeError, HarmonizeOptions, MelodyInput};

#[test]
fn test_pitches_with_default_offset() {
    // Pitches [72, 76, 79] (C5, E5, G5) with offset -12 → targets [60, 64, 67]
    let opts = HarmonizeOptions::default();
    let result = canonicalize_melody(&MelodyInput::Pitches(vec![72, 76, 79]), &opts).unwrap();
    eprintln!("pitches_default_offset: {:?}", result);
    assert_eq!(result, vec![60, 64, 67]);
}

#[test]
fn test_pcs_with_default_offset() {
    // PitchClasses [0, 4, 7] with melody_octave=5, offset=-12
    // MIDI = 12*(5+1) + pc + (-12) = 72 + pc - 12 = 60 + pc → [60, 64, 67]
    let opts = HarmonizeOptions::default();
    let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![0, 4, 7]), &opts).unwrap();
    eprintln!("pcs_default_offset: {:?}", result);
    assert_eq!(result, vec![60, 64, 67]);
}

#[test]
fn test_invalid_pitch_class() {
    let opts = HarmonizeOptions::default();
    let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![13]), &opts);
    eprintln!("invalid_pc: {:?}", result);
    assert!(matches!(result, Err(HarmonizeError::InvalidPitchClass(13))));
}

#[test]
fn test_empty_melody() {
    let opts = HarmonizeOptions::default();
    let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![]), &opts);
    eprintln!("empty_melody: {:?}", result);
    assert!(matches!(result, Err(HarmonizeError::EmptyMelody)));
}

#[test]
fn test_underflow_target() {
    // Pitch 5 with offset -12 → target = -7, out of range
    let opts = HarmonizeOptions {
        top_voice_offset: -12,
        ..Default::default()
    };
    let result = canonicalize_melody(&MelodyInput::Pitches(vec![5]), &opts);
    eprintln!("underflow: {:?}", result);
    assert!(matches!(
        result,
        Err(HarmonizeError::TargetMidiOutOfRange { position: 0, .. })
    ));
}

#[test]
fn test_overflow_target() {
    // Pitch 127 with offset +12 → target = 139, out of range
    let opts = HarmonizeOptions {
        top_voice_offset: 12,
        ..Default::default()
    };
    let result = canonicalize_melody(&MelodyInput::Pitches(vec![127]), &opts);
    eprintln!("overflow: {:?}", result);
    assert!(matches!(
        result,
        Err(HarmonizeError::TargetMidiOutOfRange { position: 0, .. })
    ));
}

#[test]
fn test_position_preserved_in_error() {
    // Second element is invalid
    let opts = HarmonizeOptions::default();
    let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![0, 13]), &opts);
    eprintln!("position_in_error: {:?}", result);
    assert!(matches!(
        result,
        Err(HarmonizeError::InvalidPitchClass(13))
    ));
}
