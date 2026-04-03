extern crate music_comp_mt as theory;

use theory::quintal::{
    classify_all, classify_orbit, enumerate_all, invert, invert_transpose, orbit, transpose,
    BaseSpace, Orbit, PcChord,
};

// ─── Group operation tests ───────────────────────────────────────────

#[test]
fn test_transpose_identity() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(transpose(&chord, 0), chord);
}

#[test]
fn test_transpose_mod12() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(transpose(&chord, 12), chord);
}

#[test]
fn test_transpose_cgda_by_1() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let expected = PcChord::new([1, 3, 8, 10]).unwrap();
    assert_eq!(transpose(&chord, 1), expected);
}

#[test]
fn test_transpose_cgda_by_7() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    // (0+7)=7, (2+7)=9, (7+7)=14%12=2, (9+7)=16%12=4 -> sorted [2,4,7,9]
    let expected = PcChord::new([2, 4, 7, 9]).unwrap();
    assert_eq!(transpose(&chord, 7), expected);
}

#[test]
fn test_invert_involution() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(invert(&invert(&chord)), chord);
}

#[test]
fn test_invert_involution_all_chords() {
    for chord in enumerate_all() {
        assert_eq!(
            invert(&invert(&chord)),
            chord,
            "inversion involution failed for {:?}",
            chord
        );
    }
}

#[test]
fn test_invert_cgda() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    // (12-0)%12=0, (12-2)%12=10, (12-7)%12=5, (12-9)%12=3 -> sorted [0,3,5,10]
    let expected = PcChord::new([0, 3, 5, 10]).unwrap();
    assert_eq!(invert(&chord), expected);
}

#[test]
fn test_invert_zero_fixpoint() {
    // Any chord containing pc 0 should still contain pc 0 after inversion
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let inverted = invert(&chord);
    assert!(inverted.pcs.contains(&0), "inversion should preserve pc 0");
}

#[test]
fn test_invert_transpose_consistency() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    for n in 0..12u8 {
        let result = invert_transpose(&chord, n);
        let manual = transpose(&invert(&chord), n);
        assert_eq!(result, manual, "invert_transpose({n}) mismatch");
    }
}

#[test]
fn test_transpose_preserves_legality() {
    for chord in enumerate_all() {
        for n in 0..12u8 {
            let transposed = transpose(&chord, n);
            assert!(
                transposed.is_legal(),
                "transpose({:?}, {}) produced illegal chord {:?}",
                chord,
                n,
                transposed
            );
        }
    }
}

#[test]
fn test_invert_preserves_legality() {
    for chord in enumerate_all() {
        let inverted = invert(&chord);
        assert!(
            inverted.is_legal(),
            "invert({:?}) produced illegal chord {:?}",
            chord,
            inverted
        );
    }
}

// ─── Orbit computation tests ────────────────────────────────────────

#[test]
fn test_orbit_size_777() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(orbit(&chord).len(), 12);
}

#[test]
fn test_orbit_size_676() {
    let chord = PcChord::new([0, 1, 6, 7]).unwrap();
    assert_eq!(orbit(&chord).len(), 6);
}

#[test]
fn test_orbit_size_686() {
    let chord = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(orbit(&chord).len(), 6);
}

#[test]
fn test_orbit_size_776() {
    let chord = PcChord::new([0, 2, 7, 8]).unwrap();
    assert_eq!(orbit(&chord).len(), 24);
}

#[test]
fn test_orbit_contains_original() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert!(orbit(&chord).contains(&chord));
}

// ─── Classification tests ───────────────────────────────────────────

#[test]
fn test_classify_orbit_777() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q777));
}

#[test]
fn test_classify_orbit_686() {
    let chord = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q686));
}

#[test]
fn test_classify_orbit_787() {
    let chord = PcChord::new([0, 3, 7, 10]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q787));
}

#[test]
fn test_classify_orbit_676() {
    let chord = PcChord::new([0, 1, 6, 7]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q676));
}

#[test]
fn test_classify_orbit_878() {
    let chord = PcChord::new([0, 3, 8, 11]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q878));
}

#[test]
fn test_classify_orbit_868() {
    let chord = PcChord::new([0, 2, 8, 10]).unwrap();
    assert_eq!(classify_orbit(&chord), Some(Orbit::Q868));
}

