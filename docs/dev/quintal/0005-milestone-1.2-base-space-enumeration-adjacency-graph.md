# Milestone 1.2: Base Space Enumeration & Adjacency Graph

## Detailed Plan

Enumerate all 228 legal chords, build the adjacency graph, compute basic graph properties.

### Files to create

- `mt/src/quintal/base_space.rs` — `BaseSpace` struct, `enumerate_all()`, `is_adjacent()` (~200 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod base_space;` and re-exports

### Types and functions

**`enumerate_all() -> Vec<PcChord>`:**

- Iterate C(12,4) = 495 combinations via 4 nested loops: a in 0..9, b in (a+1)..10, c in (b+1)..11, d in (c+1)..12
- Filter by `PcChord::is_legal()`
- Must produce exactly 228

**`is_adjacent(a: &PcChord, b: &PcChord) -> bool`:**

- Both must be legal
- Compute symmetric difference: elements in a but not b, and vice versa
- Must have exactly 1 element unique to each side
- Those 2 elements must differ by exactly 1 mod 12: `min((x+12-y)%12, (y+12-x)%12) == 1`

**`BaseSpace` struct:**

```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BaseSpace {
    chords: Vec<PcChord>,
    adjacency: Vec<Vec<usize>>,  // index-based adjacency lists
}
```

Methods:

- `new() -> Self` — enumerate + build adjacency (O(228^2) ~ 26k checks)
- `len() -> usize`
- `is_empty() -> bool`
- `chord_index(&self, chord: &PcChord) -> Option<usize>` — lookup
- `degree(&self, chord: &PcChord) -> Option<usize>`
- `degree_distribution() -> BTreeMap<usize, usize>` — must match {4=>90, 5=>48, 6=>60, 8=>30}
- `is_connected() -> bool` — BFS from vertex 0, check all 228 reached
- `chords() -> &[PcChord]`
- `neighbors(&self, chord: &PcChord) -> Option<&[usize]>`

### Test file

- `mt/tests/quintal/test_base_space.rs` (~150 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_enumerate_all_count` | `enumerate_all().len()` | 228 |
| `test_enumerate_all_unique` | All chords in result are distinct | true |
| `test_enumerate_all_all_legal` | Every chord passes `is_legal()` | true |
| `test_known_chord_present` | [0,2,7,9] is in enumerated set | true |
| `test_adjacency_single_voice_down` | `is_adjacent([0,2,7,9], [0,2,7,8])` | true (A->Ab) |
| `test_adjacency_single_voice_up` | `is_adjacent([0,2,7,9], [0,2,7,10])` | true (A->Bb), if [0,2,7,10] is legal |
| `test_not_adjacent_two_voices` | Two voices differ | false |
| `test_not_adjacent_same` | Same chord | false |
| `test_adjacency_mod12_wrap` | Adjacency wrapping around 0/11 boundary | true where applicable |
| `test_adjacency_symmetric` | `is_adjacent(a,b) == is_adjacent(b,a)` | true for all sampled pairs |
| `test_base_space_size` | `BaseSpace::new().len()` | 228 |
| `test_degree_distribution` | `degree_distribution()` | {4=>90, 5=>48, 6=>60, 8=>30} |
| `test_total_degree_sum` | Sum of all degrees | even (= 2 * edge count) |
| `test_is_connected` | `BaseSpace::new().is_connected()` | true |
| `test_cgda_degree` | Degree of [0,2,7,9] | 8 (orbit [7,7,7]) |
| `test_crossroads_degree` | Degree of [0,2,6,8] | 8 (orbit [6,8,6]) |

### Existing modules to read for patterns

- `mt/src/quintal/types.rs` — PcChord, is_legal() (from Milestone 1.1)
- `mt/src/set_class/mod.rs` — PitchClassSet patterns

## Concept Cards

### Chord Space Formal Construction (A Geometry of Music)

**Quick Definition:** The formal mathematical construction of n-note chord space as a prism whose simplicial faces are glued with a twist and whose remaining boundaries act as mirrors, symbolized as the orbifold T^n/S_n.

**Core Definition:** A chord of n pitch classes is represented by a point in n-dimensional space, determined by two sets of inequalities. This fundamental domain is a prism whose cross-sections are simplices. To convert it into a proper quotient space: the sum-zero face is glued to the sum-twelve face with a cyclic twist (corresponding to scalar transposition by one step), and boundaries containing pitch duplications act as mirrors (voice crossings reflect back). The resulting orbifold, T^n/S_n, is the space of unordered sets of n pitch classes.

---

### Higher-Dimensional Chord Spaces (A Geometry of Music)

**Quick Definition:** The n-note chord space is an n-dimensional space formed by identifying equivalent points in ordered pitch space. Its boundary consists of chords with duplicate notes, its center contains perfectly even chords.

**Core Definition:** For n-note chords, the chord space is n-dimensional. Perfectly even chords lie at the center; the boundary contains chords with duplicate notes; the space wraps around with a (360/n)-degree twist. Four-note chord space is four-dimensional with diminished seventh chords at the center; it wraps with a 90-degree twist. The boundary acts as a mirror, and motion through the center connects chords to specific transpositions determined by 12/n.

---

### Boundary Behavior in Chord Space (A Geometry of Music)

**Quick Definition:** The two types of edge behavior in chord space: horizontal edges act as mirrors (voice leadings "bounce" off them), while vertical edges are glued with a twist (voice leadings disappear off one side and reappear on the other, shifted vertically).

**Core Definition:** Chord space has two distinct types of boundary behavior. The horizontal boundaries act as mirrors: a voice leading reaching this boundary appears to reflect. The vertical boundaries are identified with a twist: a voice leading exiting the right edge reappears at a vertically reflected position on the left edge. Contact with any of the four "edges" exchanges "upward" and "downward" directions.

---

### Distance in Music (A Geometry of Music)

**Quick Definition:** Musical distance is measured in semitones using subtraction in pitch space (|p - q|) or shortest path in pitch-class space, converting the multiplicative frequency ratios of acoustics into additive differences.

**Core Definition:** Musicians are primarily sensitive not to absolute frequencies but to the ratios between them. The logarithmic mapping from frequency to pitch space converts these ratios into distances measured by subtraction. In pitch-class space, distance is defined as the shortest distance between any two pitches belonging to those pitch classes (always between 0 and 6 semitones). Transposition and inversion are the only distance-preserving transformations.

---

### Voice Leading in Pitch Space (A Geometry of Music)

**Quick Definition:** A voice leading in pitch space is a mapping from one collection of pitches to another, specifying exactly how each voice moves from its note in the first chord to its note in the second.

**Core Definition:** A voice leading in pitch space represents a mapping from one ordered collection of pitches to another, showing how individual musical voices move between chords. The "atomic constituents of musical scores" -- basic building blocks of polyphony. Geometrically: a collection of paths in linear pitch space.

---

### Discrete Voice-Leading Lattices (A Geometry of Music)

**Quick Definition:** Criteria for ensuring that discrete graphs of voice-leading relationships faithfully represent voice-leading distances, addressing the problem that many common graphs (including the Tonnetz) have local structure that does not generalize to global distances.

**Core Definition:** Many discrete music-theoretical graphs have clear local structure but unreliable global structure. Five criteria ensure faithfulness: (1) every edge represents single-step voice leading; (2) the graph contains all interscalar transpositions between any two of its chords; (3) all chords have the same size; (4) paths representing interscalar transpositions involve no ascending-descending motion in the same voice; (5) no multisets. The continuous spaces subsume the discrete graphs.

---

## Mathematical Context

**Adjacency in the [6,8] space:** Two pc chords A and B are adjacent iff there exists a bijection sigma: A -> B such that exactly one pair (a, sigma(a)) differs by +/-1 mod 12 and all other pairs are equal. Both A and B must satisfy the [6,8] constraint. This definition captures "single-semitone voice move staying in [6,8]."

**Checking all rotations:** A pc set {a,b,c,d} can be arranged in multiple orderings. The interval structure depends on which ordering is chosen. For a pc set to satisfy [6,8], at least one ordering of the 4 elements must produce 3 forward intervals (mod 12) all in {6,7,8}. Implementation must search all 24 permutations.

**Graph structure:** The base space B is a graph with 228 vertices (legal chords) and edges connecting adjacent chords. The degree distribution {4:90, 5:48, 6:60, 8:30} means there are exactly 4 distinct degree values. The graph is fully connected (BFS from any vertex reaches all 228).

**Total edge count:** Sum of degrees = 2 * |E|. Sum = 90*4 + 48*5 + 60*6 + 30*8 = 360 + 240 + 360 + 240 = 1200. So |E| = 600 edges.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read existing similar modules for patterns (see plan for which files)
4. Create the module files specified in the plan
5. Implement all types and functions
6. Write all specified tests
7. Run `cargo test` -- fix any failures
8. Run `cargo clippy` -- fix any warnings
9. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly by your implementation:

- `enumerate_all().len()` == **228**
- All 228 chords are distinct
- All 228 chords pass `is_legal()`
- `degree_distribution()` == **{4 => 90, 5 => 48, 6 => 60, 8 => 30}**
- Sum of all degrees == **1200** (= 2 * 600 edges)
- Graph is **fully connected** (BFS from any vertex reaches all 228)
- Every adjacency is **symmetric**: `is_adjacent(a,b) == is_adjacent(b,a)`
- Degree of [0,2,7,9] (C-G-D-A, orbit Q777) == **8**
- Degree of [0,2,6,8] (crossroads, orbit Q686) == **8**
- `is_adjacent([0,2,7,9], [0,2,6,9])` == **true** (G->F#, one voice moves by 1)
