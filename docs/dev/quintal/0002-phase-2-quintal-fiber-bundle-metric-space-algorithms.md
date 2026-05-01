# Phase 2: Quintal Fiber Bundle — Metric Space Algorithms

## Context

Implementing Phase 2 of the Quintal Fiber Bundle project from design spec `docs/design/02-under-review/0005-quintal-fiber-bundle-implementation-project-plan.md`, based on the paper `workbench/quintal-harmony-as-a-fiber-bundle.md`.

This phase implements shortest-path distance, geodesic enumeration, and betweenness centrality on the base space B. It builds on Phase 1's `BaseSpace` adjacency graph (228 chords, degree distribution {4:90, 5:48, 6:60, 8:30}).

**Depends on:** Phase 1 (complete). Uses `PcChord`, `BaseSpace`, `Orbit`, and group operations.

## File Structure

### New source files (`mt/src/quintal/`)

| File | Purpose | ~Lines |
|------|---------|--------|
| `distance.rs` | BFS distance, eccentricity, diameter, center | 200 |
| `geodesics.rs` | Geodesic enumeration, passing chords | 250 |
| `centrality.rs` | Brandes' betweenness centrality, saddle | 200 |

### New test files (`mt/tests/quintal/`)

| File | ~Lines |
|------|--------|
| `test_distance.rs` | 180 |
| `test_geodesics.rs` | 180 |
| `test_centrality.rs` | 140 |

### Modified files

- `mt/src/quintal/mod.rs` — add `mod distance; mod geodesics; mod centrality;` and re-exports
- `mt/tests/quintal/mod.rs` — add `mod test_distance; mod test_geodesics; mod test_centrality;`

## Milestone 2.1: Shortest-Path Distance & Eccentricity

### `distance.rs`

Extend `BaseSpace` with distance computation via BFS.

**Methods on `BaseSpace`:**

```rust
pub fn distance(&self, a: &PcChord, b: &PcChord) -> Option<u8>
pub fn all_distances_from(&self, a: &PcChord) -> HashMap<PcChord, u8>
pub fn eccentricity(&self, a: &PcChord) -> Option<u8>
pub fn diameter(&self) -> u8
pub fn center(&self) -> Vec<PcChord>
```

Alternatively, implement as free functions taking `&BaseSpace` — follow whichever pattern feels cleaner given Phase 1's `BaseSpace` design. If `BaseSpace` fields are private, methods are cleaner; if public, free functions work too.

**`distance(a, b)`:** Single BFS from `a`, return distance to `b`. O(V+E) where V=228, E~500-600.

**`all_distances_from(a)`:** Full BFS from `a`, return distance map to all 228 chords.

**`eccentricity(a)`:** `all_distances_from(a).values().max()`.

**`diameter()`:** Max over all eccentricities. Requires BFS from every vertex — O(V*(V+E)) ≈ 228 * ~800 = ~180k operations, trivially fast.

**`center()`:** All chords with minimum eccentricity. Compute all eccentricities, find min, collect matching chords.

### Verification values (Milestone 2.1)

- Diameter is exactly **8**
- Eccentricity range is **7–8**
- Center contains **54 chords** (all with eccentricity 7)
- `distance(C-G-D-A, Ab-Eb-Bb-F)` = `distance([0,2,7,9], [3,5,8,10])` = **7** (antipodal)
- `distance(C-G-D-A, C-F#-D-A)` = `distance([0,2,7,9], [0,2,6,9])` = **1** (adjacent)
- Distance is symmetric: `d(a,b) == d(b,a)` for all pairs
- Triangle inequality holds for sampled triples

## Milestone 2.2: Geodesic Enumeration

### `geodesics.rs`

**Public functions:**

```rust
pub fn geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<Vec<PcChord>>
pub fn count_geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> usize
pub fn passing_chords(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<PcChord>
```

**`geodesics(space, a, b)`:** All shortest paths from `a` to `b`.

Algorithm — modified BFS tracking all parents:
1. BFS from `a`, recording distance `dist[v]` for each vertex `v`
2. During BFS, for each vertex `v` at distance `d`, record ALL neighbors `u` at distance `d-1` as parents: `parents[v].push(u)`
3. Reconstruct all paths from `b` back to `a` by following parent links recursively
4. Reverse each path to get `a -> ... -> b`

