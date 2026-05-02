---
number: 7
title: "Melody Harmonization Feature"
author: "least total"
component: All
tags: [change-me]
created: 2026-05-01
updated: 2026-05-01
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# Melody Harmonization Feature

Draft date: 2026-05-01
Target crates: `music-comp-mt` (mt-rs), `music-theory-mcp` (ai-music-theory)
Rust edition: 2024 / toolchain 1.85+
Owners: Duncan & Claude

## Context and goal

The feature lets a user supply a melody and receive the top-K voice-led OTH chord progressions whose top voices trace out that melody, ranked by least total semitone movement. Candidate chords are drawn from the full fiber bundle E over B (all 4 inversions of every PcChord), in both quintal and quartal voicings, with the top voice pinned to a configurable offset (default −12 semitones) below each melody note.

This plan covers the first three of five phases; Phases 2 and 3 (production implementation, MCP wiring, end-to-end verification) are tracked separately and depend on Phase 1 review approval.

The phases are deliberately ordered so each builds confidence for the next:

- **Phase 0** settles a conceptual question about fiber-bundle structure that affects how candidate enumeration is written.
- **Phase 0.5** lifts symmetric quintal/quartal vocabulary into the public API surface so Phase 1 reads as one sentence in the implementation.
- **Phase 1** produces a reviewable reference sketch of `harmonize_melody` before any production code lands.

---

## Phase 0 — Empirical fiber-orbit verification

### Goal

Confirm or falsify the working hypothesis that **every legal PcChord has exactly two T1 orbits among its OTH-relevant voicings: one quintal (rooted at the {6,7,8} stacking) and one quartal (rooted at the {4,5,6} stacking) — totalling 8 distinct VoicedChords per PcChord modulo octave.**

The hypothesis was developed by working through three specific examples on paper:

- {0, 2, 7, 9} with palindromic interval structure (7,7,7): one quintal-legal stacking + one quartal-legal stacking, in two disjoint cycles. 2 orbits, 8 voicings.
- {0, 1, 6, 9} with non-palindromic (6,7,8): same structure. 2 orbits, 8 voicings.
- {0, 2, 6, 10} with (6,8,8) and chord-scale-symmetric step pattern (2,4,4,2): two distinct quintal-legal stackings *and* two distinct quartal-legal stackings exist, but they pair up into the same two T1 cycles (T1^0 and T1^1 of each cycle are both legal under the respective constraint). 2 orbits, 8 voicings.

The mechanism behind the third case is that T1 advances every voice by one chord-scale step, equivalent to rotating the stacking's "skip pattern" forward by one chord-scale index. When two distinct chord-scale starting indices both produce a legal stacking under the same skip pattern, the cycle contains two legal stackings as adjacent inversions.

We need to confirm this generalizes across all 228 PcChords. If a chord exists where the legal stackings span 3 or 4 disjoint orbits, the candidate enumeration in Phase 1 must change.

### Test location and shape

New integration test file:

```
crates/mt/tests/quintal/test_fiber_orbits.rs
```

Registered in `crates/mt/tests/quintal/mod.rs` (already declared as a submodule of `crates/mt/tests/tests.rs`).

### Algorithm

For each of the 228 PcChords in `quintal::base_space::enumerate_all()`:

1. Brute-force the 24 PC permutations.
2. For each permutation, compute the 3 forward intervals (mod 12) and classify:
   - `Quintal` if all in {6, 7, 8}
   - `Quartal` if all in {4, 5, 6}
   - `Neither` otherwise
3. For each `Quintal` and `Quartal` stacking, build the corresponding `VoicedChord` at a canonical low octave (e.g. start MIDI 12 = C0 for the bottom voice).
4. For each such `VoicedChord`, compute its T1 inversion cycle (4 entries) and store the canonical pitch-class-set fingerprint of each entry.
5. Group the legal stackings by which T1 orbit they belong to (using a `BTreeMap<OrbitFingerprint, Vec<Stacking>>`, where the orbit fingerprint is the sorted set of the 4 cycle members reduced to PC-set + relative-pitch signature).
6. Accumulate per-chord summary: `(num_quintal_orbits, num_quartal_orbits, num_legal_quintal_stackings, num_legal_quartal_stackings)`.

