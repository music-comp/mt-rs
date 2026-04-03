# Milestone 3.1: Chord-Scale & Tymoczko Inversion Operators

## Detailed Plan

Implement the chord-scale derivation, the t1/t-1 operators, inversion cycles, and L1 distance on voiced chords.

### Files to create

- `mt/src/quintal/fiber.rs` — Chord-scale, t1/t-1 operators, inversion cycle, projection, L1 distance (~300 lines)

### Files to modify

- `mt/src/quintal/mod.rs` — add `mod fiber;` and re-exports

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChordScale {
    pub pcs: [u8; 4],     // pitch classes sorted ascending within one octave
    pub steps: [u8; 4],   // cyclic step sizes between consecutive degrees
}
```

### Functions

```rust
pub fn chord_scale(chord: &VoicedChord) -> ChordScale
pub fn t1(chord: &VoicedChord) -> VoicedChord
pub fn t_minus1(chord: &VoicedChord) -> VoicedChord
pub fn inversion_cycle(chord: &VoicedChord) -> [VoicedChord; 4]
pub fn project(chord: &VoicedChord) -> PcChord
pub fn l1_distance(a: &VoicedChord, b: &VoicedChord) -> u32
```

**`chord_scale(chord)`:**

1. Extract pitch classes: `pcs[i] = pitches[i] % 12` for each voice
2. Sort the 4 pitch classes ascending within [0,11]
3. Deduplicate (should be 4 distinct for valid quintal chords)
4. Compute cyclic step sizes: `steps[i] = (pcs[(i+1) % 4] - pcs[i] + 12) % 12`
5. Steps always sum to 12 (one octave)

**`t1(chord)` — one step up the inversion cycle:**

Critical algorithm (from paper section 20):

1. Compute chord scale `cs` and step sequence
2. For each voice `p[i]` with pitch class `pc_i = p[i] % 12`:
   a. Find index `j` in chord scale where `cs.pcs[j] == pc_i`
   b. Step size = `cs.steps[j]` (interval from this degree to next)
   c. New pitch = `p[i] + step_size`
3. Sort the 4 new pitches into ascending order
4. Construct `VoicedChord` from sorted result

**`t_minus1(chord)` — one step down:**

Reverse of t1. For each voice:

1. Find index `j` where `cs.pcs[j] == pc_i`
2. Previous degree index = `(j + 3) % 4`
3. Step size down = `cs.steps[(j + 3) % 4]`
4. New pitch = `p[i] - step_size`
5. Sort ascending, construct VoicedChord

**`inversion_cycle(chord)` — all 4 inversions:**

`[chord, t1(chord), t1(t1(chord)), t1(t1(t1(chord)))]`

**`project(chord)` — pi: E -> B:**

Same as `VoicedChord::to_pc_chord()`. Included for API clarity.

**`l1_distance(a, b)` — L1 (Manhattan) distance:**

`|a[0]-b[0]| + |a[1]-b[1]| + |a[2]-b[2]| + |a[3]-b[3]|`

Uses absolute differences of MIDI pitches, NOT mod 12.

### Test file

- `mt/tests/quintal/test_fiber.rs` (~300 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_chord_scale_cgda` | chord_scale of C3-G3-D4-A4 | pcs=[0,2,7,9], steps=[2,5,2,3] |
| `test_chord_scale_steps_sum_12` | steps always sum to 12 | true |
| `test_t1_cgda_root_to_1st` | t1(48,55,62,69) | (50,57,67,72) |
| `test_t1_cgda_1st_to_2nd` | t1(50,57,67,72) | (55,60,69,74) |
| `test_t1_cgda_2nd_to_3rd` | t1(55,60,69,74) | (57,62,72,79) |
| `test_t1_fourth_equals_t12` | t1^4 = T12 (all pitches +12) | true |
| `test_t_minus1_reverses_t1` | t_minus1(t1(chord)) has same pc set | true |
| `test_inversion_cycle_cgda` | full cycle matches paper | all 4 voicings match |
| `test_inversion_cycle_intervals` | intervals match paper table | (7,7,7), (7,10,5), (5,9,5), (5,10,7) |
| `test_project_all_inversions_same` | all 4 inversions project to same PcChord | true |
| `test_only_root_in_base` | only root position has IS in [6,8] for CGDA | true |
| `test_crossroads_two_in_base` | crossroads chord has 2 inversions in [6,8] | indices 0 and 2 |
| `test_crossroads_inversion_intervals` | crossroads: (6,8,6), (6,10,6), (6,8,6), (6,10,6) | true |
| `test_l1_distance_symmetric` | l1(a,b) == l1(b,a) | true |
| `test_l1_root_to_1st_cgda` | l1 between root and 1st of CGDA | 12 |
| `test_t1_fourth_l1_cost` | total cycle: 12+12+12+36=72 | true |

### Existing modules to read for patterns

- `mt/src/quintal/types.rs` — VoicedChord, PcChord, IntervalStructure
- `mt/src/quintal/base_space.rs` — enumerate_all, BaseSpace

## Concept Cards

### Scalar Transposition (A Geometry of Music)

**Quick Definition:** Scalar transposition shifts a musical pattern by a fixed number of scale steps, preserving scalar intervals while potentially altering chromatic intervals.

