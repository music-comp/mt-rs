extern crate music_comp_mt as theory;

use theory::quintal::{
    betweenness_centrality, classify_orbit, saddle_chords, BaseSpace, Orbit, PcChord,
};

#[test]
fn test_saddle_count() {
    let space = BaseSpace::new();
    assert_eq!(saddle_chords(&space).len(), 6);
}

#[test]
fn test_saddle_are_q686() {
    let space = BaseSpace::new();
    let saddle = saddle_chords(&space);
    for chord in &saddle {
        assert_eq!(
            classify_orbit(chord),
            Some(Orbit::Q686),
            "Saddle chord {:?} is not Q686",
            chord
        );
    }
}

#[test]
fn test_saddle_centrality_approx() {
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    let saddle = saddle_chords(&space);
    for chord in &saddle {
        let c = bc[chord];
        // Paper reports ~13.9% using a different normalization convention.
        // With standard Brandes normalization (n-1)(n-2)/2, saddle
        // centrality is ~0.091. All 6 saddle chords should match.
        assert!(
            (c - 0.091).abs() < 0.005,
            "Saddle chord {:?} centrality {} not near 0.091",
            chord,
            c
        );
    }
}

#[test]
fn test_centrality_nonnegative() {
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    for (chord, &val) in &bc {
        assert!(val >= 0.0, "Negative centrality for {:?}: {}", chord, val);
    }
}

#[test]
fn test_centrality_bounded() {
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    for (chord, &val) in &bc {
        assert!(val <= 1.0, "Centrality > 1.0 for {:?}: {}", chord, val);
    }
}

#[test]
fn test_centrality_all_228() {
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    assert_eq!(bc.len(), 228);
}

#[test]
fn test_orbit_invariant_centrality() {
    // All members of Q777 should have the same centrality
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    let cgda_c = bc[&cgda];
    // Check all Q777 members
    for chord in space.chords() {
        if classify_orbit(chord) == Some(Orbit::Q777) {
            let c = bc[chord];
            assert!(
                (c - cgda_c).abs() < 1e-10,
                "Q777 member {:?} centrality {} != {}",
                chord,
                c,
                cgda_c
            );
        }
    }
}

#[test]
fn test_orbit_invariant_all_orbits() {
    // Centrality should be constant within each orbit
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    for &orb in Orbit::all() {
        let members: Vec<_> = space
            .chords()
            .iter()
            .filter(|c| classify_orbit(c) == Some(orb))
            .collect();
        if members.len() > 1 {
            let first_c = bc[members[0]];
            for &chord in &members[1..] {
                let c = bc[chord];
                assert!(
                    (c - first_c).abs() < 1e-10,
                    "Orbit {:?} member {:?} centrality {} != {}",
                    orb,
                    chord,
                    c,
                    first_c
                );
            }
        }
    }
}

#[test]
fn test_saddle_includes_known() {
    let space = BaseSpace::new();
    let saddle = saddle_chords(&space);
    let known = PcChord::new([0, 2, 6, 8]).unwrap();
    assert!(saddle.contains(&known));
}

#[test]
fn test_saddle_have_max_centrality() {
    let space = BaseSpace::new();
    let bc = betweenness_centrality(&space);
    let saddle = saddle_chords(&space);
    let min_saddle_c = saddle.iter().map(|c| bc[c]).fold(f64::INFINITY, f64::min);
    // No non-saddle chord should have higher centrality
    for chord in space.chords() {
        if !saddle.contains(chord) {
            assert!(
                bc[chord] <= min_saddle_c + 1e-10,
                "Non-saddle {:?} centrality {} exceeds saddle minimum {}",
                chord,
                bc[chord],
                min_saddle_c
            );
        }
    }
}

/// Backward-compat smoke test: the deprecated `crossroads_chords` alias still
/// exists and returns the same result as `saddle_chords`. Without this we'd
/// be silently free to remove the alias; with it, removing the alias becomes
/// a deliberate choice that breaks at least one test in the same crate.
#[test]
#[allow(deprecated)]
fn test_crossroads_chords_alias_matches_saddle_chords() {
    use theory::quintal::crossroads_chords;
    let space = BaseSpace::new();
    assert_eq!(crossroads_chords(&space), saddle_chords(&space));
}
