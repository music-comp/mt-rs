# Milestone 3.5.1: Quartal Types & Interval Vocabulary

## Preparation

Before implementing, brush up on expert Rust skills:

- Read `~/lab/oxur/ai-rust-skill/skills/claude/SKILL.md`
- Read `~/lab/oxur/ai-rust-skill/guides/*` (especially 05-type-design for newtypes, 11-anti-patterns for Deref abuse)

## Detailed Plan

Define quartal-specific types, the [4,6] interval constraint, and quartal-native constructors. The quartal module is a thin reframing layer over the quintal module — it does NOT recompute the base space or duplicate any heavy logic.

### Files to create

- `crates/mt/src/quartal/mod.rs` — module root, re-exports (~20 lines)
- `crates/mt/src/quartal/types.rs` — quartal-specific types (~120 lines)
- `crates/mt/src/quartal/interval.rs` — [4,6] constraint, interval conversion (~80 lines)
- `crates/mt/src/quartal/conversion.rs` — quartal/quintal conversion functions (~80 lines)

### Files to modify

- `crates/mt/src/lib.rs` — add `pub mod quartal;`
- `crates/mt/tests/tests.rs` — add `mod quartal;`

### Types (`types.rs`)

**`QuartalIntervalStructure`** — 3-tuple of quartal intervals:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuartalIntervalStructure(pub u8, pub u8, pub u8);
```

Methods:

- `new(a, b, c) -> Self`
- `is_legal() -> bool` — each component in {4, 5, 6}
- `intervals() -> [u8; 3]`

**`QuartalVoicedChord`** — newtype over `VoicedChord` with quartal semantics:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuartalVoicedChord(pub crate::quintal::VoicedChord);
```

Methods:

- `new(pitches: [u8; 4]) -> Result<Self, QuartalError>` — delegates to VoicedChord::new
- `pitches(&self) -> [u8; 4]`
- `quartal_interval_structure(&self) -> QuartalIntervalStructure` — reads intervals top-to-bottom: (p[3]-p[2], p[2]-p[1], p[1]-p[0])
- `quintal_interval_structure(&self) -> IntervalStructure` — delegates to inner VoicedChord
- `to_pc_chord(&self) -> Result<PcChord, QuartalError>` — delegates

### Interval functions (`interval.rs`)

```rust
/// Complement: semitones -> (12 - semitones) % 12. This is an involution.
pub fn quintal_to_quartal_interval(semitones: u8) -> u8
pub fn quartal_to_quintal_interval(semitones: u8) -> u8  // same function

/// Check if each interval is in {4, 5, 6}
pub fn is_quartal_legal(is: &QuartalIntervalStructure) -> bool

/// Convert quintal IS (i1,i2,i3) -> quartal (12-i3, 12-i2, 12-i1)
pub fn quintal_to_quartal_structure(qs: &IntervalStructure) -> QuartalIntervalStructure

/// Inverse (same operation — it's an involution)
pub fn quartal_to_quintal_structure(qs: &QuartalIntervalStructure) -> IntervalStructure
```

### Conversion functions (`conversion.rs`)

```rust
/// Reinterpret a quintal VoicedChord as quartal
pub fn to_quartal(chord: &VoicedChord) -> QuartalVoicedChord

/// Extract the quintal VoicedChord from a quartal wrapper
pub fn to_quintal(chord: &QuartalVoicedChord) -> VoicedChord

/// Read a PcChord's intervals top-to-bottom as quartal
pub fn pc_chord_quartal_intervals(chord: &PcChord) -> Option<QuartalIntervalStructure>
```

