# Milestone 2.1: Shortest-Path Distance & Eccentricity

## Detailed Plan

BFS-based shortest-path computation, diameter, eccentricity, and center on the 228-chord base space B.

### Files to create

- `mt/src/quintal/distance.rs` — distance functions on `BaseSpace` (~200 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod distance;` and re-exports

### Functions

Implement as free functions taking `&BaseSpace`, or as methods on `BaseSpace` — whichever fits the existing Phase 1 pattern (BaseSpace has private fields, so methods or friend functions are needed).

```rust
pub fn distance(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Option<u8>
pub fn all_distances_from(space: &BaseSpace, a: &PcChord) -> HashMap<PcChord, u8>
pub fn eccentricity(space: &BaseSpace, a: &PcChord) -> Option<u8>
pub fn diameter(space: &BaseSpace) -> u8
pub fn center(space: &BaseSpace) -> Vec<PcChord>
```

**`distance(space, a, b)`:** Single BFS from `a`, return distance to `b`. O(V+E) where V=228, E=600.

**`all_distances_from(space, a)`:** Full BFS from `a`, return distance map for all 228 chords.

**`eccentricity(space, a)`:** max distance from `a` to any other chord.

**`diameter(space)`:** max over all eccentricities. BFS from every vertex — O(V*(V+E)), trivially fast for V=228.

**`center(space)`:** All chords with minimum eccentricity. Compute all eccentricities, find min, collect.

### Test file

- `mt/tests/quintal/test_distance.rs` (~180 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_distance_self` | `distance(space, a, a)` | Some(0) |
| `test_distance_adjacent` | `distance([0,2,7,9], [0,2,6,9])` | Some(1) |
| `test_distance_antipodal` | `distance([0,2,7,9], [3,5,8,10])` | Some(7) |
| `test_distance_symmetric` | `d(a,b) == d(b,a)` for sampled pairs | true |
| `test_distance_triangle_inequality` | `d(a,c) <= d(a,b) + d(b,c)` for sampled triples | true |
| `test_distance_missing_chord` | chord not in space | None |
| `test_all_distances_from_cgda` | count chords at each distance from [0,2,7,9] | matches paper table |
| `test_diameter` | `diameter(space)` | 8 |
| `test_eccentricity_range` | all eccentricities in {7, 8} | true |
| `test_center_size` | `center(space).len()` | 54 |
| `test_center_eccentricity` | all center chords have eccentricity 7 | true |
| `test_cgda_in_center` | [0,2,7,9] is in center | true (ecc=7) |

### Existing modules to read for patterns

- `mt/src/quintal/base_space.rs` — BaseSpace struct, adjacency, is_connected (existing BFS pattern)

## Concept Cards

### Distance in Music (A Geometry of Music)

**Quick Definition:** Musical distance is measured in semitones using subtraction in pitch space (|p - q|) or shortest path in pitch-class space, converting the multiplicative frequency ratios of acoustics into additive differences.

**Core Definition:** Musicians are primarily sensitive not to absolute frequencies but to the ratios between them. The logarithmic mapping from frequency to pitch space converts these ratios into distances measured by subtraction: the distance between pitches p and q is |p - q| semitones. In pitch-class space, distance is defined as the shortest distance between any two pitches belonging to those pitch classes (always between 0 and 6 semitones). Transposition and inversion are the only distance-preserving transformations of these spaces.

---

### Generalized Line Segments (A Geometry of Music)

**Quick Definition:** Voice leadings in chord space are "generalized line segments" that can bounce off mirror boundaries and disappear off twisted edges to reappear on the opposite side. There is a one-to-one correspondence between these paths and voice leadings in pitch-class space.

**Core Definition:** In chord space, a voice leading is represented by a generalized line segment -- a path that starts at one chord and ends at another, but that may interact with the space's boundaries along the way. The term "generalized" distinguishes these from ordinary Euclidean line segments. The length of the path equals the size of the voice leading. Infinitely many generalized line segments can connect any two chords, each representing a different voice leading.

---

### Efficient Voice Leading (A Geometry of Music)

**Quick Definition:** Efficient voice leading is voice leading in which all voices move by short distances, the polyphonic manifestation of the general preference for conjunct melodic motion.

**Core Definition:** Efficient voice leading describes voice leadings where individual voices move by small intervals, ideally a semitone or two. It is the primary goal of contrapuntal writing. Tymoczko shows that efficient voice leading between harmonically consistent chords is possible only when the chords are nearly symmetrical. The more evenly a chord divides the octave, the smaller the voice leadings to its transpositions.

---

### Voice Leading (Oxford Handbook of Neo-Riemannian Music Theories)

**Quick Definition:** The horizontal motion of individual melodic lines (voices) as they move from chord to chord, formalized as a mapping from pitches in one chord to pitches in another, with "efficient" or "parsimonious" voice leading minimizing the distances voices travel.

**Core Definition:** A voice leading is a mapping from pitches in one chord to pitches in another, specifying which note moves to which. DVLS (Displacement Voice-Leading Size) = sum of semitones moved by all voices. AVLS = DVLS / number of voices. Semitonal voice leading: no voice moves more than one semitone. Efficient voice leadings always come in inversionally related pairs. Tymoczko catalogs all 16 semitonal voice leadings between consonant triads.

---

## Mathematical Context

**Shortest-path distance in B:** The distance between two chords in the base space B is the minimum number of single-semitone voice moves (edges in the adjacency graph) needed to travel from one to the other, where every intermediate chord satisfies the [6,8] constraint. This is the standard BFS shortest-path distance on the graph.

**Eccentricity:** For a chord v, ecc(v) = max{d(v,w) : w in B}. The maximum distance from v to any other chord.

**Diameter:** diam(B) = max{ecc(v) : v in B} = max{d(v,w) : v,w in B}. The paper proves this is exactly 8.

**Center:** The set of chords with minimum eccentricity. The paper shows 54 chords have eccentricity 7 (the minimum), including all 12 pure quintal stacks [P5,P5,P5], all 12 [P5,A5,P5] chords, and all 6 [d5,A5,d5] crossroads chords.

**Distance table from C-G-D-A (paper section 6):**

| Distance | Chords at d |
|----------|-------------|
| 0 | 1 |
| 1 | 8 |
| 2 | 18 |
| 3 | 36 |
| 4 | 45 |
| 5 | 66 |
| 6 | 44 |
| 7 | 10 |

Total: 1+8+18+36+45+66+44+10 = 228.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `mt/src/quintal/base_space.rs` for the existing BFS pattern (is_connected uses BFS)
4. Create the module files specified in the plan
5. Implement all functions
6. Write all specified tests
7. Run `cargo test` -- fix any failures
8. Run `cargo clippy` -- fix any warnings
9. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly by your implementation:

- Diameter: **8**
- Eccentricity range: **{7, 8}** (only these two values)
- Center size: **54 chords** (all with eccentricity 7)
- `distance([0,2,7,9], [3,5,8,10])` = **7** (C-G-D-A to Ab-Eb-Bb-F, antipodal)
- `distance([0,2,7,9], [0,2,6,9])` = **1** (adjacent)
- `distance(a, a)` = **0** for any chord
- Distance is symmetric
- Triangle inequality holds
- Distance distribution from C-G-D-A: **{0:1, 1:8, 2:18, 3:36, 4:45, 5:66, 6:44, 7:10}** summing to 228