### Acceptance criteria

The test asserts, for every one of the 228 PcChords:

- `num_quintal_orbits == 1`
- `num_quartal_orbits == 1`
- The quintal orbit and the quartal orbit are disjoint (no shared cycle entry modulo octave)
- The total number of distinct VoicedChords across both orbits, modulo octave, equals 8

It also reports (via `eprintln!` so it appears in `cargo test -- --nocapture`) the distribution of `num_legal_quintal_stackings` across chords (we expect this to be 1 or 2 per chord; values higher than 2 would be surprising and worth investigating).

### What if the hypothesis is falsified

If any chord produces 3+ orbits or fewer than 8 total voicings, halt and discuss before continuing. The candidate-enumeration design in Phase 1 assumes exactly 2 orbits per chord; a different result reshapes that step. Specifically:

- More than 2 orbits → the harmonize candidate pool grows, and we need to decide which orbits are "OTH-relevant" (the ones with at least one legal stacking) versus "fiber-but-non-OTH" (e.g. close-position cycles that aren't anyone's idea of a quintal/quartal voicing).
- Fewer than 8 voicings → some chord has a degenerate fiber where quintal and quartal voicings coincide at some inversion. Investigate before proceeding.

### Test-design notes

- This is a pure-computation test, no I/O, no fixtures. Should run in well under a second.
- Use `BaseSpace::new()` (cached internally, but instantiate once for the test).
- Re-use the existing `PERMUTATIONS_4` constant from `quintal::types` if exposed; otherwise duplicate locally with a comment pointing at the original.
- Do not depend on the Phase 0.5 API additions — Phase 0 must be runnable against the current codebase.

### Deliverable

A single test file plus a one-line update to the test module declaration. Commit with a message referencing `[6,8]` framework documentation: e.g. `test(quintal): verify exactly two T1 orbits per PcChord (1 quintal + 1 quartal)`.

---

## Phase 0.5 — Symmetric quintal/quartal API surface

### Goal

Expose a parallel, mirror-image API for quintal and quartal perspectives, so that Phase 1 (and any future feature) can write code that treats the two perspectives symmetrically rather than reaching into asymmetric internal helpers.

The current state has asymmetries:

- `quintal::base_space::BaseSpace` exists; there is no `quartal::base_space`.
- `quartal::from_stacked_fourths_voiced(root_pc, intervals)` builds a `QuartalVoicedChord` from stacking parameters; there is no `quintal::from_stacked_fifths_voiced` parallel.
- Neither side has a `xxx_root(pc_chord, octave) -> VoicedChord` constructor that goes from a `PcChord` (an abstract pitch-class set) directly to a canonical voiced root in either perspective. Today the path is `pc_chord.interval_structure()` → manual stacking, which leaks pre-API ceremony into every caller.

### API additions

#### `quintal::quintal_root(pc_chord, base_octave) -> Result<VoicedChord, QuintalError>`

Builds the canonical quintal-root `VoicedChord` for a given `PcChord`, with the bottom voice at MIDI `12 * base_octave + first_pc_in_legal_stacking`.

Implementation:

1. Call `pc_chord.interval_structure()` to get the (first) legal quintal interval triple `(i1, i2, i3)`.
2. Determine the bottom PC of the legal stacking by finding which permutation produced that interval triple. (Either expose this from `PcChord` as `legal_quintal_stacking_permutation()` or recompute inline via brute force — recommend exposing it.)
3. Stack from `bottom_midi = 12 * base_octave + bottom_pc`: `[bottom, bottom+i1, bottom+i1+i2, bottom+i1+i2+i3]`.
4. Construct via `VoicedChord::new(pitches)`.

Errors:

- `PcChord` admits no legal quintal stacking (cannot occur for chords from `BaseSpace`, but the public API should still return `Result` rather than panic).
- Resulting MIDI pitches exceed the u8 range (only relevant for absurd `base_octave` values; document the safe range).

#### `quartal::quartal_root(pc_chord, base_octave) -> Result<QuartalVoicedChord, QuartalError>`

Mirror of the above. Builds the canonical quartal-root `QuartalVoicedChord`.