**`count_geodesics(space, a, b)`:** Count without materializing paths. During BFS, track `sigma[v]` = number of shortest paths from `a` to `v`. Initialize `sigma[a] = 1`. For each vertex `v` discovered at distance `d`, sum sigma of all parents: `sigma[v] = sum(sigma[u] for u in parents[v])`. Return `sigma[b]`.

**`passing_chords(space, a, b)`:** All chords `z` where `d(a,z) + d(z,b) == d(a,b)`. Requires BFS from both `a` and `b`, then scan all chords.

### Verification values (Milestone 2.2)

- Geodesics from C-G-D-A at distance 1: exactly **8 paths** of length 1
- Geodesic count from C-G-D-A to Ab-Eb-Bb-F (distance 7): **298**
- Every path in geodesics has correct length `d(a,b)`
- Every chord in `passing_chords` satisfies `d(a,z) + d(z,b) == d(a,b)`
- Geodesic count table from C-G-D-A (from paper section 6):

| Distance | Chords at d | Max geodesics |
|----------|-------------|---------------|
| 1 | 8 | 1 |
| 2 | 18 | 2 |
| 3 | 36 | 6 |
| 4 | 45 | 24 |
| 5 | 66 | 40 |
| 6 | 44 | 176 |
| 7 | 10 | 298 |

## Milestone 2.3: Betweenness Centrality

### `centrality.rs`

**Public functions:**

```rust
pub fn betweenness_centrality(space: &BaseSpace) -> HashMap<PcChord, f64>
pub fn saddle_chords(space: &BaseSpace) -> Vec<PcChord>
```

**`betweenness_centrality(space)`:** Brandes' algorithm — O(VE) rather than O(V^3).

Algorithm (Brandes 2001):
1. Initialize `centrality[v] = 0.0` for all vertices
2. For each source vertex `s`:
   a. BFS from `s`, computing `dist[v]` and `sigma[v]` (number of shortest paths)
   b. Stack vertices in order of discovery (for reverse traversal)
   c. Initialize `delta[v] = 0.0` for all `v`
   d. Process vertices in reverse BFS order (furthest first):
      - For each predecessor `p` of `v`: `delta[p] += (sigma[p] / sigma[v]) * (1.0 + delta[v])`
   e. For each `v != s`: `centrality[v] += delta[v]`
3. Normalize: divide all values by `(n-1)*(n-2)/2` for undirected graph (n=228)

**`saddle_chords(space)`:** Top 6 chords by betweenness centrality. All should be [d5,A5,d5] orbit members.

### Verification values (Milestone 2.3)

- Top 6 chords by centrality are all **[d5,A5,d5] orbit** members (the `Q686` orbit)
- Each saddle chord has approximately **13.9%** betweenness centrality
- Centrality values sum correctly after normalization
- Centrality is T/I-invariant: all members of an orbit have the same centrality
- The 6 saddle chords: `{0,2,6,8}` and its 5 transpositions

## Key Verification Values (Phase 2 Summary)

- Diameter: **8**
- Eccentricity range: **7–8**
- Center size: **54 chords**
- C-G-D-A to antipodal: distance **7**, **298** geodesics
- Saddle: **6** chords, all Q686 orbit, each **~13.9%** betweenness
- Total chord count remains **228** (no new chords added)

## Build Order

1. **Milestone 2.1** first — distance computation is foundation for 2.2 and 2.3
2. **Milestone 2.2** second — geodesics use BFS distance infrastructure
3. **Milestone 2.3** third — centrality uses BFS + path counting from 2.1/2.2

## Verification

```bash
cargo test --test tests quintal::test_distance
cargo test --test tests quintal::test_geodesics
cargo test --test tests quintal::test_centrality
cargo clippy
cargo fmt --check
```

## Notes

- No new dependencies needed
- All algorithms are standard graph algorithms (BFS, Brandes) — nothing exotic
- The graph is small (228 vertices, ~500–600 edges), so brute-force approaches are fine
- `distance.rs` methods may be `impl BaseSpace` methods or free functions taking `&BaseSpace` — follow Phase 1's pattern
- Consider caching the all-pairs distance matrix in `BaseSpace` if multiple callers need it — but this is an optimization, not a requirement
