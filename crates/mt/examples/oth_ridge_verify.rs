//! Verification harness for OTH ridge / connector-profile claims.
//!
//! Independently re-derives the structural facts in
//! `oth_ridge_theoretical_roots.md` using only the public mt-rs `quintal`
//! API. Used to audit a previous Claude session whose Python helpers were
//! buggy — the goal is for every claim to be checked from primitives the
//! library is already known to compute correctly (`BaseSpace`,
//! `classify_orbit`, `betweenness_centrality`, `inversion_cycle`).
//!
//! Run with:
//!
//! ```bash
//! cargo run --release -p music-comp-mt --example oth_ridge_verify
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};

use music_comp_mt::quintal::{
    betweenness_centrality, classify_orbit, inversion_cycle, BaseSpace, IntervalStructure, Orbit,
    PcChord, VoicedChord,
};

const RIDGE_ORBITS: [Orbit; 3] = [Orbit::Q777, Orbit::Q787, Orbit::Q686];

fn header(s: &str) {
    println!("\n=== {} ===", s);
}

fn section(s: &str) {
    println!("\n--- {} ---", s);
}

fn check(label: &str, ok: bool) {
    let mark = if ok { "PASS" } else { "FAIL" };
    println!("  [{mark}] {label}");
}

/// Try to build a [6,8] voicing for a given pcset by trying each pitch class
/// as the bottom note and checking that the inherited stacking lies in
/// {6, 7, 8}. Mirrors the private `pc_chord_to_voiced` helper in
/// `verification.rs` but exposed locally so we can reason about it.
fn pc_to_voicing(chord: &PcChord, base_octave: u8) -> Option<VoicedChord> {
    let is = chord.interval_structure()?;
    for &start_pc in &chord.pcs {
        let base = base_octave * 12 + start_pc;
        let pitches = [
            base,
            base + is.0,
            base + is.0 + is.1,
            base + is.0 + is.1 + is.2,
        ];
        if let Ok(vc) = VoicedChord::new(pitches) {
            if let Ok(pc) = vc.to_pc_chord() {
                if pc == *chord {
                    return Some(vc);
                }
            }
        }
    }
    None
}

fn orbit_recipe(o: Orbit) -> IntervalStructure {
    o.representative()
}

fn recipe_str(is: IntervalStructure) -> String {
    format!("({},{},{})", is.0, is.1, is.2)
}