Implementation: derive the quartal interval triple by complementing-and-reversing the quintal interval structure (i.e. `(12 - i3, 12 - i2, 12 - i1)`), and the quartal bottom PC is the quintal *top* PC of the legal stacking. (This was confirmed in the design discussion as a generalization that holds for all cases.) Then call the existing `from_stacked_fourths_voiced`.

#### `quartal::base_space() -> BaseSpace`

Re-export of `quintal::BaseSpace::new()` (the underlying base space is the same — the 228 PcChords are the same set; only the perspective changes). Add as a thin alias for symmetric naming; document explicitly that "the base space is shared, only the voicing perspective differs."

Optional: if the team prefers a stronger conceptual separation, expose a `quartal::BaseSpace` *newtype* that wraps the quintal one. This is more ceremony but makes the symmetric naming complete. Recommend the alias approach for now; promote to a newtype if downstream API users find the bare alias confusing.

#### `quintal::min_voice_leading_l1(a, b) -> u32` and `quartal::min_voice_leading_l1(a, b) -> u32`

The current `quintal::fiber::l1_distance(a, b)` computes the *positional* L1 distance — voices matched by stored array position, not by optimal assignment. This is correct for the Universal L1 Law verification (which depends on positional identity through the fiber) but not for general voice-leading distance.

`voice_leading::minimal_movement(from, to) -> VoiceLeading` already provides the assignment-optimal version on `&[Note]`. The harmonize feature wants this on `&VoicedChord` directly, without the `Note` round-trip. Add a thin helper that does the 24-permutation brute force on `[u8; 4]` arrays. Document the difference between this and the positional `l1_distance` clearly to avoid future confusion.

This helper could live in `voice_leading::` as a `VoicedChord`-specialized variant, or alongside `quintal::fiber::l1_distance`. Recommend `voice_leading::min_voiced_chord_l1` as the canonical home, with re-exports from both `quintal` and `quartal` for symmetric naming.

### Docstring fixes

The doc comment in `crates/mt/src/quartal/voicing.rs` claims:

> "These are the same 4 voiced chords as the quintal cycle but in reverse order: quartal `[inv0, inv1, inv2, inv3]` = quintal `[inv0, inv3, inv2, inv1]`."

This is correct only when the input chord is the same starting voicing — i.e. quintal and quartal cycles starting from the *same* `VoicedChord` are the same 4 chords because `t_minus1` is the inverse of `t1`. It is *not* a statement about the relationship between the quintal-rooted cycle and the quartal-rooted cycle of the same `PcChord` (which the worked example {0, 2, 7, 9} shows are 8 distinct voicings).

Tighten the doc comment to make the "starting from the same chord" qualifier explicit, and add a separate note (cross-referenced from `quartal::quartal_root`) that the *quintal-rooted* and *quartal-rooted* cycles of a `PcChord` are disjoint.

### Tests

Add tests in `crates/mt/tests/quintal/test_constructors.rs` and `crates/mt/tests/quartal/test_constructors.rs`:

- `quintal_root` produces a `VoicedChord` whose interval structure matches `pc_chord.interval_structure()`.
- `quartal_root` produces a `QuartalVoicedChord` whose interval structure is the reverse-complement of the quintal interval structure.
- `quintal_root(pc_chord, octave_a).pitches[3] - 12 * (octave_a - octave_b) == quintal_root(pc_chord, octave_b).pitches[3]` — confirms the octave parameter does what it says.
- `min_voice_leading_l1` agrees with `voice_leading::minimal_movement(...).total_distance` after `Note` conversion (round-trip equivalence).
- Round-trip through `quintal_root` → `to_pc_chord` returns the same `PcChord`.

### Acceptance criteria