### Error type

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuartalError {
    /// Interval not in {4, 5, 6}
    IllegalInterval(u8),
    /// Wrong number of intervals (expected 3 for 4-note chord)
    WrongIntervalCount(usize),
    /// Pitches not in ascending order
    NotAscending,
    /// Duplicate pitch classes
    DuplicatePitchClasses,
}
```

### Test files

- `crates/mt/tests/quartal/mod.rs`
- `crates/mt/tests/quartal/test_types.rs` (~250 lines)

### Test cases

| Test | Description | Expected |
|------|-------------|----------|
| `test_complement_p5_p4` | `quintal_to_quartal_interval(7)` | 5 |
| `test_complement_d5_a4` | `quintal_to_quartal_interval(6)` | 6 (tritone self-dual) |
| `test_complement_a5_d4` | `quintal_to_quartal_interval(8)` | 4 |
| `test_complement_involution` | round-trip for all {4,5,6} | identity |
| `test_structure_777_to_555` | quintal (7,7,7) -> quartal (5,5,5) | true |
| `test_structure_686_to_646` | quintal (6,8,6) -> quartal (6,4,6) | true |
| `test_structure_round_trip` | quintal->quartal->quintal | identity |
| `test_is_quartal_legal_555` | (5,5,5) legal | true |
| `test_is_quartal_legal_456` | (4,5,6) legal | true |
| `test_is_quartal_not_legal` | (3,5,5) not legal | true |
| `test_all_228_quartal_legal` | every chord in B has a quartal-legal reading | true |
| `test_quartal_voiced_cgda` | quartal reading of (48,55,62,69) | (5,5,5) top-to-bottom |
| `test_to_quartal_round_trip` | to_quartal(to_quintal(qc)) == qc | true |
| `test_pc_chord_quartal_intervals` | [0,2,7,9] quartal IS | (5,5,5) |
| `test_quartal_interval_structure_methods` | QuartalIntervalStructure methods | correct |

### Existing modules to read for patterns

- `crates/mt/src/quintal/types.rs` — IntervalStructure, PcChord, VoicedChord (wrapping targets)
- `crates/mt/src/quintal/duality.rs` — quartal_reading, reverse_interval_structure (already implemented!)
- `crates/mt/src/quintal/error.rs` — QuintalError pattern

## Concept Cards

### Quartal Harmony (Twentieth-Century Harmony)

**Quick Definition:** A harmonic system built by superimposing intervals of the fourth, creating a distinctly twentieth-century sound. Rootless — any member can function as root. Spacing must preserve fourths to maintain quartal identity.

---

### Four-Note Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** Quartal structures of four stacked fourths, more resonant than three-note types. P4+P4+P4 spans a minor tenth. Three inverted forms each with different intervallic content. Moving through inversions = harmonic movement without root change.

---

### Three-Note Quartal Chords (Twentieth-Century Harmony)

**Quick Definition:** The basic quartal formation: two superimposed fourths. Types include P4+P4 (most consonant), P4+A4, A4+P4, A4+A4 (most dissonant). Inversions introduce fifths but preserve quartal identity.

---

### Quartal Voicings (A Geometry of Music)

**Quick Definition:** Chord voicings built from stacks of perfect fourths rather than thirds, creating an "open" modern sound. Quartal voicings make tritone substitution logic transparent.

---

### Transposition Symmetry (A Geometry of Music)

**Quick Definition:** The T symmetry groups musical objects related by uniform transposition, defining "chord type." Geometrically: rotation on the pitch-class circle.

---

### Inversion Symmetry (A Geometry of Music)

**Quick Definition:** The I symmetry groups objects related by pitch-space reflection. Adding I to OPTC gives OPTIC (set classes). Major and minor triads are the canonical example.

---

### Set Class (A Geometry of Music)

**Quick Definition:** Groups all chords related by any OPTIC symmetry including transposition and inversion. Members share arc-length sequences on the pitch-class circle.

---

## Mathematical Context

**Quartal/Quintal interval duality:** Every interval has a complement mod 12. P5 (7) complements P4 (5): 7+5=12. A5 (8) complements d4 (4): 8+4=12. The tritone (6) is self-complementary: 6+6=12.

**Reading direction:** A quintal chord C-G-D-A read bottom-to-top has intervals (7,7,7) — three P5s. The same chord read top-to-bottom has intervals (5,5,5) — three P4s. Same pitch classes, dual vocabularies.

**The [4,6] constraint:** The quartal analogue of [6,8]. An interval is "quartal-legal" if it is 4, 5, or 6 semitones (d4, P4, A4/tritone). For each quintal interval i in {6,7,8}, the quartal complement (12-i) is in {6,5,4} = {4,5,6}.

**Interval structure reversal:** If a pc chord has quintal IS (i1,i2,i3) bottom-to-top, its quartal IS top-to-bottom is (12-i3, 12-i2, 12-i1). Reversal = reading direction change; complementation = fourths-vs-fifths change.

## Implementation Instructions

1. Read CLAUDE.md for project conventions
2. Read SKILL.md and linked Rust guides (05-type-design for newtypes, 11-anti-patterns)
3. Read `crates/mt/src/quintal/types.rs` for the types being wrapped
4. Read `crates/mt/src/quintal/duality.rs` — quartal_reading already exists there!
5. Create the module files specified in the plan
6. Implement all types and functions
7. Write all specified tests
8. Run `cargo test` -- fix any failures
9. Run `cargo clippy` -- fix any warnings
10. Run `cargo fmt` -- ensure formatting

## Verification Values

- `quintal_to_quartal_interval(7)` == **5** (P5 -> P4)
- `quintal_to_quartal_interval(6)` == **6** (tritone self-dual)
- `quintal_to_quartal_interval(8)` == **4** (A5 -> d4)
- Quintal (7,7,7) -> Quartal **(5,5,5)**
- Quintal (6,8,6) -> Quartal **(6,4,6)**
- **All 228 chords** satisfy both [6,8] (quintal) and [4,6] (quartal) when read in the appropriate direction
- Structure conversion is an **involution** (round-trips to identity)
- C-G-D-A quartal reading: **(5,5,5)** top-to-bottom
