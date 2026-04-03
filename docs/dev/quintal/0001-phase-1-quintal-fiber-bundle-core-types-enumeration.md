# Phase 1: Quintal Fiber Bundle — Core Types & Enumeration

## Context

Implementing Phase 1 of the Quintal Fiber Bundle project from design spec `docs/design/02-under-review/0005-quintal-fiber-bundle-implementation-project-plan.md`, based on the paper `workbench/quintal-harmony-as-a-fiber-bundle.md`.

This phase creates the `quintal` module with foundational types, enumerates the 228-chord base space B, builds its adjacency graph, and classifies all chords into 14 T/I orbits. This is the foundation for Phases 2-4 (metrics, fiber bundle, MCP tools).

## File Structure

### New source files (`mt/src/quintal/`)

| File | Purpose | ~Lines |
|------|---------|--------|
| `mod.rs` | Module root, re-exports | 30 |
| `error.rs` | `QuintalError` enum | 40 |
| `types.rs` | `PcChord`, `VoicedChord`, `IntervalStructure`, `FiberClass` | 220 |
| `base_space.rs` | `BaseSpace`, `enumerate_all()`, `is_adjacent()` | 200 |
| `group.rs` | `transpose`, `invert`, `invert_transpose`, `orbit()` | 100 |
| `orbit.rs` | `Orbit` enum (14 variants), classification | 220 |

### New test files (`mt/tests/quintal/`)

| File | ~Lines |
|------|--------|
| `mod.rs` | 10 |
| `test_types.rs` | 200 |
| `test_base_space.rs` | 150 |
| `test_orbit.rs` | 200 |

### Modified files

- `mt/src/lib.rs` — add `pub mod quintal;`
- `mt/tests/tests.rs` — add `mod quintal;`

## Milestone 1.1: Core Types

### `error.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuintalError {
    PitchClassOutOfRange(u8),
    DuplicatePitchClasses,
    WrongCardinality(usize),
    NotAscending,
}
```

Display + Error impls. Follow `ChordError` pattern from `mt/src/chord/mod.rs`.

### `types.rs`

**`IntervalStructure`** — newtype `(u8, u8, u8)`:

- Derives: `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord` + serde cfg_attr
- `new(a, b, c) -> Self`
- `is_legal() -> bool` — each component in {6, 7, 8}
- `intervals() -> [u8; 3]`

**`PcChord`** — `{ pcs: [u8; 4] }` (sorted ascending, 0-11, unique):

- Derives: `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord` + serde cfg_attr
- `new(pcs: [u8; 4]) -> Result<Self, QuintalError>` — validates range, uniqueness, sorts
- `from_unsorted(pcs: &[u8]) -> Result<Self, QuintalError>` — validates cardinality, range, uniqueness, sorts
- `interval_structure() -> Option<IntervalStructure>` — finds valid quintal ordering (see critical note below)
- `is_legal() -> bool` — any permutation produces 3 intervals all in {6,7,8}
- `pcs() -> [u8; 4]` — accessor

**Critical: interval structure != sorted gaps.** For {0,2,7,9}, sorted gaps are (2,5,2), but the quintal interval structure is (7,7,7) from the ordering C->G->D->A = 0->7->2->9. Must search all 24 permutations (or fix one element, try 6 orderings of remaining 3, for each of the 4 starting elements). Use a const array of 24 permutations of [0,1,2,3].

**`VoicedChord`** — `{ pitches: [u8; 4] }` (strictly ascending MIDI):

- Same derives as PcChord
- `new(pitches: [u8; 4]) -> Result<Self, QuintalError>` — validates p[0]<p[1]<p[2]<p[3]
- `interval_structure() -> IntervalStructure` — direct: (p[1]-p[0], p[2]-p[1], p[3]-p[2])
- `to_pc_chord() -> PcChord` — each pitch mod 12, sort, construct (projection pi: E->B)

**`FiberClass`** — `enum { ClassA, ClassB }`:

- Derives: `Debug, Clone, Copy, PartialEq, Eq, Hash` + serde cfg_attr
- Stub for now; classification logic added in Phase 3

### `mod.rs`

Re-exports all public types, functions, and error.

## Milestone 1.2: Enumeration & Adjacency

### `base_space.rs`

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
pub struct BaseSpace {
    chords: Vec<PcChord>,
    adjacency: Vec<Vec<usize>>,  // index-based adjacency lists
}
```

- `new() -> Self` — enumerate + build adjacency (O(228^2) ~ 26k checks)
- `len() -> usize`
- `chord_index(&self, chord: &PcChord) -> Option<usize>` — lookup
- `degree(&self, chord: &PcChord) -> Option<usize>`
- `degree_distribution() -> BTreeMap<usize, usize>` — must match {4=>90, 5=>48, 6=>60, 8=>30}
- `is_connected() -> bool` — BFS from vertex 0, check all 228 reached
- `chords() -> &[PcChord]`
- `neighbors(&self, chord: &PcChord) -> Option<&[usize]>`