- New API symbols exported from `quintal::` and `quartal::` modules and visible through `music_comp_mt::quintal::` / `music_comp_mt::quartal::`.
- Existing tests still pass (`cargo test`).
- New tests pass.
- `cargo clippy -- -D warnings` is clean (the project's existing 4 `module_inception` warnings are expected and tolerated; do not introduce new ones).
- Updated rustdoc renders correctly: `cargo doc --no-deps --open` and visually verify the "Module quintal" and "Module quartal" pages now have parallel structure.

### Risks

- The "alias vs newtype" choice for `quartal::base_space` carries some downstream cost if changed later. Land it as an alias initially and document the choice; revisit if API consumers find it confusing.
- The `legal_quintal_stacking_permutation()` exposure on `PcChord` is the riskiest API addition because it's a fundamental piece of internal machinery. Consider whether it should be `pub(crate)` for now and only promoted to `pub` when a downstream caller actually needs it. Recommend `pub(crate)` initially.

---

## Phase 1 — `harmonize_melody` reference sketch

### Goal

Produce a reviewable reference implementation of `harmonize_melody` in `crates/mt/src/quintal/harmonize.rs` (or a dedicated `crates/mt/src/harmonize/` module — see "Module location"), with full type signatures, algorithm, edge-case handling, and unit tests. Phase 1 deliverable is the *code* but with the understanding that it may be revised once Duncan reviews the sketch before Phase 2 (production implementation hardening, MCP wiring).

### Module location

Two options under consideration:

- **`crates/mt/src/quintal/harmonize.rs`** — keeps the feature inside the `quintal` module since the candidate enumeration walks the quintal `BaseSpace`. Uses both quintal and quartal sub-APIs.
- **`crates/mt/src/harmonize/mod.rs`** — top-level module signaling that harmonization is a cross-cutting concern that uses both quintal and quartal voicings symmetrically.

Recommend the second (top-level) given the symmetric quartal-API work in Phase 0.5. Reading `mt::harmonize::*` matches the conceptual frame better than `mt::quintal::harmonize::*`. Implementation cost is identical.

### Public API

```rust
//! Voice-led OTH chord progressions for melodies.
//!
//! Given a melody (a sequence of pitches or pitch classes), enumerate the
//! top-K voice-led progressions whose top-voice line traces out that melody,
//! drawing from the full fiber bundle E (all 4 inversions per PcChord) in
//! both quintal and quartal voicings.

use crate::quintal::{PcChord, VoicedChord};

/// Input melody: either pitch classes (caller-supplied octave-free) or
/// MIDI pitches (caller-supplied with octaves).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MelodyInput {
    /// Each element is a pitch class in `0..=11`.
    PitchClasses(Vec<u8>),
    /// Each element is a MIDI pitch in `0..=127`.
    Pitches(Vec<u8>),
}

/// Which voicing perspective(s) to draw candidates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DualityScope {
    QuintalOnly,
    QuartalOnly,
    Both,
}

impl Default for DualityScope {
    fn default() -> Self { DualityScope::Both }
}

/// Configuration for [`harmonize_melody`].
#[derive(Debug, Clone)]
pub struct HarmonizeOptions {
    /// Semitone offset from each melody pitch to that chord's top voice.
    /// Negative pulls the harmony below the melody. Default: -12 (one octave).
    pub top_voice_offset: i8,
    /// Used only with [`MelodyInput::PitchClasses`]: the octave to place the
    /// implied melody in before applying `top_voice_offset`. Default: 5.
    pub melody_octave: i8,
    /// Voicing perspective(s) to draw from. Default: [`DualityScope::Both`].
    pub duality: DualityScope,
    /// Number of distinct progressions to return, ordered by ascending total
    /// movement. Default: 10.
    pub k: usize,
}

impl Default for HarmonizeOptions {
    fn default() -> Self {
        HarmonizeOptions {
            top_voice_offset: -12,
            melody_octave: 5,
            duality: DualityScope::Both,
            k: 10,
        }
    }
}

/// One harmonization result: a sequence of voiced chords plus voice-leading
/// statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Harmonization {
    /// One `VoicedChord` per melody position.
    pub chords: Vec<VoicedChord>,
    /// Per-step minimal voice-leading L1 distances. Length = chords.len() - 1.
    pub per_step_movements: Vec<u32>,
    /// Sum of `per_step_movements`.
    pub total_movement: u32,
}

/// Errors returned by [`harmonize_melody`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HarmonizeError {
    #[error("melody must contain at least one note")]
    EmptyMelody,
    #[error("melody pitch class {0} is outside the valid range 0..=11")]
    InvalidPitchClass(u8),
    #[error(
        "no candidate chord found for melody position {position} \
         (top pitch {top_midi}); this should be unreachable for any \
         legal melody pitch in the supported MIDI range"
    )]
    NoCandidatesForPosition { position: usize, top_midi: u8 },
    #[error("requested K={requested} but only {available} distinct progressions exist")]
    InsufficientProgressions { requested: usize, available: usize },
}

/// Harmonize a melody with the top-K voice-led OTH chord progressions.
///
/// Each returned [`Harmonization`] has, for every melody position,
/// a [`VoicedChord`] whose top voice (`pitches[3]`) equals the melody pitch
/// at that position plus `options.top_voice_offset`. Progressions are
/// ranked by ascending `total_movement`.
///
/// # Errors
///
/// Returns [`HarmonizeError::EmptyMelody`] if `melody` contains no notes,
/// [`HarmonizeError::InvalidPitchClass`] if a [`MelodyInput::PitchClasses`]
/// entry is outside `0..=11`, or [`HarmonizeError::InsufficientProgressions`]
/// if fewer than `options.k` distinct progressions exist for the input.
///
/// # Examples
///
/// ```
/// use music_comp_mt::harmonize::{harmonize_melody, HarmonizeOptions, MelodyInput};
///
/// // Harmonize the melody [C, E, G] with default options.
/// let result = harmonize_melody(
///     MelodyInput::PitchClasses(vec![0, 4, 7]),
///     HarmonizeOptions::default(),
/// )?;
/// assert!(!result.is_empty());
/// # Ok::<(), music_comp_mt::harmonize::HarmonizeError>(())
/// ```
pub fn harmonize_melody(
    melody: MelodyInput,
    options: HarmonizeOptions,
) -> Result<Vec<Harmonization>, HarmonizeError>;
```

Notes on the API design choices:

- `#[non_exhaustive]` on `MelodyInput`, `DualityScope`, `HarmonizeError` so future variants don't break SemVer (TD-07).
- `HarmonizeOptions` is `Clone` (caller can reuse) but not `Copy` (it has no large data, but adding a future `Vec<...>` field would silently break Copy). `Default` impl encodes the defaults the user agreed to.
- `Result<Vec<Harmonization>, HarmonizeError>` returns the K results in ascending-cost order; an empty `Vec` is *not* an error case (e.g. K=0 returns `Ok(vec![])`).
- Errors are typed (no `anyhow` in a library — EH-04). The error variants enumerate the only failure modes; document them in the rustdoc `# Errors` section (DC-04).

### Algorithm

#### Step 1 — Canonicalize melody to target top MIDI pitches

```rust
fn canonicalize_melody(
    melody: &MelodyInput,
    options: &HarmonizeOptions,
) -> Result<Vec<u8>, HarmonizeError> {
    match melody {
        MelodyInput::PitchClasses(pcs) => {
            // Reject empty.
            // For each pc: validate in 0..=11; compute target_top = (12 * (melody_octave + 1)) + pc + top_voice_offset
            //   (the +1 is the standard MIDI octave convention where C-1 = 0; matches Note::midi_pitch())
            // Result is a Vec<u8> of MIDI pitches, each constrained to 0..=127 (clip or error if out of range).
        }
        MelodyInput::Pitches(midi) => {
            // Reject empty.
            // For each pitch: target_top = pitch + top_voice_offset (clamped to 0..=127, error if would underflow).
        }
    }
}
```

The function returns a `Vec<u8>` of target top MIDI pitches, one per melody position. After this step, both `MelodyInput` variants are unified.

#### Step 2 — Candidate enumeration per position

For each target top MIDI pitch `T_i`:

```rust
fn candidates_for_top(
    target_top_midi: u8,
    duality: DualityScope,
    space: &BaseSpace,
) -> Vec<VoicedChord> {
    let target_pc = target_top_midi % 12;
    let mut candidates = Vec::new();
    let base_octave = 0;  // arbitrary; we shift to target_top_midi at the end

    for pc_chord in space.chords() {
        if !pc_chord.pcs.contains(&target_pc) { continue; }

        if matches!(duality, DualityScope::QuintalOnly | DualityScope::Both) {
            let q_root = quintal::quintal_root(pc_chord, base_octave)?;
            for inv in quintal::inversion_cycle(&q_root) {
                if inv.pitches[3] % 12 == target_pc {
                    candidates.push(shift_to_top(inv, target_top_midi));
                    break;  // exactly one inversion per cycle has the target PC on top
                }
            }
        }

        if matches!(duality, DualityScope::QuartalOnly | DualityScope::Both) {
            let qv_root = quartal::quartal_root(pc_chord, base_octave)?;
            for inv in quartal::quartal_inversion_cycle(&qv_root) {
                if inv.0.pitches[3] % 12 == target_pc {
                    candidates.push(shift_to_top(inv.0, target_top_midi));
                    break;
                }
            }
        }
    }

    candidates
}

/// Shift a VoicedChord vertically so its top voice equals `target_midi`.
/// Preconditions: target_midi % 12 == chord.pitches[3] % 12.
fn shift_to_top(chord: VoicedChord, target_midi: u8) -> VoicedChord {
    let delta = target_midi as i32 - chord.pitches[3] as i32;
    debug_assert!(delta % 12 == 0);
    let octaves = delta / 12;
    let shifted = chord.pitches.map(|p| (p as i32 + octaves * 12) as u8);
    VoicedChord::new(shifted).expect("octave shift preserves ascending order")
}
```

Per-position candidate count: 76 with single duality, 152 with `Both`. This was derived from "228 PcChords × (4 PCs each / 12 total PCs) = 76 PcChords contain any given PC, each contributing exactly one inversion per cycle with that PC on top."

If `candidates.is_empty()` for any position, return `HarmonizeError::NoCandidatesForPosition`. Per the count argument this shouldn't occur for any in-range MIDI pitch, but the error variant exists for safety in case a target PC has no chord containing it (which can't happen for {0..=11}, but the `Result` discipline requires we account for it — see EH-02).

