extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, quartal_orbits_by_betweenness, quartal_saddle_chords, QuartalIntervalStructure,
    QuartalOrbit,
};
use theory::quintal::{saddle_chords as quintal_saddle_chords, PcChord};

#[test]
fn test_quartal_saddle_chords_count_and_is() {
    let space = base_space();
    let saddle = quartal_saddle_chords(&space);
    assert_eq!(saddle.len(), 6);
    // Saddle orbit Q646 is palindromic: every pair has the same IS (6, 4, 6).
    for (chord, is) in &saddle {
        assert_eq!(
            *is,
            QuartalIntervalStructure(6, 4, 6),
            "chord {:?} did not produce QuartalIntervalStructure(6, 4, 6)",
            chord
        );
    }
}

#[test]
fn test_quartal_saddle_chords_match_quintal() {
    // The PcChord set returned by quartal_saddle_chords (drop the IS) must
    // equal the Vec<PcChord> returned by quintal::saddle_chords.
    let space = base_space();
    let quartal_pcs: Vec<PcChord> = quartal_saddle_chords(&space)
        .into_iter()
        .map(|(chord, _)| chord)
        .collect();
    let quintal_pcs = quintal_saddle_chords(&space);
    assert_eq!(quartal_pcs, quintal_pcs);
}

#[test]
fn test_quartal_orbits_by_betweenness_count() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    assert_eq!(ranked.len(), 14);
}

#[test]
fn test_quartal_orbits_by_betweenness_top_is_saddle() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    assert_eq!(ranked[0].0, QuartalOrbit::Q646);
    // The saddle's max betweenness is documented as ~0.139; be generous and
    // assert it's noticeably above the next-highest orbit's value.
    assert!(
        ranked[0].1 > ranked[1].1,
        "Saddle (Q646) betweenness should strictly exceed the next orbit's"
    );
}

#[test]
fn test_quartal_orbits_by_betweenness_descending() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    for window in ranked.windows(2) {
        assert!(
            window[0].1 >= window[1].1,
            "betweenness must be non-strictly descending: {:?} < {:?}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn test_quartal_orbits_by_betweenness_no_duplicates() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    let mut seen = std::collections::BTreeSet::new();
    for (orbit, _) in &ranked {
        assert!(
            seen.insert(*orbit),
            "duplicate QuartalOrbit in ranking: {:?}",
            orbit
        );
    }
    // Every variant must appear.
    for variant in QuartalOrbit::all() {
        assert!(
            seen.contains(variant),
            "QuartalOrbit::{:?} missing from ranking",
            variant
        );
    }
}
