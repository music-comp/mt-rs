//! Distance matrix dump: all-pairs BFS distances over the quintal base space.
//!
//! For every chord in the 228-chord base space, runs a single-source BFS via
//! [`all_distances_from`] and emits CSV rows of the form:
//!
//! ```text
//! source_pcs,source_orbit,target_pcs,target_orbit,distance
//! ```
//!
//! where `*_pcs` is a 4-tuple like `0-2-7-9` (dash-separated, sorted ascending —
//! the canonical form held inside [`PcChord`]). The source is excluded from its
//! own per-source rows (we emit `source != target` pairs only). Total output:
//! 228 × 227 = 51,756 data rows + 1 header row.
//!
//! Orbit identifiers are emitted via the [`Orbit`] enum's `Debug` format, which
//! is the variant name verbatim (e.g. `Q777`, `Q686`) — the same short form used
//! by external analyses (e.g. `oth_chord_graph.dot`). The `Display` impl is
//! deliberately *not* used here because it expands to `Q(7,7,7) [analogy]`,
//! which contains commas and would corrupt the CSV.
//!
//! Run with:
//! ```sh
//! cargo run --release -p music-comp-mt --example distance_matrix \
//!     > chord_distances.csv
//! ```
//!
//! Output is deterministic: rows are ordered by `BaseSpace::chords()` for the
//! source and then again for the target, so successive runs produce byte-for-byte
//! identical CSVs.

use music_comp_mt::quintal::{all_distances_from, classify_orbit, BaseSpace, PcChord};

fn pcs_to_str(pcs: &[u8; 4]) -> String {
    format!("{}-{}-{}-{}", pcs[0], pcs[1], pcs[2], pcs[3])
}

fn main() {
    let space = BaseSpace::new();
    let chords: Vec<PcChord> = space.chords().to_vec();

    println!("source_pcs,source_orbit,target_pcs,target_orbit,distance");

    for source in &chords {
        let source_orbit = classify_orbit(source)
            .expect("every chord in the base space is classified into an orbit");
        let distances = all_distances_from(&space, source);

        // Iterate over `chords` (not the HashMap) so row ordering is deterministic.
        for target in &chords {
            if target == source {
                continue;
            }
            let d = distances
                .get(target)
                .copied()
                .expect("base space is connected; every other chord is reachable");
            let target_orbit = classify_orbit(target)
                .expect("every chord in the base space is classified into an orbit");
            println!(
                "{},{:?},{},{:?},{}",
                pcs_to_str(&source.pcs),
                source_orbit,
                pcs_to_str(&target.pcs),
                target_orbit,
                d
            );
        }
    }
}
