use crate::quartal::{quartal_inversion_cycle, quartal_root};
use crate::quintal::{inversion_cycle, quintal_root, BaseSpace, VoicedChord};

use super::{DualityScope, HarmonizeError};

/// Enumerate all candidate VoicedChords for a given target top MIDI pitch.
///
/// For each PcChord in the base space that contains `target_top_midi % 12`,
/// finds the inversion whose top voice has that PC and shifts it to the
/// exact target MIDI pitch. Candidates come from quintal and/or quartal
/// cycles depending on `duality`.
pub fn candidates_for_top(
    target_top_midi: u8,
    duality: DualityScope,
    space: &BaseSpace,
    position: usize,
) -> Result<Vec<VoicedChord>, HarmonizeError> {
    // Build roots at octave 4 so the full inversion cycle stays in valid MIDI
    // range. t_minus1 can underflow if applied to very low pitches (e.g., MIDI 1),
    // and t1 applied 3 times climbs ~36 semitones above the root. Octave 4 (bottom
    // ≥ 48) gives headroom in both directions. The shift_to_top call afterward
    // moves to the actual target.
    const BASE_OCTAVE: u8 = 4;

    let target_pc = target_top_midi % 12;
    let mut candidates = Vec::new();

    for pc_chord in space.chords() {
        if !pc_chord.pcs.contains(&target_pc) {
            continue;
        }

        if matches!(duality, DualityScope::QuintalOnly | DualityScope::Both) {
            let q_root = quintal_root(pc_chord, BASE_OCTAVE)
                .expect("Phase 0 verifies all 228 chords have legal stackings");
            for inv in inversion_cycle(&q_root) {
                if inv.pitches[3] % 12 == target_pc {
                    candidates.push(shift_to_top(&inv, target_top_midi, position)?);
                    break;
                }
            }
        }

        if matches!(duality, DualityScope::QuartalOnly | DualityScope::Both) {
            let qv_root = quartal_root(pc_chord, BASE_OCTAVE)
                .expect("Phase 0 verifies all 228 chords have legal stackings");
            for inv in quartal_inversion_cycle(&qv_root) {
                if inv.as_voiced().pitches[3] % 12 == target_pc {
                    candidates.push(shift_to_top(inv.as_voiced(), target_top_midi, position)?);
                    break;
                }
            }
        }
    }

    Ok(candidates)
}

/// Shift a VoicedChord vertically so `pitches[3] == target_midi`.
///
/// Precondition: `target_midi % 12 == chord.pitches[3] % 12` (same PC).
/// Returns `Err(TargetMidiOutOfRange)` if the shift would push any voice
/// outside MIDI range [0, 127].
pub fn shift_to_top(
    chord: &VoicedChord,
    target_midi: u8,
    position: usize,
) -> Result<VoicedChord, HarmonizeError> {
    let delta = target_midi as i32 - chord.pitches[3] as i32;
    debug_assert!(delta % 12 == 0, "shift_to_top: PC mismatch");
    // Only bottom can underflow: top_after = target_midi (u8 ≤ 127) by
    // construction, and bottom < top for ascending chords.
    let bottom_after = chord.pitches[0] as i32 + delta;
    if bottom_after < 0 {
        return Err(HarmonizeError::TargetMidiOutOfRange {
            position,
            target_midi,
        });
    }
    let shifted = chord.pitches.map(|p| (p as i32 + delta) as u8);
    Ok(VoicedChord { pitches: shifted })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_identity() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let result = shift_to_top(&c, 69, 0).unwrap();
        eprintln!("identity: input={:?}, target=69, output={:?}", c.pitches, result.pitches);
        assert_eq!(result.pitches, c.pitches);
    }

    #[test]
    fn test_shift_octave_up() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let result = shift_to_top(&c, 81, 0).unwrap();
        eprintln!("octave_up: input={:?}, target=81, output={:?}", c.pitches, result.pitches);
        assert_eq!(result.pitches, [60, 67, 74, 81]);
    }

    #[test]
    fn test_shift_octave_down() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let result = shift_to_top(&c, 57, 0).unwrap();
        eprintln!("octave_down: input={:?}, target=57, output={:?}", c.pitches, result.pitches);
        assert_eq!(result.pitches, [36, 43, 50, 57]);
    }

    #[test]
    fn test_shift_underflow() {
        // Chord with bottom at 48; shift down so bottom would go to -12
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        // target = 69 - 60 = 9 → delta = 9 - 69 = -60, bottom = 48 - 60 = -12
        let result = shift_to_top(&c, 9, 3);
        eprintln!("underflow: input={:?}, target=9, result={:?}", c.pitches, result);
        assert!(matches!(
            result,
            Err(HarmonizeError::TargetMidiOutOfRange { position: 3, target_midi: 9 })
        ));
    }

    #[test]
    fn test_overflow_unreachable_for_u8_target() {
        // For any ascending VoicedChord and any target_midi: u8 (≤ 127),
        // top_after = target_midi ≤ 127, and bottom_after < top_after
        // (because pitches are strictly ascending). So bottom_after ≤ 127.
        // Overflow is structurally impossible — verified here at the upper
        // boundary. Underflow handling is verified by `test_shift_underflow`.
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        // c.pitches[3] = 69, PC 9. Highest PC-9 MIDI ≤ 127 is 117.
        // delta = +48; bottom = 96, top = 117. Both in [0, 127].
        assert!(shift_to_top(&c, 117, 0).is_ok());
    }

    #[test]
    fn test_shift_result_is_ascending() {
        let c = VoicedChord::new([36, 43, 50, 57]).unwrap();
        let result = shift_to_top(&c, 69, 0).unwrap();
        eprintln!("ascending: input={:?}, target=69, output={:?}", c.pitches, result.pitches);
        assert!(result.pitches[0] < result.pitches[1]);
        assert!(result.pitches[1] < result.pitches[2]);
        assert!(result.pitches[2] < result.pitches[3]);
        assert_eq!(result.pitches[3], 69);
    }
}
