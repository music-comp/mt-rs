# Milestone 3.2: L1 Distance, Fiber Classification & Universal L1 Law

## Detailed Plan

Classify fibers by how many inversions re-enter [6,8], verify the Universal L1 Law across all 228 chords.

### Files to create

- `mt/src/quintal/verification.rs` — L1 pattern verification, fiber class computation (~200 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod verification;` and re-exports

### Functions

```rust
pub fn inversions_in_base(chord: &VoicedChord) -> Vec<usize>
pub fn inversion_l1_distances(chord: &VoicedChord) -> [u32; 4]
pub fn fiber_class(chord: &PcChord) -> FiberClass
pub fn verify_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>
pub fn verify_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>
```

**`inversions_in_base(chord)`:**

- Compute inversion cycle via `inversion_cycle(chord)`
- For each inversion (index 0-3), check if its `interval_structure().is_legal()`
- Return the indices where it's legal

**`inversion_l1_distances(chord)`:**

- Compute inversion cycle: `[inv0, inv1, inv2, inv3]`
- Compute `t1(inv3)` to get root' (= root + T12)
- Return `[l1(inv0,inv1), l1(inv1,inv2), l1(inv2,inv3), l1(inv3,root')]`

**`fiber_class(chord)`:**

- Need a representative VoicedChord for the PcChord
- Construct by placing at a default register (e.g., starting at MIDI 48)
- Count how many inversions are in [6,8]
- 1 -> ClassA, 2 -> ClassB

**`verify_universal_l1_law(space)`:**

- For every chord in B (all 228):
  - Construct a VoicedChord in default register
  - Compute `inversion_l1_distances`
  - Verify pattern is `[12, 12, 12, 36]`
- Return `Ok(())` if all pass, `Err(counterexamples)` otherwise

**`verify_fiber_classes(space)`:**

- For each orbit, pick one representative chord
- Compute its fiber class
- Return mapping Orbit -> FiberClass

### Helper: PcChord -> VoicedChord

Need a function to construct a VoicedChord from a PcChord in a default register. Algorithm:

1. Take the sorted pcs [a, b, c, d]
2. Find the permutation that gives the legal interval structure (same search as `interval_structure()`)
3. Place the first pc at MIDI 48, then add intervals cumulatively
4. The result is a voiced chord in ascending order with all intervals in [6,8]

### Test file

- `mt/tests/quintal/test_verification.rs` (~250 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_inversions_in_base_cgda` | CGDA root position | [0] (only root in [6,8]) |
| `test_inversions_in_base_saddle` | saddle chord | [0, 2] (root and 2nd) |
| `test_l1_distances_cgda` | L1 pattern for CGDA | [12, 12, 12, 36] |
| `test_l1_distances_all_sum_72` | total cycle cost | 72 for all tested |
| `test_universal_l1_law` | verify for all 228 | Ok(()) |
| `test_fiber_class_q777` | Q777 orbit | ClassA |
| `test_fiber_class_q686` | Q686 orbit (saddle) | ClassB |
| `test_fiber_class_q676` | Q676 orbit | ClassB |
| `test_fiber_class_q688` | Q688 orbit | ClassB |
| `test_verify_fiber_classes_count` | 3 ClassB orbits, 11 ClassA | true |
| `test_l1_symmetric` | l1(a,b) == l1(b,a) | true |
| `test_l1_triangle_inequality` | sampled triples | true |
| `test_saddle_two_inversions` | exactly 2 inversions in [6,8] | true |

### Existing modules to read for patterns

- `mt/src/quintal/fiber.rs` — inversion_cycle, t1, l1_distance, chord_scale (from Milestone 3.1)
- `mt/src/quintal/types.rs` — PcChord, VoicedChord, FiberClass, IntervalStructure
- `mt/src/quintal/orbit.rs` — Orbit enum, classify_orbit
- `mt/src/quintal/base_space.rs` — BaseSpace, enumerate_all

## Concept Cards

### Chord Space Formal Construction (A Geometry of Music)

**Quick Definition:** The formal mathematical construction of n-note chord space as a prism whose simplicial faces are glued with a twist and whose remaining boundaries act as mirrors.

**Core Definition:** A chord of n pitch classes is represented by a point in n-dimensional space. The fundamental domain is a prism whose cross-sections are simplices. The sum-zero face is glued to the sum-twelve face with a cyclic twist (transposition), and boundaries containing pitch duplications act as mirrors. The resulting orbifold T^n/S_n is the space of unordered sets of n pitch classes.

---

### Cross Sections of Chord Space (A Geometry of Music)

**Quick Definition:** Vertical (2D) or horizontal (3D) slices through chord space containing all chords whose pitch classes sum to the same value.

**Core Definition:** Every chord type appears in every cross section. Cross sections are musically significant because restricting attention to a cross section is equivalent to studying only the purely contrary component of voice leading. Line segments within a cross section represent the contrary components of individually T-related voice leadings.

---

### Horizontal and Vertical Motion (A Geometry of Music)

**Quick Definition:** In rotated chord space, horizontal motion = parallel (both voices same direction/amount), vertical = perfect contrary (opposite directions, equal amounts).

**Core Definition:** Any voice leading can be decomposed into horizontal and vertical components. Horizontal component = (d1+d2)/2 semitones of transposition. Vertical component = (d1-d2)/2 semitones of contrary motion. This is the musical analogue of vector analysis.

---

### Decomposition into Parallel and Contrary Motion (A Geometry of Music)

**Quick Definition:** Any voice leading can be mathematically decomposed into a pure parallel component and a pure contrary component.

**Core Definition:** The parallel component transposes both voices by (d1+d2)/2 semitones. The contrary component moves them by (d1-d2)/2 semitones in opposite directions. The decomposition separates relative motion (contrary = counterpoint) from shared transpositional motion (parallel = pitch level shift). Restricting attention to a cross section studies only the contrary component.

---

## Mathematical Context

**L1 (Manhattan) distance on voiced chords:** For two voiced chords a = (a1,a2,a3,a4) and b = (b1,b2,b3,b4), the L1 distance is |a1-b1| + |a2-b2| + |a3-b3| + |a4-b4|. This is NOT the graph distance in B. L1 measures total voice displacement in pitch space.

**Universal L1 Law:** For EVERY chord in B, the L1 distances between consecutive inversions in the Tymoczko cycle follow the pattern [12, 12, 12, 36]. The first three steps each cost 12 semitones; the "closing" step (from 3rd inversion back to root one octave higher) costs 36. Total cycle cost: 12+12+12+36 = 72 = 6*12. This holds universally across all 14 orbits.

**Proof sketch:** Each application of t1 moves every voice up by exactly one chord-scale step. The chord-scale steps sum to 12 (one octave) by definition. Therefore each t1 application moves total pitch content by exactly 12 semitones, so L1 = 12.

**Fiber classification:**

- **Class A (11 orbits):** exactly 1 of 4 inversions satisfies [6,8] — the root position
- **Class B (3 orbits: Q676, Q686, Q688):** exactly 2 of 4 inversions satisfy [6,8]
- Class B orbits have chord-scale step sequences with period-2 or palindromic symmetry

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `mt/src/quintal/fiber.rs` for t1, inversion_cycle, l1_distance (from Milestone 3.1)
4. Read `mt/src/quintal/types.rs` for FiberClass enum
5. Create the module files specified in the plan
6. Implement all functions
7. Write all specified tests
8. Run `cargo test` -- fix any failures
9. Run `cargo clippy` -- fix any warnings
10. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly:

- **Universal L1 Law: [12, 12, 12, 36] for ALL 228 chords** (no exceptions)
- Total cycle cost = **72** for every chord
- L1 between root and 1st inversion of C-G-D-A: **12**
- Fiber class of Q777: **ClassA**
- Fiber class of Q686 (saddle): **ClassB**
- Fiber class of Q676: **ClassB**
- Fiber class of Q688: **ClassB**
- **3 ClassB orbits**, **11 ClassA orbits**
- Saddle chord has exactly **2** inversions in [6,8] (indices 0 and 2)
- L1 distance is symmetric
- L1 satisfies triangle inequality
