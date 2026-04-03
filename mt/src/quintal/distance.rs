//! Shortest-path distance and eccentricity computations on the base space graph.
//!
//! All functions operate on the [`BaseSpace`] graph using BFS (breadth-first
//! search), which is optimal for unweighted graphs. The base space has 228
//! vertices and approximately 828 edges, so even full all-pairs BFS completes
//! in microseconds.

use std::collections::{HashMap, VecDeque};

use super::base_space::BaseSpace;
use super::PcChord;

/// Run BFS from `source_idx` and return the shortest-path distance to every
/// vertex. Unreachable vertices receive `None`.
///
/// Uses `u8` for distances since the base space diameter is at most 8.
fn bfs_distances(space: &BaseSpace, source_idx: usize) -> Vec<Option<u8>> {
    let n = space.len();
    let mut dist: Vec<Option<u8>> = vec![None; n];
    let mut queue = VecDeque::new();

    dist[source_idx] = Some(0);
    queue.push_back(source_idx);

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

/// Shortest-path distance between two chords in the base space.
///
/// Returns `None` if either chord is not present in the space or if
/// the two chords are not connected (which cannot happen in the quintal
/// base space since it is connected).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{distance, BaseSpace, PcChord};
///
/// let space = BaseSpace::new();
/// let a = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(distance(&space, &a, &a), Some(0));
/// ```
pub fn distance(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Option<u8> {
    let ai = space.chord_index(a)?;
    let bi = space.chord_index(b)?;
    let dists = bfs_distances(space, ai);
    dists[bi]
}

/// Shortest-path distances from one chord to all others in the base space.
///
/// Returns a map from each reachable chord to its distance from `a`.
/// Returns an empty map if `a` is not in the space.
pub fn all_distances_from(space: &BaseSpace, a: &PcChord) -> HashMap<PcChord, u8> {
    let mut result = HashMap::new();
    if let Some(ai) = space.chord_index(a) {
        let dists = bfs_distances(space, ai);
        for (i, d) in dists.iter().enumerate() {
            if let Some(dist) = d {
                result.insert(space.chords()[i], *dist);
            }
        }
    }
    result
}

/// Eccentricity of a chord: the maximum shortest-path distance from that
/// chord to any other chord in the base space.
///
/// Returns `None` if the chord is not in the space.
pub fn eccentricity(space: &BaseSpace, a: &PcChord) -> Option<u8> {
    let ai = space.chord_index(a)?;
    let dists = bfs_distances(space, ai);
    dists.iter().filter_map(|d| *d).max()
}

/// Diameter of the base space: the maximum eccentricity across all chords,
/// equivalently the longest shortest path between any pair of chords.
pub fn diameter(space: &BaseSpace) -> u8 {
    let mut max_ecc = 0u8;
    for i in 0..space.len() {
        let dists = bfs_distances(space, i);
        let ecc = dists.iter().filter_map(|d| *d).max().unwrap_or(0);
        if ecc > max_ecc {
            max_ecc = ecc;
        }
    }
    max_ecc
}

/// Center of the base space: all chords whose eccentricity equals the
/// minimum eccentricity (the radius).
///
/// In graph theory the center is the set of vertices that minimize the
/// maximum distance to any other vertex.
pub fn center(space: &BaseSpace) -> Vec<PcChord> {
    let eccs: Vec<u8> = (0..space.len())
        .map(|i| {
            let dists = bfs_distances(space, i);
            dists.iter().filter_map(|d| *d).max().unwrap_or(0)
        })
        .collect();

    let min_ecc = eccs.iter().copied().min().unwrap_or(0);
    eccs.iter()
        .enumerate()
        .filter(|(_, &e)| e == min_ecc)
        .map(|(i, _)| space.chords()[i])
        .collect()
}
