# Tier 3 Advanced Music Theory — Design Spec

**Date**: 2026-03-20
**Status**: Approved
**Depends on**: Tier 1 + Tier 2 enhancements (completed)

## Goal

Add 4 advanced music theory modules: neo-Riemannian transformations, pitch-class set theory, first-species counterpoint checking, and figured bass realization. These cover graduate-level theory topics and complete the library's academic coverage.

## Features

### 1. Neo-Riemannian Transformations

**Module**: `src/neo_riemannian/mod.rs`

Three operations on major/minor triads, each moving one voice by minimal distance:

| Op | Major → | Minor → |
|----|---------|---------|
| P (Parallel) | Lower 3rd by 1 (C-E-G → C-Eb-G) | Raise 3rd by 1 (C-Eb-G → C-E-G) |
| R (Relative) | Lower 5th by 2 (C-E-G → C-E-A = A-C-E) | Raise root by 2 (A-C-E → C-E-G... actually G-C-E? No: raise root A→B? No.) |
| L (Leading-tone) | Lower root by 1 (C-E-G → B-E-G = E-G-B) | Raise 5th by 1 (E-G-B → E-G-C = C-E-G) |

More precisely, using pitch-class arithmetic:
- **P**: For major triad {0,4,7} on root r: flip 3rd (r+4 → r+3). For minor {0,3,7}: flip 3rd (r+3 → r+4).
- **R**: For major {0,4,7}: move 5th down 2 (r+7 → r+9 mod 12, re-root). For minor {0,3,7}: move root up 2.
- **L**: For major {0,4,7}: move root down 1 (r → r-1, re-root). For minor {0,3,7}: move 5th up 1.

**API**:
```rust
pub enum NROperation { P, R, L }

pub fn transform(chord: &Chord, op: NROperation) -> Result<Chord, NRError>
pub fn transform_chain(chord: &Chord, ops: &[NROperation]) -> Result<Vec<Chord>, NRError>
```

`transform_chain` returns each intermediate chord (for showing the path).

Only works on major/minor triads. Returns error for other chord types.

### 2. Pitch-Class Set Theory

**Module**: `src/set_class/mod.rs` + `src/set_class/forte.rs`

**Core type**: `PitchClassSet` wrapping `BTreeSet<u8>` (values 0-11).

**Operations**:
- `PitchClassSet::new(pitches)` — construct from pitches or integers
- `transpose(&self, n: u8) -> PitchClassSet` — T_n: add n mod 12 to each pc
- `invert(&self, n: u8) -> PitchClassSet` — I_n: (n - pc) mod 12 for each pc
- `normal_form(&self) -> Vec<u8>` — most compact rotation
- `prime_form(&self) -> Vec<u8>` — lexicographically smallest of all T_n and I_n normal forms
- `forte_number(&self) -> Option<String>` — lookup in Forte table (e.g., "3-11")
- `interval_vector(&self) -> [u8; 6]` — count of each interval class ic1-ic6

**Forte table** (`forte.rs`): Static array of all 224 prime forms mapped to Forte numbers. Only needs cardinalities 3-9 (smaller and larger are trivial or complements).

### 3. First-Species Counterpoint

**Module**: `src/counterpoint/mod.rs`

Check two melodic lines against first-species (note-against-note) counterpoint rules.

**Rules checked**:
1. All vertical intervals must be consonant (unison, 3rd, 5th, 6th, 8ve, 10th)
2. No parallel perfect consonances (parallel 5ths, parallel octaves, parallel unisons)
3. No direct/hidden 5ths or octaves (both voices move in same direction to a perfect interval)
4. First interval must be a perfect consonance (unison, 5th, or octave)
5. Last interval must be a perfect consonance
6. No voice crossing (upper voice below lower voice)

**API**:
```rust
pub struct Violation {
    pub position: usize,     // which beat (0-indexed)
    pub rule: &'static str,  // rule identifier
    pub description: String,  // human-readable explanation
}

pub struct CounterpointResult {
    pub valid: bool,
    pub violations: Vec<Violation>,
}

pub fn check_first_species(
    cantus_firmus: &[Note],
    counterpoint: &[Note],
) -> CounterpointResult
```

Both voices must have the same length. Returns all violations found (not just the first).

### 4. Figured Bass Realization

**Module**: `src/figured_bass/mod.rs`

Parse figured bass symbols and produce chord voicings above a bass line.

**Figure types**:
- Empty = root position triad (implied 5/3)
- `6` = first inversion triad (6/3)
- `64` = second inversion triad (6/4)
- `7` = root position seventh (7/5/3)
- `65` = first inversion seventh (6/5/3)
- `43` = second inversion seventh (4/3)
- `42` or `2` = third inversion seventh (4/2)
- Accidentals: `#6`, `b3`, `n7` modify individual intervals

**API**:
```rust
pub struct Figure {
    pub interval: u8,        // interval above bass (e.g., 3, 5, 6, 7)
    pub accidental: i8,      // 0=as-in-key, 1=sharp, -1=flat
}

pub struct Realization {
    pub bass: Note,
    pub upper_voices: Vec<Note>,
}

pub fn realize(
    bass_notes: &[Note],
    figures: &[Vec<Figure>],
    key_tonic: Pitch,
    key_mode: Mode,
) -> Vec<Realization>
```

Figured bass realization uses the key signature to determine default accidentals. Each figure specifies an interval above the bass; the realization places those intervals in close voicing above the bass.

## Architecture

4 new top-level modules:
```
src/neo_riemannian/mod.rs
src/set_class/mod.rs
src/set_class/forte.rs
src/counterpoint/mod.rs
src/figured_bass/mod.rs
```

All new types get serde derives behind the feature flag. No new dependencies.

## Testing

Each module gets its own test file. Tests use musically known-correct values:
- Neo-Riemannian: verified P/R/L on C major, A minor, chains like PRL
- Set theory: verified against Forte's published tables (e.g., {0,4,7} = 3-11, major triad)
- Counterpoint: Fux-style examples with known violations
- Figured bass: standard Baroque patterns with known realizations

## Success Criteria

- All existing 204 tests continue to pass
- Each new module has 5+ tests
- `cargo clippy` clean
- Serde feature still compiles
