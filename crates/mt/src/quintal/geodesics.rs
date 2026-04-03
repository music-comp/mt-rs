//! Geodesic (shortest-path) enumeration and counting on the base space graph.
//!
//! Provides functions to enumerate all shortest paths between two chords,
//! count them without materializing, and find all metrically intermediate
//! ("passing") chords.

use std::collections::VecDeque;

use super::base_space::BaseSpace;
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
