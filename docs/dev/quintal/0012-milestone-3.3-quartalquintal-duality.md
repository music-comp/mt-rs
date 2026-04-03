# Milestone 3.3: Quartal/Quintal Duality

## Detailed Plan

Formalize and verify the quartal/quintal duality theorem: quartal and quintal are orientation reversal on the Z4 fiber.

### Files to create

- `mt/src/quintal/duality.rs` — Duality functions and orbit self-duality verification (~150 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod duality;` and re-exports

### Functions

```rust
pub fn quartal_reading(chord: &VoicedChord) -> IntervalStructure
pub fn quintal_reading(chord: &VoicedChord) -> IntervalStructure
pub fn reverse_interval_structure(is: &IntervalStructure) -> IntervalStructure
pub fn t1_reversal_equivalence(chord: &VoicedChord) -> bool
pub fn orbit_self_duality(orbit: &Orbit, space: &BaseSpace) -> bool
pub fn verify_all_orbits_self_dual(space: &BaseSpace) -> bool
```

**`quartal_reading(chord)`:**

Read intervals top-to-bottom (fourths perspective). For ascending pitches [p0, p1, p2, p3]: `(p3-p2, p2-p1, p1-p0)` — the reversal of the quintal reading.

**`quintal_reading(chord)`:**

Read intervals bottom-to-top (fifths perspective). Same as `VoicedChord::interval_structure()`: `(p1-p0, p2-p1, p3-p2)`.

**`reverse_interval_structure(is)`:**

`IntervalStructure(is.2, is.1, is.0)` — swap first and third components.

**`t1_reversal_equivalence(chord)`:**

Verify that the t-1 cycle visits the same 4 pitch-class sets as the t1 cycle, in reverse order:
1. Compute t1 cycle: [inv0, inv1, inv2, inv3]
2. Compute t-1 cycle: [inv0, t_minus1(inv0), t_minus1(t_minus1(inv0)), ...]
3. The t-1 cycle should visit the same pc sets as [inv0, inv3, inv2, inv1]

**`orbit_self_duality(orbit, space)`:**

1. Pick a representative chord from the orbit
2. Compute its interval structure (i1, i2, i3)
3. Compute the reversed IS (i3, i2, i1)
4. Find a chord with the reversed IS among legal chords
5. Check that both chords are in the same T/I orbit

**`verify_all_orbits_self_dual(space)`:**

Check `orbit_self_duality` for all 14 orbits. All must be self-dual.

### Test file

- `mt/tests/quintal/test_duality.rs` (~180 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_quintal_reading_cgda` | quintal reading of C-G-D-A | (7, 7, 7) |
| `test_quartal_reading_cgda` | quartal reading of C-G-D-A | (7, 7, 7) — palindromic |
| `test_reverse_palindromic` | reverse of (7,7,7) | (7,7,7) — same |
| `test_reverse_asymmetric` | reverse of (7,7,6) | (6,7,7) |
| `test_reverse_is_involution` | reverse(reverse(is)) == is | true |
| `test_t1_reversal_cgda` | t-1 cycle is reverse of t1 cycle | true |
| `test_t1_reversal_crossroads` | t-1 reversal for crossroads | true |
| `test_t1_reversal_all_orbits` | t1 reversal for a representative of each orbit | true |
| `test_orbit_self_duality_q777` | Q777 is self-dual | true |
| `test_orbit_self_duality_q776` | Q776 is self-dual ((7,7,6) and (6,7,7) same T/I orbit) | true |
| `test_all_orbits_self_dual` | all 14 orbits self-dual | true |
| `test_l1_pattern_same_both_directions` | L1 [12,12,12,36] same for both t1 and t-1 | true |
| `test_quartal_quintal_same_fiber` | quartal and quintal readings are same pc set | true |

### Existing modules to read for patterns

- `mt/src/quintal/fiber.rs` — t1, t_minus1, inversion_cycle, chord_scale (from Milestone 3.1)
- `mt/src/quintal/types.rs` — VoicedChord, IntervalStructure
- `mt/src/quintal/orbit.rs` — Orbit enum, classify_orbit
- `mt/src/quintal/base_space.rs` — BaseSpace, enumerate_all

## Concept Cards

### Quartal Harmony (Twentieth-Century Harmony)

**Quick Definition:** A harmonic system in which chords are built by superimposing intervals of the fourth, creating a distinctly twentieth-century sound.

**Core Definition:** Chords by fourths are built by superimposing intervals of the fourth. This rootless harmony places the burden of key verification upon the most active melodic line. Any member can function as root. Spacing must preserve fourths to maintain quartal identity.

---

### Quartal Voicings (A Geometry of Music)

**Quick Definition:** Chord voicings built from stacks of perfect fourths rather than thirds, creating an "open" modern sound associated with McCoy Tyner and the post-1950s jazz tradition.

**Core Definition:** Postwar jazz musicians reconceived traditional chords as quartal rather than tertian. Three-note quartal voicings are now a standard jazz piano technique. Quartal voicings make the logic of tritone substitution transparent. Associated with Stravinsky, Bartok, Hindemith.

---

### Quartal-Tertian Pivotal Structures (Twentieth-Century Harmony)

**Quick Definition:** Chords containing equal numbers of thirds and fourths that can function as either tertian or quartal sonorities, serving as pivot points between the two harmonic systems.

**Core Definition:** If thirds overrun a six- or seven-note chord, the ear hears a thirteenth formation. If fourths overrun, a chord by fourths sounds. If the number of thirds and fourths is equal, the chord may be used as a pivotal structure and regarded as belonging to either category. Voicing/arrangement determines perceived identity.

---

### Compound Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** Quartal chords combined with thirds (compound construction), where a major or minor third added above or below creates fresh colors.

**Core Definition:** A third may be added above or below a three-note chord by fourths. Major third = consonant; minor third = less consonant. The compound chord with major third is effective as a cadential tonic. A five-note form (third above + below) is lush and bridges tertian and quartal.

---

### Tritone Resolution in Quartal Harmony (Twentieth-Century Harmony)

**Quick Definition:** The treatment of the augmented fourth within quartal chords, where the upper note resolves to the nearest scale tone and the tritone moves most easily at the top of four-note chords.

**Core Definition:** The upper note of the tritone resolves to the nearest scale tone. If equidistant neighbors, either direction is taken. Tritone at top of four-note chord = easiest resolution. Pedal point lessens resolution need. Only the upper note has strong directional tendency (unlike tertian where both resolve).

---

## Mathematical Context

**Quartal/quintal duality as orientation reversal on the fiber:**

The t1 operator traverses the Z4 fiber in one direction (quintal = "stacked fifths, reading up"). The t-1 operator traverses the same fiber in the reverse direction (quartal = "stacked fourths, reading down"). They visit the same 4 chords in reverse order.

**Formally:** If the inversion cycle under t1 is [C0, C1, C2, C3], then the cycle under t-1 is [C0, C3, C2, C1] — the same set, reversed (with C0 as the shared starting point since it's a cycle).

**Orbit self-duality:** The quartal dual of interval structure (i1,i2,i3) is (i3,i2,i1) — the same intervals read backwards. Under the T/I group, (i1,i2,i3) and (i3,i2,i1) always belong to the same orbit, because pitch-class inversion In reverses interval sequences. This is why we get 14 T/I orbits from 20 T-orbits: 6 asymmetric pairs collapse.

**Z2 symmetry:** The fiber Z4 has a natural Z2 action (orientation reversal: k -> -k mod 4). This Z2 is the quartal/quintal duality.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read `mt/src/quintal/fiber.rs` for t1, t_minus1, inversion_cycle (from Milestone 3.1)
4. Read `mt/src/quintal/orbit.rs` for Orbit, classify_orbit
5. Create the module files specified in the plan
6. Implement all functions
7. Write all specified tests
8. Run `cargo test` -- fix any failures
9. Run `cargo clippy` -- fix any warnings
10. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly:

- C-G-D-A quintal reading: **(7, 7, 7)**
- C-G-D-A quartal reading: **(7, 7, 7)** (palindromic)
- Interval reversal of (7,7,7) -> (7,7,7) — same orbit
- Interval reversal of (7,7,6) -> (6,7,7) — different T-orbit, **same T/I orbit**
- t-1 cycle visits **same 4 chords** as t1 cycle in reverse order
- **All 14 orbits** are self-dual
- The 20 T-orbits collapse to 14 T/I orbits (6 asymmetric pairs merged)
- L1 distance pattern **[12, 12, 12, 36]** identical for both traversal directions
- `reverse_interval_structure` is an involution