#### Step 3 — Top-K Viterbi

Standard k-best Viterbi (also called list Viterbi) over the layered DAG:

- Layers `0..N` correspond to melody positions.
- Layer `i` has `M_i` candidates (exactly the output of `candidates_for_top` for that position's target).
- Edge weight `c_i → c_{i+1}` is `voice_leading::min_voiced_chord_l1(c_i, c_{i+1})` (the new helper from Phase 0.5).

The k-best DP keeps, for each candidate at each layer, the K best paths reaching it (with predecessor pointers). At the final layer, gather all `M_{N-1} × K` candidates and pick the K best globally.

```rust
fn top_k_viterbi(
    layers: &[Vec<VoicedChord>],
    k: usize,
) -> Vec<Harmonization> {
    // For each (layer_idx, candidate_idx), maintain a Vec<PathState> of up to k entries:
    //   PathState { cost: u32, prev_layer_candidate: Option<usize>, prev_path_in_predecessor: Option<usize> }
    // Sorted ascending by cost.
    //
    // Layer 0: each candidate has a single PathState { cost: 0, prev: None, prev_path: None }.
    //
    // For layer i+1, for each candidate c_new:
    //   For each candidate c_prev in layer i:
    //     Compute edge_cost = min_voiced_chord_l1(c_prev, c_new).
    //     For each PathState p in c_prev's k-best list:
    //       Push (p.cost + edge_cost, c_prev_idx, p_idx) into a working buffer.
    //   Sort the buffer ascending by cost; take the first k; assign to c_new's k-best list.
    //
    // Final: gather all (cost, layer_N-1_candidate_idx, path_in_predecessor_idx) across the last layer's
    // k-best lists, sort, take top K globally, and reconstruct each path by backtracking through the
    // predecessor pointers.
}
```

Complexity: `O(N · M² · K · log K)` where `N` is melody length, `M` is per-position candidate count, `K` is requested progressions. With `N=20, M=152, K=10`, this is ≈ 1.5M elementary operations. Trivial — under a millisecond on any machine.

#### Step 4 — Construct `Harmonization` results

For each of the K best paths returned by Viterbi:

- `chords`: the sequence of `VoicedChord`s along the path.
- `per_step_movements`: the edge weights along the path.
- `total_movement`: sum of `per_step_movements`.

Return them in ascending `total_movement` order. If fewer than K distinct paths exist (e.g. melody has 1 note → only single-chord "progressions" exist), return what's available; if `k` was strictly greater than what's available *and the caller supplied k explicitly via options*, we have a design choice:

- Option A: Return what's available silently, document this in rustdoc.
- Option B: Return `HarmonizeError::InsufficientProgressions { requested, available }`.

Recommend Option A — for a brainstorming tool, "give me what you've got" is the right semantics. The error variant is defined in the public API but never returned by the reference implementation; document it as reserved for future use (or remove it from the sketch). **Open question for review:** keep the error variant or drop it?

### Edge cases

- **Single-note melody**: Returns up to `k` single-chord "progressions" with `total_movement = 0`. The K best are just the K candidates with… nothing to rank by, since per_step_movements is empty. **Open question:** what's the secondary sort key for single-note melodies? Possibilities: orbit centrality (tonics first), interval structure (palindromic first), arbitrary (insertion order). Recommend punting this until a real use case asks for it; for now, return them in the order `candidates_for_top` produces.
- **Melody with repeated consecutive notes** (e.g. `[C, C, E]`): The candidate sets for adjacent positions can overlap, so a "stay on the same chord" edge with weight 0 is naturally selected by Viterbi. Confirm this in tests.
- **Melody pitch outside MIDI range** (after applying `top_voice_offset`): Error or clamp? Recommend error; the user should know if their offset pushes the harmony out of range.
- **K = 0**: Return `Ok(vec![])`. Don't error.
- **DualityScope inconsistency**: All three variants are valid; no input combination should error. Test each with a 4-note melody.

### Tests

Co-located in `crates/mt/tests/harmonize/test_harmonize.rs` (registering a new `harmonize` test submodule):

1. `test_canonicalize_melody_pitches_with_default_offset` — `[C5, E5, G5]` with default options canonicalizes to `[C4, E4, G4]` (MIDI 60, 64, 67).
2. `test_canonicalize_melody_pcs_with_default_offset` — `[0, 4, 7]` with `melody_octave=5` canonicalizes to the same `[60, 64, 67]`.
3. `test_candidates_for_c4_with_quintal_only` — assert exactly 76 candidates returned, all with `pitches[3] == 60`.
4. `test_candidates_for_c4_with_both_dualities` — assert exactly 152 candidates returned.
5. `test_top_voice_pinned_for_short_melody` — for a 3-note melody, every chord in every returned harmonization has `pitches[3]` equal to the canonicalized target.
6. `test_total_movement_is_monotonic_in_returned_order` — for K=10, the returned `total_movement` values are non-decreasing.
7. `test_repeated_notes_admit_zero_movement_step` — for melody `[C, C]`, the top harmonization has `total_movement = 0` (achieved by choosing the same chord both positions).
8. `test_quartal_only_excludes_quintal_voicings` — for a one-note melody with `DualityScope::QuartalOnly`, every returned chord has interval structure in {4,5,6}-reverse-of-quintal terms.
9. `test_empty_melody_errors` — `HarmonizeError::EmptyMelody`.
10. `test_invalid_pitch_class_errors` — `MelodyInput::PitchClasses(vec![13])` returns `HarmonizeError::InvalidPitchClass`.
11. `test_default_options_match_documented_defaults` — sanity check on `Default` impl.

Doctests on `harmonize_melody` rustdoc validate the example.

### Acceptance criteria for Phase 1

- All public types and the `harmonize_melody` function compile and have full rustdoc with `# Errors` and `# Examples` (DC-02, DC-04).
- All tests above pass.
- `cargo clippy -- -D warnings` is clean.
- `cargo doc --no-deps` produces complete docs for the new module with no broken intra-doc links.
- Reference sketch is reviewed and approved by Duncan before Phase 2 (production hardening: profiling, performance optimizations if needed, MCP wiring) begins.

### Risks

- **API stability**: The public types here will be exposed via the MCP tool in Phase 2. Once the tool is in use, changing the types is a breaking change. Spend a little extra care on the types and field names; consider using a [builder pattern](https://rust-unofficial.github.io/patterns/idioms/ctor.html) for `HarmonizeOptions` if more options are likely to be added (current count = 4, manageable; revisit at 6+).
- **Top-K Viterbi correctness**: The k-best DP has classic off-by-one and pruning bugs. Test 6 above is the integration test; consider also a property-style test that compares against brute-force enumeration for tiny inputs (M ≤ 4, N ≤ 3).
- **`min_voiced_chord_l1` scope creep**: This belongs to Phase 0.5, but if its scope grows during implementation (e.g. "should it support 3-voice or 5-voice chords?"), defer the expansion — the harmonize feature only needs the 4-voice case.

---

## Cross-phase concerns

### Error handling

Library-grade typed errors via `thiserror` per EH-04. No `panic!` / `unwrap` / `expect` outside test code (AP-03, AP-06). Every public `Result`-returning function has a `# Errors` rustdoc section listing the variants it can return.

The MCP server crate (Phase 2 onward) will translate `HarmonizeError` to its own error type via `From` impls, following the existing pattern in `mcp-server/crates/server/src/error.rs`.

### Documentation

Crate-level rustdoc in `harmonize/mod.rs` should briefly state the OTH framing and the conceptual model (pinned top voice + voice-leading-ranked progressions). Per DC-14, this orientation paragraph is the first thing a user reads when navigating to the module from the crate root.

Cross-link to:

- `quintal::BaseSpace` (the candidate pool source)
- `quintal::quintal_root`, `quartal::quartal_root` (Phase 0.5 additions)
- `voice_leading::min_voiced_chord_l1` (the edge-weight function)

### Cargo and feature gates

No new optional features needed. The harmonize module depends only on `quintal`, `quartal`, and `voice_leading` — all of which are already in core (no `midi` feature gate). Add no new dependencies.

If `serde` derives are wanted on `Harmonization` for MCP serialization later, gate them under the existing `serde` feature flag (consistent with other types in the crate).

### Workspace layout

No changes to `Cargo.toml` files. No new crates. All work lands in `crates/mt/`.

---

## Open design questions carried forward

These don't block Phase 0 or 0.5 but should be resolved during Phase 1 review:

1. **Module path**: `crates/mt/src/harmonize/mod.rs` (top-level) vs. `crates/mt/src/quintal/harmonize.rs` (under quintal). Recommend the former; flag for confirmation.
2. **Error variant `InsufficientProgressions`**: keep as part of the public surface or drop? Recommend drop — silently return what's available.
3. **Single-note melody secondary sort key**: not yet specified. Recommend punt until a real use case asks.
4. **`HarmonizeOptions::Default` choice for `melody_octave`**: 5 (so default top voice lands at octave 4 after `−12` offset). Confirm.
5. **`shift_to_top` precondition handling**: `debug_assert!` on the precondition (top PC matches target PC) is right for an internal helper, but if the function ever becomes public it needs a real `Result`. Keep `pub(crate)` for now.
6. **Performance**: at the ~500K-edge-evaluation scale, no optimization needed. If long melodies (N > 50) become a real use case, profile and consider caching `min_voiced_chord_l1` results by `(VoicedChord, VoicedChord)` pair — many will recur across positions.

---

## Summary table

| Phase | Goal | Deliverable | Acceptance |
|-------|------|------------|------------|
| 0 | Verify 2 T1 orbits per PcChord | `tests/quintal/test_fiber_orbits.rs` | Test passes for all 228 chords |
| 0.5 | Symmetric quintal/quartal API | `quintal_root`, `quartal_root`, `quartal::base_space`, `min_voiced_chord_l1`, doc fixes, tests | New tests + existing tests pass; clippy clean |
| 1 | `harmonize_melody` reference sketch | `crates/mt/src/harmonize/mod.rs` + tests | Compiles, tests pass, rustdoc complete, design-reviewed |

## Related guidelines (Rust skill cross-references)

- TD-07 `#[non_exhaustive]` on public enums
- EH-02 / EH-04 `Result` everywhere; `thiserror` for libraries
- API-12 / API-14 Owned returns; sealed extension surfaces
- DC-02 / DC-04 / DC-14 Crate-level orientation, `# Errors`, `# Examples`
- AP-03 / AP-06 No `unwrap` / `expect` outside test code
- ID-10 `impl AsRef<Path>` etc. — N/A here (no file I/O)
- TR-26 Sealed traits — only relevant if we expose extension points later
