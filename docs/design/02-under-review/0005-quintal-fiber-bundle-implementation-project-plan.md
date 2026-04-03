---
number: 5
title: "Quintal Fiber Bundle Implementation: Project Plan"
author: "exactly one"
component: All
tags: [change-me]
created: 2026-04-02
updated: 2026-04-02
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# Quintal Fiber Bundle Implementation: Project Plan

## Big-Picture Overview

This project implements the voice-leading geometry for quintal/quartal harmony as developed in *"Quintal Harmony as a Fiber Bundle"* across two Rust codebases:

- **`mt-rs`** — Core music theory library: new `quintal` module with types, metric space B, extended space E, and all associated algorithms
- **`ai-music-theory`** — MCP server: three new tools (`quintal_inversions`, `voiced_geodesic`, `passing_chords_voiced`) exposing the quintal geometry to AI consumers

The project is divided into **4 phases**, each containing **2–4 milestones** sized for a single Claude Code session (Opus 4.6, 1M context).

---

## Phase 1: Core Types & Enumeration (mt-rs)

**Goal:** Define all fundamental types and enumerate the 228-chord base space B with its adjacency graph.

### Milestone 1.1: Core Quintal Types

**Scope:** New module `mt/src/quintal/` with foundational types.

**Deliverables:**

- `types.rs` — Core type definitions:
  - `PcChord` — unordered 4-element pitch-class set (stored as sorted `[u8; 4]` with values 0–11). Implements `Eq`, `Hash`, `Ord`. Constructor validates uniqueness and range.
  - `VoicedChord` — ordered ascending 4-tuple of MIDI pitches (`[u8; 4]` with `p[0] < p[1] < p[2] < p[3]`). Constructor validates ascending order.
  - `IntervalStructure` — 3-tuple of semitones `(u8, u8, u8)` representing consecutive intervals `(i₁, i₂, i₃)`. Method `is_legal() -> bool` checks each ∈ {6, 7, 8}.
  - `Orbit` — enum of the 14 T/I orbit types, each carrying its representative interval structure, orbit size, degree, and analogy label (e.g., `MajorAnalogue` for [P5,P5,P5]).
  - `FiberClass` — enum `{ ClassA, ClassB }` indicating how many inversions re-enter [6,8] (1 vs 2).
- `mod.rs` — Module root re-exporting public API
- `conversions.rs` — `PcChord::interval_structure()`, `VoicedChord::interval_structure()`, `VoicedChord::to_pc_chord()` (the projection π: E → B)

**Concept cards for CC to read:**

- `pitch-and-pitch-class` (fundamentals)
- `pitch-class` (post-tonal-theory)
- `pitch-vs-pitch-class` (open-music-theory)
- `set-class` (geometry-of-music)
- `transposition-symmetry` (geometry-of-music)
- `inversion-symmetry` (geometry-of-music)
- `quartal-harmony` (20th-century-harmony)
- `four-note-quartal-chords` (20th-century-harmony)

**Mathematical context for CC (no concept cards available):**

> **ℤ₁₂ (integers mod 12):** Pitch classes are elements of the cyclic group ℤ₁₂ = {0,1,...,11} under addition mod 12. This models octave equivalence: C=0, C♯=1, ..., B=11.
>
> **Pitch-class set:** An unordered subset of ℤ₁₂. A 4-element pc set is a subset {a,b,c,d} ⊂ ℤ₁₂ with |{a,b,c,d}| = 4. Two pc sets are the same if they contain the same elements regardless of ordering.
>
> **Interval structure:** For a pc set sorted as a₀ < a₁ < a₂ < a₃ (within one octave), the interval structure is the 3-tuple ((a₁−a₀) mod 12, (a₂−a₁) mod 12, (a₃−a₂) mod 12). The [6,8] constraint requires each component ∈ {6,7,8}.
>
> **T/I group (ℤ₁₂ ⋊ ℤ₂):** The semidirect product of transposition (Tₙ: add n mod 12 to each pc) and inversion (I: replace each pc x with (12−x) mod 12, then optionally transpose). This group has order 24 and partitions the 228 legal chords into 14 equivalence classes (orbits).

**Tests:**

- Constructors reject invalid inputs (duplicate pcs, out-of-range values, non-ascending voices)
- `IntervalStructure::is_legal()` correctly classifies all 27 combinations
- `VoicedChord::to_pc_chord()` round-trips correctly
- `PcChord::interval_structure()` matches known examples from the paper (e.g., {C,G,D,A} → (7,7,7))

**Estimated session size:** ~400 lines of Rust + ~300 lines of tests

---

### Milestone 1.2: Base Space Enumeration & Adjacency Graph

**Scope:** Enumerate all 228 legal chords, build the adjacency graph, compute basic graph properties.

**Deliverables:**