**Core Definition:** Scalar transposition adds a constant to each scale degree number, using "scale degree arithmetic." Unlike chromatic transposition which preserves exact semitone intervals, scalar transposition preserves scalar intervals while allowing chromatic intervals to vary. Can act on notes outside the scale via fractional scale degree numbers.

---

### Interscalar Transposition in 20th-Century Music (A Geometry of Music)

**Quick Definition:** The technique of mapping music from one scale type to another while preserving the scalar step pattern.

**Core Definition:** Interscalar transposition maps the notes of one scale onto the notes of another preserving the ordering of scale degrees. It systematically exploits all possible ways of mapping one chord type onto another. Connected to the geometry of chord space.

---

### Combining Scalar and Chromatic Transposition (A Geometry of Music)

**Quick Definition:** Any strongly crossing-free voice leading between chords of the same type can be decomposed into a scalar transposition and a chromatic transposition that nearly cancel each other out.

**Core Definition:** The voice leading is efficient when the scalar and chromatic components nearly cancel. For n-note chords, only n interscalar templates exist; these combine with chromatic transpositions to generate all efficient voice leadings. This is the culminating theoretical result of Part I.

---

### Strongly Crossing-Free Voice Leading (A Geometry of Music)

**Quick Definition:** A voice leading that remains crossing-free no matter how its voices are distributed in register. Every strongly crossing-free voice leading is an interscalar transposition, and vice versa.

**Core Definition:** The key theorem: a voice leading is strongly crossing-free if and only if it is a scalar or interscalar transposition. Since removing crossings never increases voice-leading size, there is always a maximally efficient voice leading that is strongly crossing-free. Dramatically reduces the search space for efficient voice leadings.

---

### Chord Progressions vs Voice Leadings (A Geometry of Music)

**Quick Definition:** In chord space, a chord progression is a pair of points (no specific path between them), while a voice leading is a specific path connecting those points.

**Core Definition:** A chord progression specifies "where" but not "how"; a voice leading specifies both. Because there can be infinitely many paths between any two points, the same chord progression can be realized by infinitely many different voice leadings. The size of the voice leading corresponds to the length of the path.

---

## Mathematical Context

**Tymoczko's interscalar transposition (t1):** Given a voiced chord (p1, p2, p3, p4) in ascending order, with pitch-class set S = {pc(p1), pc(p2), pc(p3), pc(p4)}, the chord scale is S sorted within one octave. The step sequence is the circular sequence of intervals between consecutive chord-scale degrees.

**Computing t1:** For each voice pi with pitch class pci, find the next chord-scale degree above pci in the circular ordering of S. The step size is the interval from pci to that next degree. Add this step to pi to get the new pitch. Then re-sort the result into ascending order.

**Concrete algorithm for t1(p1, p2, p3, p4):**

1. Compute the chord scale: sort the 4 pitch classes into ascending order within [0,11]: cs = [c0, c1, c2, c3]
2. For each voice pi, find its pitch class pci = pi mod 12
3. Find the index j such that cs[j] = pci
4. The next scale degree is cs[(j+1) mod 4]
5. The step size is (cs[(j+1) mod 4] - cs[j] + 12) mod 12
6. New pitch = pi + step_size
7. Sort the 4 new pitches into ascending order

**Key property:** t1^4 = T12 (four applications raise all pitches by one octave). The cycle always closes after exactly 4 steps.

**t-1 is the reverse:** each voice moves DOWN to the previous chord-scale degree.

**Verified example — C3-G3-D4-A4:**

- Chord scale {C,D,G,A}, steps [2,5,2,3]
- Root: (48,55,62,69) intervals (7,7,7) — in [6,8]
- t1 -> (50,57,67,72) intervals (7,10,5) — NOT in [6,8]
- t1 -> (55,60,69,74) intervals (5,9,5) — NOT in [6,8]
- t1 -> (57,62,72,79) intervals (5,10,7) — NOT in [6,8]
- t1 -> (60,67,74,81) = (48+12,55+12,62+12,69+12) = T12(root)

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides (especially 11-anti-patterns, 05-type-design)
3. Read `mt/src/quintal/types.rs` for VoicedChord API
4. Create the module files specified in the plan
5. Implement all types and functions
6. Write all specified tests
7. Run `cargo test` -- fix any failures
8. Run `cargo clippy` -- fix any warnings
9. Run `cargo fmt` -- ensure formatting

## Verification Values

The following values MUST be reproduced exactly:

- Chord scale of {C,D,G,A} = {0,2,7,9}: steps **[2, 5, 2, 3]**
- C3-G3-D4-A4 = (48,55,62,69) inversion cycle:
  - Root: **(48, 55, 62, 69)** intervals (7, 7, 7) span 21
  - 1st: **(50, 57, 67, 72)** intervals (7, 10, 5) span 22
  - 2nd: **(55, 60, 69, 74)** intervals (5, 9, 5) span 19
  - 3rd: **(57, 62, 72, 79)** intervals (5, 10, 7) span 22
- `t1^4` = T12 for all tested chords
- `project` maps all 4 inversions to the **same PcChord**
- Only root position has IS in [6,8] for C-G-D-A
- Crossroads chord (48,54,62,68) cycle: root (6,8,6) yes, 1st (6,10,6) no, 2nd (6,8,6) yes, 3rd (6,10,6) no
- L1 between root and 1st inversion of C-G-D-A: **12**
