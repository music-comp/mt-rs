# Milestone 3.5.3: Quartal-Native Constructors & Integration Tests

## Preparation

Before implementing, brush up on expert Rust skills:

- Read `~/lab/oxur/ai-rust-skill/skills/claude/SKILL.md`
- Read `~/lab/oxur/ai-rust-skill/guides/*` (especially 03-error-handling, 05-type-design, 11-anti-patterns)

## Detailed Plan

Idiomatic quartal chord construction, comprehensive cross-module integration tests, and Persichetti-derived verification.

### Files to create

- `crates/mt/src/quartal/constructors.rs` — quartal-native chord builders (~120 lines)
- `crates/mt/src/quartal/error.rs` — quartal error type (~40 lines)
- `crates/mt/tests/quartal/test_constructors.rs` (~200 lines)
- `crates/mt/tests/quartal/test_quartal_quintal_identity.rs` — exhaustive duality tests (~200 lines)

### Files to modify

- `crates/mt/src/quartal/mod.rs` — add modules and re-exports

### Error type (`error.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuartalError {
    /// Interval value not in {4, 5, 6}
    IllegalInterval(u8),
    /// Expected exactly 3 intervals for a 4-note chord
    WrongIntervalCount(usize),
    /// Pitched not in ascending order
    NotAscending,
    /// Duplicate pitch classes after mod 12
    DuplicatePitchClasses,
    /// Pitch class out of range (must be 0..=11)
    PitchClassOutOfRange(u8),
}
```

Display + Error impls. Follow QuintalError pattern.

### Constructors (`constructors.rs`)

```rust
/// Build a PcChord by stacking fourths upward from a root pitch class.
/// Each interval must be in {4, 5, 6}. Exactly 3 intervals required.
///
/// Example: from_stacked_fourths(9, &[5,5,5]) builds A-D-G-C = {9,2,7,0}
pub fn from_stacked_fourths(root_pc: u8, intervals: &[u8]) -> Result<PcChord, QuartalError>

/// Build a QuartalVoicedChord by stacking fourths upward from a root MIDI pitch.
pub fn from_stacked_fourths_voiced(
    root_pitch: u8,
    intervals: &[u8],
) -> Result<QuartalVoicedChord, QuartalError>

/// Convenience: three perfect fourths from a root.
/// Example: pure_quartal_stack(0) = C-F-Bb-Eb = {0,5,10,3}
pub fn pure_quartal_stack(root_pc: u8) -> PcChord

