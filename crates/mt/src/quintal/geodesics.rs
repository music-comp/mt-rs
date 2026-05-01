//! Geodesic (shortest-path) enumeration and counting on the base space graph.
//!
//! Provides functions to enumerate all shortest paths between two chords,
//! count them without materializing, find all metrically intermediate
//! ("passing") chords, and compute the full geodesic-distribution profile
//! from a single source.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, VecDeque};

use super::base_space::BaseSpace;
use super::orbit::{classify_orbit, Orbit};
use super::PcChord;

/// BFS from `source` recording distances, parent lists, and shortest-path counts.
///
/// Returns `(dist, parents, sigma)` where:
/// - `dist[v]` is `Some(d)` if `v` is reachable at distance `d`, else `None`
/// - `parents[v]` is the list of all neighbors of `v` at distance `d-1`
/// - `sigma[v]` is the number of shortest paths from `source` to `v`
fn bfs_with_parents(
    space: &BaseSpace,
    source: usize,
) -> (Vec<Option<u8>>, Vec<Vec<usize>>, Vec<usize>) {
    let n = space.len();
    let mut dist: Vec<Option<u8>> = vec![None; n];
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut sigma: Vec<usize> = vec![0; n];
    let mut queue = VecDeque::new();

    dist[source] = Some(0);
    sigma[source] = 1;
    queue.push_back(source);

    while let Some(current) = queue.pop_front() {
        let current_dist = dist[current].unwrap();
        for &ni in space.neighbors_by_index(current) {
            match dist[ni] {
                None => {
                    // First visit: set distance, record parent, propagate count.
                    dist[ni] = Some(current_dist + 1);
                    parents[ni].push(current);
                    sigma[ni] = sigma[current];
                    queue.push_back(ni);
                }
                Some(d) if d == current_dist + 1 => {
                    // Another shortest path through `current`.
                    parents[ni].push(current);
                    sigma[ni] += sigma[current];
                }
                _ => {}
            }
        }
    }

    (dist, parents, sigma)
}

/// Simple BFS returning only distances (no parent tracking).
fn bfs_dist_only(space: &BaseSpace, source: usize) -> Vec<Option<u8>> {
    let n = space.len();
    let mut dist: Vec<Option<u8>> = vec![None; n];
    let mut queue = VecDeque::new();

    dist[source] = Some(0);
    queue.push_back(source);

    while let Some(current) = queue.pop_front() {
        let current_dist = dist[current].unwrap();
        for &ni in space.neighbors_by_index(current) {
            if dist[ni].is_none() {
                dist[ni] = Some(current_dist + 1);
                queue.push_back(ni);
            }
        }
    }

    dist
}

/// Recursively collect all shortest paths from `source` to `current` using
/// the parent lists produced by BFS.
fn collect_paths(parents: &[Vec<usize>], current: usize, source: usize) -> Vec<Vec<usize>> {
    if current == source {
        return vec![vec![source]];
    }
    let mut all = Vec::new();
    for &p in &parents[current] {
        for mut path in collect_paths(parents, p, source) {
            path.push(current);
            all.push(path);
        }
    }
    all
}

/// All shortest paths from chord `a` to chord `b` in the base space.
///
/// Each path is a sequence of `PcChord`s starting at `a` and ending at `b`.
/// Returns an empty `Vec` if either chord is not in the space or if
/// no path exists (which cannot happen in the connected quintal base space).
///
/// Special case: if `a == b`, returns `vec![vec![a]]`.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{geodesics, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let a = PcChord::new([0, 2, 7, 9]).unwrap();
/// let paths = geodesics(&space, &a, &a);
/// assert_eq!(paths, vec![vec![a]]);
/// ```
pub fn geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<Vec<PcChord>> {
    let ai = match space.chord_index(a) {
        Some(i) => i,
        None => return vec![],
    };
    let bi = match space.chord_index(b) {
        Some(i) => i,
        None => return vec![],
    };
    if ai == bi {
        return vec![vec![*a]];
    }
    let (dist, parents, _) = bfs_with_parents(space, ai);
    if dist[bi].is_none() {
        return vec![];
    }
    let index_paths = collect_paths(&parents, bi, ai);
    index_paths
        .into_iter()
        .map(|path| path.into_iter().map(|i| space.chords()[i]).collect())
        .collect()
}

/// Count the number of shortest paths from chord `a` to chord `b` without
/// materializing them.
///
/// Returns 0 if either chord is not in the space. Returns 1 if `a == b`.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{count_geodesics, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let a = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(count_geodesics(&space, &a, &a), 1);
/// ```
pub fn count_geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> usize {
    let ai = match space.chord_index(a) {
        Some(i) => i,
        None => return 0,
    };
    let bi = match space.chord_index(b) {
        Some(i) => i,
        None => return 0,
    };
    if ai == bi {
        return 1;
    }
    let (dist, _, sigma) = bfs_with_parents(space, ai);
    if dist[bi].is_none() {
        0
    } else {
        sigma[bi]
    }
}

