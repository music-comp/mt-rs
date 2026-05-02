extern crate music_comp_mt as theory;

use std::collections::BTreeSet;
use theory::harmonize::candidates::candidates_for_top;
use theory::harmonize::DualityScope;
use theory::quintal::BaseSpace;

#[test]
fn test_quintal_only_76_candidates() {
    let space = BaseSpace::new();
    let candidates = candidates_for_top(60, DualityScope::QuintalOnly, &space, 0).unwrap();
    eprintln!("quintal_only count: {}", candidates.len());
    assert_eq!(candidates.len(), 76);
    for (i, c) in candidates.iter().enumerate() {
        assert_eq!(
            c.pitches[3], 60,
            "candidate {} has top voice {} != 60",
            i, c.pitches[3]
        );
    }
    let lowest_bottom = candidates.iter().map(|c| c.pitches[0]).min().unwrap();
    let highest_bottom = candidates.iter().map(|c| c.pitches[0]).max().unwrap();
    eprintln!(
        "quintal_only: lowest bottom={}, highest bottom={}",
        lowest_bottom, highest_bottom
    );
}

#[test]
fn test_quartal_only_76_candidates() {
    let space = BaseSpace::new();
    let candidates = candidates_for_top(60, DualityScope::QuartalOnly, &space, 0).unwrap();
    eprintln!("quartal_only count: {}", candidates.len());
    assert_eq!(candidates.len(), 76);
    for (i, c) in candidates.iter().enumerate() {
        assert_eq!(
            c.pitches[3], 60,
            "candidate {} has top voice {} != 60",
            i, c.pitches[3]
        );
    }
    let lowest_bottom = candidates.iter().map(|c| c.pitches[0]).min().unwrap();
    let highest_bottom = candidates.iter().map(|c| c.pitches[0]).max().unwrap();
    eprintln!(
        "quartal_only: lowest bottom={}, highest bottom={}",
        lowest_bottom, highest_bottom
    );
}

#[test]
fn test_both_152_distinct_candidates() {
    let space = BaseSpace::new();
    let candidates = candidates_for_top(60, DualityScope::Both, &space, 0).unwrap();
    eprintln!("both count: {}", candidates.len());
    assert_eq!(candidates.len(), 152);

    for (i, c) in candidates.iter().enumerate() {
        assert_eq!(
            c.pitches[3], 60,
            "candidate {} has top voice {} != 60",
            i, c.pitches[3]
        );
        assert!(
            c.pitches[0] < c.pitches[1]
                && c.pitches[1] < c.pitches[2]
                && c.pitches[2] < c.pitches[3],
            "candidate {} not ascending: {:?}",
            i,
            c.pitches
        );
    }

    let unique: BTreeSet<_> = candidates.iter().map(|c| c.pitches).collect();
    eprintln!("both unique count: {}", unique.len());
    assert_eq!(unique.len(), 152, "all 152 candidates must be distinct");

    let lowest_bottom = candidates.iter().map(|c| c.pitches[0]).min().unwrap();
    let highest_bottom = candidates.iter().map(|c| c.pitches[0]).max().unwrap();
    eprintln!(
        "both: lowest bottom={}, highest bottom={}",
        lowest_bottom, highest_bottom
    );
}

#[test]
fn test_specific_chord_cgda() {
    let space = BaseSpace::new();
    let candidates = candidates_for_top(60, DualityScope::QuintalOnly, &space, 0).unwrap();
    // PcChord {0,2,7,9}: quintal root at octave 0 = [0, 7, 14, 21]
    // Inversion cycle: find the one with top voice PC 0 (= C = MIDI 60 target)
    // quintal_root([0,2,7,9], 0) has interval (7,7,7): pitches [0, 7, 14, 21]
    // pitches[3] = 21, PC = 21%12 = 9. Not PC 0.
    // t1 of that: each voice moves up by chord-scale step. chord_scale of {0,2,7,9}
    // sorted = [0,2,7,9], steps = [2,5,2,3]. Voice PCs: 0→+2, 7→+5(mod12), 2→+2(mod12)=14, 9→+3=12
    // Actually let me just find it in the output.
    let cgda = candidates.iter().find(|c| {
        let pcs: BTreeSet<u8> = c.pitches.iter().map(|&p| p % 12).collect();
        pcs == [0, 2, 7, 9].iter().copied().collect()
    });
    if let Some(c) = cgda {
        eprintln!("CGDA candidate with top=60: {:?}", c.pitches);
    } else {
        eprintln!("CGDA candidate NOT FOUND in quintal candidates for PC 0");
    }
}