/// Adjacent chords with their quartal interval structures.
pub fn quartal_neighbors(
    chord: &PcChord,
    space: &BaseSpace,
) -> Vec<(PcChord, QuartalIntervalStructure)>
```

**`from_stacked_fourths` algorithm:**

1. Validate `intervals.len() == 3`
2. Validate each interval in {4, 5, 6}
3. Validate root_pc in 0..=11
4. Compute pcs: `[root, (root+i1)%12, (root+i1+i2)%12, (root+i1+i2+i3)%12]`
5. Construct PcChord (sorts internally)

**`from_stacked_fourths_voiced` algorithm:**

1. Same validation
2. Compute pitches: `[root, root+i1, root+i1+i2, root+i1+i2+i3]` (NOT mod 12 — MIDI pitches)
3. Construct QuartalVoicedChord

**`pure_quartal_stack`:**
`from_stacked_fourths(root_pc, &[5, 5, 5]).unwrap()`

**`quartal_neighbors`:**

1. Get neighbors from `space.neighbors(chord)`
2. For each neighbor, compute its quartal interval structure via `pc_chord_quartal_intervals`
3. Return as Vec of (PcChord, QuartalIntervalStructure) pairs

### Test files

**`test_constructors.rs`:**

| Test | Description | Expected |
|------|-------------|----------|
| `test_pure_quartal_stack_c` | `pure_quartal_stack(0)` | {0, 3, 5, 10} (C-Eb-F-Bb reading as stacked 4ths) |
| `test_pure_quartal_stack_a` | `pure_quartal_stack(9)` | {0, 2, 7, 9} (= C-G-D-A, same as quintal!) |
| `test_from_stacked_fourths_valid` | `from_stacked_fourths(9, &[5,5,5])` | Ok, pcs = [0,2,7,9] |
| `test_from_stacked_fourths_mixed` | `from_stacked_fourths(0, &[5,6,5])` | Ok (P4,A4,P4) |
| `test_from_stacked_fourths_bad_interval` | interval 3 | Err(IllegalInterval(3)) |
| `test_from_stacked_fourths_bad_interval_7` | interval 7 | Err(IllegalInterval(7)) |
| `test_from_stacked_fourths_wrong_count` | 2 intervals | Err(WrongIntervalCount(2)) |
| `test_from_stacked_fourths_too_many` | 4 intervals | Err(WrongIntervalCount(4)) |
| `test_from_stacked_fourths_bad_root` | root_pc 13 | Err(PitchClassOutOfRange(13)) |
| `test_voiced_stacked_fourths` | from_stacked_fourths_voiced(57, &[5,5,5]) | pitches [57,62,67,72] = A3-D4-G4-C5 |
| `test_all_stacked_in_base_space` | every pure_quartal_stack(n) is in BaseSpace | true for n in 0..12 |
| `test_quartal_neighbors` | quartal_neighbors returns non-empty with valid IS | true |

**`test_quartal_quintal_identity.rs`** — exhaustive cross-module tests:

| Test | Description | Expected |
|------|-------------|----------|
| `test_all_228_dual_intervals` | for every chord: quintal IS (i1,i2,i3) and quartal IS (12-i3,12-i2,12-i1) describe the same chord | true |
| `test_all_orbits_same_chords` | each quintal orbit and corresponding quartal orbit contain the same chords | true |
| `test_all_fibers_same_chords` | quartal and quintal cycles visit the same 4 voiced chords for every chord | true |
| `test_saddle_dual_perspective` | 6 saddle chords have degree 8, max betweenness, 2 inversions in [6,8] from BOTH perspectives | true |
| `test_distance_identical` | distance via quartal path == distance via quintal path for sampled pairs | true |
| `test_quartal_root_vs_quintal_root` | pure_quartal_stack(0) has quartal root 0, quintal root 3 | true |

### Existing modules to read for patterns

- `crates/mt/src/quartal/types.rs` — QuartalVoicedChord, QuartalIntervalStructure (from 3.5.1)
- `crates/mt/src/quartal/voicing.rs` — quartal traversal (from 3.5.2)
- `crates/mt/src/quartal/interval.rs` — conversion functions (from 3.5.1)
- `crates/mt/src/quintal/error.rs` — QuintalError pattern
- `crates/mt/src/quintal/base_space.rs` — BaseSpace, neighbors

## Concept Cards

### Quartal Harmony (Twentieth-Century Harmony)

**Quick Definition:** Chords built by superimposing fourths. Rootless — any member can function as root. Spacing must preserve fourths to maintain quartal identity.

---

### Quartal-Tertian Pivotal Structures (Twentieth-Century Harmony)

**Quick Definition:** Chords with equal numbers of thirds and fourths that function as either tertian or quartal sonorities. Serve as pivot points between the two harmonic systems. Voicing determines perceived identity.

---

### Twelve-Note Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** Extended quartal formations stacking all twelve pitch classes in fourths. The 12-note quartal chord contains all chromatic tones arranged as a chain of perfect and augmented fourths.

---

### Quartal Voice Leading (Twentieth-Century Harmony)

**Quick Definition:** Any chord tone may skip a fourth or seventh if others remain stationary. Guided by melodic purpose, not codified rules. Florid voice addition enables greater harmonic freedom.

---

### Quartal Voicings (A Geometry of Music)

**Quick Definition:** Jazz quartal voicings built from stacks of perfect fourths. Make tritone substitution logic transparent. Associated with McCoy Tyner and post-1950s jazz.

---

## Mathematical Context

**Quartal stacking direction:** Quartal chords are conventionally described as stacking fourths *upward*: C-F-Bb-Eb is "C with three perfect fourths stacked above." This is the reverse of the quintal convention (stacking fifths upward: C-G-D-A).

**Quartal root vs quintal root:** A quartal stack C-F-Bb-Eb has quartal root C (bottom of the fourth-stack). The same chord read as quintal is Eb-Bb-F-C with quintal root Eb. The pc set {0,3,5,10} is the same either way. "Root" is a property of the reading convention.

**Persichetti's observation:** "Quartal chords are extremely useful in their inverted forms." This is verified by our framework: the inversion cycle produces voicings with different acoustic character but identical pitch-class content. For Class B orbits, 2 of 4 inversions remain in [4,6].

**Key identity:** `pure_quartal_stack(n)` and `pure_quintal_stack((n + 3*5) % 12)` describe the same pitch-class set. The quartal root is related to the quintal root by the total span of three P4s = 15 semitones = 3 mod 12.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides (03-error-handling, 11-anti-patterns)
3. Read `crates/mt/src/quintal/error.rs` for error pattern
4. Read `crates/mt/src/quartal/` for existing quartal types (from 3.5.1, 3.5.2)
5. Create the module files
6. Implement all types and functions
7. Write all specified tests (especially the exhaustive duality tests)
8. Run `cargo test`, `cargo clippy`, `cargo fmt`

## Verification Values

- `pure_quartal_stack(0)` = **{0, 3, 5, 10}** (C-Eb-F-Bb as stacked P4s)
- `pure_quartal_stack(9)` = **{0, 2, 7, 9}** (same as quintal C-G-D-A!)
- `from_stacked_fourths(9, &[5,5,5])` = **[0, 2, 7, 9]** sorted
- `from_stacked_fourths_voiced(57, &[5,5,5])` = **[57, 62, 67, 72]** (A3-D4-G4-C5)
- Every `pure_quartal_stack(n)` for n in 0..12 is **in the BaseSpace** (all 12 are legal)
- **All 228 chords** satisfy both [6,8] quintal and [4,6] quartal constraints
- **6 saddle chords** identified identically from both perspectives
- Quartal root 0 -> quintal root **3** (offset by 3*5 mod 12 = 3)
- Distance between any two chords is **identical** via quartal or quintal paths