fn main() {
    let space = BaseSpace::new();

    // ------------------------------------------------------------------
    // 1. Cardinality of B and the orbit decomposition.
    // ------------------------------------------------------------------
    header("Layer 1 — base-space cardinality and orbits");
    println!("|B| = {}", space.len());
    check("|B| = 228", space.len() == 228);

    let mut orbit_chords: BTreeMap<Orbit, Vec<PcChord>> = BTreeMap::new();
    for &c in space.chords() {
        let o = classify_orbit(&c).expect("every chord in B classifies");
        orbit_chords.entry(o).or_default().push(c);
    }
    check("# orbits = 14", orbit_chords.len() == 14);

    let total_from_orbits: usize = orbit_chords.values().map(|v| v.len()).sum();
    check("orbit sizes sum to 228", total_from_orbits == 228);

    section("orbit table (size, declared degree, recipe)");
    let mut size_distribution: BTreeMap<usize, usize> = BTreeMap::new();
    for o in Orbit::all() {
        let chords = &orbit_chords[o];
        let declared_size = o.size();
        let declared_degree = o.degree();
        // Confirm declared size matches the actual orbit count in B.
        check(
            &format!(
                "{} declared size {} matches actual {}",
                o,
                declared_size,
                chords.len()
            ),
            declared_size == chords.len(),
        );
        // Confirm every chord in the orbit has the same degree as declared.
        let actual_degrees: BTreeSet<usize> =
            chords.iter().map(|c| space.degree(c).unwrap()).collect();
        check(
            &format!(
                "{} all chords have degree {} (saw {:?})",
                o, declared_degree, actual_degrees
            ),
            actual_degrees.len() == 1 && actual_degrees.contains(&declared_degree),
        );
        *size_distribution.entry(declared_size).or_insert(0) += 1;
        println!(
            "  {:5} size={:>2} degree={} recipe={}",
            format!("{:?}", o),
            declared_size,
            declared_degree,
            recipe_str(orbit_recipe(*o)),
        );
    }
    println!("size distribution: {:?}", size_distribution);

    // ------------------------------------------------------------------
    // 2. Summit and ridge.
    // ------------------------------------------------------------------
    header("Layer 2 — Summit + ridge identification");
    let max_degree = Orbit::all().iter().map(|o| o.degree()).max().unwrap();
    check("max orbit degree = 8", max_degree == 8);

    let degree8_orbits: Vec<Orbit> = Orbit::all()
        .iter()
        .copied()
        .filter(|o| o.degree() == 8)
        .collect();
    println!("degree-8 orbits: {:?}", degree8_orbits);
    check(
        "degree-8 orbits = {Q777, Q787, Q686}",
        degree8_orbits.len() == 3
            && degree8_orbits.contains(&Orbit::Q777)
            && degree8_orbits.contains(&Orbit::Q787)
            && degree8_orbits.contains(&Orbit::Q686),
    );

    let ridge_size: usize = degree8_orbits.iter().map(|o| o.size()).sum();
    println!("|ridge| (sum of degree-8 orbit sizes) = {}", ridge_size);
    check("|ridge| = 30 (= 12 + 12 + 6)", ridge_size == 30);

    // Summit uniqueness as the orbit of *unique* maximum degree?
    // Strictly false: three orbits all hit degree 8. So "Summit" is a
    // distinguished point only after a second criterion is applied.
    println!("  NOTE: 'Summit' is not the unique max-degree orbit — three orbits tie at degree 8.");
    println!("        The Summit / Plateau / Saddle distinction must come from another property.");

    // ------------------------------------------------------------------
    // 3. Saddle triple property.
    // ------------------------------------------------------------------
    header("Layer 3 — Saddle triple-property check");
    let bc: HashMap<PcChord, f64> = betweenness_centrality(&space);

    // Aggregate betweenness by orbit.
    let mut orbit_bc_sum: BTreeMap<Orbit, f64> = BTreeMap::new();
    let mut orbit_bc_count: BTreeMap<Orbit, usize> = BTreeMap::new();
    for (chord, &val) in &bc {
        let o = classify_orbit(chord).unwrap();
        *orbit_bc_sum.entry(o).or_insert(0.0) += val;
        *orbit_bc_count.entry(o).or_insert(0) += 1;
    }
    section("orbit-level betweenness (mean over orbit)");
    let mut orbit_bc_mean: Vec<(Orbit, f64)> = orbit_bc_sum
        .iter()
        .map(|(o, sum)| (*o, sum / orbit_bc_count[o] as f64))
        .collect();
    orbit_bc_mean.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (o, mean) in &orbit_bc_mean {
        println!("  {:?}  {:.6}", o, mean);
    }

    let max_bc_orbit = orbit_bc_mean[0].0;
    println!("orbit of max mean betweenness: {:?}", max_bc_orbit);
    check(
        "max-betweenness orbit is Q686 (Saddle)",
        max_bc_orbit == Orbit::Q686,
    );

    // Inversion stability: how many of the four T1-cycle inversions of a
    // representative voicing land in B (i.e., have an interval structure
    // in [6,8]). Class B chords give 2; Class A chords give 1.
    section("inversion stability per orbit (count of inversions whose stacking is legal)");
    let mut inv_in_base: BTreeMap<Orbit, usize> = BTreeMap::new();
    for o in Orbit::all() {
        let rep_pc = orbit_chords[o][0];
        let voicing = pc_to_voicing(&rep_pc, 4).expect("orbit reps voice cleanly at C4");
        let cycle = inversion_cycle(&voicing);
        let count = cycle
            .iter()
            .filter(|v| v.interval_structure().is_legal())
            .count();
        inv_in_base.insert(*o, count);
        println!(
            "  {:?}  recipe={}  inversions-in-B={}",
            o,
            recipe_str(orbit_recipe(*o)),
            count
        );
    }
    let max_inv_stability = *inv_in_base.values().max().unwrap();
    let max_inv_orbits: Vec<Orbit> = inv_in_base
        .iter()
        .filter(|(_, &v)| v == max_inv_stability)
        .map(|(o, _)| *o)
        .collect();
    println!(
        "max inversion stability = {} achieved by: {:?}",
        max_inv_stability, max_inv_orbits
    );
    check(
        "Q686 attains max inversion stability",
        max_inv_orbits.contains(&Orbit::Q686),
    );
    let q686_unique = max_inv_orbits.len() == 1 && max_inv_orbits[0] == Orbit::Q686;
    check(
        "Q686 is the *unique* max-inversion-stability orbit",
        q686_unique,
    );

    // Max degree among all orbits is 8 — Q686 ties; not strictly maximum.
    let q686_degree = Orbit::Q686.degree();
    println!(
        "Q686 degree = {}, global max degree = {}",
        q686_degree, max_degree
    );
    check(
        "Q686 has max degree (tied with Q777, Q787)",
        q686_degree == max_degree,
    );

    // ------------------------------------------------------------------
    // 4. Orbit-adjacency multigraph (foundational for connector profiles).
    // ------------------------------------------------------------------
    header("Layer 4 — orbit-adjacency multigraph");
    // For each pair (i, j), edge_count[(i,j)] = number of *unordered* edges
    // in B with one endpoint in orbit i and the other in orbit j.
    // For directed-adjacency-from-orbit-i, we use directed_count[(i,j)].
    let mut directed_count: BTreeMap<(Orbit, Orbit), usize> = BTreeMap::new();
    for &c in space.chords() {
        let oi = classify_orbit(&c).unwrap();
        let neigh = space.neighbors(&c).unwrap();
        for &nidx in neigh {
            let oj = classify_orbit(&space.chords()[nidx]).unwrap();
            *directed_count.entry((oi, oj)).or_insert(0) += 1;
        }
    }
    // Per-chord neighbor count from orbit i to orbit j is
    // directed_count[(i,j)] / |orbit i| because the orbit acts transitively.
    let per_chord_count = |from: Orbit, to: Orbit| -> f64 {
        let total = *directed_count.get(&(from, to)).unwrap_or(&0);
        total as f64 / from.size() as f64
    };

    // Sanity: per-chord total neighbors should equal declared degree for
    // every orbit.
    section("sanity: sum of per-chord neighbor counts = declared degree");
    for &from in Orbit::all() {
        let total: f64 = Orbit::all()
            .iter()
            .map(|&to| per_chord_count(from, to))
            .sum();
        check(
            &format!("{:?} per-chord neighbor sum = {}", from, from.degree()),
            (total - from.degree() as f64).abs() < 1e-9,
        );
    }

    // ------------------------------------------------------------------
    // 5. Connector-profile theorem (Theorem D).
    // ------------------------------------------------------------------
    header("Layer 5 — connector-profile theorem (Theorem D)");
    let connectors = [Orbit::Q776, Orbit::Q786, Orbit::Q867, Orbit::Q877];
    section("per-chord neighbor counts FROM each connector orbit TO each ridge orbit");
    println!(
        "  {:>5} | {:>6} {:>6} {:>6}",
        "from", "Q777", "Q787", "Q686"
    );
    for &c in &connectors {
        let to_summit = per_chord_count(c, Orbit::Q777);
        let to_plateau = per_chord_count(c, Orbit::Q787);
        let to_saddle = per_chord_count(c, Orbit::Q686);
        println!(
            "  {:>5} | {:>6.2} {:>6.2} {:>6.2}",
            format!("{:?}", c),
            to_summit,
            to_plateau,
            to_saddle
        );
    }

    let theorem_d = [
        (Orbit::Q776, [1.0, 0.0, 1.0]),
        (Orbit::Q786, [1.0, 2.0, 1.0]),
        (Orbit::Q867, [1.0, 0.0, 0.0]),
        (Orbit::Q877, [1.0, 1.0, 0.0]),
    ];
    section("verifying Theorem D values exactly");
    for (c, [s, p, sa]) in theorem_d {
        let to_s = per_chord_count(c, Orbit::Q777);
        let to_p = per_chord_count(c, Orbit::Q787);
        let to_sa = per_chord_count(c, Orbit::Q686);
        check(
            &format!(
                "{:?} -> (Summit={:.2}, Plateau={:.2}, Saddle={:.2}) expected ({}, {}, {})",
                c, to_s, to_p, to_sa, s, p, sa
            ),
            (to_s - s).abs() < 1e-9 && (to_p - p).abs() < 1e-9 && (to_sa - sa).abs() < 1e-9,
        );
    }

    // ------------------------------------------------------------------
    // 6. Ridge-edge theorem and length-2 ridge walks.
    // ------------------------------------------------------------------
    header("Layer 6 — ridge-edge theorem + length-2 ridge walks");
    let ridge_set: BTreeSet<PcChord> = RIDGE_ORBITS
        .iter()
        .flat_map(|o| orbit_chords[o].iter().copied())
        .collect();
    println!("|ridge_set| = {}", ridge_set.len());

    // Direct ridge-to-ridge edges in B?
    let mut ridge_to_ridge_edges = 0usize;
    for &r in &ridge_set {
        for &nidx in space.neighbors(&r).unwrap() {
            let n = space.chords()[nidx];
            if ridge_set.contains(&n) {
                ridge_to_ridge_edges += 1;
            }
        }
    }
    check(
        "no atomic edges between ridge chords (ridge-edge theorem)",
        ridge_to_ridge_edges == 0,
    );

    // Length-2 walks: ridge -> connector -> ridge.
    // Catalog by (orbit_of_endpoint_a, orbit_of_endpoint_b, connector_orbit),
    // canonicalizing endpoint pair to be unordered.
    let mut walk_catalog: BTreeMap<(Orbit, Orbit, Orbit), usize> = BTreeMap::new();
    let mut total_walks_ordered = 0usize;
    for &a in &ridge_set {
        let oa = classify_orbit(&a).unwrap();
        for &mid_idx in space.neighbors(&a).unwrap() {
            let m = space.chords()[mid_idx];
            let om = classify_orbit(&m).unwrap();
            for &b_idx in space.neighbors(&m).unwrap() {
                let b = space.chords()[b_idx];
                if a == b {
                    continue;
                }
                if !ridge_set.contains(&b) {
                    continue;
                }
                let ob = classify_orbit(&b).unwrap();
                let pair = if (oa as u8) <= (ob as u8) {
                    (oa, ob)
                } else {
                    (ob, oa)
                };
                *walk_catalog.entry((pair.0, pair.1, om)).or_insert(0) += 1;
                total_walks_ordered += 1;
            }
        }
    }

    section("length-2 ordered walks by (endpoint-orbit-pair, connector orbit)");
    let mut walk_rows: Vec<((Orbit, Orbit, Orbit), usize)> =
        walk_catalog.iter().map(|(k, v)| (*k, *v)).collect();
    walk_rows.sort_by_key(|row| std::cmp::Reverse(row.1));
    for ((oa, ob, om), count) in &walk_rows {
        println!("  {{{:?}, {:?}}} via {:?}  count = {}", oa, ob, om, count);
    }
    println!(
        "total ordered length-2 ridge walks = {}",
        total_walks_ordered
    );
    check(
        "total ordered length-2 ridge walks = 384",
        total_walks_ordered == 384,
    );
    check("# distinct walk types = 6", walk_catalog.len() == 6);

    // Q786-mediated walks fraction.
    let via_q786: usize = walk_catalog
        .iter()
        .filter(|((_, _, om), _)| *om == Orbit::Q786)
        .map(|(_, c)| *c)
        .sum();
    let frac = via_q786 as f64 / total_walks_ordered as f64;
    println!(
        "ordered walks via Q786 = {} / {} = {:.4}",
        via_q786, total_walks_ordered, frac
    );
    check(
        "Q786 mediates 75% of all length-2 ridge walks (NOT 87.5%)",
        (frac - 0.75).abs() < 1e-9,
    );

    // ------------------------------------------------------------------
    // 7. Recipe-perturbation graph at the orbit level.
    // ------------------------------------------------------------------
    header("Layer 7 — recipe-perturbation reachability per orbit");
    section("R(orbit) := {orbit' : exists chord in orbit with a B-neighbor in orbit'}");
    for &from in Orbit::all() {
        let mut reachable: BTreeSet<Orbit> = BTreeSet::new();
        for &c in &orbit_chords[&from] {
            for &nidx in space.neighbors(&c).unwrap() {
                let n = space.chords()[nidx];
                let to = classify_orbit(&n).unwrap();
                reachable.insert(to);
            }
        }
        let r_str: Vec<String> = reachable.iter().map(|o| format!("{:?}", o)).collect();
        println!("  R({:?}) = {{{}}}", from, r_str.join(", "));
    }

    // Universal connector check: which orbits lie in R(Q777) ∩ R(Q787) ∩ R(Q686)?
    let mut tri_intersection: BTreeSet<Orbit> = BTreeSet::new();
    for &candidate in Orbit::all() {
        let in_all = RIDGE_ORBITS.iter().all(|&ridge| {
            orbit_chords[&ridge].iter().any(|c| {
                space
                    .neighbors(c)
                    .unwrap()
                    .iter()
                    .any(|&nidx| classify_orbit(&space.chords()[nidx]) == Some(candidate))
            })
        });
        if in_all {
            tri_intersection.insert(candidate);
        }
    }
    println!(
        "R(Q777) ∩ R(Q787) ∩ R(Q686) = {:?}",
        tri_intersection.iter().collect::<Vec<_>>()
    );
    check(
        "Q786 is the unique orbit reachable from all three ridge orbits in 1 step",
        tri_intersection.len() == 1 && tri_intersection.contains(&Orbit::Q786),
    );

    // Saddle wing-asymmetry: is Q877 ∉ R(Q686)?
    let q877_from_q686 = orbit_chords[&Orbit::Q686].iter().any(|c| {
        space
            .neighbors(c)
            .unwrap()
            .iter()
            .any(|&nidx| classify_orbit(&space.chords()[nidx]) == Some(Orbit::Q877))
    });
    check(
        "Saddle (Q686) cannot reach Q877 in one step (wing asymmetry)",
        !q877_from_q686,
    );

    // ------------------------------------------------------------------
    // 8. Pcset-vs-voicing distinction at the [6,8] level.
    // ------------------------------------------------------------------
    header("Layer 8 — pcset adjacency vs single-voice register-stable voicing motion");
    // For each ridge chord, build a [6,8] voicing at C4. Then for each
    // pcset-neighbor (single-voice perturbation in B), determine whether
    // *some* [6,8] voicing of the neighbor is reachable from the source
    // voicing by changing exactly one voice (i.e., 3 of the 4 voicing
    // pitches stay identical). If not, the move requires a fiber-rotation
    // of the destination voicing.
    let mut total_pcset_edges = 0usize;
    let mut requires_rotation = 0usize;
    let mut sample_witnesses: Vec<(PcChord, PcChord, [u8; 4], [u8; 4])> = Vec::new();
    for &src in &ridge_set {
        let src_v = pc_to_voicing(&src, 4).unwrap();
        for &nidx in space.neighbors(&src).unwrap() {
            total_pcset_edges += 1;
            let dst = space.chords()[nidx];
            // Try every legal stacking of dst at every base octave that is
            // close enough; for our purpose, accept "single-voice motion"
            // if ANY [6,8] voicing of dst at base-octave 3, 4, or 5 differs
            // from src_v in exactly one of the four pitch positions
            // (treating the four positions as bottom..top order).
            let mut found_single_voice = false;
            'outer: for octave in 3..=5u8 {
                if let Some(dst_v) = pc_to_voicing(&dst, octave) {
                    let diff = src_v
                        .pitches
                        .iter()
                        .zip(dst_v.pitches.iter())
                        .filter(|(a, b)| a != b)
                        .count();
                    if diff == 1 {
                        found_single_voice = true;
                        break 'outer;
                    }
                }
                // Also try alternate stackings (e.g., when an interval is
                // forced to be 6 or 8 at a different rotation): we must
                // search over interval-structure rotations of dst that are
                // legal in [6,8]. The default voicing builder picks the
                // first legal rotation; here we enumerate.
                if let Some(is) = dst.interval_structure() {
                    let rotations = [
                        IntervalStructure(is.0, is.1, is.2),
                        IntervalStructure(is.1, is.2, is.0),
                        IntervalStructure(is.2, is.0, is.1),
                    ];
                    for rot in rotations {
                        if !rot.is_legal() {
                            continue;
                        }
                        for &start_pc in &dst.pcs {
                            let base = octave * 12 + start_pc;
                            let pitches = [
                                base,
                                base + rot.0,
                                base + rot.0 + rot.1,
                                base + rot.0 + rot.1 + rot.2,
                            ];
                            if let Ok(vc) = VoicedChord::new(pitches) {
                                if let Ok(pc) = vc.to_pc_chord() {
                                    if pc == dst {
                                        let diff = src_v
                                            .pitches
                                            .iter()
                                            .zip(vc.pitches.iter())
                                            .filter(|(a, b)| a != b)
                                            .count();
                                        if diff == 1 {
                                            found_single_voice = true;
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !found_single_voice {
                requires_rotation += 1;
                if sample_witnesses.len() < 5 {
                    let src_pcs = src.pcs;
                    let dst_pcs = dst.pcs;
                    sample_witnesses.push((src, dst, src_pcs, dst_pcs));
                }
            }
        }
    }
    println!(
        "of {} pcset edges out of ridge chords, {} require fiber rotation at the voicing level",
        total_pcset_edges, requires_rotation
    );
    check(
        "AT LEAST ONE pcset edge needs voicing rotation (pcset-vs-voicing distinction is real)",
        requires_rotation > 0,
    );
    if !sample_witnesses.is_empty() {
        section("witness pairs (pcset adjacency requiring fiber rotation)");
        for (src, dst, sp, dp) in sample_witnesses {
            println!(
                "  {:?}  pcs={:?} -- atomic --> {:?}  pcs={:?}",
                classify_orbit(&src).unwrap(),
                sp,
                classify_orbit(&dst).unwrap(),
                dp,
            );
        }
    }

    println!("\nDone.");
}