- `enumeration.rs`:
  - `enumerate_all() -> Vec<PcChord>` — brute-force all C(12,4) = 495 four-note pc sets, filter by [6,8] constraint across all rotations of the sorted pc set. Must produce exactly 228 chords.
  - `is_adjacent(a: &PcChord, b: &PcChord) -> bool` — true iff a and b differ by exactly one semitone in exactly one voice (single-semitone voice move) AND both are legal.
- `graph.rs`:
  - `BaseSpace` struct holding:
    - `chords: Vec<PcChord>` (the 228 vertices)
    - `adjacency: HashMap<PcChord, Vec<PcChord>>` (adjacency lists)
  - `BaseSpace::new() -> Self` — builds the graph
  - `BaseSpace::degree(chord: &PcChord) -> u8` — vertex degree
  - `BaseSpace::degree_distribution() -> BTreeMap<u8, usize>` — must match {4→90, 5→48, 6→60, 8→30}

**Concept cards for CC to read:**

- `chord-space-formal-construction` (geometry-of-music)
- `higher-dimensional-chord-spaces` (geometry-of-music)
- `boundary-behavior` (geometry-of-music)
- `distance-in-music` (geometry-of-music)
- `voice-leading-in-pitch-space` (geometry-of-music)

**Mathematical context for CC:**

> **Adjacency in the [6,8] space:** Two pc chords A and B are adjacent iff there exists a bijection σ: A → B such that exactly one pair (a, σ(a)) differs by ±1 mod 12 and all other pairs are equal. Both A and B must satisfy the [6,8] constraint. This definition captures "single-semitone voice move staying in [6,8]."
>
> **Checking all rotations:** A pc set {a,b,c,d} can be arranged in multiple cyclic orderings. The interval structure depends on which note is considered "bottom." For a pc set to satisfy [6,8], at least one cyclic ordering must produce all intervals in {6,7,8}. Concretely: sort the pcs as a₀ < a₁ < a₂ < a₃, then check all 4 rotations: (a₀,a₁,a₂,a₃), (a₁,a₂,a₃,a₀+12), etc., computing consecutive differences for each. If any rotation has all differences in {6,7,8}, the chord is legal.

**Tests:**

- Exactly 228 chords enumerated
- Degree distribution matches paper: {4:90, 5:48, 6:60, 8:30}
- Known adjacencies verified (e.g., C-G-D-A is adjacent to C-F♯-D-A)
- Graph is fully connected (BFS from any vertex reaches all 228)
- Every adjacency is symmetric

**Estimated session size:** ~350 lines of Rust + ~250 lines of tests

---

### Milestone 1.3: Orbit Classification

**Scope:** Implement T/I group actions, classify all 228 chords into the 14 orbits.

**Deliverables:**

- `symmetry.rs`:
  - `transpose(chord: &PcChord, n: u8) -> PcChord` — Tₙ: add n mod 12 to each pc
  - `invert(chord: &PcChord) -> PcChord` — I: replace each pc x with (12−x) mod 12
  - `invert_transpose(chord: &PcChord, n: u8) -> PcChord` — TₙI composition
  - `orbit(chord: &PcChord) -> Vec<PcChord>` — compute full T/I orbit (up to 24 elements, deduplicated)
  - `classify_orbit(chord: &PcChord) -> Orbit` — determine which of the 14 orbit types
  - `classify_all(chords: &[PcChord]) -> BTreeMap<Orbit, Vec<PcChord>>` — partition all 228

**Mathematical context for CC:**

> **Transposition Tₙ:** Acts on a pc set S by adding n to each element mod 12. Tₙ(S) = {(s+n) mod 12 : s ∈ S}. The set of all transpositions forms the cyclic group ℤ₁₂.
>
> **Inversion I:** Acts on a pc set S by replacing each element with its mod-12 complement. I(S) = {(12−s) mod 12 : s ∈ S}. Note: this maps 0→0, 1→11, 2→10, etc.
>
> **T/I orbit:** The set of all chords reachable from a given chord by any combination of transpositions and inversions. For each chord C, its orbit is {TₙI^k(C) : n ∈ {0,...,11}, k ∈ {0,1}}. An orbit's size divides 24. A chord with no T/I symmetry has orbit size 24; one with a non-trivial stabilizer has a smaller orbit.
>
> **The 14 orbit representatives and their sizes:** (see paper §8 table). Orbits of size 6 have a stabilizer of order 4; orbits of size 12 have stabilizer of order 2; orbits of size 24 are generic (trivial stabilizer). Total: 6+6+12+12+12+12+24+24+24+24+24+24+12+12 = 228.

**Tests:**

- `transpose` is cyclic: T₁₂ = identity
- `invert` is involutory: I² = identity
- Exactly 14 distinct orbits produced
- Orbit sizes match paper table (two size-6, four size-12, eight size-24, summing to 228)
- Each orbit's degree is uniform (all members have same degree)
- Analogy labels assigned correctly for the four "named" orbits

**Estimated session size:** ~300 lines of Rust + ~250 lines of tests

---

## Phase 2: Metric Space Algorithms (mt-rs)

