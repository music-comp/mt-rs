//! Betweenness centrality for the quintal base space graph.
//!
//! Implements Brandes' algorithm for betweenness centrality on the
//! 228-vertex base space, identifying the "saddle" chords that lie on
//! the greatest fraction of shortest paths.

use std::collections::{HashMap, VecDeque};

use super::base_space::BaseSpace;
use super::PcChord;

/// Compute the normalized betweenness centrality for every chord in the
/// base space using Brandes' algorithm.
///
/// Betweenness centrality measures how often a chord lies on shortest
/// paths between other chords. The result is normalized for an undirected
/// graph by dividing by `(n-1)(n-2)/2`, so values fall in `[0, 1]`.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{betweenness_centrality, BaseSpace};
///
/// let space = BaseSpace::new();
/// let bc = betweenness_centrality(&space);
/// assert_eq!(bc.len(), 228);
/// ```
pub fn betweenness_centrality(space: &BaseSpace) -> HashMap<PcChord, f64> {
    let n = space.len();
    let mut cb = vec![0.0f64; n];

    for s in 0..n {
        // BFS from source s
        let mut stack = Vec::new();
        let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut sigma = vec![0usize; n];
        let mut dist: Vec<i32> = vec![-1; n];
        let mut queue = VecDeque::new();

        sigma[s] = 1;
        dist[s] = 0;
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in space.neighbors_by_index(v) {
                // First discovery of w
                if dist[w] < 0 {
                    dist[w] = dist[v] + 1;
                    queue.push_back(w);
                }
                // Shortest path to w via v
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            }
        }

        // Back-propagate dependencies
        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                delta[v] += (sigma[v] as f64 / sigma[w] as f64) * (1.0 + delta[w]);
            }
            if w != s {
                cb[w] += delta[w];
            }
        }
    }

    // Normalize for undirected graph: divide by (n-1)(n-2)/2
    let norm = ((n - 1) * (n - 2)) as f64 / 2.0;

    let mut result = HashMap::new();
    for (i, &val) in cb.iter().enumerate() {
        result.insert(space.chords()[i], val / norm);
    }
    result
}

/// Return the top 6 chords by betweenness centrality (the "saddle" chords).
///
/// In the quintal base space these are the chords that act as critical
/// junctions, lying on the greatest proportion of shortest paths. They
/// are expected to belong to the Q686 ("augmented analogue") orbit with
/// a centrality of approximately 0.139.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{saddle_chords, BaseSpace};
///
/// let space = BaseSpace::new();
/// let saddle = saddle_chords(&space);
/// assert_eq!(saddle.len(), 6);
/// ```
pub fn saddle_chords(space: &BaseSpace) -> Vec<PcChord> {
    let bc = betweenness_centrality(space);
    let mut sorted: Vec<(PcChord, f64)> = bc.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted.into_iter().take(6).map(|(chord, _)| chord).collect()
}

/// Deprecated alias for [`saddle_chords`].
///
/// The project terminology has migrated from "Crossroads" to "Saddle"; this
/// thin wrapper preserves backward compatibility for downstream callers.
/// New code should call [`saddle_chords`] directly.
#[deprecated(since = "0.5.2", note = "renamed to `saddle_chords`")]
pub fn crossroads_chords(space: &BaseSpace) -> Vec<PcChord> {
    saddle_chords(space)
}
