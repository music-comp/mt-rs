extern crate music_comp_mt as theory;

use theory::quartal::{
    quartal_inversion_cycle, quartal_l1_distances, t_quartal, t_quartal_reverse, to_quartal,
    QuartalIntervalStructure, QuartalOrbit,
};
use theory::quintal::{self, inversion_cycle, BaseSpace, Orbit, VoicedChord};

// --- t_quartal ---

#[test]
fn test_t_quartal_is_t_minus1() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    let result = t_quartal(&qvc);
    let expected = quintal::t_minus1(&vc);
    assert_eq!(result.0, expected);
}

#[test]
fn test_t_quartal_reverse_is_t1() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    let result = t_quartal_reverse(&qvc);
    let expected = quintal::t1(&vc);
    assert_eq!(result.0, expected);
}

#[test]
fn test_t_quartal_round_trip() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    let forward = t_quartal(&qvc);
    let back = t_quartal_reverse(&forward);
    assert_eq!(back.0.to_pc_chord().unwrap(), qvc.0.to_pc_chord().unwrap());
}

#[test]
fn test_t_quartal_fourth_is_t_minus12() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    let q1 = t_quartal(&qvc);
    let q2 = t_quartal(&q1);
    let q3 = t_quartal(&q2);
    let q4 = t_quartal(&q3);
    // Should be original pitches - 12
    for i in 0..4 {
        assert_eq!(q4.pitches()[i], vc.pitches[i] - 12);
    }
}

// --- quartal_inversion_cycle ---

#[test]
fn test_quartal_cycle_reverses_quintal() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);

    let q_cycle = quartal_inversion_cycle(&qvc);
    let q5_cycle = inversion_cycle(&vc);

    // Quartal [inv0, inv1, inv2, inv3] = quintal [inv0, inv3, inv2, inv1]
    assert_eq!(
        q_cycle[0].0.to_pc_chord().unwrap(),
        q5_cycle[0].to_pc_chord().unwrap()
    );
    assert_eq!(
        q_cycle[1].0.to_pc_chord().unwrap(),
        q5_cycle[3].to_pc_chord().unwrap()
    );
    assert_eq!(
        q_cycle[2].0.to_pc_chord().unwrap(),
        q5_cycle[2].to_pc_chord().unwrap()
    );
    assert_eq!(
        q_cycle[3].0.to_pc_chord().unwrap(),
        q5_cycle[1].to_pc_chord().unwrap()
    );
}

#[test]
fn test_quartal_cycle_same_set_as_quintal() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);

    let q_pcs: std::collections::BTreeSet<_> = quartal_inversion_cycle(&qvc)
        .iter()
        .map(|c| c.0.to_pc_chord().unwrap())
        .collect();
    let q5_pcs: std::collections::BTreeSet<_> = inversion_cycle(&vc)
        .iter()
        .map(|c| c.to_pc_chord().unwrap())
        .collect();

    assert_eq!(q_pcs, q5_pcs);
}

// --- quartal_l1_distances ---

#[test]
fn test_quartal_l1_pattern() {
    let vc = VoicedChord::new([48, 55, 62, 69]).unwrap();
    let qvc = to_quartal(&vc);
    assert_eq!(quartal_l1_distances(&qvc), [12, 12, 12, 36]);
}

#[test]
fn test_quartal_l1_all_228() {
    let space = BaseSpace::new();
    for &chord in space.chords() {
        // Build a voiced chord for this pc chord
        if let Some(is) = chord.interval_structure() {
            // Try to build from first valid ordering
            for &start_pc in &chord.pcs {
                let base = 48 + start_pc;
                let pitches = [
                    base,
                    base + is.0,
                    base + is.0 + is.1,
                    base + is.0 + is.1 + is.2,
                ];
                if let Ok(vc) = VoicedChord::new(pitches) {
                    if let Ok(pc) = vc.to_pc_chord() {
                        if pc == chord {
                            let qvc = to_quartal(&vc);
                            let dists = quartal_l1_distances(&qvc);
                            assert_eq!(
                                dists,
                                [12, 12, 12, 36],
                                "Quartal L1 failed for {:?}",
                                chord
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}

// --- QuartalOrbit ---

#[test]
fn test_quartal_orbit_bijection() {
    for &q_orbit in Orbit::all() {
        let quartal = QuartalOrbit::from_quintal(&q_orbit);
        let back = quartal.to_quintal();
        assert_eq!(back, q_orbit, "Round-trip failed for {:?}", q_orbit);
    }
}

#[test]
fn test_quartal_orbit_all_14() {
    assert_eq!(QuartalOrbit::all().len(), 14);
}

#[test]
fn test_quartal_orbit_sizes_preserved() {
    for &q_orbit in Orbit::all() {
        let quartal = QuartalOrbit::from_quintal(&q_orbit);
        assert_eq!(quartal.size(), q_orbit.size());
    }
}

#[test]
fn test_quartal_orbit_degrees_preserved() {
    for &q_orbit in Orbit::all() {
        let quartal = QuartalOrbit::from_quintal(&q_orbit);
        assert_eq!(quartal.degree(), q_orbit.degree());
    }
}

#[test]
fn test_quartal_analogies() {
    assert_eq!(QuartalOrbit::Q555.analogy(), Some("major"));
    assert_eq!(QuartalOrbit::Q545.analogy(), Some("minor"));
    assert_eq!(QuartalOrbit::Q656.analogy(), Some("diminished"));
    assert_eq!(QuartalOrbit::Q646.analogy(), Some("augmented"));
    assert_eq!(QuartalOrbit::Q454.analogy(), None);
}

#[test]
fn test_quartal_orbit_display() {
    let s = format!("{}", QuartalOrbit::Q555);
    assert_eq!(s, "[P4,P4,P4] [major]");
}

#[test]
fn test_quartal_orbit_display_no_analogy() {
    let s = format!("{}", QuartalOrbit::Q454);
    assert_eq!(s, "[d4,P4,d4]");
}

#[test]
fn test_quartal_orbit_representatives() {
    assert_eq!(
        QuartalOrbit::Q555.representative(),
        QuartalIntervalStructure(5, 5, 5)
    );
    assert_eq!(
        QuartalOrbit::Q646.representative(),
        QuartalIntervalStructure(6, 4, 6)
    );
}

// --- re-exports ---

#[test]
fn test_reexported_base_space() {
    let space = theory::quartal::BaseSpace::new();
    assert_eq!(space.len(), 228);
}

#[test]
fn test_reexported_distance() {
    let space = theory::quartal::BaseSpace::new();
    let a = theory::quartal::PcChord::new([0, 2, 7, 9]).unwrap();
    let b = theory::quartal::PcChord::new([0, 2, 6, 9]).unwrap();
    let d_quartal = theory::quartal::distance(&space, &a, &b);
    let d_quintal = theory::quintal::distance(&space, &a, &b);
    assert_eq!(d_quartal, d_quintal);
}
