extern crate music_comp_mt as theory;

use theory::quintal::{
    inversion_cycle, l1_distance, orbit_self_duality, quartal_reading, quintal_reading,
    reverse_interval_structure, t1_reversal_equivalence, t_minus1, verify_all_orbits_self_dual,
    BaseSpace, IntervalStructure, Orbit, VoicedChord,
};

// --- readings ---

#[test]
fn test_quintal_reading_cgda() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(quintal_reading(&chord), IntervalStructure(7, 7, 7));
}

#[test]
fn test_quartal_reading_cgda() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    // Palindromic: quartal = quintal for (7,7,7)
    assert_eq!(quartal_reading(&chord), IntervalStructure(7, 7, 7));
}

#[test]
fn test_quartal_is_reverse_of_quintal() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(
        quartal_reading(&chord),
        reverse_interval_structure(&quintal_reading(&chord))
    );
}

// --- reverse_interval_structure ---

#[test]
fn test_reverse_palindromic() {
    let is = IntervalStructure(7, 7, 7);
    assert_eq!(reverse_interval_structure(&is), is);
}

#[test]
fn test_reverse_asymmetric() {
    let is = IntervalStructure(7, 7, 6);
    assert_eq!(reverse_interval_structure(&is), IntervalStructure(6, 7, 7));
}

#[test]
fn test_reverse_is_involution() {
    let is = IntervalStructure(8, 6, 7);
    assert_eq!(
        reverse_interval_structure(&reverse_interval_structure(&is)),
        is
    );
}

// --- t1_reversal_equivalence ---

#[test]
fn test_t1_reversal_cgda() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert!(t1_reversal_equivalence(&chord));
}

#[test]
fn test_t1_reversal_crossroads() {
    let chord = VoicedChord::new([48, 54, 62, 68]).unwrap();
    assert!(t1_reversal_equivalence(&chord));
}

#[test]
fn test_t1_reversal_various_chords() {
    // Test with a few different legal voiced chords
    let chords = [
        [48, 55, 62, 69], // Q777
        [48, 54, 62, 68], // Q686
        [48, 55, 62, 68], // Q776
        [48, 56, 63, 70], // Q877
    ];
    for pitches in &chords {
        let chord = VoicedChord::new(*pitches).unwrap();
        assert!(
            t1_reversal_equivalence(&chord),
            "t1 reversal failed for {:?}",
            pitches
        );
    }
}

// --- orbit_self_duality ---

#[test]
fn test_orbit_self_duality_q777() {
    let space = BaseSpace::new();
    assert!(orbit_self_duality(&Orbit::Q777, &space));
}

#[test]
fn test_orbit_self_duality_q776() {
    // (7,7,6) reversed is (6,7,7) — different T-orbit but same T/I orbit
    let space = BaseSpace::new();
    assert!(orbit_self_duality(&Orbit::Q776, &space));
}

#[test]
fn test_all_orbits_self_dual() {
    let space = BaseSpace::new();
    assert!(verify_all_orbits_self_dual(&space));
}

// --- L1 same in both directions ---

#[test]
fn test_l1_pattern_same_both_directions() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let t1_cycle = inversion_cycle(&chord);

    // Forward (t1) consecutive L1 distances
    let fwd = [
        l1_distance(&t1_cycle[0], &t1_cycle[1]),
        l1_distance(&t1_cycle[1], &t1_cycle[2]),
        l1_distance(&t1_cycle[2], &t1_cycle[3]),
        l1_distance(&t1_cycle[3], &t1_cycle[0]),
    ];

    // Backward (t_minus1) consecutive L1 distances
    let tm1 = t_minus1(&chord);
    let tm2 = t_minus1(&tm1);
    let tm3 = t_minus1(&tm2);
    let bwd = [
        l1_distance(&t1_cycle[0], &tm1),
        l1_distance(&tm1, &tm2),
        l1_distance(&tm2, &tm3),
        l1_distance(&tm3, &t1_cycle[0]),
    ];

    // Both should follow [12, 12, 12, 36]
    assert_eq!(fwd, [12, 12, 12, 36]);
    assert_eq!(bwd, [12, 12, 12, 36]);
}

#[test]
fn test_quartal_quintal_same_pc_set() {
    // Both readings of a chord produce the same pitch-class set
    // (they're just different orderings of the same notes)
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let q5 = quintal_reading(&chord);
    let q4 = quartal_reading(&chord);
    // Both are interval structures of the same chord, just read differently
    // The sum of intervals should be the same (total span)
    assert_eq!(q5.0 + q5.1 + q5.2, q4.0 + q4.1 + q4.2);
}
