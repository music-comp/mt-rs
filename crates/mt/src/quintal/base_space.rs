use std::collections::{BTreeMap, VecDeque};

use super::PcChord;

/// Enumerate all C(12,4) = 495 four-note pitch-class sets and filter
/// by the \[6,8\] legality constraint. Produces exactly 228 chords.
///
/// Chords are returned in lexicographic order of their pitch-class arrays.
pub fn enumerate_all() -> Vec<PcChord> {
    let mut chords = Vec::new();
    for a in 0..9u8 {
        for b in (a + 1)..10 {
            for c in (b + 1)..11 {
                for d in (c + 1)..12 {
                    // new() sorts, but inputs are already ascending.
                    let chord = PcChord::new([a, b, c, d]).unwrap();
                    if chord.is_legal() {
                        chords.push(chord);
                    }
                }
            }
        }
    }
    chords
}

/// Two `PcChord`s are adjacent if they share exactly 3 pitch classes
/// and the two differing pitch classes differ by exactly 1 mod 12.
///
/// Both chords must be legal for adjacency to hold.
pub fn is_adjacent(a: &PcChord, b: &PcChord) -> bool {
    if !a.is_legal() || !b.is_legal() {
        return false;
    }

    let mut only_a = Vec::new();
    let mut only_b = Vec::new();

    for &pc in &a.pcs {
        if !b.pcs.contains(&pc) {
            only_a.push(pc);
        }
    }
    for &pc in &b.pcs {
        if !a.pcs.contains(&pc) {
            only_b.push(pc);
        }
    }

    if only_a.len() != 1 || only_b.len() != 1 {
        return false;
    }

    let x = only_a[0];
    let y = only_b[0];
    let diff = (x as i16 - y as i16).rem_euclid(12);
    diff == 1 || diff == 11
}

/// The base space B: the graph of all 228 legal quintal chords
/// with adjacency defined by single-semitone voice moves.
///
/// Each chord is connected to its neighbors by edges representing
/// the minimal voice-leading motion of a single pitch class moving
/// by one semitone.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BaseSpace {
    chords: Vec<PcChord>,
    adjacency: Vec<Vec<usize>>,
}

impl BaseSpace {
    /// Build the complete base space by enumerating all legal chords
    /// and computing the full adjacency graph.
    pub fn new() -> Self {
        let chords = enumerate_all();
        let n = chords.len();
        let mut adjacency = vec![Vec::new(); n];

        for i in 0..n {
            for j in (i + 1)..n {
                if is_adjacent(&chords[i], &chords[j]) {
                    adjacency[i].push(j);
                    adjacency[j].push(i);
                }
            }
        }

        BaseSpace { chords, adjacency }
    }

    /// Returns the number of chords in the base space.
    pub fn len(&self) -> usize {
        self.chords.len()
    }

    /// Returns `true` if the base space contains no chords.
    pub fn is_empty(&self) -> bool {
        self.chords.is_empty()
    }

    /// Returns a slice of all chords in the base space.
    pub fn chords(&self) -> &[PcChord] {
        &self.chords
    }

    /// Returns the index of the given chord in the base space, or `None`
    /// if the chord is not present.
    ///
    /// Uses binary search since chords are stored in sorted order.
    pub fn chord_index(&self, chord: &PcChord) -> Option<usize> {
        self.chords.binary_search(chord).ok()
    }

    /// Returns the indices of all neighbors of the given chord, or `None`
    /// if the chord is not in the base space.
    pub fn neighbors(&self, chord: &PcChord) -> Option<&[usize]> {
        self.chord_index(chord)
            .map(|i| self.adjacency[i].as_slice())
    }

    /// Returns the neighbor indices for a chord given by its index.
    ///
    /// This is a crate-internal method used by the distance module for
    /// efficient index-based BFS without repeated binary searches.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    pub(crate) fn neighbors_by_index(&self, idx: usize) -> &[usize] {
        &self.adjacency[idx]
    }

    /// Returns the degree (number of neighbors) of the given chord, or
    /// `None` if the chord is not in the base space.
    pub fn degree(&self, chord: &PcChord) -> Option<usize> {
        self.chord_index(chord).map(|i| self.adjacency[i].len())
    }

    /// Returns a map from degree to the count of chords having that degree.
    pub fn degree_distribution(&self) -> BTreeMap<usize, usize> {
        let mut dist = BTreeMap::new();
        for neighbors in &self.adjacency {
            *dist.entry(neighbors.len()).or_insert(0) += 1;
        }
        dist
    }

    /// Returns `true` if the base space graph is connected (every chord
    /// is reachable from every other chord via adjacency edges).
    pub fn is_connected(&self) -> bool {
        if self.chords.is_empty() {
            return true;
        }

        let n = self.chords.len();
        let mut visited = vec![false; n];
        let mut queue = VecDeque::new();

        visited[0] = true;
        queue.push_back(0);

        let mut count = 1usize;
        while let Some(current) = queue.pop_front() {
            for &neighbor in &self.adjacency[current] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    count += 1;
                    queue.push_back(neighbor);
                }
            }
        }

        count == n
    }
}

impl Default for BaseSpace {
    fn default() -> Self {
        Self::new()
    }
}
