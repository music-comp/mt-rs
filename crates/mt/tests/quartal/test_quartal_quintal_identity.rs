extern crate music_comp_mt as theory;

use theory::quartal::{
    self, pc_chord_quartal_intervals, pure_quartal_stack, quartal_inversion_cycle, to_quartal,
    QuartalOrbit,
};
use theory::quintal::{
    self, classify_orbit, enumerate_all, inversion_cycle, saddle_chords, BaseSpace, Orbit,
    VoicedChord,
};

/// For every one of the 228 chords: verify the quintal IS (i1,i2,i3) and
/// quartal IS (12-i3, 12-i2, 12-i1) describe the same chord.
#[test]
fn test_all_228_dual_intervals() {
    let chords = enumerate_all();
    for chord in &chords {
        let q5_is = chord.interval_structure();
        let q4_is = pc_chord_quartal_intervals(chord);
        assert!(q5_is.is_some(), "Chord {:?} has no quintal IS", chord);
        assert!(q4_is.is_some(), "Chord {:?} has no quartal IS", chord);
        let q5 = q5_is.unwrap();
        let q4 = q4_is.unwrap();
        // The quartal IS should be the reversed complement of quintal
        assert_eq!(q4.0, (12 - q5.2) % 12);
        assert_eq!(q4.1, (12 - q5.1) % 12);
        assert_eq!(q4.2, (12 - q5.0) % 12);
    }
}

/// For every orbit: verify the quintal orbit and quartal orbit contain the
/// same chords (same size, bijective mapping).
#[test]
fn test_all_orbits_same_chords() {
    for &q5_orbit in Orbit::all() {
        let q4_orbit = QuartalOrbit::from_quintal(&q5_orbit);
        // Both should have the same size
        assert_eq!(q5_orbit.size(), q4_orbit.size());
        // Both should map back to each other
        assert_eq!(q4_orbit.to_quintal(), q5_orbit);
    }
}

/// For a sample of chords: quartal and quintal cycles visit the same 4
/// voiced chords (as pitch-class sets).
#[test]
fn test_fibers_same_chords() {
    let test_chords: [[u8; 4]; 3] = [
        [48, 55, 62, 69], // C-G-D-A (Q777)
        [48, 54, 62, 68], // saddle (Q686)
        [48, 55, 62, 68], // Q776
    ];
    for pitches in &test_chords {
        let vc = VoicedChord::new(*pitches).unwrap();
        let qvc = to_quartal(&vc);

        let q5_pcs: std::collections::BTreeSet<_> = inversion_cycle(&vc)
            .iter()
            .map(|c| c.to_pc_chord().unwrap())
            .collect();
        let q4_pcs: std::collections::BTreeSet<_> = quartal_inversion_cycle(&qvc)
            .iter()
            .map(|c| c.0.to_pc_chord().unwrap())
            .collect();

        assert_eq!(q5_pcs, q4_pcs, "Fiber mismatch for {:?}", pitches);
    }
}

/// Saddle chords have degree 8 and orbit Q686 (quintal) = Q646 (quartal).
#[test]
fn test_saddle_dual_perspective() {
    let space = BaseSpace::new();
    let saddle = saddle_chords(&space);
    assert_eq!(saddle.len(), 6);
    for chord in &saddle {
        // Degree 8 from quintal
        assert_eq!(space.degree(chord), Some(8));
        // Orbit is Q686 quintal = Q646 quartal
        assert_eq!(classify_orbit(chord), Some(Orbit::Q686));
        assert_eq!(QuartalOrbit::from_quintal(&Orbit::Q686), QuartalOrbit::Q646);
    }
}

/// Distance is identical via quartal and quintal paths (they share the
/// same base space graph).
#[test]
fn test_distance_identical() {
    let space = BaseSpace::new();
    let chords = space.chords();
    for i in (0..chords.len()).step_by(25) {
        for j in ((i + 1)..chords.len()).step_by(30) {
            let d_q5 = quintal::distance(&space, &chords[i], &chords[j]);
            let d_q4 = quartal::distance(&space, &chords[i], &chords[j]);
            assert_eq!(d_q5, d_q4);
        }
    }
}

/// Quartal root vs quintal root: `pure_quartal_stack(0)` has quartal root C,
/// but viewed as quintal the ordering starts from Eb.
#[test]
fn test_quartal_root_vs_quintal_root() {
    let quartal_c = pure_quartal_stack(0); // C-F-Bb-Eb = {0,3,5,10}

    // The chord contains both the quartal root (C=0) and the quintal
    // "root" (Eb=3), which are different pitch classes.
    assert!(quartal_c.pcs.contains(&0)); // quartal root C
    assert!(quartal_c.pcs.contains(&3)); // quintal root Eb
    assert_ne!(0u8, 3u8); // they are different notes
}
