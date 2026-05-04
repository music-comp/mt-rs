extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t_quartal_reversal_equivalence, to_quartal, verify_all_orbits_self_dual,
    QuartalIntervalStructure,
};
use theory::quintal::{IntervalStructure, Orbit, VoicedChord};

// Q555 (= quintal Q777) Summit, voiced quintally as Eb-Bb-F-C — the
// legal-quintal stacking of pcs {0,3,5,10}. Palindromic IS.
// See plan §2 D-T0-003 for why "constructed via pure_quartal_stack" reads
// as "PcChord wrapped via to_quartal of its legal quintal voicing."
fn q555_summit_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([51, 58, 65, 72]).unwrap(); // Eb3-Bb3-F4-C5
    to_quartal(&vc)
}

// Q646 (= quintal Q686) Saddle, voiced quintally as C-F#-D-G# — the
// legal-quintal stacking of pcs {0,2,6,8}. Palindromic IS.
fn q646_saddle_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([48, 54, 62, 68]).unwrap(); // C3-F#3-D4-G#4
    to_quartal(&vc)
}

// Q554 (= quintal Q877), voiced quintally as C-G#-D#-A# — the
// legal-quintal stacking of pcs {0,3,8,10}. Bottom-up IS (8,7,7) — ASYMMETRIC.
// This fixture is the asymmetric-chord case that catches reversal bugs in
// quartal_reading; see plan v2 revision history.
fn q554_asymmetric_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([60, 68, 75, 82]).unwrap(); // C4-G#4-D#5-A#5
    to_quartal(&vc)
}

#[test]
fn test_quartal_reading_q555_summit() {
    let chord = q555_summit_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(5, 5, 5));
}

#[test]
fn test_quintal_reading_q555_summit() {
    let chord = q555_summit_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(7, 7, 7));
}

#[test]
fn test_quartal_reading_q646_saddle() {
    let chord = q646_saddle_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(6, 4, 6));
}

#[test]
fn test_quintal_reading_q646_saddle() {
    let chord = q646_saddle_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(6, 8, 6));
}

#[test]
fn test_quartal_reading_q554_asymmetric() {
    // Asymmetric chord: bottom-up IS (8,7,7) → quartal IS (5,5,4).
    // If quartal_reading is implemented by composing quintal::quartal_reading
    // (top-down) with quintal_to_quartal_structure (which expects bottom-up),
    // the result will be (4,5,5) instead of (5,5,4) — this test catches that.
    let chord = q554_asymmetric_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(5, 5, 4));
}

#[test]
fn test_quintal_reading_q554_asymmetric() {
    let chord = q554_asymmetric_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(8, 7, 7));
}

#[test]
fn test_t_quartal_reversal_equivalence_holds() {
    // Sample one chord per orbit class; mirrors the spec §7.1 intent.
    let test_pitches: [[u8; 4]; 4] = [
        [48, 55, 62, 69], // Q777
        [48, 54, 62, 68], // Q686 saddle
        [48, 55, 62, 68], // Q776
        [48, 56, 63, 70], // Q877
    ];
    for pitches in &test_pitches {
        let qvc = to_quartal(&VoicedChord::new(*pitches).unwrap());
        assert!(
            t_quartal_reversal_equivalence(&qvc),
            "t_quartal reversal failed for {:?}",
            pitches
        );
    }
}

#[test]
fn test_self_duality_re_export() {
    // verify_all_orbits_self_dual must be reachable via crate::quartal::* and
    // return true on the canonical base space.
    let space = base_space();
    assert!(verify_all_orbits_self_dual(&space));
}

#[test]
fn test_orbit_self_duality_re_export() {
    let space = base_space();
    assert!(orbit_self_duality(&Orbit::Q777, &space));
}

#[test]
fn test_reverse_interval_structure_re_export() {
    let is = IntervalStructure(7, 7, 6);
    assert_eq!(reverse_interval_structure(&is), IntervalStructure(6, 7, 7));
}
