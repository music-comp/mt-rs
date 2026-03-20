# Tier 2 Music Theory Enhancements — Design Spec

**Date**: 2026-03-20
**Status**: Approved
**Depends on**: Tier 1 enhancements (completed)

## Goal

Add 6 comprehensive music theory computation features: common tones, pivot chords, chord-scale compatibility, voice leading, compound intervals, and secondary dominant detection. These complete the library's coverage of standard undergraduate harmony curriculum.

## Features

### 1. Common Tones

**Module**: `src/harmony/common_tones.rs`
**Public API**:
```rust
pub fn common_tones(a: &[Pitch], b: &[Pitch]) -> Vec<Pitch>
```

**Behavior**: Returns pitch classes present in both inputs. Compares by `as_u8()` (enharmonic equivalence). Returns pitches using the spelling from the first argument. Order follows the first argument's order.

**Example**: `common_tones([C, E, G], [C, F, A])` returns `[C]`.

### 2. Pivot Chords

**Module**: `src/harmony/pivot.rs`
**Public API**:
```rust
pub struct PivotChord {
    pub chord: Chord,
    pub roman_in_a: RomanNumeral,
    pub roman_in_b: RomanNumeral,
}

pub fn pivot_chords(
    key_a_tonic: Pitch, key_a_mode: Mode,
    key_b_tonic: Pitch, key_b_mode: Mode,
) -> Vec<PivotChord>
```

**Behavior**: Generates diatonic triads for both keys. For each chord in key A, checks if any chord in key B has the same pitch-class set. Returns matches with Roman numeral labels in both keys.

**Example**: `pivot_chords(C, Ionian, G, Ionian)` returns chords like G major (V in C, I in G), D minor (ii in C, — wait, not diatonic to G with same quality), etc.

### 3. Chord-Scale Compatibility

**Module**: `src/harmony/compatibility.rs`
**Public API**:
```rust
pub fn compatible_scales(chord: &Chord) -> Vec<(Pitch, Mode)>
```

**Behavior**: Finds all scales whose pitch-class set contains all chord tones. Reuses `Scale::identify()` logic internally — extracts pitch classes from `chord.notes()` and delegates.

**Example**: `compatible_scales(Cmaj7)` returns C Ionian, C Lydian, G Mixolydian, etc.

### 4. Voice Leading

**Module**: `src/voice_leading/mod.rs` (new module)
**Public API**:
```rust
pub struct VoiceMovement {
    pub from: Note,
    pub to: Note,
    pub semitones: i8,  // positive=up, negative=down
}

pub struct VoiceLeading {
    pub movements: Vec<VoiceMovement>,
    pub total_distance: u8,  // sum of absolute semitone movements
}

pub fn minimal_movement(from: &[Note], to: &[Note]) -> VoiceLeading
```

**Behavior**: For two chord voicings of equal size (up to 4 voices), finds the voice assignment that minimizes total semitone movement. Uses brute-force permutation for correctness (max 4! = 24 permutations). If chord sizes differ, pads the smaller with unison movements.

**Distance metric**: Sum of absolute semitone differences between paired voices (taxicab/L1 distance in pitch space).

**Example**: C major [C4, E4, G4] to E minor [E4, G4, B4] — optimal: C4→B3(-1), E4→E4(0), G4→G4(0) = total 1 semitone.

### 5. Compound Intervals

**Module**: Modify `src/interval/interval.rs`
**Changes**:
- Extend `Number` enum with: `Ninth`, `Tenth`, `Eleventh`, `Twelfth`, `Thirteenth`, `Fourteenth`, `Fifteenth`
- Extend `from_semitone()` to handle 13-24 (compound = simple + octave)
- Add `Interval::is_compound(&self) -> bool`
- Add `Interval::simple(&self) -> Interval` (strip octave component)

**Mapping** (semitones 13-24):
| Semitones | Quality | Number |
|-----------|---------|--------|
| 13 | Minor | Ninth |
| 14 | Major | Ninth |
| 15 | Minor | Tenth |
| 16 | Major | Tenth |
| 17 | Perfect | Eleventh |
| 18 | Augmented | Eleventh |
| 19 | Perfect | Twelfth |
| 20 | Minor | Thirteenth |
| 21 | Major | Thirteenth |
| 22 | Minor | Fourteenth |
| 23 | Major | Fourteenth |
| 24 | Perfect | Fifteenth |

### 6. Secondary Dominant Detection

**Module**: `src/analysis/secondary.rs`
**Public API**:
```rust
pub struct SecondaryDominant {
    pub label: String,         // e.g., "V/vi", "V7/IV"
    pub target_degree: u8,     // the scale degree being tonicized
    pub target_chord: DiatonicChord,
}

pub fn secondary_dominant(
    key_tonic: Pitch,
    key_mode: Mode,
    chord: &Chord,
) -> Option<SecondaryDominant>
```

**Behavior**: A chord is a secondary dominant (V/x) if:
1. It has Major or Dominant quality
2. Its root is a perfect 5th (7 semitones) above a diatonic chord root
3. The target chord is NOT the tonic (V/I is just V, not a secondary dominant)

Returns None if the chord doesn't function as a secondary dominant. Also detects vii/x (diminished chord a semitone below a diatonic chord root).

**Example**: In C major, D major chord → V/V (root D is P5 above G, which is degree V).

## Architecture

All features extend existing modules:
- `src/harmony/` gains 3 new files (common_tones.rs, pivot.rs, compatibility.rs)
- `src/voice_leading/` is a new module (single file for now)
- `src/interval/interval.rs` gains compound interval support
- `src/analysis/` gains secondary.rs

No new dependencies. All types derive serde behind the feature flag.

## Testing

Each feature gets its own test file in the corresponding `tests/` directory. Tests verify:
- Happy paths with musically known-correct values
- Edge cases (empty inputs, enharmonic equivalents, extreme keys)
- All 6 features tested independently

## Success Criteria

- All existing 166 tests continue to pass
- Each new feature has 4+ tests
- `cargo clippy` clean (4 expected module_inception warnings)
- Serde feature still compiles and round-trips
