# Milestone 3.5.2: Quartal Traversal & Fiber Access

## Preparation

Before implementing, brush up on expert Rust skills:

- Read `~/lab/oxur/ai-rust-skill/skills/claude/SKILL.md`
- Read `~/lab/oxur/ai-rust-skill/guides/*` (especially 05-type-design, 06-traits, 11-anti-patterns)

## Detailed Plan

Quartal-primary inversion traversal (t_minus1 as the "forward" direction), quartal orbit labels, and quartal-native access to the shared base space.

### Files to create

- `crates/mt/src/quartal/voicing.rs` — quartal traversal operators (~150 lines)
- `crates/mt/src/quartal/orbit.rs` — quartal orbit labels (~120 lines)

### Files to modify

- `crates/mt/src/quartal/mod.rs` — add modules and re-exports

### Voicing functions (`voicing.rs`)

```rust
/// One step in the quartal direction (= t_minus1 in quintal terms).
/// Each voice moves DOWN to the previous chord-scale degree.
pub fn t_quartal(chord: &QuartalVoicedChord) -> QuartalVoicedChord

/// One step in the reverse-quartal direction (= t1 in quintal terms).
pub fn t_quartal_reverse(chord: &QuartalVoicedChord) -> QuartalVoicedChord

/// The quartal inversion cycle: traversed via t_quartal (= t_minus1).
/// Visits the same 4 chords as the quintal cycle, in reverse order.
/// Quartal labels: [inv0, inv1, inv2, inv3] = quintal [inv0, inv3, inv2, inv1].
pub fn quartal_inversion_cycle(chord: &QuartalVoicedChord) -> [QuartalVoicedChord; 4]

/// L1 distances between consecutive quartal inversions, closing back to root.
/// Returns [d(qinv0,qinv1), d(qinv1,qinv2), d(qinv2,qinv3), d(qinv3,qinv0)].
/// Must be [12, 12, 12, 36] — Universal L1 Law holds in both directions.
pub fn quartal_l1_distances(chord: &QuartalVoicedChord) -> [u32; 4]
```

**Implementation notes:**

- `t_quartal` delegates to `quintal::t_minus1` on the inner VoicedChord, then wraps result
- `t_quartal_reverse` delegates to `quintal::t1`
- `quartal_inversion_cycle`: apply t_quartal 3 times: [chord, t_q(chord), t_q^2(chord), t_q^3(chord)]
- `quartal_l1_distances`: use `quintal::l1_distance` on inner VoicedChords

### Quartal orbit labels (`orbit.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuartalOrbit {
    Q555,  // [P4,P4,P4] size 12, degree 8, "major analogue" (= quintal Q777)
    Q565,  // [P4,A4,P4] size 12, degree 4 (= quintal Q767)
    Q545,  // [P4,d4,P4] size 12, degree 8, "minor analogue" (= quintal Q787)
    Q656,  // [A4,P4,A4] size 6, degree 4, "diminished analogue" (= quintal Q676)
    Q646,  // [A4,d4,A4] size 6, degree 8, "augmented analogue" (= quintal Q686)
    Q454,  // [d4,P4,d4] size 12, degree 4 (= quintal Q878)
    Q464,  // [d4,A4,d4] size 12, degree 4 (= quintal Q868)
    Q655,  // [A4,P4,P4] size 24, degree 5 (= quintal Q776)
    Q455,  // [d4,P4,P4] size 24, degree 6 (= quintal Q877)
    Q465,  // [d4,A4,P4] size 24, degree 4 (= quintal Q867)
    Q456,  // [d4,P4,A4] size 24, degree 5 (= quintal Q876)
    Q445,  // [d4,d4,P4] size 24, degree 4 (= quintal Q788)
    Q645,  // [A4,d4,P4] size 24, degree 6 (= quintal Q786)
    Q446,  // [d4,d4,A4] size 12, degree 6 (= quintal Q688)
}
```

Methods:

- `representative(&self) -> QuartalIntervalStructure`
- `size(&self) -> usize`
- `degree(&self) -> usize`
- `analogy(&self) -> Option<&'static str>`
- `all() -> &'static [QuartalOrbit; 14]`
- `from_quintal(orbit: &Orbit) -> QuartalOrbit` — bijection
- `to_quintal(&self) -> Orbit` — inverse bijection

### Re-exports (in `mod.rs`)

Re-export shared infrastructure from quintal:

```rust
// Shared types (orientation-independent)
pub use crate::quintal::{BaseSpace, PcChord, FiberClass};

// Shared graph algorithms
pub use crate::quintal::{
    enumerate_all, is_adjacent,
    distance, all_distances_from, eccentricity, diameter, center,
    geodesics, count_geodesics, passing_chords,
    betweenness_centrality, saddle_chords,
};
```

### Test file