/// All chords metrically between `a` and `b`: every chord `z` where
/// `d(a, z) + d(z, b) == d(a, b)`.
///
/// The endpoints `a` and `b` are excluded from the result. Returns an
/// empty `Vec` if either chord is missing, if `a == b`, or if no
/// intermediate chords exist (i.e., `a` and `b` are adjacent).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{passing_chords, distance, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let a = PcChord::new([0, 2, 7, 9]).unwrap();
/// let b = PcChord::new([0, 2, 6, 9]).unwrap();
/// // Adjacent chords have no passing chords.
/// assert!(passing_chords(&space, &a, &b).is_empty());
/// ```
pub fn passing_chords(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<PcChord> {
    let ai = match space.chord_index(a) {
        Some(i) => i,
        None => return vec![],
    };
    let bi = match space.chord_index(b) {
        Some(i) => i,
        None => return vec![],
    };
    if ai == bi {
        return vec![];
    }
    let dist_a = bfs_dist_only(space, ai);
    let dist_b = bfs_dist_only(space, bi);
    let d_ab = match dist_a[bi] {
        Some(d) => d,
        None => return vec![],
    };

    let mut result = Vec::new();
    for i in 0..space.len() {
        if i == ai || i == bi {
            continue;
        }
        if let (Some(da), Some(db)) = (dist_a[i], dist_b[i]) {
            if da + db == d_ab {
                result.push(space.chords()[i]);
            }
        }
    }
    result
}

// ───────────────────────────── Distribution API ─────────────────────────────

/// The (distances, geodesic counts) pair returned by a single BFS pass.
///
/// **Both maps include the source** with `distance = 0` and
/// `geodesic_count = 1`, matching the σ(source | source) = 1 base case used
/// throughout the spec. For a connected source on `B`, both maps have
/// `len() == 228` (one entry per chord in the space).
///
/// If you want a profile that *excludes* the source — one row per *other*
/// chord — use [`geodesic_distribution`] instead.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistAndGeodesicCounts {
    /// Shortest-path distance from the source to each reachable chord.
    /// **Includes the source** at distance 0.
    pub distances: HashMap<PcChord, u8>,
    /// σ(target | source) — the count of distinct shortest paths from the
    /// source to each reachable chord. **Includes the source** with
    /// `geodesic_count = 1` (the σ(source | source) = 1 base case).
    pub geodesic_counts: HashMap<PcChord, u64>,
}

/// One row of the per-target geodesic profile.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicProfileEntry {
    /// The target chord this row is about.
    pub chord: PcChord,
    /// The orbit `chord` belongs to (always classifiable for chords in `B`).
    pub orbit: Orbit,
    /// Shortest-path distance from the source.
    pub distance: u8,
    /// σ(target | source) — the count of distinct shortest paths.
    pub geodesic_count: u64,
}

/// One row of the aggregate distribution table (one per distance bucket).
///
/// `max_chords` pairs each tying chord with its orbit so the two-vec
/// "must stay the same length" smell is replaced by a single vector of
/// `(chord, orbit)` tuples.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicBucket {
    /// The distance this bucket represents.
    pub distance: u8,
    /// How many chords lie at exactly this distance.
    pub chords_at_d: usize,
    /// Mean geodesic count across the bucket's chords.
    pub avg_geodesics: f64,
    /// Largest geodesic count in the bucket.
    pub max_geodesics: u64,
    /// Every chord that ties for `max_geodesics`, paired with its orbit.
    /// Sorted by `pcs` ascending for determinism.
    pub max_chords: Vec<(PcChord, Orbit)>,
}

/// Full result: aggregate buckets + per-chord detail + sanity totals.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicDistribution {
    /// The source chord this profile was generated from.
    pub source: PcChord,
    /// The orbit the source belongs to.
    pub source_orbit: Orbit,
    /// The eccentricity of the source — the largest distance to any
    /// reachable chord.
    pub eccentricity: u8,
    /// Total reachable chords **excluding the source itself**. On the
    /// connected quintal base space this is always |B| − 1 = 227.
    pub reachable_chords: usize,
    /// Buckets ordered by `distance` ascending. The graph is connected and
    /// every distance `1..=eccentricity` is populated for the sources we
    /// care about, but the API does not guarantee dense indexing — read
    /// `bucket.distance` rather than indexing by `d - 1`.
    pub buckets: Vec<GeodesicBucket>,
    /// Per-target detail, sorted by
    /// (distance ascending, geodesic_count descending, pcs ascending).
    ///
    /// **Excludes the source** — one row per *other* chord. For any in-space
    /// source, `per_chord.len() == reachable_chords` (= 227 on the connected
    /// base space) and `per_chord.iter().all(|e| e.distance >= 1)`.
    /// If you need σ(source | source) = 1, use
    /// [`distances_and_geodesic_counts`] instead.
    pub per_chord: Vec<GeodesicProfileEntry>,
}

