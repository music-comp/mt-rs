use super::{HarmonizeError, HarmonizeOptions, MelodyInput};

pub(crate) fn canonicalize_melody(
    melody: &MelodyInput,
    options: &HarmonizeOptions,
) -> Result<Vec<u8>, HarmonizeError> {
    match melody {
        MelodyInput::PitchClasses(pcs) => {
            if pcs.is_empty() {
                return Err(HarmonizeError::EmptyMelody);
            }
            pcs.iter()
                .enumerate()
                .map(|(position, &pc)| {
                    if pc > 11 {
                        return Err(HarmonizeError::InvalidPitchClass(pc));
                    }
                    let midi = 12i16 * (options.melody_octave as i16 + 1)
                        + pc as i16
                        + options.top_voice_offset as i16;
                    if !(0..=127).contains(&midi) {
                        return Err(HarmonizeError::TargetMidiOutOfRange {
                            position,
                            target_midi: midi,
                        });
                    }
                    Ok(midi as u8)
                })
                .collect()
        }
        MelodyInput::Pitches(pitches) => {
            if pitches.is_empty() {
                return Err(HarmonizeError::EmptyMelody);
            }
            pitches
                .iter()
                .enumerate()
                .map(|(position, &p)| {
                    let target = p as i16 + options.top_voice_offset as i16;
                    if !(0..=127).contains(&target) {
                        return Err(HarmonizeError::TargetMidiOutOfRange {
                            position,
                            target_midi: target,
                        });
                    }
                    Ok(target as u8)
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitches_with_default_offset() {
        let opts = HarmonizeOptions::default();
        let result = canonicalize_melody(&MelodyInput::Pitches(vec![72, 76, 79]), &opts).unwrap();
        assert_eq!(result, vec![60, 64, 67]);
    }

    #[test]
    fn test_pcs_with_default_offset() {
        let opts = HarmonizeOptions::default();
        let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![0, 4, 7]), &opts).unwrap();
        assert_eq!(result, vec![60, 64, 67]);
    }

    #[test]
    fn test_invalid_pitch_class() {
        let opts = HarmonizeOptions::default();
        let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![13]), &opts);
        assert!(matches!(result, Err(HarmonizeError::InvalidPitchClass(13))));
    }

    #[test]
    fn test_empty_melody() {
        let opts = HarmonizeOptions::default();
        let result = canonicalize_melody(&MelodyInput::PitchClasses(vec![]), &opts);
        assert!(matches!(result, Err(HarmonizeError::EmptyMelody)));
    }

    #[test]
    fn test_underflow_target() {
        let opts = HarmonizeOptions {
            top_voice_offset: -12,
            ..Default::default()
        };
        let result = canonicalize_melody(&MelodyInput::Pitches(vec![5]), &opts);
        assert!(matches!(
            result,
            Err(HarmonizeError::TargetMidiOutOfRange { position: 0, .. })
        ));
    }

    #[test]
    fn test_overflow_target() {
        let opts = HarmonizeOptions {
            top_voice_offset: 12,
            ..Default::default()
        };
        let result = canonicalize_melody(&MelodyInput::Pitches(vec![127]), &opts);
        assert!(matches!(
            result,
            Err(HarmonizeError::TargetMidiOutOfRange { position: 0, .. })
        ));
    }

    #[test]
    fn test_position_field_carries_correct_index() {
        let opts = HarmonizeOptions {
            top_voice_offset: -12,
            ..Default::default()
        };
        let result = canonicalize_melody(&MelodyInput::Pitches(vec![60, 5, 60]), &opts);
        assert!(matches!(
            result,
            Err(HarmonizeError::TargetMidiOutOfRange { position: 1, .. })
        ));
    }
}