- `crates/mt/tests/quartal/test_voicing.rs` (~350 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_t_quartal_is_t_minus1` | t_quartal(chord).inner == t_minus1(chord.inner) | true |
| `test_t_quartal_reverse_is_t1` | t_quartal_reverse(chord).inner == t1(chord.inner) | true |
| `test_quartal_cycle_reverses_quintal` | quartal cycle visits same 4 pc sets in reverse order | true |
| `test_quartal_cycle_cgda` | quartal cycle of C-G-D-A: [root, 3rd, 2nd, 1st] in quintal terms | exact pitches |
| `test_quartal_l1_pattern` | quartal L1 distances | [12, 12, 12, 36] |
| `test_quartal_l1_all_228` | quartal L1 for all 228 chords | [12,12,12,36] for all |
| `test_t_quartal_fourth_is_t_minus12` | t_quartal^4 = T_minus12 (down one octave) | true |
| `test_t_quartal_round_trip` | t_quartal(t_quartal_reverse(chord)) same pc set | true |
| `test_quartal_orbit_bijection` | QuartalOrbit::from_quintal is a bijection | all 14 map uniquely |
| `test_quartal_orbit_sizes_preserved` | each quartal orbit has same size as corresponding quintal | true |
| `test_quartal_orbit_degrees_preserved` | degrees preserved | true |
| `test_quartal_analogies` | Q555=major, Q545=minor, Q656=dim, Q646=aug | true |
| `test_quartal_orbit_round_trip` | from_quintal(to_quintal(qo)) == qo | true |
| `test_reexported_base_space` | quartal::BaseSpace::new().len() == 228 | true |
| `test_reexported_distance` | same result via quartal:: and quintal:: paths | true |

### Existing modules to read for patterns

- `crates/mt/src/quintal/fiber.rs` — t1, t_minus1, inversion_cycle, l1_distance
- `crates/mt/src/quintal/orbit.rs` — Orbit enum pattern, methods
- `crates/mt/src/quartal/types.rs` — QuartalVoicedChord (from Milestone 3.5.1)
- `crates/mt/src/quartal/interval.rs` — conversion functions (from Milestone 3.5.1)

## Concept Cards

### Quartal Voice Leading (Twentieth-Century Harmony)

**Quick Definition:** Any chord tone may skip a fourth or seventh if others remain stationary. Quartal voice leading is guided by melodic purpose, not codified rules. More flexible than tertian voice leading due to rootless ambiguity.

---

### Tritone Resolution in Quartal Harmony (Twentieth-Century Harmony)

**Quick Definition:** The upper note of the tritone resolves to the nearest scale tone. Tritone at top of four-note chord = easiest resolution. Only the upper note has strong directional tendency (unlike tertian where both resolve).

---

### Compound Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** Quartal chords combined with thirds. Major third = consonant, minor third = less consonant. Five-note form (third above + below) is lush and bridges tertian and quartal.

---

### Multi-Note Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** Extended quartal formations of five or more stacked fourths, eventually encompassing all twelve tones. As more fourths are added, the chord approaches a complete chromatic aggregate.

---

## Mathematical Context

**Quartal traversal = t_minus1:** The quartal "forward" inversion operator moves each voice DOWN to the previous chord-scale degree. This is t_minus1 in the quintal convention. It traverses the fiber in reverse order.

**Quartal inversion labeling:** If quintal labels the fiber [inv0, inv1, inv2, inv3] via t1, quartal labels the same fiber [inv0, inv3, inv2, inv1] via t_minus1. So quintal's "1st inversion" is quartal's "3rd inversion" and vice versa. Root position (inv0) is shared.

**Quartal orbit naming:** Each quintal IS (i1,i2,i3) has quartal dual (12-i3, 12-i2, 12-i1). All 14 T/I orbits are self-dual: every orbit contains both its quintal representative and quartal dual (proven in paper section 26).

**t_quartal^4 = T_minus12:** Four applications of the quartal operator lower all pitches by one octave (the reverse of quintal's t1^4 = T12).

**Universal L1 Law in quartal direction:** The L1 pattern [12, 12, 12, 36] holds for the quartal traversal direction as well, because the fiber is metrically uniform — L1 distance depends only on the set of chords visited, not the traversal direction.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `crates/mt/src/quintal/fiber.rs` for t1/t_minus1 that quartal wraps
4. Read `crates/mt/src/quintal/orbit.rs` for Orbit enum pattern
5. Read `crates/mt/src/quartal/types.rs` for QuartalVoicedChord (from 3.5.1)
6. Create the module files
7. Implement all types and functions
8. Write all specified tests
9. Run `cargo test`, `cargo clippy`, `cargo fmt`

## Verification Values

- Quartal L1 distances: **[12, 12, 12, 36]** for ALL 228 chords
- `t_quartal^4` = **T_minus12** (all pitches -12)
- Quartal cycle of C-G-D-A visits same 4 voiced chords as quintal, in **reverse order**
- **14 QuartalOrbits**, bijection with 14 quintal Orbits
- Orbit sizes and degrees are **preserved** under relabeling
- Quartal [P4,P4,P4] (Q555): size **12**, degree **8** (= quintal Q777)
- Quartal [A4,d4,A4] (Q646): size **6**, degree **8** (= quintal Q686 saddle)
- Re-exported `BaseSpace` produces **228 chords** with identical degree distribution
