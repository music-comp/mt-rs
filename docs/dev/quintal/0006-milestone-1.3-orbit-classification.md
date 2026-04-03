# Milestone 1.3: Orbit Classification

## Detailed Plan

Implement T/I group actions, classify all 228 chords into the 14 orbits.

### Files to create

- `mt/src/quintal/group.rs` — `transpose`, `invert`, `invert_transpose`, `orbit()` (~100 lines)
- `mt/src/quintal/orbit.rs` — `Orbit` enum (14 variants), classification logic (~220 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod group; mod orbit;` and re-exports

### Functions (`group.rs`)

```rust
/// T_n: transpose all pitch classes by n mod 12
pub fn transpose(chord: &PcChord, n: u8) -> PcChord

/// I: invert each pitch class: x -> (12 - x) mod 12
pub fn invert(chord: &PcChord) -> PcChord

/// T_n . I: invert then transpose by n
pub fn invert_transpose(chord: &PcChord, n: u8) -> PcChord

/// Compute the full T/I orbit of a chord (up to 24 elements, deduplicated, sorted)
pub fn orbit(chord: &PcChord) -> Vec<PcChord>
```

- `transpose`: map each pc -> (pc+n)%12, sort, construct PcChord
- `invert`: map each pc -> (12-pc)%12, sort. Note (12-0)%12 = 0
- `invert_transpose`: map each pc -> ((12-pc)+n)%12, sort
- `orbit`: BTreeSet collecting T_n for n in 0..12 and T_n(I(chord)) for n in 0..12, then to sorted Vec

### Types (`orbit.rs`)

**`Orbit` enum** -- 14 variants named by interval structure:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Orbit {
    Q777,  // [P5,P5,P5] size 12, degree 8, "major analogue"
    Q767,  // [P5,d5,P5] size 12, degree 4
    Q787,  // [P5,A5,P5] size 12, degree 8, "minor analogue"
    Q676,  // [d5,P5,d5] size 6, degree 4, "diminished analogue"
    Q686,  // [d5,A5,d5] size 6, degree 8, "augmented analogue"
    Q878,  // [A5,P5,A5] size 12, degree 4
    Q868,  // [A5,d5,A5] size 12, degree 4
    Q776,  // [P5,P5,d5] size 24, degree 5
    Q877,  // [A5,P5,P5] size 24, degree 6
    Q867,  // [A5,d5,P5] size 24, degree 4
    Q876,  // [A5,P5,d5] size 24, degree 5
    Q788,  // [P5,A5,A5] size 24, degree 4
    Q786,  // [P5,A5,d5] size 24, degree 6
    Q688,  // [d5,A5,A5] size 12, degree 6
}
```

Methods:

- `representative(&self) -> IntervalStructure` -- returns the canonical IS for this orbit
- `size(&self) -> usize` -- 6, 12, or 24
- `degree(&self) -> usize` -- 4, 5, 6, or 8
- `analogy(&self) -> Option<&'static str>` -- "major", "minor", "diminished", "augmented", or None
- `all() -> &'static [Orbit; 14]` -- all 14 variants

### Classification functions (`orbit.rs`)

```rust
pub fn classify_orbit(chord: &PcChord) -> Option<Orbit>
pub fn classify_all(chords: &[PcChord]) -> BTreeMap<Orbit, Vec<PcChord>>
```

**Classification approach (eager lookup table):**

1. For each of 14 orbits, compute a representative PcChord from the interval structure starting at pc 0
2. Compute the full T/I orbit of each representative
3. Build a `HashMap<PcChord, Orbit>` mapping every chord in each orbit to the orbit type
4. `classify_orbit()` is a simple lookup
5. Consider using `std::sync::LazyLock` for the lookup table, or building it in `classify_all()`

### The 14 orbit representatives

| Variant | Interval Structure | Representative (from pc 0) | Size | Degree | Analogy |
|---------|-------------------|---------------------------|------|--------|---------|
| Q777 | (7,7,7) | {0, 7, 2, 9} -> sorted [0,2,7,9] | 12 | 8 | major |
| Q767 | (7,6,7) | {0, 7, 1, 8} -> sorted [0,1,7,8] | 12 | 4 | -- |
| Q787 | (7,8,7) | {0, 7, 3, 10} -> sorted [0,3,7,10] | 12 | 8 | minor |
| Q676 | (6,7,6) | {0, 6, 1, 7} -> sorted [0,1,6,7] | 6 | 4 | dim |
| Q686 | (6,8,6) | {0, 6, 2, 8} -> sorted [0,2,6,8] | 6 | 8 | aug |
| Q878 | (8,7,8) | {0, 8, 3, 11} -> sorted [0,3,8,11] | 12 | 4 | -- |
| Q868 | (8,6,8) | {0, 8, 2, 10} -> sorted [0,2,8,10] | 12 | 4 | -- |
| Q776 | (7,7,6) | {0, 7, 2, 8} -> sorted [0,2,7,8] | 24 | 5 | -- |
| Q877 | (8,7,7) | {0, 8, 3, 10} -> sorted [0,3,8,10] | 24 | 6 | -- |
| Q867 | (8,6,7) | {0, 8, 2, 9} -> sorted [0,2,8,9] | 24 | 4 | -- |
| Q876 | (8,7,6) | {0, 8, 3, 9} -> sorted [0,3,8,9] | 24 | 5 | -- |
| Q788 | (7,8,8) | {0, 7, 3, 11} -> sorted [0,3,7,11] | 24 | 4 | -- |
| Q786 | (7,8,6) | {0, 7, 3, 9} -> sorted [0,3,7,9] | 24 | 6 | -- |
| Q688 | (6,8,8) | {0, 6, 2, 10} -> sorted [0,2,6,10] | 12 | 6 | -- |

**Note:** Some representatives above may need verification -- the key is that starting from pc 0, applying the interval structure as forward steps (mod 12) gives the 4 pitch classes. The PcChord representation always stores them sorted.

### Test file

- `mt/tests/quintal/test_orbit.rs` (~200 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_transpose_identity` | `transpose(chord, 0)` | Same chord |
| `test_transpose_mod12` | `transpose(chord, 12)` | Same chord |
| `test_transpose_cgda_by_1` | `transpose([0,2,7,9], 1)` | [1,3,8,10] |
| `test_transpose_cgda_by_7` | `transpose([0,2,7,9], 7)` | [4,7,9,2] -> sorted [2,4,7,9] |
| `test_invert_involution` | `invert(invert(chord))` | Same chord (I^2 = id) |
| `test_invert_cgda` | `invert([0,2,7,9])` | [(12-0)%12, (12-2)%12, (12-7)%12, (12-9)%12] = [0,10,5,3] -> sorted [0,3,5,10] |
| `test_invert_zero_fixpoint` | `invert` fixes pc 0 | pc 0 in result |
| `test_orbit_size_777` | `orbit([0,2,7,9]).len()` | 12 |
| `test_orbit_size_676` | orbit of a Q676 chord | 6 |
| `test_orbit_size_686` | `orbit([0,2,6,8]).len()` | 6 |
| `test_orbit_size_generic` | orbit of a Q776 chord | 24 |
| `test_classify_orbit_777` | `classify_orbit([0,2,7,9])` | Some(Orbit::Q777) |
| `test_classify_orbit_686` | `classify_orbit([0,2,6,8])` | Some(Orbit::Q686) |
| `test_classify_orbit_787` | classify a [7,8,7] chord | Some(Orbit::Q787) |
| `test_classify_all_14_orbits` | Number of distinct orbits in classify_all | 14 |
| `test_classify_all_total` | Sum of all orbit members | 228 |
| `test_orbit_sizes_match` | Each orbit's actual size matches `Orbit::size()` | true for all 14 |
| `test_orbit_degrees_match` | Each orbit's members all have same degree (from BaseSpace) | true for all 14 |
| `test_transpose_preserves_legality` | transpose of legal chord is legal | true |
| `test_invert_preserves_legality` | invert of legal chord is legal | true |
| `test_orbit_representative_structures` | Each orbit's representative matches its IS | true for all 14 |

### Existing modules to read for patterns

- `mt/src/quintal/types.rs` — PcChord (from Milestone 1.1)
- `mt/src/quintal/base_space.rs` — enumerate_all, BaseSpace (from Milestone 1.2)
- `mt/src/set_class/mod.rs` — transpose/invert on PitchClassSet (existing pattern)
- `mt/src/neo_riemannian/mod.rs` — enum pattern for operations

## Concept Cards

### Transposition (A Geometry of Music)

**Quick Definition:** Transposition moves every pitch in the same direction by the same amount, corresponding geometrically to translation in pitch space or rotation in pitch-class space.

**Core Definition:** Transposition is one of only two distance-preserving transformations of musical space (the other being inversion). In pitch space, Tx(p) = p + x, shifting every point the same distance in the same direction. In pitch-class space, transposition corresponds to rotation of the circle. Transpositions have a "size" and can be distinguished by this size. Transposition preserves all intervallic relationships between pitches.

---

### Transpositional Set Class / Chord Type (A Geometry of Music)

**Quick Definition:** A transpositional set class (or chord type) groups together all chords related by transposition, so that all major triads, for instance, belong to the same chord type.

**Core Definition:** A transpositional set class is an equivalence class formed by four OPTIC symmetry operations: O, P, T, and C (OPTC). Two chords belong to the same chord type if one can be rotated into the other on the pitch-class circle. Such chords share the same sequence of arc-length distances between adjacent notes. Does not consider inversion -- major and minor triads are different chord types.

---

### Set Class (Open Music Theory)

**Quick Definition:** A set class is a group of all pitch-class sets related by transposition (Tn) or inversion (In). All members share the same interval content.

**Core Definition:** Set class is the most abstract level of harmonic/melodic classification in set theory. A set class groups together all pc sets that are transpositionally or inversionally equivalent. All members of a set class share the same interval vector. Set classes are named by their prime form and catalogued by Forte number. There are 220 set classes total (from cardinality 0 to 12). Size = 24 / (number of self-mapping operations). Major and minor triads belong to the same set class (037) because they are inversionally related.

---

### Symmetry and Set Class Size (Introduction to Post-Tonal Theory)

**Quick Definition:** The inverse relationship between a set class's degree of symmetry and the number of distinct sets it contains: dividing 24 by the total number of self-mapping operations gives the number of sets in the class.

**Core Definition:** The size of a set class (the number of distinct pitch-class sets it contains) is inversely proportional to its degree of symmetry. For any set class, the number of sets = 24 / (number of self-mapping operations). Most set classes have degree (1, 0) and contain 24 distinct sets. The more symmetrical the set, the fewer distinct members the set class contains. Examples: (0167) degree (2, 2) -> 24 / 4 = 6 sets; (0369) degree (4, 4) -> 24 / 8 = 3 sets.

---

### List of Set Classes (Introduction to Post-Tonal Theory)

**Quick Definition:** The List of Set Classes is a comprehensive catalog providing prime forms, Forte names, interval-class vectors, and symmetry information for all possible set classes.

**Core Definition:** The list is organized by cardinality. Tetrachords: 29 set classes. Each entry provides: (1) prime form, (2) Forte name, (3) interval-class vector, (4) symmetry measures. Sets with more than six elements are listed across from their complements. The thousands of possible sets reduce to a manageable catalog.

---

## Mathematical Context

**Transposition Tn:** Acts on a pc set S by adding n to each element mod 12. Tn(S) = {(s+n) mod 12 : s in S}. The set of all transpositions forms the cyclic group Z12.

**Inversion I:** Acts on a pc set S by replacing each element with its mod-12 complement. I(S) = {(12-s) mod 12 : s in S}. Note: this maps 0->0, 1->11, 2->10, etc.

**T/I orbit:** The set of all chords reachable from a given chord by any combination of transpositions and inversions. For each chord C, its orbit is {Tn(I^k(C)) : n in {0,...,11}, k in {0,1}}. An orbit's size divides 24. A chord with no T/I symmetry has orbit size 24; one with a non-trivial stabilizer has a smaller orbit.

**The 14 orbit sizes:** Orbits of size 6 have a stabilizer of order 4; orbits of size 12 have stabilizer of order 2; orbits of size 24 are generic (trivial stabilizer). Total: 6+6+12+12+12+12+24+24+24+24+24+24+12+12 = 228.

**Orbit degree uniformity:** All members of an orbit have the same degree in the adjacency graph. This is because transposition and inversion are graph isometries -- they preserve adjacency. So degree is an orbit invariant.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides
3. Read existing similar modules for patterns: `mt/src/set_class/mod.rs` (transpose/invert), `mt/src/neo_riemannian/mod.rs` (enum pattern)
4. Create the module files specified in the plan
5. Implement all types and functions
6. Write all specified tests
7. Run `cargo test` -- fix any failures
8. Run `cargo clippy` -- fix any warnings
9. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly by your implementation:

- `transpose(chord, 0)` == chord (identity)
- `transpose(chord, 12)` == chord (mod 12 identity)
- `invert(invert(chord))` == chord (involution)
- `transpose([0,2,7,9], 1)` == [1,3,8,10]
- `invert([0,2,7,9])` == [0,3,5,10]
- `orbit([0,2,7,9]).len()` == **12** (Q777, stabilizer order 2)
- `orbit([0,2,6,8]).len()` == **6** (Q686, stabilizer order 4)
- Exactly **14** distinct orbits produced from all 228 chords
- Orbit sizes sum to **228**
- Orbit size distribution: **two size-6, four size-12, eight size-24**
  - Size 6: Q676, Q686
  - Size 12: Q777, Q767, Q787, Q878, Q868, Q688
  - Size 24: Q776, Q877, Q867, Q876, Q788, Q786
  - Wait -- that's 2 + 6 + 6 = 14 but sizes are 2*6 + 6*12 + 6*24 = 12+72+144 = 228. Let me recount from the paper:
  - Size 6 (2 orbits): Q676, Q686 -> 12
  - Size 12 (4 orbits): Q777, Q767, Q787, Q878 -> wait, need 4. From the table: Q777(12), Q767(12), Q787(12), Q878(12), Q868(12), Q688(12) = 6 orbits of size 12 = 72. Q776(24), Q877(24), Q867(24), Q876(24), Q788(24), Q786(24) = 6 orbits of size 24 = 144. Total = 12+72+144 = 228. So it's 2 size-6, 6 size-12, 6 size-24.
- **Corrected orbit size distribution: two size-6, six size-12, six size-24**
- Each orbit's members all have the same degree in the adjacency graph
- `classify_orbit([0,2,7,9])` == Some(Orbit::Q777)
- `classify_orbit([0,2,6,8])` == Some(Orbit::Q686)
- Transpose and invert both preserve legality (map legal chords to legal chords)
