# Milestone 2.3: Betweenness Centrality

## Detailed Plan

Compute betweenness centrality for all chords using Brandes' algorithm; identify the 6 crossroads chords.

### Files to create

- `mt/src/quintal/centrality.rs` — Brandes' algorithm, crossroads identification (~200 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod centrality;` and re-exports

### Functions

```rust
pub fn betweenness_centrality(space: &BaseSpace) -> HashMap<PcChord, f64>
pub fn crossroads_chords(space: &BaseSpace) -> Vec<PcChord>
```

**`betweenness_centrality(space)`** -- Brandes' algorithm, O(VE).

Algorithm (Brandes 2001):
1. Initialize `centrality[v] = 0.0` for all vertices
2. For each source vertex `s`:
   a. BFS from `s`, computing `dist[v]` and `sigma[v]` (number of shortest paths from `s` to `v`)
   b. Push vertices onto a stack in order of discovery (for reverse traversal)
   c. Track predecessors: `pred[v]` = list of vertices `u` where `dist[u] + 1 == dist[v]` and `u` is a neighbor of `v`
   d. Initialize `delta[v] = 0.0` for all `v`
   e. Pop vertices from stack (reverse BFS order, furthest first):
      - For each predecessor `p` of `v`: `delta[p] += (sigma[p] as f64 / sigma[v] as f64) * (1.0 + delta[v])`
   f. For each `v != s`: `centrality[v] += delta[v]`
3. Normalize: divide all values by `(n-1)*(n-2)/2` for undirected graph where n=228
   - Normalization factor = (227 * 226) / 2 = 25,651

**`crossroads_chords(space)`** -- top 6 chords by betweenness centrality.

1. Compute betweenness_centrality
2. Sort by centrality descending
3. Return top 6
4. Assert all are Q686 orbit members (or just return them and let the test verify)

### Test file

- `mt/tests/quintal/test_centrality.rs` (~140 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_crossroads_count` | `crossroads_chords(space).len()` | 6 |
| `test_crossroads_are_q686` | all crossroads chords have IS (6,8,6) | true |
| `test_crossroads_centrality_approx` | each crossroads ~13.9% betweenness | within tolerance |
| `test_centrality_nonnegative` | all centrality values >= 0 | true |
| `test_centrality_normalized` | max centrality <= 1.0 | true |
| `test_orbit_invariant_centrality` | all members of Q777 have same centrality | true |
| `test_centrality_all_chords` | centrality computed for all 228 | true |
| `test_crossroads_includes_known` | [0,2,6,8] is a crossroads chord | true |

### Existing modules to read for patterns

- `mt/src/quintal/distance.rs` — BFS pattern from Milestone 2.1
- `mt/src/quintal/geodesics.rs` — sigma/parent tracking from Milestone 2.2
- `mt/src/quintal/base_space.rs` — BaseSpace API

## Concept Cards

### Voice-Leading Graph (Oxford Handbook of Neo-Riemannian Music Theories)

**Quick Definition:** Geometric representations of chords as points in multidimensional space where proximity reflects voice-leading efficiency, enabling the visualization and calculation of parsimonious connections between harmonies.

**Core Definition:** Voice-leading graphs are spatial models where nodes represent chords, edges connect chords with efficient voice-leading relationships, and distance corresponds to total voice-leading displacement. A chord class is highly "voice-leadable" when multiple other chords lie at minimal distance, these connections form regular patterns, and the chord can participate in parsimonious progressions. The graph of all triadic P, L, R connections forms a torus with 24 vertices, 36 edges, vertex degree 3 (chicken-wire torus).

---

### Musical Maps (Audacious Euphony)

**Quick Definition:** Geometric and graphical representations of musical space that reflect proximity judgments between musical objects, enabling visualization of syntactic relationships and compositional paths.

**Core Definition:** Musical maps are geometric models of pitch space. "The supreme advantage afforded by musical maps is their capacity to reflect judgments about the psychological proximity of musical objects or states." A good musical map "acts as a stage upon which imaginative performances are mounted, thus serving the same function as a geographical map for a child with a toy car." Different maps foreground different relationships -- no single map captures all information.

---

### Toroidal Tonnetz (Oxford Handbook of Neo-Riemannian Music Theories)

**Quick Definition:** The geometric shape of the Tonnetz under equal temperament, where enharmonic and syntonic equivalence cause the infinite plane to wrap into a torus containing exactly 12 pitch classes and 24 triadic regions.

**Core Definition:** The toroidal Tonnetz arises when equal temperament is assumed. The generating intervals become cyclic: 12 perfect fifths close the circle, and 3 major thirds equal one octave. This causes the infinite plane to wrap into a torus (product of two circles), a finite, bounded surface containing exactly 12 pitch-class nodes, 24 triadic regions, 36 edges. Every triad has exactly 3 PLR neighbors. LP cycles trace "longitudinal" loops (hexatonic tubes). PR cycles trace "latitudinal" loops (octatonic bands).

---

## Mathematical Context

**Betweenness centrality (Brandes' algorithm):** For each vertex s, perform BFS to compute shortest-path distances and counts (sigma). Then, traversing vertices in reverse BFS order, accumulate dependency scores delta. The centrality of vertex v is the sum of delta values across all sources, normalized by (n-1)(n-2)/2 for an undirected graph.

**Normalization:** For an undirected graph with n vertices, the normalization factor is (n-1)(n-2)/2. For n=228, this is 227*226/2 = 25,651. Dividing raw centrality by this gives values in [0, 1].

**Expected result:** The 6 [d5,A5,d5] chords (Q686 orbit) each have ~13.9% betweenness centrality, making them the dominant crossroads of the space. These are: {0,2,6,8}, {1,3,7,9}, {2,4,8,10}, {3,5,9,11}, {4,6,10,0}, {5,7,11,1}.

**Centrality as orbit invariant:** Betweenness centrality is invariant under graph isometries. Since transposition and inversion are isometries of B, all members of a T/I orbit have the same centrality. This means centrality is constant across each of the 14 orbits.

**Crossroads significance (paper section 7):** The crossroads chords are the most structurally important in the entire space -- every shortest path tends to pass through them. They sit at the intersection of major voice-leading highways in the quintal space.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `mt/src/quintal/distance.rs` and `mt/src/quintal/geodesics.rs` for BFS patterns
4. Read `mt/src/quintal/base_space.rs` for BaseSpace API
5. Create the module files specified in the plan
6. Implement all functions
7. Write all specified tests
8. Run `cargo test` -- fix any failures
9. Run `cargo clippy` -- fix any warnings
10. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly by your implementation:

- **6** crossroads chords, all Q686 orbit members
- Each crossroads chord has approximately **13.9%** (0.139) betweenness centrality
- All centrality values are **non-negative**
- All normalized centrality values are **<= 1.0**
- Centrality is **orbit-invariant**: all members of Q777 have the same centrality, all members of Q686 have the same centrality, etc.
- The 6 crossroads chords are: **{0,2,6,8}** and its 5 transpositions by 2 semitones each (the Q686 orbit has size 6)
- Centrality is computed for all **228** chords
- Crossroads chords have **maximum** betweenness among all 228 chords
