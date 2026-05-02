# Phase 1 — `harmonize_melody` Reference Sketch: Implementation Plan

## Context

Phase 0 confirmed 2 T1 orbits per PcChord (8 voicings). Phase 0.5 delivered `quintal_root`, `quartal_root`, and `min_voiced_chord_l1`. Phase 1 produces the reference implementation of `harmonize_melody` — the function that takes a melody and returns the top-K voice-led OTH chord progressions ranked by least total semitone movement.

---

## Deliverables

### New source files
1. `crates/mt/src/harmonize/mod.rs` — public types, `harmonize_melody` entry point
2. `crates/mt/src/harmonize/canonicalize.rs` — melody canonicalization (Step 1)
3. `crates/mt/src/harmonize/candidates.rs` — candidate enumeration (Step 2)
4. `crates/mt/src/harmonize/viterbi.rs` — top-K Viterbi DP (Step 3)

### Modified source files
5. `crates/mt/src/lib.rs` — add `pub mod harmonize;`
6. `crates/mt/Cargo.toml` — add `thiserror = "2"` dependency

### New test files
7. `crates/mt/tests/harmonize/mod.rs` — test module declaration
8. `crates/mt/tests/harmonize/test_harmonize.rs` — 11 tests per design doc

### Modified test files
9. `crates/mt/tests/tests.rs` — add `mod harmonize;`

---

## Implementation Details

### Step 1: Add `thiserror` dependency

**File:** `crates/mt/Cargo.toml`

```toml
[dependencies]
thiserror = "2"
```

The design doc mandates `thiserror` for library error types (EH-04). Current error types in `quintal` and `quartal` are manual; `HarmonizeError` will be the first to use the derive macro.

---

### Step 2: `crates/mt/src/harmonize/mod.rs` — Public API

The module-level doc, types, and `harmonize_melody` function follow the design doc's API sketch verbatim:

```rust
//! Voice-led OTH chord progressions for melodies.
//!
//! Given a melody (a sequence of pitches or pitch classes), enumerate the
//! top-K voice-led progressions whose top-voice line traces out that melody,
//! drawing from the full fiber bundle E (all 4 inversions per PcChord) in
//! both quintal and quartal voicings.
//!
//! # Architecture
//!
//! 1. **Canonicalize** the melody to target top MIDI pitches (one per position).
//! 2. **Enumerate candidates** for each position: all VoicedChords whose top
//!    voice matches the target pitch class, shifted to the target octave.
//! 3. **Top-K Viterbi** finds the K lowest-cost paths through the layered DAG
//!    where edge weights are [`min_voiced_chord_l1`](crate::voice_leading::min_voiced_chord_l1).
//!
//! See [`crate::quintal::BaseSpace`] for the 228-chord candidate pool,
//! [`crate::quintal::quintal_root`] / [`crate::quartal::quartal_root`] for
//! canonical voicing construction.

mod canonicalize;
mod candidates;
mod viterbi;

use crate::quintal::VoicedChord;

// ... public types and harmonize_melody function ...
```

**Types (exactly per design doc):**
- `MelodyInput` — enum with `PitchClasses(Vec<u8>)` and `Pitches(Vec<u8>)`, `#[non_exhaustive]`
- `DualityScope` — enum (`QuintalOnly`, `QuartalOnly`, `Both`), `#[non_exhaustive]`, `Default = Both`
- `HarmonizeOptions` — struct with `top_voice_offset: i8`, `melody_octave: i8`, `duality: DualityScope`, `k: usize`; `Default` impl per doc
- `Harmonization` — struct with `chords: Vec<VoicedChord>`, `per_step_movements: Vec<u32>`, `total_movement: u32`; `#[cfg_attr(feature = "serde", derive(...))]`
- `HarmonizeError` — `#[derive(Debug, thiserror::Error)]`, `#[non_exhaustive]`, variants: `EmptyMelody`, `InvalidPitchClass(u8)`, `NoCandidatesForPosition { position, top_midi }`, `InsufficientProgressions { requested, available }`

**Main function:**
```rust
pub fn harmonize_melody(
    melody: MelodyInput,
    options: HarmonizeOptions,
) -> Result<Vec<Harmonization>, HarmonizeError>
```

