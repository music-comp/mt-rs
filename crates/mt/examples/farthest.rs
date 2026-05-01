use music_comp_mt::quintal::{
    all_distances_from, betweenness_centrality, count_geodesics, saddle_chords, BaseSpace, PcChord,
};

fn main() {
    let space = BaseSpace::new();
    let from = PcChord::from_unsorted(&[0, 2, 7, 9]).unwrap();
    let dists = all_distances_from(&space, &from);
    let mut by_dist: std::collections::BTreeMap<u8, Vec<PcChord>> = Default::default();
    for (c, d) in &dists {
        by_dist.entry(*d).or_default().push(*c);
    }
    println!("Distance distribution from C-D-G-A {:?}:", from.pcs());
    let mut total = 0;
    for (d, set) in &by_dist {
        println!("  d={}: {} chords", d, set.len());
        total += set.len();
    }
    println!("  total: {}", total);

    let dmax = *by_dist.keys().max().unwrap();
    println!("\nFarthest chords (d={}):", dmax);
    for c in &by_dist[&dmax] {
        let pcs = c.pcs();
        let g = count_geodesics(&space, &from, c);
        println!("  {:?} - {} geodesics", pcs, g);
    }

    // Also show d=6 and d=7
    if let Some(set) = by_dist.get(&6) {
        let max_g = set
            .iter()
            .map(|c| count_geodesics(&space, &from, c))
            .max()
            .unwrap_or(0);
        println!("\nMax geodesics at d=6: {}", max_g);
    }

    // Saddle betweenness
    let bc = betweenness_centrality(&space);
    let cr = saddle_chords(&space);
    println!("\nSaddle chords (n={}):", cr.len());
    for c in &cr {
        let v = bc.get(c).copied().unwrap_or(0.0);
        println!("  {:?} bc={:.6}", c.pcs(), v);
    }
}
