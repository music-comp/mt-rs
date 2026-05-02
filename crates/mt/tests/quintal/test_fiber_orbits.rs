extern crate music_comp_mt as theory;

use std::collections::{BTreeMap, BTreeSet};
use theory::quintal::{inversion_cycle, BaseSpace, VoicedChord};

/// All 24 permutations of indices [0, 1, 2, 3].
/// Duplicated from crates/mt/src/quintal/types.rs (not public).
const PERMUTATIONS_4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// Extract voice-order pitch classes from a VoicedChord (octave-invariant).
fn voice_order_pcs(vc: &VoicedChord) -> [u8; 4] {
    [
        vc.pitches[0] % 12,
        vc.pitches[1] % 12,
        vc.pitches[2] % 12,
        vc.pitches[3] % 12,
    ]
}

/// Compute the orbit fingerprint: the set of voice-order PC tuples
/// across all 4 members of the T1 cycle.
fn orbit_fingerprint(voiced: &VoicedChord) -> BTreeSet<[u8; 4]> {
    inversion_cycle(voiced)
        .iter()
        .map(|vc| voice_order_pcs(vc))
        .collect()
}

/// Build a VoicedChord from a stacking (pitch classes in voicing order) and
/// its computed intervals, anchored at MIDI octave 1 (bottom voice = 12 + bottom_pc).
fn build_voiced(ordered_pcs: &[u8; 4], i1: u16, i2: u16, i3: u16) -> VoicedChord {
    let bottom = 12 + ordered_pcs[0];
    let pitches: [u8; 4] = [
        bottom,
        bottom + i1 as u8,
        bottom + i1 as u8 + i2 as u8,
        bottom + i1 as u8 + i2 as u8 + i3 as u8,
    ];
    VoicedChord::new(pitches).unwrap_or_else(|_| {
        panic!(
            "failed to build VoicedChord from pcs {:?}, intervals ({},{},{}), pitches {:?}",
            ordered_pcs, i1, i2, i3, pitches
        )
    })
}

#[test]
fn test_exactly_two_orbits_per_pc_chord() {
    let space = BaseSpace::new();

    let mut quintal_stacking_distribution: BTreeMap<usize, usize> = BTreeMap::new();
    let mut quartal_stacking_distribution: BTreeMap<usize, usize> = BTreeMap::new();
    let mut voicing_count_distribution: BTreeMap<usize, usize> = BTreeMap::new();
    let mut failures: Vec<String> = Vec::new();

    for pc_chord in space.chords() {
        let pcs = pc_chord.pcs;

        let mut quintal_voiced: Vec<VoicedChord> = Vec::new();
        let mut quartal_voiced: Vec<VoicedChord> = Vec::new();

        for perm in &PERMUTATIONS_4 {
            let ordered = [pcs[perm[0]], pcs[perm[1]], pcs[perm[2]], pcs[perm[3]]];

            let i1 = (ordered[1] as u16 + 12 - ordered[0] as u16) % 12;
            let i2 = (ordered[2] as u16 + 12 - ordered[1] as u16) % 12;
            let i3 = (ordered[3] as u16 + 12 - ordered[2] as u16) % 12;

            let is_quintal =
                (6..=8).contains(&i1) && (6..=8).contains(&i2) && (6..=8).contains(&i3);
            let is_quartal =
                (4..=6).contains(&i1) && (4..=6).contains(&i2) && (4..=6).contains(&i3);

            if is_quintal {
                quintal_voiced.push(build_voiced(&ordered, i1, i2, i3));
            }
            if is_quartal {
                quartal_voiced.push(build_voiced(&ordered, i1, i2, i3));
            }
        }

        *quintal_stacking_distribution
            .entry(quintal_voiced.len())
            .or_insert(0) += 1;
        *quartal_stacking_distribution
            .entry(quartal_voiced.len())
            .or_insert(0) += 1;

        // Group quintal stackings by orbit fingerprint
        let mut quintal_orbits: BTreeMap<BTreeSet<[u8; 4]>, Vec<VoicedChord>> = BTreeMap::new();
        for vc in &quintal_voiced {
            let fp = orbit_fingerprint(vc);
            quintal_orbits.entry(fp).or_default().push(*vc);
        }

        // Group quartal stackings by orbit fingerprint
        let mut quartal_orbits: BTreeMap<BTreeSet<[u8; 4]>, Vec<VoicedChord>> = BTreeMap::new();
        for vc in &quartal_voiced {
            let fp = orbit_fingerprint(vc);
            quartal_orbits.entry(fp).or_default().push(*vc);
        }

        // Check exactly 1 quintal orbit
        if quintal_orbits.len() != 1 {
            failures.push(format!(
                "PcChord {:?}: expected 1 quintal orbit, got {} (stackings: {:?})",
                pcs,
                quintal_orbits.len(),
                quintal_voiced
            ));
        }

        // Check exactly 1 quartal orbit
        if quartal_orbits.len() != 1 {
            failures.push(format!(
                "PcChord {:?}: expected 1 quartal orbit, got {} (stackings: {:?})",
                pcs,
                quartal_orbits.len(),
                quartal_voiced
            ));
        }

        // Check orbits are disjoint
        if quintal_orbits.len() == 1 && quartal_orbits.len() == 1 {
            let quintal_fp = quintal_orbits.keys().next().unwrap();
            let quartal_fp = quartal_orbits.keys().next().unwrap();
            let overlap: Vec<_> = quintal_fp.intersection(quartal_fp).collect();
            if !overlap.is_empty() {
                failures.push(format!(
                    "PcChord {:?}: quintal and quartal orbits share {} voicings: {:?}",
                    pcs,
                    overlap.len(),
                    overlap
                ));
            }

            // Count total distinct voicings modulo octave
            let mut all_voicings: BTreeSet<[u8; 4]> = BTreeSet::new();
            all_voicings.extend(quintal_fp);
            all_voicings.extend(quartal_fp);
            *voicing_count_distribution
                .entry(all_voicings.len())
                .or_insert(0) += 1;

            if all_voicings.len() != 8 {
                failures.push(format!(
                    "PcChord {:?}: expected 8 total voicings modulo octave, got {}",
                    pcs,
                    all_voicings.len()
                ));
            }
        }
    }

    // Report results
    eprintln!("\n=== Fiber-Orbit Verification Results ===");
    eprintln!(
        "Quintal legal-stacking distribution (num_stackings -> num_chords): {:?}",
        quintal_stacking_distribution
    );
    eprintln!(
        "Quartal legal-stacking distribution (num_stackings -> num_chords): {:?}",
        quartal_stacking_distribution
    );
    eprintln!(
        "Total voicing count distribution (num_voicings -> num_chords): {:?}",
        voicing_count_distribution
    );

    if failures.is_empty() {
        eprintln!("\nAll 228 PcChords verified:");
        eprintln!("  - exactly 1 quintal T1 orbit per chord");
        eprintln!("  - exactly 1 quartal T1 orbit per chord");
        eprintln!("  - orbits are disjoint");
        eprintln!("  - 8 distinct VoicedChords per chord modulo octave");
    } else {
        eprintln!("\n{} failures:", failures.len());
        for f in &failures {
            eprintln!("  {}", f);
        }
    }

    assert!(
        failures.is_empty(),
        "{} chords failed orbit verification. Run with --nocapture for details.",
        failures.len()
    );
}
