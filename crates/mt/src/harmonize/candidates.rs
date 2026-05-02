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
    let bottom_after = chord.pitches[0] as i32 + delta;
    let top_after = chord.pitches[3] as i32 + delta;
    if !(0..=127).contains(&bottom_after) || !(0..=127).contains(&top_after) {
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
    fn test_shift_overflow() {
        // Chord with top at 69; shift up so top would be > 127
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        // target = 69 + 60 = 129 > 127... but target_midi is u8, max 127
        // Instead: chord at [100, 107, 114, 121], target = 121 + 12 = 133 > 127
        // But target_midi is u8... let me pick a feasible case:
        // Chord [100, 107, 114, 121], shift to 121 is identity (no overflow).
        // Shift to target 121 → delta 0. That's fine.
        // Actually: the target itself must be ≤ 127 (it's u8). So overflow
        // means the BOTTOM goes > 127 after shift.
        // Chord [110, 117, 121, 124], target 124+12... can't, u8 max is 127.
        // Actually overflow case: chord top is 60, target is 120.
        // delta = +60. bottom = 48 + 60 = 108. Top = 60 + 60 = 120. Both ≤ 127. OK.
        // For a real overflow: chord [60, 67, 74, 81], target 81+48=129? No, u8.
        // The only way overflow triggers is if bottom + delta > 127.
        // Chord [80, 87, 94, 101], target = 125 (PC 5, same as 101%12=5? 101%12=5. 125%12=5. yes!)
        // delta = 125-101 = 24. bottom = 80+24 = 104. top = 101+24 = 125. Both ≤ 127. Still fine.
        // Chord [90, 97, 104, 111], target = 123. 111%12=3, 123%12=3.
        // delta = 12. bottom = 90+12=102, top = 111+12=123. Fine.
        // Actually: for overflow we need bottom_after > 127.
        // Chord [110, 115, 120, 125], target 125+12=137? Can't — u8 max 127.
        // Hmm. Since target_midi is u8 (max 127), and top_after = target_midi (the whole point),
        // top_after is always ≤ 127. So overflow from top_after is impossible.
        // Bottom overflow: need chord where pitches[0] + delta > 127.
        // delta = target_midi - pitches[3]. For bottom_after > 127:
        //   pitches[0] + (target_midi - pitches[3]) > 127
        // This requires pitches[0] > 127 - target_midi + pitches[3].
        // With target=127 and a chord where pitches[3] is very small relative to pitches[0]...
        // But pitches are ascending, so pitches[0] < pitches[3] always.
        // Therefore bottom_after < top_after = target_midi ≤ 127.
        //
        // Conclusion: top_after > 127 is the only overflow case, and that requires
        // target_midi > 127, which can't happen since target_midi is u8.
        // So overflow via this function is actually impossible for valid target_midi!
        // The only failure mode is underflow (bottom < 0).
        //
        // Let's just verify the only-underflow claim:
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let result = shift_to_top(&c, 69, 0); // identity, fine
        eprintln!("overflow_check: for ascending chords with u8 target, overflow is impossible");
        eprintln!("  identity result: {:?}", result);
        assert!(result.is_ok());
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