**Goal:** Implement shortest-path distance, geodesic enumeration, and betweenness centrality on B.

### Milestone 2.1: Shortest-Path Distance & Eccentricity

**Scope:** BFS-based shortest-path computation, diameter, eccentricity.

**Deliverables:**

- Extend `BaseSpace` in `graph.rs`:
  - `distance(a: &PcChord, b: &PcChord) -> u8` — BFS shortest path
  - `all_distances_from(a: &PcChord) -> HashMap<PcChord, u8>` — single-source BFS
  - `eccentricity(a: &PcChord) -> u8` — max distance from a to any other chord
  - `diameter() -> u8` — max over all eccentricities (must be 8)
  - `center() -> Vec<PcChord>` — chords with minimum eccentricity (must be 54 chords with eccentricity 7)

**Tests:**

- Diameter is exactly 8
- Eccentricity range is 7–8
- Center contains 54 chords
- Distance from C-G-D-A to A♭-E♭-B♭-F is 7 (antipodal)
- Distance from C-G-D-A to C-F♯-D-A is 1 (adjacent)
- Distance is symmetric: d(a,b) = d(b,a) for all sampled pairs
- Triangle inequality holds for sampled triples

**Estimated session size:** ~250 lines of Rust + ~200 lines of tests

---

### Milestone 2.2: Geodesic Enumeration

**Scope:** Enumerate all shortest paths between two chords.

**Deliverables:**

- `geodesics.rs`:
  - `geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<Vec<PcChord>>` — all shortest paths from a to b. Uses modified BFS that tracks all parents at each distance level, then reconstructs paths.
  - `count_geodesics(space: &BaseSpace, a: &PcChord, b: &PcChord) -> usize` — count without materializing paths
  - `passing_chords(space: &BaseSpace, a: &PcChord, b: &PcChord) -> Vec<PcChord>` — all chords that lie on any geodesic from a to b (i.e., Z where d(a,Z) + d(Z,b) = d(a,b))

**Mathematical context for CC:**

> **Geodesic:** A shortest path in the graph. If d(A,B) = k, a geodesic is a sequence A = C₀, C₁, ..., Cₖ = B where each Cᵢ and Cᵢ₊₁ are adjacent and k is minimal.
>
> **Betweenness (metric):** A chord Z is metrically between A and B if d(A,Z) + d(Z,B) = d(A,B). Z lies on at least one geodesic from A to B.
>
> **Geodesic count:** From C-G-D-A to A♭-E♭-B♭-F (distance 7), there are 298 distinct geodesics. The count grows dramatically with distance (see paper §6 table).

**Tests:**

- Geodesics from C-G-D-A at distance 1: exactly 8 paths of length 1
- Geodesic count to antipodal chord: 298
- Every path in the geodesic list has correct length d(a,b)
- Every chord in passing_chords satisfies d(a,Z) + d(Z,b) = d(a,b)
- Geodesics example from paper §9 verified step by step

**Estimated session size:** ~300 lines of Rust + ~200 lines of tests

---

### Milestone 2.3: Betweenness Centrality

**Scope:** Compute betweenness centrality for all chords; identify crossroads chords.

**Deliverables:**

- `centrality.rs`:
  - `betweenness_centrality(space: &BaseSpace) -> HashMap<PcChord, f64>` — normalized betweenness centrality using Brandes' algorithm (O(VE) rather than O(V³))
  - `crossroads_chords(space: &BaseSpace) -> Vec<PcChord>` — the 6 chords with highest betweenness (all [d5,A5,d5] orbit members)

**Mathematical context for CC:**

