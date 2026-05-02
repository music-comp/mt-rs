use super::{HarmonizeError, HarmonizeOptions, MelodyInput};

pub fn canonicalize_melody(
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
                            target_midi: midi.clamp(0, 255) as u8,
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
                            target_midi: target.clamp(0, 255) as u8,
                        });
                    }
                    Ok(target as u8)
                })
                .collect()
        }
    }
}