Per design doc recommendation (Option A): return what's available silently when fewer than K paths exist. The `InsufficientProgressions` variant remains defined but is never returned — documented as reserved.

---

### Step 3: `crates/mt/src/harmonize/canonicalize.rs`

```rust
use super::{HarmonizeError, HarmonizeOptions, MelodyInput};

/// Canonicalize a melody to target top MIDI pitches.
pub(super) fn canonicalize_melody(
    melody: &MelodyInput,
    options: &HarmonizeOptions,
) -> Result<Vec<u8>, HarmonizeError> {
    match melody {
        MelodyInput::PitchClasses(pcs) => {
            if pcs.is_empty() {
                return Err(HarmonizeError::EmptyMelody);
            }
            pcs.iter()
                .map(|&pc| {
                    if pc > 11 {
                        return Err(HarmonizeError::InvalidPitchClass(pc));
                    }
                    // melody_octave 5, offset -12 → octave 4 = MIDI 48+pc
                    // General: MIDI = 12 * (melody_octave + 1) + pc + offset
                    let midi = 12i16 * (options.melody_octave as i16 + 1)
                        + pc as i16
                        + options.top_voice_offset as i16;
                    if midi < 0 || midi > 127 {
                        return Err(HarmonizeError::NoCandidatesForPosition {
                            position: 0, // will be refined in caller
                            top_midi: 0,
                        });
                    }
                    Ok(midi as u8)
                })
                .collect()
        }
        MelodyInput::Pitches(pitches) => {
            if pitches.is_empty() {
                return Err(HarmonizeError::EmptyMelody);
            }
            pitches.iter()
                .map(|&p| {
                    let target = p as i16 + options.top_voice_offset as i16;
                    if target < 0 || target > 127 {
                        return Err(HarmonizeError::NoCandidatesForPosition {
                            position: 0,
                            top_midi: p,
                        });
                    }
                    Ok(target as u8)
                })
                .collect()
        }
    }
}
```

---

### Step 4: `crates/mt/src/harmonize/candidates.rs`

```rust
use crate::quintal::{
    inversion_cycle, quintal_root, BaseSpace, VoicedChord,
};
use crate::quartal::{quartal_inversion_cycle, quartal_root};

use super::{DualityScope, HarmonizeError};

/// Enumerate all candidate VoicedChords for a given target top MIDI pitch.
pub(super) fn candidates_for_top(
    target_top_midi: u8,
    duality: DualityScope,
    space: &BaseSpace,
) -> Result<Vec<VoicedChord>, HarmonizeError> {
    let target_pc = target_top_midi % 12;
    let mut candidates = Vec::new();

    for pc_chord in space.chords() {
        if !pc_chord.pcs.contains(&target_pc) {
            continue;
        }

        if matches!(duality, DualityScope::QuintalOnly | DualityScope::Both) {
            if let Ok(q_root) = quintal_root(pc_chord, 0) {
                for inv in inversion_cycle(&q_root) {
                    if inv.pitches[3] % 12 == target_pc {
                        candidates.push(shift_to_top(&inv, target_top_midi));
                        break;
                    }
                }
            }
        }

        if matches!(duality, DualityScope::QuartalOnly | DualityScope::Both) {
            if let Ok(qv_root) = quartal_root(pc_chord, 0) {
                for inv in quartal_inversion_cycle(&qv_root) {
                    if inv.0.pitches[3] % 12 == target_pc {
                        candidates.push(shift_to_top(&inv.0, target_top_midi));
                        break;
                    }
                }
            }
        }
    }

    Ok(candidates)
}

/// Shift a VoicedChord vertically so pitches[3] == target_midi.
fn shift_to_top(chord: &VoicedChord, target_midi: u8) -> VoicedChord {
    let delta = target_midi as i32 - chord.pitches[3] as i32;
    debug_assert!(delta % 12 == 0, "shift_to_top: PC mismatch");
    let shifted = chord.pitches.map(|p| (p as i32 + delta) as u8);
    VoicedChord { pitches: shifted }
}
```

**Key insight:** `shift_to_top` bypasses `VoicedChord::new()` validation because an octave shift of an ascending sequence is still ascending. We directly construct with the struct literal since `pitches` is `pub`. The `debug_assert!` catches misuse in tests.

---

### Step 5: `crates/mt/src/harmonize/viterbi.rs`