> **Betweenness centrality (Brandes' algorithm):** For each vertex s, perform BFS to compute shortest-path distances and counts (σ). Then, traversing vertices in reverse BFS order, accumulate dependency scores δ. The centrality of vertex v is the sum of δ values across all sources, normalized by (n-1)(n-2)/2 for an undirected graph.
>
> **Expected result:** The 6 [d5,A5,d5] chords each have ~13.9% betweenness centrality, making them the dominant crossroads of the space.

**Tests:**

- Top 6 chords by centrality are all [d5,A5,d5] orbit members
- Each crossroads chord has approximately 13.9% betweenness
- Centrality values sum correctly (normalization check)
- Centrality is invariant under transposition: T₁(C) has same centrality as C

**Estimated session size:** ~250 lines of Rust + ~150 lines of tests

---

## Phase 3: Extended Space E & Fiber Bundle (mt-rs)

**Goal:** Implement Tymoczko inversions, the fiber bundle structure, the Universal L1 Law, and quartal/quintal duality.

### Milestone 3.1: Chord-Scale & Tymoczko Inversion Operators

**Scope:** Implement the chord-scale derivation and the t₁/t₋₁ operators.

**Deliverables:**

- `fiber.rs`:
  - `chord_scale(chord: &VoicedChord) -> Vec<u8>` — extract pitch classes, sort in ascending order within one octave, return as the chord's intrinsic scale (step sizes between consecutive elements)
  - `chord_scale_steps(chord: &VoicedChord) -> Vec<u8>` — the step sizes [s₀, s₁, ..., s₃] where sᵢ = (pcᵢ₊₁ − pcᵢ) mod 12, with wrap-around
  - `t1(chord: &VoicedChord) -> VoicedChord` — one step up the inversion cycle: each voice moves up to the next chord-scale degree in its local register
  - `t_minus1(chord: &VoicedChord) -> VoicedChord` — one step down
  - `inversion_cycle(chord: &VoicedChord) -> [VoicedChord; 4]` — all 4 inversions (root, 1st, 2nd, 3rd)
  - `project(chord: &VoicedChord) -> PcChord` — π: E → B, the canonical projection

**Mathematical context for CC (critical — no concept cards exist for this):**

> **Tymoczko's interscalar transposition (t₁):** Given a voiced chord (p₁, p₂, p₃, p₄) in ascending order, with pitch-class set S = {pc(p₁), pc(p₂), pc(p₃), pc(p₄)}, the chord scale is S sorted within one octave. The step sequence is the circular sequence of intervals between consecutive chord-scale degrees.
>
> **Computing t₁:** For each voice pᵢ with pitch class pcᵢ, find the next chord-scale degree above pcᵢ in the circular ordering of S. The step size is the interval from pcᵢ to that next degree. Add this step to pᵢ to get the new pitch. Then re-sort the result into ascending order.
>
> **Concrete algorithm for t₁(p₁, p₂, p₃, p₄):**
>
> 1. Compute the chord scale: sort the 4 pitch classes into ascending order within [0,11]: cs = [c₀, c₁, c₂, c₃]
> 2. For each voice pᵢ, find its pitch class pcᵢ = pᵢ mod 12
> 3. Find the index j such that cs[j] = pcᵢ
> 4. The next scale degree is cs[(j+1) mod 4]
> 5. The step size is (cs[(j+1) mod 4] − cs[j] + 12) mod 12
> 6. New pitch = pᵢ + step_size
> 7. Sort the 4 new pitches into ascending order
>
> **Key property:** t₁⁴ = T₁₂ (four applications raise all pitches by one octave). The cycle always closes after exactly 4 steps.
>
> **t₋₁ is the reverse:** each voice moves DOWN to the previous chord-scale degree. t₋₁ = t₁³ ∘ T₋₁₂ (equivalently, t₋₁(chord) is the chord that t₁ maps TO chord, shifted down an octave).
>
> **Verified example — C3-G3-D4-A4:**
>
> - Chord scale {C,D,G,A}, steps [2,5,2,3]
> - Root: (48,55,62,69) intervals (7,7,7) — in [6,8]
> - t₁ → (50,57,67,72) → sort → (50,57,67,72) intervals (7,10,5) — NOT in [6,8]
> - t₁ → (55,60,69,74) intervals (5,9,5) — NOT in [6,8]
> - t₁ → (57,62,72,79) intervals (5,10,7) — NOT in [6,8]
> - t₁ → (60,67,74,81) = (48+12,55+12,62+12,69+12) = T₁₂(root) ✓

**Tests:**

- C3-G3-D4-A4 inversion cycle matches paper exactly (all 4 voicings, all intervals)
- t₁⁴ = T₁₂ for all tested chords
- t₋₁ reverses t₁: t₋₁(t₁(C)) has same pc set as C (shifted by one inversion position)
- `project` maps all 4 inversions of any chord to the same PcChord
- Chord scale of {C,D,G,A} is [2,5,2,3]
- Crossroads chord C-F♯-D-A♭ inversion cycle: root (6,8,6)✓, 1st (6,10,6)✗, 2nd (6,8,6)✓, 3rd (6,10,6)✗

**Estimated session size:** ~400 lines of Rust + ~350 lines of tests

---

### Milestone 3.2: L1 Distance, Fiber Classification & Universal L1 Law

**Scope:** Implement L1 distance on voiced chords, classify fibers, verify the Universal L1 Law.

**Deliverables:**

- Extend `fiber.rs`:
  - `l1_distance(a: &VoicedChord, b: &VoicedChord) -> u32` — sum of absolute pitch differences: Σ|aᵢ - bᵢ|
  - `fiber_class(chord: &PcChord, space: &BaseSpace) -> FiberClass` — determine whether 1 or 2 inversions re-enter [6,8]
  - `inversions_in_base(chord: &VoicedChord) -> Vec<usize>` — which inversion indices (0–3) have interval structure satisfying [6,8]
  - `inversion_l1_distances(chord: &VoicedChord) -> [u32; 4]` — L1 distances between consecutive inversions: [d(inv0→inv1), d(inv1→inv2), d(inv2→inv3), d(inv3→root')]

- `verification.rs`:
  - `verify_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>` — for every chord in B, compute its inversion cycle and verify the [12,12,12,36] pattern. Returns Err with counterexamples if any.
  - `verify_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>` — map each orbit to its fiber class

**Mathematical context for CC:**

> **L1 (Manhattan) distance on voiced chords:** For two voiced chords a = (a₁,a₂,a₃,a₄) and b = (b₁,b₂,b₃,b₄), the L1 distance is |a₁−b₁| + |a₂−b₂| + |a₃−b₃| + |a₄−b₄|. This is NOT the same as the graph distance in B (which counts single-semitone moves). L1 measures total voice displacement in pitch space.
>
> **Universal L1 Law:** For EVERY chord in B, the L1 distances between consecutive inversions in the Tymoczko cycle follow the pattern [12, 12, 12, 36]. The first three steps each cost 12 semitones; the "closing" step (from 3rd inversion back to root one octave higher) costs 36. Total cycle cost: 12+12+12+36 = 72 = 6×12. This holds universally across all 14 orbits.
>
> **Fiber classification:**
>
> - Class A (10 orbits): exactly 1 of 4 inversions satisfies [6,8] — the root position
> - Class B (3 orbits: [d5,P5,d5], [d5,A5,d5], [d5,A5,A5]): exactly 2 of 4 inversions satisfy [6,8]
> - Class B orbits have chord-scale step sequences with period-2 or palindromic symmetry

**Tests:**

- L1 distance is symmetric and satisfies triangle inequality
- Universal L1 Law verified for ALL 228 chords (no exceptions)
- L1 between root and 1st inversion of C-G-D-A is 12
- Fiber class of [P5,P5,P5] orbit is ClassA
- Fiber class of [d5,A5,d5] orbit is ClassB
- Crossroads chord has exactly 2 inversions in [6,8] (indices 0 and 2)
- Total cycle cost is 72 for every chord

**Estimated session size:** ~300 lines of Rust + ~300 lines of tests

---

### Milestone 3.3: Quartal/Quintal Duality

**Scope:** Formalize and verify the duality theorem.

**Deliverables:**

- `duality.rs`:
  - `quartal_reading(chord: &VoicedChord) -> IntervalStructure` — read intervals top-to-bottom (fourths perspective)
  - `quintal_reading(chord: &VoicedChord) -> IntervalStructure` — read intervals bottom-to-top (fifths perspective)
  - `t1_reversal_equivalence(chord: &VoicedChord) -> bool` — verify that t₋₁ traverses the same fiber as t₁ in reverse
  - `orbit_self_duality(orbit: &Orbit, space: &BaseSpace) -> bool` — verify that interval reversal maps orbit to itself
  - `verify_all_orbits_self_dual(space: &BaseSpace) -> bool` — all 14 orbits are self-dual

**Mathematical context for CC:**

> **Quartal/quintal duality as orientation reversal on the fiber:**
> The t₁ operator traverses the ℤ₄ fiber in one direction (quintal = "stacked fifths, reading up"). The t₋₁ operator traverses the same fiber in the reverse direction (quartal = "stacked fourths, reading down"). They visit the same 4 chords in reverse order.
>
> **Formally:** If the inversion cycle under t₁ is [C₀, C₁, C₂, C₃], then the cycle under t₋₁ is [C₀, C₃, C₂, C₁] — the same set, reversed (with C₀ as the shared starting point since it's a cycle).
>
> **Orbit self-duality:** The quartal dual of interval structure (i₁,i₂,i₃) is (i₃,i₂,i₁) — the same intervals read backwards. Under the T/I group, (i₁,i₂,i₃) and (i₃,i₂,i₁) always belong to the same orbit, because pitch-class inversion Iₙ reverses interval sequences. This is why we get 14 T/I orbits from 20 T-orbits: 6 asymmetric pairs collapse.
>
> **ℤ₂ symmetry:** The fiber ℤ₄ has a natural ℤ₂ action (orientation reversal: k ↦ −k mod 4). This ℤ₂ is the quartal/quintal duality.

**Tests:**

- For C-G-D-A: quintal reading (7,7,7), quartal reading from A-D-G-C gives (5,5,5) = complementary fourths
- t₋₁ cycle visits same 4 chords as t₁ cycle in reverse order
- All 14 orbits are self-dual
- Interval reversal of (7,7,7) → (7,7,7) (palindromic, same orbit)
- Interval reversal of (7,7,6) → (6,7,7) (different T-orbit, same T/I orbit)

**Estimated session size:** ~200 lines of Rust + ~200 lines of tests

---

## Phase 4: MCP Server Tools (ai-music-theory)

**Goal:** Expose the quintal geometry through three new MCP tools in the ai-music-theory server.

### Milestone 4.1: `quintal_inversions` MCP Tool

**Scope:** Given a voiced chord, return its complete inversion cycle with all metadata.

**Deliverables:**

- New tool registration in the MCP server's tool registry
- Tool implementation that:
  1. Parses input voiced chord (MIDI pitches or note names)
  2. Validates it projects to a legal [6,8] chord
  3. Computes full inversion cycle via `inversion_cycle()`
  4. For each inversion, returns: voiced chord, interval structure, outer span, L1 distance from root, whether it's in [6,8], fiber class
  5. Returns orbit classification and inversion interval signature

**Input format:** `{ "chord": [48, 55, 62, 69] }` or `{ "chord": "C3-G3-D4-A4" }`

**Output format:**

```json
{
  "pc_chord": [0, 2, 7, 9],
  "orbit": { "name": "P5_P5_P5", "analogy": "major", "degree": 8 },
  "fiber_class": "A",
  "chord_scale": { "pcs": [0, 2, 7, 9], "steps": [2, 5, 2, 3] },
  "inversions": [
    { "index": 0, "voices": [48,55,62,69], "intervals": [7,7,7], "span": 21, "l1_from_root": 0, "in_base": true },
    { "index": 1, "voices": [50,57,67,72], "intervals": [7,10,5], "span": 22, "l1_from_root": 12, "in_base": false },
    ...
  ],
  "l1_pattern": [12, 12, 12, 36],
  "total_cycle_cost": 72
}
```

**Tests:**

- Paper example C3-G3-D4-A4 returns exact values from §20
- Crossroads chord returns fiber_class "B" with 2 inversions in_base
- Invalid chord (intervals outside [6,8]) returns appropriate error
- Note-name parsing works for sharps and flats

**Estimated session size:** ~350 lines of Rust + ~200 lines of tests

---

### Milestone 4.2: `voiced_geodesic` MCP Tool

**Scope:** Given two voiced chords, find shortest paths in B between their root-position projections, then show how inversion position evolves along the path.

**Deliverables:**

- Tool implementation that:
  1. Projects both voiced chords to B via π
  2. Computes geodesics between the two pc chords in B
  3. For each step in each geodesic, shows the root-position voiced chord and notes which inversion of the source/target it corresponds to
  4. Returns L1 costs for each step

**Input format:** `{ "source": [48,55,62,69], "target": [56,63,70,77] }`

**Output format:**

```json
{
  "source_projection": [0,2,7,9],
  "target_projection": [8,10,3,5],
  "distance": 7,
  "geodesic_count": 298,
  "geodesics": [
    {
      "path": [
        { "pc_chord": [0,2,7,9], "intervals": [7,7,7], "orbit": "P5_P5_P5" },
        { "pc_chord": [0,2,6,9], "intervals": [6,8,7], "orbit": "..." },
        ...
      ]
    }
  ]
}
```

**Tests:**

- C-G-D-A to A♭-E♭-B♭-F: distance 7, 298 geodesics
- Adjacent chords: distance 1, 1 geodesic
- Same chord projected from different inversions: distance 0

**Estimated session size:** ~300 lines of Rust + ~200 lines of tests

---

### Milestone 4.3: `passing_chords_voiced` MCP Tool

**Scope:** Given source and target voiced chords, enumerate passing chords in both B and E.

**Deliverables:**

- Tool implementation that:
  1. Computes passing chords in B (chords on geodesics between projections)
  2. For each passing chord in B, computes its full inversion cycle to show E-space passing options
  3. Groups results by distance from source
  4. Annotates each with orbit, degree, fiber class

**Input format:** `{ "source": [48,55,62,69], "target": [53,60,67,74] }`

**Output format:**

```json
{
  "base_space_passing": {
    "distance": 4,
    "by_distance": {
      "1": [{ "pc_chord": [0,2,6,9], "intervals": [6,8,7], "degree": 6 }, ...],
      "2": [...],
      "3": [...]
    }
  },
  "extended_passing": [
    {
      "pc_chord": [0,2,6,9],
      "inversions": [
        { "index": 0, "voices": [...], "in_base": true },
        ...
      ]
    }
  ]
}
```

**Tests:**

- Paper example from §9: C-G-D-A to F-C-G-D, verify passing chords match
- Paper example: C-G-D-A to C-E♭-G-B♭ (distance 2), verify 2 passing chords
- Extended passing chords include inversions not in [6,8]

**Estimated session size:** ~300 lines of Rust + ~200 lines of tests

---

## Concept Card Reference Index

This section maps each milestone to the concept cards Claude Code should read from the MCP server before beginning work.

### Phase 1 Cards (Core Types & Enumeration)

| Milestone | Concept Cards (by ID) |
|---|---|
| 1.1 Types | `pitch-and-pitch-class`, `pitch-class` (post-tonal), `pitch-vs-pitch-class`, `set-class` (geometry-of-music), `transposition-symmetry`, `inversion-symmetry`, `quartal-harmony`, `four-note-quartal-chords`, `quartal-voice-leading` |
| 1.2 Enumeration | `chord-space-formal-construction`, `higher-dimensional-chord-spaces`, `boundary-behavior`, `distance-in-music`, `voice-leading-in-pitch-space`, `discrete-voice-leading-lattices` |
| 1.3 Orbits | `transposition`, `transpositional-set-class`, `set-class` (geometry-of-music), `set-class` (open-music-theory), `symmetry-and-set-class-size`, `list-of-set-classes` |

### Phase 2 Cards (Metric Space Algorithms)

| Milestone | Concept Cards (by ID) |
|---|---|
| 2.1 Distance | `distance-in-music`, `generalized-line-segments`, `efficient-voice-leading`, `voice-leading` (neo-riemannian) |
| 2.2 Geodesics | `generalized-line-segments`, `voice-leading-in-pitch-space`, `parsimonious-voice-leading`, `semitonal-voice-leadings` |
| 2.3 Centrality | `voice-leading-graph`, `musical-maps`, `toroidal-tonnetz` |

### Phase 3 Cards (Extended Space E)

| Milestone | Concept Cards (by ID) |
|---|---|
| 3.1 Inversion | `scalar-transposition`, `interscalar-transposition-twentieth-century`, `combining-scalar-chromatic-transposition`, `strongly-crossing-free-voice-leading`, `chord-progressions-vs-voice-leadings` |
| 3.2 L1 & Fibers | `chord-space-formal-construction`, `cross-sections-of-chord-space`, `horizontal-vertical-motion`, `decomposition-into-parallel-contrary` |
| 3.3 Duality | `quartal-harmony`, `quartal-voicings`, `quartal-pivotal-structures`, `compound-quartal-chords`, `tritone-resolution-in-quartal-harmony` |

### Phase 4 Cards (MCP Tools)

All Phase 1–3 cards are potentially relevant. Additionally:

- `voicing-as-set-class` (tonality-owners-manual)
- `triads-in-chromatic-space` (audacious-euphony)

### Mathematical Foundations Cards (cross-cutting)

These cards from the `mathematical-foundations`, `algebra-in-music`, `modular-arithmetic`, and `generalized-interval-systems` categories should be loaded for any milestone where CC needs to reason about group theory:

| Topic | Cards |
|---|---|
| Groups | `group` (both sources), `cyclic-group-and-generator`, `commutative-group`, `associativity`, `binary-composition` |
| Modular arithmetic | `modular-arithmetic`, `modular-arithmetic-and-intervals`, `modular-equivalence-on-the-integers` |
| Equivalence | `equivalence-class`, `equivalence-relation`, `equivalence-classes`, `congruence` |
| Homomorphisms | `homomorphism`, `anti-homomorphism`, `interval-group-isomorphisms` |
| GIS foundations | `chromatic-pitch-space`, `diatonic-pitch-space`, `commutative-vs-noncommutative-gis`, `inversion-transposition-combination` |

---

## Claude Code Prompt Templates

### Template A: Phase-Level Planning Prompt

Use this to generate a detailed implementation plan for a phase. Paste the relevant concept cards, phase number, and milestone descriptions.

```
You are working on the mt-rs Rust music theory library (https://github.com/music-comp/mt-rs).

## Task
Create a detailed implementation plan for Phase {N} of the Quintal Fiber Bundle project.
The phase contains the following milestones:

{PASTE MILESTONE DESCRIPTIONS FROM THIS PLAN}

## Context

### Paper Reference
{PASTE RELEVANT SECTIONS OF THE FIBER BUNDLE PAPER}

### Concept Cards
The following concept cards from the music-theory MCP server provide
formal definitions relevant to this phase. Read each one carefully
using the `get_concept` tool before writing the plan:

{LIST CONCEPT CARD IDS FROM THE REFERENCE INDEX ABOVE}

### Mathematical Context
{PASTE THE "Mathematical context for CC" BLOCKS FROM EACH MILESTONE}

### Existing Codebase
Read the following files to understand the existing mt-rs architecture:
- mt/src/lib.rs (module structure)
- mt/Cargo.toml (dependencies)
- CLAUDE.md (project conventions)
- Any existing files in mt/src/ that are structurally similar to what
  we're building (e.g., mt/src/chord/mod.rs, mt/src/interval/mod.rs,
  mt/src/set_class/mod.rs for type patterns; mt/src/neo_riemannian/mod.rs
  for transformation patterns)

### Rust Best Practices
Read the SKILL.md and its linked guides. Key constraints:
- All new code goes in mt/src/quintal/
- Tests go in mt/tests/quintal/
- Use proptest for property-based testing where applicable
- Follow existing module conventions (pub re-exports, error types)
- Target: `cargo test` passes, `cargo clippy` clean, `cargo fmt` clean

## Deliverable
Produce a detailed implementation plan for each milestone in this phase.
For each milestone, specify:
1. Exact file paths and module structure
2. All public types with their fields and derives
3. All public functions with signatures
4. All test cases with expected values
5. Dependencies on previous milestones
6. Estimated line counts

Do NOT write implementation code yet — produce only the plan.
```

### Template B: Milestone Implementation Prompt

Use this to implement a single milestone. Paste the detailed plan from Template A's output.

```
You are implementing Milestone {M.N} of the Quintal Fiber Bundle project
in the mt-rs Rust music theory library.

## Detailed Plan
{PASTE THE DETAILED PLAN FOR THIS MILESTONE FROM TEMPLATE A's OUTPUT}

## Concept Cards
Read the following concept cards before writing code:
{LIST CONCEPT CARD IDS}

Use the music-theory MCP server's `get_concept` tool to retrieve each card.

## Mathematical Context
{PASTE THE "Mathematical context for CC" BLOCK FOR THIS MILESTONE}

## Implementation Instructions
1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read existing similar modules for patterns (see plan for which files)
4. Create the module files specified in the plan
5. Implement all types and functions
6. Write all specified tests
7. Run `cargo test` — fix any failures
8. Run `cargo clippy` — fix any warnings
9. Run `cargo fmt` — ensure formatting

## Verification Values
The following values MUST be reproduced exactly by your implementation:
{PASTE KEY NUMERICAL RESULTS FROM THE PAPER, e.g.:
- 228 total legal chords
- Degree distribution: {4:90, 5:48, 6:60, 8:30}
- Diameter: 8
- Eccentricity range: 7–8
- 54 center chords
- 14 T/I orbits
- C-G-D-A inversion cycle: root (48,55,62,69), 1st (50,57,67,72),
  2nd (55,60,69,74), 3rd (57,62,72,79)
- Universal L1 pattern: [12,12,12,36]
- 6 crossroads chords, each ~13.9% betweenness
}
```

### Template C: MCP Tool Implementation Prompt

Use this for Phase 4 milestones (ai-music-theory codebase).

```
You are implementing the {TOOL_NAME} MCP tool in the ai-music-theory
server (https://github.com/music-comp/ai-music-theory).

## Tool Specification
{PASTE MILESTONE DESCRIPTION INCLUDING INPUT/OUTPUT FORMATS}

## Dependencies
This tool depends on the quintal module in mt-rs. The mt-rs crate
is a dependency of ai-music-theory. The following types and functions
from mt-rs are available:
{LIST ALL PUBLIC TYPES AND FUNCTIONS FROM PHASES 1-3}

## Existing MCP Server Architecture
Read the following files to understand how existing tools are registered:
- The fabryk-mcp crate structure
- Existing tool implementations (search for examples of tool registration)
- The MCP server's tool traits and handler patterns

Read CLAUDE.md and SKILL.md before starting.

## Concept Cards
Read these concept cards for domain context:
{LIST RELEVANT CONCEPT CARD IDS}

## Implementation Instructions
1. Register the new tool in the tool registry
2. Implement the tool handler
3. Parse and validate input
4. Call mt-rs quintal functions
5. Format output as specified
6. Write integration tests
7. Run full test suite
```

---

## Session Sizing Notes

Each milestone is designed to fit within a single Claude Code session:

- **Lines of new code:** 200–400 per milestone
- **Lines of tests:** 150–350 per milestone
- **Total per milestone:** 400–750 lines
- **Concept cards to read:** 5–10 per milestone
- **Paper sections to reference:** 2–5 per milestone

The largest milestone is 3.1 (Tymoczko inversions) at ~750 lines, which is well within a 1M-token context budget given that it needs to read ~10 concept cards and several paper sections.

---

## Dependency Graph

```
Phase 1:
  1.1 Types ──────────────┐
  1.2 Enumeration ────────┤
  1.3 Orbits ─────────────┘─── requires 1.1, 1.2

Phase 2:                     requires Phase 1
  2.1 Distance ───────────┐
  2.2 Geodesics ──────────┤── requires 2.1
  2.3 Centrality ─────────┘── requires 2.1

Phase 3:                     requires Phase 1
  3.1 Inversions ─────────┐
  3.2 L1 & Fibers ────────┤── requires 3.1, 1.3
  3.3 Duality ────────────┘── requires 3.1

Phase 4:                     requires Phases 1–3
  4.1 quintal_inversions ─┐── requires 3.1, 3.2
  4.2 voiced_geodesic ────┤── requires 2.2, 3.1
  4.3 passing_chords ─────┘── requires 2.2, 3.1
```

Within each phase, milestones can mostly be done sequentially. Phase 2 and Phase 3 are independent of each other and could theoretically be parallelized, but Phase 4 requires both.

---

## Total Project Estimate

| Phase | Milestones | Est. New Code | Est. Tests |
|-------|-----------|---------------|------------|
| 1 | 3 | ~1,050 lines | ~800 lines |
| 2 | 3 | ~800 lines | ~550 lines |
| 3 | 3 | ~900 lines | ~850 lines |
| 4 | 3 | ~950 lines | ~600 lines |
| **Total** | **12** | **~3,700 lines** | **~2,800 lines** |

Grand total: approximately 6,500 lines of Rust across 12 Claude Code sessions.