/// Single-pass BFS exposing both distance and σ counts for every reachable
/// chord. Returns `None` if `source` is not a member of `space`.
///
/// **The source itself is included** in both maps with distance 0 and
/// σ = 1. For a connected source on `B`, both maps have `len() == 228`.
/// If you want one row per *other* chord (the source excluded), use
/// [`geodesic_distribution`] instead.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{distances_and_geodesic_counts, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// let result = distances_and_geodesic_counts(&space, &cgda).unwrap();
/// assert_eq!(result.distances.len(), 228);
/// assert_eq!(result.distances[&cgda], 0);
/// assert_eq!(result.geodesic_counts[&cgda], 1);
/// ```
#[must_use]
pub fn distances_and_geodesic_counts(
    space: &BaseSpace,
    source: &PcChord,
) -> Option<DistAndGeodesicCounts> {
    let si = space.chord_index(source)?;
    let (dist, _parents, sigma) = bfs_with_parents(space, si);
    let mut distances = HashMap::with_capacity(space.len());
    let mut geodesic_counts = HashMap::with_capacity(space.len());
    for (i, chord) in space.chords().iter().enumerate() {
        if let Some(d) = dist[i] {
            distances.insert(*chord, d);
            geodesic_counts.insert(*chord, sigma[i] as u64);
        }
    }
    Some(DistAndGeodesicCounts {
        distances,
        geodesic_counts,
    })
}

/// Compute the full §6 geodesic-distribution profile from `source` over the
/// base space.
///
/// Aggregates the single-pass BFS result of [`distances_and_geodesic_counts`]
/// into per-distance buckets (count, average σ, max σ, and the chord
/// identities tying for max σ) plus a per-target detail vector. Returns
/// `None` if `source` is not a member of `space`.
///
/// **The source itself is not included** as a row in `per_chord` and is
/// excluded from every bucket. With |B| = 228 and the source in the base
/// space, `per_chord.len() == reachable_chords == 227` — one row per
/// *other* chord. If you need σ(source | source) = 1, use
/// [`distances_and_geodesic_counts`] instead.
///
/// One BFS per call — O(|V| + |E|) on the quintal base space, sub-millisecond
/// for |B| = 228.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{geodesic_distribution, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// let dist = geodesic_distribution(&space, &cgda).unwrap();
/// assert_eq!(dist.reachable_chords, 227);
/// assert_eq!(dist.per_chord.len(), 227);  // source excluded
/// ```
#[must_use]
pub fn geodesic_distribution(space: &BaseSpace, source: &PcChord) -> Option<GeodesicDistribution> {
    let DistAndGeodesicCounts {
        distances,
        geodesic_counts,
    } = distances_and_geodesic_counts(space, source)?;

    // Bucket targets by distance, excluding the source (distance 0).
    let mut by_distance: BTreeMap<u8, Vec<PcChord>> = BTreeMap::new();
    for (chord, &d) in &distances {
        if d == 0 {
            continue;
        }
        by_distance.entry(d).or_default().push(*chord);
    }

    let buckets: Vec<GeodesicBucket> = by_distance
        .iter()
        .map(|(&distance, members)| {
            let chords_at_d = members.len();
            let sum_geodesics: u64 = members.iter().map(|c| geodesic_counts[c]).sum();
            let avg_geodesics = sum_geodesics as f64 / chords_at_d as f64;
            let max_geodesics = members
                .iter()
                .map(|c| geodesic_counts[c])
                .max()
                .expect("bucket is non-empty by construction");

            let mut max_chords: Vec<(PcChord, Orbit)> = members
                .iter()
                .filter(|c| geodesic_counts[*c] == max_geodesics)
                .map(|c| {
                    let orbit =
                        classify_orbit(c).expect("BaseSpace chords are always classifiable");
                    (*c, orbit)
                })
                .collect();
            max_chords.sort_by_key(|(c, _)| c.pcs);

            GeodesicBucket {
                distance,
                chords_at_d,
                avg_geodesics,
                max_geodesics,
                max_chords,
            }
        })
        .collect();

    let eccentricity = buckets.last().map(|b| b.distance).unwrap_or(0);
    let reachable_chords: usize = buckets.iter().map(|b| b.chords_at_d).sum();

    let mut per_chord: Vec<GeodesicProfileEntry> = distances
        .iter()
        .filter(|(_, &distance)| distance != 0)
        .map(|(chord, &distance)| GeodesicProfileEntry {
            chord: *chord,
            orbit: classify_orbit(chord).expect("BaseSpace chords are always classifiable"),
            distance,
            geodesic_count: geodesic_counts[chord],
        })
        .collect();
    per_chord.sort_by_key(|e| (e.distance, Reverse(e.geodesic_count), e.chord.pcs));

    Some(GeodesicDistribution {
        source: *source,
        source_orbit: classify_orbit(source).expect("BaseSpace chords are always classifiable"),
        eccentricity,
        reachable_chords,
        buckets,
        per_chord,
    })
}