The k-best Viterbi DP:

```rust
use crate::quintal::VoicedChord;
use crate::voice_leading::min_voiced_chord_l1;

use super::Harmonization;

/// Internal path state for k-best tracking.
#[derive(Clone)]
struct PathState {
    cost: u32,
    prev_candidate: Option<usize>,
    prev_path: Option<usize>,
}

/// Run the top-K Viterbi algorithm over layered candidate sets.
pub(super) fn top_k_viterbi(
    layers: &[Vec<VoicedChord>],
    k: usize,
) -> Vec<Harmonization> {
    if layers.is_empty() || k == 0 {
        return Vec::new();
    }

    let n = layers.len();

    // dp[layer][candidate] = Vec<PathState>, sorted by cost ascending, max len k
    let mut dp: Vec<Vec<Vec<PathState>>> = Vec::with_capacity(n);

    // Layer 0: each candidate starts with cost 0, no predecessor
    let first_layer: Vec<Vec<PathState>> = layers[0]
        .iter()
        .map(|_| vec![PathState { cost: 0, prev_candidate: None, prev_path: None }])
        .collect();
    dp.push(first_layer);

    // Forward pass: layers 1..n
    for layer_idx in 1..n {
        let mut layer_dp: Vec<Vec<PathState>> = Vec::with_capacity(layers[layer_idx].len());

        for (c_new_idx, c_new) in layers[layer_idx].iter().enumerate() {
            let mut candidates_for_state: Vec<PathState> = Vec::new();

            for (c_prev_idx, c_prev) in layers[layer_idx - 1].iter().enumerate() {
                let edge_cost = min_voiced_chord_l1(c_prev, c_new);

                for (p_idx, p_state) in dp[layer_idx - 1][c_prev_idx].iter().enumerate() {
                    candidates_for_state.push(PathState {
                        cost: p_state.cost + edge_cost,
                        prev_candidate: Some(c_prev_idx),
                        prev_path: Some(p_idx),
                    });
                }
            }

            // Sort by cost, keep only the best k
            candidates_for_state.sort_by_key(|ps| ps.cost);
            candidates_for_state.truncate(k);
            layer_dp.push(candidates_for_state);
        }

        dp.push(layer_dp);
    }

    // Gather the global best K paths from the final layer
    let last = n - 1;
    let mut finals: Vec<(u32, usize, usize)> = Vec::new(); // (cost, candidate_idx, path_idx)
    for (c_idx, paths) in dp[last].iter().enumerate() {
        for (p_idx, ps) in paths.iter().enumerate() {
            finals.push((ps.cost, c_idx, p_idx));
        }
    }
    finals.sort_by_key(|&(cost, _, _)| cost);
    finals.truncate(k);

    // Backtrack to reconstruct each path
    finals.iter().map(|&(total_cost, mut c_idx, mut p_idx)| {
        let mut chord_indices: Vec<usize> = Vec::with_capacity(n);
        chord_indices.push(c_idx);

        for layer_idx in (1..n).rev() {
            let state = &dp[layer_idx][c_idx][p_idx];
            c_idx = state.prev_candidate.unwrap();
            p_idx = state.prev_path.unwrap();
            chord_indices.push(c_idx);
        }
        chord_indices.reverse();

        let chords: Vec<VoicedChord> = chord_indices
            .iter()
            .enumerate()
            .map(|(layer, &ci)| layers[layer][ci])
            .collect();

        let per_step_movements: Vec<u32> = chords
            .windows(2)
            .map(|w| min_voiced_chord_l1(&w[0], &w[1]))
            .collect();

        Harmonization {
            chords,
            per_step_movements,
            total_movement: total_cost,
        }
    }).collect()
}
```

---

### Step 6: Wire `harmonize_melody` together in `mod.rs`

The top-level function:

```rust
pub fn harmonize_melody(
    melody: MelodyInput,
    options: HarmonizeOptions,
) -> Result<Vec<Harmonization>, HarmonizeError> {
    if options.k == 0 {
        return Ok(Vec::new());
    }

    let targets = canonicalize::canonicalize_melody(&melody, &options)?;
    let space = crate::quintal::BaseSpace::new();

    let mut layers: Vec<Vec<VoicedChord>> = Vec::with_capacity(targets.len());
    for (pos, &target) in targets.iter().enumerate() {
        let cands = candidates::candidates_for_top(target, options.duality, &space)?;
        if cands.is_empty() {
            return Err(HarmonizeError::NoCandidatesForPosition {
                position: pos,
                top_midi: target,
            });
        }
        layers.push(cands);
    }

    Ok(viterbi::top_k_viterbi(&layers, options.k))
}
```

