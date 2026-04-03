# Milestone 2.2: Geodesic Enumeration

## Detailed Plan

Enumerate all shortest paths between two chords, count geodesics efficiently, and find passing chords.

### Files to create

- `mt/src/quintal/geodesics.rs` — geodesic enumeration, counting, passing chords (~250 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod geodesics;` and re-exports

### Functions

```rust
pub fn geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<Vec<PcChord>>
pub fn count_geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> usize
pub fn passing_chords(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<PcChord>
```

**`geodesics(space, a, b)`** -- all shortest paths from `a` to `b`.

Algorithm (modified BFS tracking all parents):
1. BFS from `a`, recording `dist[v]` for each vertex `v`
2. During BFS, for each vertex `v` at distance `d`, record ALL neighbors `u` at distance `d-1` as parents: `parents[v].push(u)`
3. Reconstruct all paths from `b` back to `a` by recursively following parent links
4. Reverse each path to get `a -> ... -> b`

**`count_geodesics(space, a, b)`** -- count without materializing paths.

During BFS, track `sigma[v]` = number of shortest paths from `a` to `v`:
- Initialize `sigma[a] = 1`
- For each vertex `v` discovered at distance `d`: `sigma[v] = sum(sigma[u] for u in parents[v])`
- Return `sigma[b]`

**`passing_chords(space, a, b)`** -- all chords metrically between `a` and `b`.

A chord `z` is a passing chord if `d(a,z) + d(z,b) == d(a,b)`. Implementation:
1. BFS from `a` to get `dist_a[v]` for all `v`
2. BFS from `b` to get `dist_b[v]` for all `v`
3. Let `d_ab = dist_a[b]`
4. Collect all `z` where `dist_a[z] + dist_b[z] == d_ab` and `z != a` and `z != b`

### Test file

- `mt/tests/quintal/test_geodesics.rs` (~180 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_geodesics_same_chord` | `geodesics(space, a, a)` | single path `[a]` |
| `test_geodesics_adjacent` | geodesics between adjacent chords | single path of length 1 |
| `test_geodesics_distance_1_count` | from C-G-D-A, all distance-1 chords | 8 paths, each length 1 |
| `test_geodesics_path_lengths` | all paths have correct length `d(a,b)` | true |
| `test_geodesics_path_adjacency` | consecutive chords in each path are adjacent | true |
| `test_geodesics_antipodal_count` | `count_geodesics([0,2,7,9], [3,5,8,10])` | 298 |
| `test_count_matches_len` | `count_geodesics(a,b) == geodesics(a,b).len()` | true for sampled pairs |
| `test_passing_chords_distance_1` | passing chords for adjacent pair | empty (no intermediate) |
| `test_passing_chords_metric` | every passing chord satisfies `d(a,z)+d(z,b)==d(a,b)` | true |
| `test_passing_chords_exclude_endpoints` | `a` and `b` not in passing_chords | true |
| `test_geodesic_count_table` | max geodesic count at each distance from C-G-D-A | matches paper table |

### Existing modules to read for patterns

- `mt/src/quintal/distance.rs` — BFS infrastructure from Milestone 2.1
- `mt/src/quintal/base_space.rs` — BaseSpace, neighbors, chord_index

## Concept Cards

### Generalized Line Segments (A Geometry of Music)

**Quick Definition:** Voice leadings in chord space are "generalized line segments" that can bounce off mirror boundaries and disappear off twisted edges to reappear on the opposite side.

**Core Definition:** In chord space, a voice leading is represented by a generalized line segment -- a path that starts at one chord and ends at another, but that may interact with the space's boundaries along the way. The length of the path equals the size of the voice leading. Infinitely many generalized line segments can connect any two chords, each representing a different voice leading.

---

### Voice Leading in Pitch Space (A Geometry of Music)

**Quick Definition:** A voice leading in pitch space is a mapping from one collection of pitches to another, specifying exactly how each voice moves from its note in the first chord to its note in the second.

**Core Definition:** A voice leading in pitch space represents a mapping from one ordered collection of pitches to another, showing how individual musical voices move between chords. The "atomic constituents of musical scores." Geometrically: a collection of paths in linear pitch space.

---

### Parsimonious Voice Leading (Open Music Theory)

**Quick Definition:** Voice leading in which no single voice moves more than a step (whole step or half step), with half-step motion being the most parsimonious; the central principle underlying Neo-Riemannian theory.

**Core Definition:** Parsimonious voice leading principles: most parsimonious = half-step motion; still parsimonious = whole-step motion; not parsimonious = leaps. P and L transformations are maximally parsimonious (DVLS = 1). The Tonnetz and Cube Dance visualize parsimonious relationships -- adjacent nodes are parsimoniusly connected. Lower total semitone displacement = more parsimonious. Parsimonious music can be coherent WITHOUT functional harmony.

---

### Semitonal Voice Leadings Between Triads (Oxford Handbook of Neo-Riemannian Music Theories)

**Quick Definition:** The complete catalog of 16 voice leadings between consonant triads where no individual voice moves more than one semitone, representing all maximally efficient connections between triads.

**Core Definition:** Exactly 16 semitonal voice leadings exist between consonant triads (starting from C major or C minor). They group by inversional equivalence, retrograde equivalence, and individual transpositional equivalence. P and L appear as single-semitone voice leadings (DVLS = 1). Major-third relations involve two semitone moves (DVLS = 2). Perfect-fifth relations require more total motion and do not appear in the catalog.

---

## Mathematical Context

**Geodesic:** A shortest path in the graph. If d(A,B) = k, a geodesic is a sequence A = C0, C1, ..., Ck = B where each Ci and Ci+1 are adjacent and k is minimal.

**Betweenness (metric):** A chord Z is metrically between A and B if d(A,Z) + d(Z,B) = d(A,B). Z lies on at least one geodesic from A to B.

**Geodesic count:** From C-G-D-A to Ab-Eb-Bb-F (distance 7), there are 298 distinct geodesics. The count grows dramatically with distance (see paper section 6 table).

**Geodesic count table from C-G-D-A (paper section 6):**

| Distance | Chords at d | Avg geodesics | Max geodesics |
|----------|-------------|---------------|---------------|
| 1 | 8 | 1 | 1 |
| 2 | 18 | 1.9 | 2 |
| 3 | 36 | 3.5 | 6 |
| 4 | 45 | 7.8 | 24 |
| 5 | 66 | 17.8 | 40 |
| 6 | 44 | 67.9 | 176 |
| 7 | 10 | 226.8 | 298 |

**Passing chords (paper section 9):** The paper gives explicit examples of passing chords. From C-G-D-A to F-C-G-D (distance 4), one geodesic is: C-G-D-A -> C-F#-D-A -> D-A-F-C -> F-C-Ab-D -> F-C-G-D.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `mt/src/quintal/distance.rs` for BFS patterns (from Milestone 2.1)
4. Read `mt/src/quintal/base_space.rs` for BaseSpace API
5. Create the module files specified in the plan
6. Implement all functions
7. Write all specified tests
8. Run `cargo test` -- fix any failures
9. Run `cargo clippy` -- fix any warnings
10. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly by your implementation:

- Geodesics from C-G-D-A at distance 1: exactly **8 paths**, each length 1
- Geodesic count from C-G-D-A to Ab-Eb-Bb-F (distance 7): **298**
- Every geodesic has length equal to `d(a,b)`
- Every consecutive pair in a geodesic is adjacent
- Every passing chord satisfies the metric betweenness condition
- `count_geodesics` matches `geodesics().len()` for all tested pairs
- Max geodesic count at distance 7 from C-G-D-A: **298**
- Max geodesic count at distance 6 from C-G-D-A: **176**
