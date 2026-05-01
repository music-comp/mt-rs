extern crate music_comp_mt as theory;
use theory::quintal::{
    chord_scale, inversion_cycle, l1_distance, project, t1, t_minus1, IntervalStructure,
    VoicedChord,
};

// --- chord_scale ---

#[test]
fn test_chord_scale_cgda() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cs = chord_scale(&chord);
    assert_eq!(cs.pcs, [0, 2, 7, 9]);
    assert_eq!(cs.steps, [2, 5, 2, 3]);
}

#[test]
fn test_chord_scale_steps_sum_12() {
    let chord = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cs = chord_scale(&chord);
    let sum: u8 = cs.steps.iter().sum();
    assert_eq!(sum, 12);
}

#[test]
fn test_chord_scale_saddle() {
    // C-F#-D-Ab = [0,2,6,8] voiced as (48, 54, 62, 68)
    let chord = VoicedChord::new([48, 54, 62, 68]).unwrap();
    let cs = chord_scale(&chord);
    assert_eq!(cs.pcs, [0, 2, 6, 8]);
    assert_eq!(cs.steps, [2, 4, 2, 4]);
}

// --- t1 ---

#[test]
fn test_t1_cgda_root_to_1st() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let first = t1(&root);
    assert_eq!(first.pitches, [50, 57, 67, 72]);
}

#[test]
fn test_t1_cgda_1st_to_2nd() {
    let first = VoicedChord::new([50, 57, 67, 72]).unwrap();
    let second = t1(&first);
    assert_eq!(second.pitches, [55, 60, 69, 74]);
}

#[test]
fn test_t1_cgda_2nd_to_3rd() {
    let second = VoicedChord::new([55, 60, 69, 74]).unwrap();
    let third = t1(&second);
    assert_eq!(third.pitches, [57, 62, 72, 79]);
}

#[test]
fn test_t1_fourth_equals_t12() {
    // t1^4 should equal T12 (all pitches + 12)
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let inv1 = t1(&root);
    let inv2 = t1(&inv1);
    let inv3 = t1(&inv2);
    let root_prime = t1(&inv3);
    assert_eq!(root_prime.pitches, [60, 67, 74, 81]);
    // That's root + 12 in each voice
    for i in 0..4 {
        assert_eq!(root_prime.pitches[i], root.pitches[i] + 12);
    }
}

// --- t_minus1 ---

#[test]
fn test_t_minus1_reverses_t1() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let inv1 = t1(&root);
    let back = t_minus1(&inv1);
    // Should have same pitch-class set as root
    assert_eq!(back.to_pc_chord().unwrap(), root.to_pc_chord().unwrap());
}

// --- inversion_cycle ---

#[test]
fn test_inversion_cycle_cgda() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cycle = inversion_cycle(&root);
    assert_eq!(cycle[0].pitches, [48, 55, 62, 69]);
    assert_eq!(cycle[1].pitches, [50, 57, 67, 72]);
    assert_eq!(cycle[2].pitches, [55, 60, 69, 74]);
    assert_eq!(cycle[3].pitches, [57, 62, 72, 79]);
}

#[test]
fn test_inversion_cycle_intervals() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cycle = inversion_cycle(&root);
    assert_eq!(cycle[0].interval_structure(), IntervalStructure(7, 7, 7));
    assert_eq!(cycle[1].interval_structure(), IntervalStructure(7, 10, 5));
    assert_eq!(cycle[2].interval_structure(), IntervalStructure(5, 9, 5));
    assert_eq!(cycle[3].interval_structure(), IntervalStructure(5, 10, 7));
}

#[test]
fn test_project_all_inversions_same() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cycle = inversion_cycle(&root);
    let pc = project(&cycle[0]).unwrap();
    for inv in &cycle[1..] {
        assert_eq!(project(inv).unwrap(), pc);
    }
}

#[test]
fn test_only_root_in_base_cgda() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cycle = inversion_cycle(&root);
    assert!(cycle[0].interval_structure().is_legal());
    assert!(!cycle[1].interval_structure().is_legal());
    assert!(!cycle[2].interval_structure().is_legal());
    assert!(!cycle[3].interval_structure().is_legal());
}

#[test]
fn test_saddle_two_in_base() {
    let root = VoicedChord::new([48, 54, 62, 68]).unwrap();
    let cycle = inversion_cycle(&root);
    let in_base: Vec<usize> = cycle
        .iter()
        .enumerate()
        .filter(|(_, inv)| inv.interval_structure().is_legal())
        .map(|(i, _)| i)
        .collect();
    assert_eq!(in_base, vec![0, 2]);
}

#[test]
fn test_saddle_intervals() {
    let root = VoicedChord::new([48, 54, 62, 68]).unwrap();
    let cycle = inversion_cycle(&root);
    assert_eq!(cycle[0].interval_structure(), IntervalStructure(6, 8, 6));
    assert_eq!(cycle[1].interval_structure(), IntervalStructure(6, 10, 6));
    assert_eq!(cycle[2].interval_structure(), IntervalStructure(6, 8, 6));
    assert_eq!(cycle[3].interval_structure(), IntervalStructure(6, 10, 6));
}

// --- l1_distance ---

#[test]
fn test_l1_distance_symmetric() {
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let b = VoicedChord::new([50, 57, 67, 72]).unwrap();
    assert_eq!(l1_distance(&a, &b), l1_distance(&b, &a));
}

#[test]
fn test_l1_root_to_1st_cgda() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let first = VoicedChord::new([50, 57, 67, 72]).unwrap();
    assert_eq!(l1_distance(&root, &first), 12);
}

#[test]
fn test_l1_cycle_cost_72() {
    let root = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let cycle = inversion_cycle(&root);
    // Each consecutive t1 step costs L1 = 12 (sum of chord-scale steps).
    let d01 = l1_distance(&cycle[0], &cycle[1]);
    let d12 = l1_distance(&cycle[1], &cycle[2]);
    let d23 = l1_distance(&cycle[2], &cycle[3]);
    assert_eq!(d01, 12);
    assert_eq!(d12, 12);
    assert_eq!(d23, 12);

    // The "closing" distance back to the ORIGINAL root (same octave)
    // is 36 because all four voices have ascended 3*12=36 semitones
    // over the preceding three steps. This gives the paper's
    // Universal L1 Law pattern [12, 12, 12, 36] with total 72.
    let d30 = l1_distance(&cycle[3], &cycle[0]);
    assert_eq!(d30, 36);
    assert_eq!(d01 + d12 + d23 + d30, 72);

    // The forward t1 step from inv3 to root' (T12) also costs 12:
    let root_prime = t1(&cycle[3]);
    assert_eq!(l1_distance(&cycle[3], &root_prime), 12);
}

#[test]
fn test_l1_self_zero() {
    let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
    assert_eq!(l1_distance(&a, &a), 0);
}