## Milestone 1.3: Orbit Classification

### `group.rs`

```rust
pub fn transpose(chord: &PcChord, n: u8) -> PcChord
pub fn invert(chord: &PcChord) -> PcChord           // x -> (12-x) % 12
pub fn invert_transpose(chord: &PcChord, n: u8) -> PcChord  // T_n . I
pub fn orbit(chord: &PcChord) -> Vec<PcChord>        // full T/I orbit, deduplicated, sorted
```

- `transpose`: map each pc -> (pc+n)%12, sort, construct PcChord
- `invert`: map each pc -> (12-pc)%12, sort. Note (12-0)%12 = 0
- `orbit`: BTreeSet collecting T_n for n in 0..12 and T_n.I for n in 0..12, then to sorted Vec

### `orbit.rs`

**`Orbit` enum** — 14 variants named by interval structure:

| Variant | IS | Size | Degree | Analogy |
|---------|-----|------|--------|---------|
| `Q777` | (7,7,7) | 12 | 8 | major |
| `Q767` | (7,6,7) | 12 | 4 | — |
| `Q787` | (7,8,7) | 12 | 8 | minor |
| `Q676` | (6,7,6) | 6 | 4 | dim |
| `Q686` | (6,8,6) | 6 | 8 | aug |
| `Q878` | (8,7,8) | 12 | 4 | — |
| `Q868` | (8,6,8) | 12 | 4 | — |
| `Q776` | (7,7,6) | 24 | 5 | — |
| `Q877` | (8,7,7) | 24 | 6 | — |
| `Q867` | (8,6,7) | 24 | 4 | — |
| `Q876` | (8,7,6) | 24 | 5 | — |
| `Q788` | (7,8,8) | 24 | 4 | — |
| `Q786` | (7,8,6) | 24 | 6 | — |
| `Q688` | (6,8,8) | 12 | 6 | — |

Derives: `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord` + serde cfg_attr

Methods:

- `representative(&self) -> IntervalStructure`
- `size(&self) -> usize`
- `degree(&self) -> usize`
- `analogy(&self) -> Option<&'static str>`
- `all() -> &'static [Orbit; 14]`

**Classification approach:** Build lookup table eagerly.

```rust
pub fn classify_orbit(chord: &PcChord) -> Option<Orbit>
pub fn classify_all(chords: &[PcChord]) -> BTreeMap<Orbit, Vec<PcChord>>
```

Implementation of `classify_orbit`:

1. For each of 14 orbits, compute a representative PcChord from the interval structure (start at pc 0)
2. Compute the full T/I orbit of the representative
3. Check if the input chord is in that orbit
4. Return the matching orbit

Alternative (more efficient): precompute a `HashMap<PcChord, Orbit>` once for all 228 legal chords, then classify_orbit is a lookup. Build via: for each of 14 orbit reps, compute orbit, map all members -> Orbit variant.

## Key Verification Values

These values MUST match exactly:

- `enumerate_all().len() == 228`
- Degree distribution: `{4=>90, 5=>48, 6=>60, 8=>30}`
- Graph is connected (BFS reaches all 228)
- `PcChord::new([0,2,7,9]).interval_structure() == Some(IntervalStructure(7,7,7))`
- `PcChord::new([0,2,6,8]).interval_structure() == Some(IntervalStructure(6,8,6))`
- 14 distinct orbits, sizes summing to 228
- Orbit sizes: two 6s, four 12s, eight 24s: 2*6 + 4*12 + 8*24 = 12+48+192 = 252 ... wait: 6+6+12+12+12+12+24+24+24+24+24+24+12+12 = 228
- `transpose(chord, 12) == chord` (identity)
- `invert(invert(chord)) == chord` (involution)

## Build Order

1. **Milestone 1.1** first (types/error) — foundation for everything
2. **Milestone 1.2** second (enumeration/adjacency) — produces the 228 chords
3. **Milestone 1.3** third (group/orbit) — classifies using the enumerated chords

## Verification

```bash
cargo test --lib quintal      # unit tests
cargo test --test tests quintal  # integration tests
cargo clippy                   # lint clean
cargo fmt --check              # format clean
```

## Notes

- No new dependencies needed (no proptest for Phase 1; standard tests with known values suffice)
- Follow existing patterns: serde cfg_attr on all public types, Display + Error on error types
- Public fields (not getters) per existing convention
- All constructors that can fail return `Result<Self, QuintalError>`