#[test]
fn test_classify_all_14_orbits() {
    let chords = enumerate_all();
    let classified = classify_all(&chords);
    assert_eq!(
        classified.len(),
        14,
        "expected 14 orbits, got {}",
        classified.len()
    );
}

#[test]
fn test_classify_all_total() {
    let chords = enumerate_all();
    let classified = classify_all(&chords);
    let total: usize = classified.values().map(|v| v.len()).sum();
    assert_eq!(total, 228, "expected 228 total chords, got {total}");
}

#[test]
fn test_orbit_sizes_match() {
    let chords = enumerate_all();
    let classified = classify_all(&chords);
    for (&orb, members) in &classified {
        assert_eq!(
            members.len(),
            orb.size(),
            "orbit {:?} expected size {}, got {}",
            orb,
            orb.size(),
            members.len()
        );
    }
}

#[test]
fn test_orbit_degrees_match() {
    let chords = enumerate_all();
    let classified = classify_all(&chords);
    let base = BaseSpace::new();

    for (&orb, members) in &classified {
        for chord in members {
            let degree = base.degree(chord).expect("chord should be in base space");
            assert_eq!(
                degree,
                orb.degree(),
                "chord {:?} in orbit {:?} has degree {}, expected {}",
                chord,
                orb,
                degree,
                orb.degree()
            );
        }
    }
}

#[test]
fn test_orbit_representative_structures() {
    for &orb in Orbit::all() {
        let is = orb.representative();
        assert!(
            is.is_legal(),
            "orbit {:?} has illegal representative IS {:?}",
            orb,
            is
        );
        // Verify the IS name matches the intervals
        let [a, b, c] = is.intervals();
        let name = format!("Q{}{}{}", a, b, c);
        let debug_name = format!("{:?}", orb);
        assert_eq!(
            name, debug_name,
            "orbit {:?} representative mismatch: IS ({},{},{}) vs name",
            orb, a, b, c
        );
    }
}

// ─── Orbit method tests ─────────────────────────────────────────────

#[test]
fn test_orbit_all_has_14() {
    assert_eq!(Orbit::all().len(), 14);
}

#[test]
fn test_orbit_representative_roundtrip() {
    for &orb in Orbit::all() {
        let is = orb.representative();
        // Build the representative PcChord from the IS starting at pc 0
        let p0: u8 = 0;
        let p1 = (p0 + is.0) % 12;
        let p2 = (p1 + is.1) % 12;
        let p3 = (p2 + is.2) % 12;
        let chord = PcChord::new([p0, p1, p2, p3]).unwrap();
        let classified = classify_orbit(&chord);
        assert_eq!(
            classified,
            Some(orb),
            "representative of {:?} classified as {:?}",
            orb,
            classified
        );
    }
}

#[test]
fn test_orbit_analogy() {
    assert_eq!(Orbit::Q777.analogy(), Some("major"));
    assert_eq!(Orbit::Q787.analogy(), Some("minor"));
    assert_eq!(Orbit::Q676.analogy(), Some("diminished"));
    assert_eq!(Orbit::Q686.analogy(), Some("augmented"));
    assert_eq!(Orbit::Q776.analogy(), None);
    assert_eq!(Orbit::Q878.analogy(), None);
}

#[test]
fn test_orbit_display() {
    assert_eq!(format!("{}", Orbit::Q777), "Q(7,7,7) [major]");
    assert_eq!(format!("{}", Orbit::Q787), "Q(7,8,7) [minor]");
    assert_eq!(format!("{}", Orbit::Q776), "Q(7,7,6)");
}

#[test]
fn test_orbit_sizes_sum_to_228() {
    let total: usize = Orbit::all().iter().map(|o| o.size()).sum();
    assert_eq!(total, 228, "orbit sizes should sum to 228, got {total}");
}

#[test]
fn test_transposed_chord_same_orbit() {
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    for n in 0..12u8 {
        let transposed = transpose(&chord, n);
        assert_eq!(
            classify_orbit(&transposed),
            Some(Orbit::Q777),
            "transpose by {n} should stay in Q777"
        );
    }
}

#[test]
fn test_inverted_chord_same_orbit() {
    // Q777 is self-inverse (palindromic IS), so inversion stays in Q777
    let chord = PcChord::new([0, 2, 7, 9]).unwrap();
    let inverted = invert(&chord);
    assert_eq!(classify_orbit(&inverted), Some(Orbit::Q777));
}