---

### Step 7: Register in `lib.rs`

Add `pub mod harmonize;` to `crates/mt/src/lib.rs` (alphabetically between `harmony` and `interval`).

---

### Step 8: Tests

**File:** `crates/mt/tests/harmonize/mod.rs`
```rust
mod test_harmonize;
```

**File:** `crates/mt/tests/harmonize/test_harmonize.rs`

11 tests per design doc:

1. `test_canonicalize_melody_pitches_with_default_offset` — `Pitches(vec![72, 76, 79])` + offset -12 → `[60, 64, 67]`
2. `test_canonicalize_melody_pcs_with_default_offset` — `PitchClasses(vec![0, 4, 7])`, melody_octave=5, offset=-12 → `[60, 64, 67]`
3. `test_candidates_for_c4_with_quintal_only` — assert 76 candidates, all with `pitches[3] == 60`
4. `test_candidates_for_c4_with_both_dualities` — assert 152 candidates
5. `test_top_voice_pinned_for_short_melody` — 3-note melody, every chord in every result has correct top voice
6. `test_total_movement_is_monotonic_in_returned_order` — K=10, `total_movement` values non-decreasing
7. `test_repeated_notes_admit_zero_movement_step` — melody `[C, C]` (same pitch twice), best result has `total_movement == 0`
8. `test_quartal_only_excludes_quintal_voicings` — 1-note melody with `QuartalOnly`, all candidates have quartal-range raw intervals
9. `test_empty_melody_errors` — `EmptyMelody`
10. `test_invalid_pitch_class_errors` — `PitchClasses(vec![13])` → `InvalidPitchClass(13)`
11. `test_default_options_match_documented_defaults` — sanity check on `Default` impl values

**Register in `tests/tests.rs`:** add `mod harmonize;`

---

### Step 9: Doctest on `harmonize_melody`

The docstring example in the design doc is the doctest. It exercises `PitchClasses(vec![0, 4, 7])` with `HarmonizeOptions::default()`.

---

## Implementation Order

1. Add `thiserror = "2"` to `Cargo.toml`
2. Create `crates/mt/src/harmonize/mod.rs` with all public types and `harmonize_melody`
3. Create `crates/mt/src/harmonize/canonicalize.rs`
4. Create `crates/mt/src/harmonize/candidates.rs`
5. Create `crates/mt/src/harmonize/viterbi.rs`
6. Add `pub mod harmonize;` to `lib.rs`
7. `cargo check` — verify compilation
8. Create test files and register
9. `cargo test` — all tests pass
10. `cargo clippy -- -D warnings` — lint clean
11. `cargo doc --no-deps` — docs build, no broken links

---

## Design Decisions (resolved per doc recommendations)

| Question | Resolution |
|----------|-----------|
| Module path | `crates/mt/src/harmonize/mod.rs` (top-level) |
| `InsufficientProgressions` | Keep variant defined, never returned (Option A — return what's available) |
| Single-note sort | Insertion order from `candidates_for_top` |
| `melody_octave` default | 5 (target top lands at octave 4 after −12 offset) |
| `shift_to_top` visibility | Private helper (`fn`, not `pub`) |
| Performance | No caching; trivial at expected scale |

---

## Verification

```bash
cargo test                          # all tests pass (580+ expected)
cargo clippy -- -D warnings         # lint clean
cargo doc --no-deps                 # docs build, no broken intra-doc links

# Specific new tests
cargo test harmonize
cargo test test_candidates_for_c4
cargo test test_total_movement
```

---

## Acceptance Criteria (from design doc)

- All public types and `harmonize_melody` compile with full rustdoc including `# Errors` and `# Examples`
- All 11 tests pass
- Doctest passes
- `cargo clippy -- -D warnings` clean
- `cargo doc --no-deps` produces complete docs for the `harmonize` module with no broken intra-doc links
- Reference sketch ready for Duncan's review before Phase 2
