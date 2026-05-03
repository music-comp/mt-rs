---
number: 8
title: "Melody Harmonization — Phase 2: Production Hardening & MCP Integration"
author: "least total"
component: "music-comp-mt, music-comp-mt-cli, music-theory-mcp"
tags: [phase-2, production, mcp, public-api]
created: 2026-05-02
updated: 2026-05-02
state: Draft
supersedes: null
superseded-by: null
version: 1.0
related: ["0007-melody-harmonization-feature-implementation-plan-phases-0-0.5-1.md"]
---

# Melody Harmonization — Phase 2: Production Hardening & MCP Integration

Draft date: 2026-05-02
Target crates: `music-comp-mt` (mt-rs), `music-comp-mt-cli` (mt-rs), `music-theory-mcp` (ai-music-theory)
Rust edition: 2021 / toolchain 1.80+
Owners: Duncan & Claude

## Context and goal

Phase 1 landed the reference sketch of `harmonize_melody` (commit `7c89f63`) — public types, the candidate-enumeration → Viterbi pipeline, integration tests, doctest, all eight MUST-FIX items addressed. The implementation is correct and the math holds. What it isn't, yet, is *finished*.

Phase 2 takes the Phase 1 sketch from "compiles, tests pass, math verified" to "ready for public consumption and MCP integration." Three audiences are in scope:

1. **Rust library users** who depend on `music-comp-mt` from crates.io and call `harmonize_melody` directly.
2. **CLI users** who run `mt harmonize ...` from a terminal.
3. **MCP/Claude users** who invoke a `harmonize_melody` tool from a conversational context.

For each, the experience must be as polished as the existing `mt oth modes`, `mt oth orbits`, and `mt oth geodesic-distribution` surfaces. That means rich documentation, accurate errors, OTH-aware output formatting (orbit names, fiber class, parent scale), reasonable defaults, stable types, and integration tests that exercise the public path end-to-end.

This plan is organized in nine sub-phases, each independently reviewable. They run roughly sequentially but Phases 2.1–2.4 can be parallelized if helpful.

---

## Phase 2.1 — Code cleanup from Phase 1 review

### Goal

Apply the small-but-real items from the Phase 1 review that were marked "non-blocking but worth tightening." None individually changes behavior; together they make the code review-clean.

### Deliverables

**`test_specific_chord_cgda` becomes a real test.** Currently uses `if let Some / else` and prints rather than asserting. Replace with:

```rust
let cgda = candidates
    .iter()
    .find(|c| {
        let pcs: BTreeSet<u8> = c.pitches.iter().map(|&p| p % 12).collect();
        pcs == [0, 2, 7, 9].iter().copied().collect()
    })
    .expect("CGDA must be in quintal candidates for top=60");
eprintln!("CGDA candidate with top=60: {:?}", cgda.pitches);
assert_eq!(cgda.pitches[3], 60);
```

**`test_shift_overflow` either becomes accurate or goes away.** The existing test correctly reasons that overflow is unreachable for `u8 target_midi` — but the test name promises overflow coverage and the body is an identity shift. Pick one: rename to `test_shift_overflow_is_unreachable_for_u8_target` (and assert the reasoning explicitly), or delete it. Recommend renaming with a clear assertion that documents the invariant.

**Remove the redundant `top_after` bounds check in `shift_to_top`.** Since `top_after = target_midi as i32` and `target_midi: u8`, `top_after ∈ 0..=255` always; combined with the precondition that target ≤ 127 (canonicalized), the `top_after > 127` arm is dead code. Simplify to:

```rust
let bottom_after = chord.pitches[0] as i32 + delta;
if bottom_after < 0 {
    return Err(HarmonizeError::TargetMidiOutOfRange { position, target_midi });
}
```

Add a comment explaining why only the bottom needs to be checked.

**Rename misleading integration test names.** In `crates/mt/tests/harmonize/test_harmonize.rs`, two tests inherit names from the design doc that no longer reflect what they actually test:

- `test_canonicalize_melody_pitches_with_default_offset` → `test_harmonize_with_pitches_default_offset`
- `test_canonicalize_melody_pcs_with_default_offset` → `test_harmonize_with_pcs_default_offset`

(The standalone canonicalize tests live in `test_canonicalize.rs` with their original names; the integration-layer tests should reflect that they exercise `harmonize_melody` end-to-end.)

**Add a position-field correctness test for `TargetMidiOutOfRange`.** The existing canonicalize tests use `..` to ignore the field, leaving the `.iter().enumerate()` change unverified at the assertion layer. Add:

```rust
#[test]
fn test_position_field_carries_correct_index() {
    let opts = HarmonizeOptions { top_voice_offset: -12, ..Default::default() };
    let result = canonicalize_melody(
        &MelodyInput::Pitches(vec![60, 5, 60]),
        &opts,
    );
    eprintln!("position_field_check: {:?}", result);
    assert!(matches!(
        result,
        Err(HarmonizeError::TargetMidiOutOfRange { position: 1, .. })
    ));
}
```

### Acceptance criteria

All previously-passing tests still pass; the renamed and new tests pass; `cargo clippy -- -D warnings` clean; `cargo doc --no-deps` clean.

---

## Phase 2.2 — Error refinement

### Goal

Make error reporting accurate, complete, and consistent with the rest of the crate.

### Deliverables

**`TargetMidiOutOfRange` reports the actual computed pitch.** Currently `target_midi: u8` is clamped to `0..=255`, so an underflow case (e.g., `pitch=5, offset=-12 → -7`) reports `target_midi: 0`, which is misleading. Two paths:

- *Option A:* Change the error variant to `TargetMidiOutOfRange { position: usize, target_midi: i16 }`. Faithful reporting; minor semver impact (the variant is `#[non_exhaustive]` in carrying-struct shape, so adding/changing fields is allowed).
- *Option B:* Keep `u8` and document that the field is clamped on out-of-range.

Recommend Option A. The cost is one type change; the benefit is the user can read the actual offending value in the error message.

**Audit `Display` messages for `HarmonizeError`.** Each variant should produce a single human-readable sentence, no jargon, no MIDI-specific language unless necessary. Current messages are reasonable; this is a final pass. Verify with `cargo expand` or a unit test that constructs each variant and checks the formatted message.

**Drop `InsufficientProgressions` from the public surface.** Per the design doc's Option A resolution and the actual implementation behavior (silently return what's available when fewer than K paths exist), this variant is defined but never returned. Remove it. If we want a future-proofing slot, add a doc comment on the enum noting that variants may be added under the `#[non_exhaustive]` guarantee.

**Add `From<QuintalError>` and `From<QuartalError>` impls** if any internal call sites would benefit from `?`-conversion. Audit `candidates.rs` — currently uses `.expect(...)` on quintal/quartal_root, which is correct (Phase-0-verified invariant). No `From` impls needed in the harmonize module today; document this decision in the rustdoc on `HarmonizeError`.

**Document the precise meaning of every error variant in the rustdoc** under a `# Errors` section on the variant itself, not just on `harmonize_melody`. This is the level of detail expected from a 1.0-quality library.

### Acceptance criteria

The `TargetMidiOutOfRange` field type change compiles, all tests pass with the new field, and the error message renders the actual i16 value. `InsufficientProgressions` is gone. Each variant has its own rustdoc paragraph. `cargo doc --no-deps` produces no broken intra-doc links.

---

## Phase 2.3 — API stability and visibility tightening

### Goal

Lock down the public API so we can commit to it under SemVer. Reduce the surface that's exposed beyond what users need.

### Deliverables

**Tighten `pub mod canonicalize;` and `pub mod candidates;` to `pub(crate) mod`.** Phase 1 made these `pub` so integration tests could reach internals; for production we want internals private. Move the test files' contents into in-module `#[cfg(test)] mod tests` blocks, or convert the integration tests to public-API tests that exercise the same paths via `harmonize_melody`. Recommend the second: the integration layer should test the public function, not internals. Standalone Phase-1 tests stay as in-module unit tests for fine-grained verification.

**Audit every `pub` symbol in `crates/mt/src/harmonize/mod.rs`.** Each one is a SemVer commitment. Specifically:

- `MelodyInput`, `DualityScope`, `HarmonizeOptions`, `Harmonization`, `HarmonizeError` — all stable, all `#[non_exhaustive]` where appropriate. ✓
- `harmonize_melody` — stable. ✓
- Any internal helper accidentally exposed — none, after Phase 2.3 above.

**Decide the `HarmonizeOptions` builder question.** The design doc flagged builder pattern as "revisit at 6+ fields"; we're at 4. For a 1.0 surface, recommend: skip the builder, keep `Default` + struct-update syntax (`HarmonizeOptions { k: 5, ..Default::default() }`). Add a doc-test showing this idiom in the rustdoc on `HarmonizeOptions`. Revisit in v0.7 if the field count grows.

**Add `serde` derives behind the existing `serde` feature flag** to all public types: `MelodyInput`, `DualityScope`, `HarmonizeOptions`, `HarmonizeError`. `Harmonization` already has them. Required for MCP serialization in Phase 2.9. Use the same `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` pattern as the existing types.

**Re-export from `crates/mt/src/lib.rs`** the surface a typical caller wants:

```rust
pub use harmonize::{
    harmonize_melody, DualityScope, Harmonization, HarmonizeError,
    HarmonizeOptions, MelodyInput,
};
```

So users can `use music_comp_mt::harmonize_melody;` instead of `use music_comp_mt::harmonize::harmonize_melody;`. Match the convention used for `quintal::quintal_root` etc. (which are *not* re-exported at the crate root — actually verify this and pick a consistent rule). Recommend: re-export at crate root only the most-likely-used items per module, and require `music_comp_mt::harmonize::*` for less-common items.

### Acceptance criteria

All public symbols audited and documented. `pub(crate) mod canonicalize`/`pub(crate) mod candidates` compile and all tests still pass. Serde derives gated correctly. A `cargo public-api diff` baseline is captured for future SemVer regression detection (optional but recommended).

---

## Phase 2.4 — Documentation completeness

### Goal

Every public type and function has rustdoc that an unfamiliar reader could use to write working code without reading the implementation. The OTH framing is surfaced where it adds value (orbit names, fiber class) without requiring the reader to know OTH theory.

### Deliverables

**Module-level rustdoc** at the top of `crates/mt/src/harmonize/mod.rs` becomes a real introduction:

- One-paragraph overview: what the module does.
- A "Conceptual model" subsection: melody pinned at top voice; harmony drawn from the OTH 228-chord space; ranked by least total semitone movement.
- A "Quick example" subsection with a 5–10 line copy-paste example that runs as a doctest.
- A "How it works" subsection naming the three internal stages (canonicalize, enumerate, Viterbi) without exposing internals — for users who want to understand the model.
- A "When to use this" subsection: who this is for (composers, theorists, ML harmonization workflows).
- Cross-references to `quintal::BaseSpace`, `quintal::Orbit`, the OTH glossary doc.

**Per-function rustdoc** with three sections each: a one-sentence summary, an `# Errors` section enumerating every variant the function can return, and an `# Examples` section with a runnable doctest. Apply to `harmonize_melody` (already has this — verify completeness) and to all `pub` types' methods if any.

**Per-type rustdoc** for `MelodyInput`, `HarmonizeOptions`, `Harmonization`, `DualityScope`, `HarmonizeError`. Each variant of each enum gets its own paragraph. Each field of each struct gets its own paragraph (already partially done in Phase 1 — verify).

**Intra-doc links everywhere.** Replace bare references (`MelodyInput`, `BaseSpace`) with `[`MelodyInput`]`, `[`crate::quintal::BaseSpace`]`, etc. Run `cargo doc --no-deps` with `RUSTDOCFLAGS="-D warnings"` to fail-fast on broken links.

**Add a doctest that exercises the OTH-specific framing.** Something like:

```rust
/// # OTH framing
///
/// Each returned [`Harmonization`] consists of voiced chords drawn from the
/// 228-chord OTH base space. The progression is voice-led — the K best
/// progressions minimize total L1 voice movement across the melody.
///
/// ```
/// use music_comp_mt::harmonize::*;
/// // [doctest body]
/// # Ok::<(), HarmonizeError>(())
/// ```
```

**`# Errors` sections must enumerate variants explicitly,** matching the project convention from `quintal_root`, `quartal_root`, etc. No "may return an error" hand-waving.

### Acceptance criteria

`cargo doc --no-deps` with `RUSTDOCFLAGS="-D warnings"` is clean. Every public symbol renders a populated doc page. The "Module harmonize" landing page reads as a self-contained introduction. All doctests pass under `cargo test --doc`.

---

## Phase 2.5 — Examples and README

### Goal

A user landing on the crates.io page or the GitHub README can find working examples within 30 seconds.

### Deliverables

**Add `crates/mt/examples/harmonize_melody.rs`** — a runnable example showing realistic usage:

- Construct a 4-note melody (e.g., the opening of "Twinkle Twinkle Little Star" or a simple ascending scale).
- Call `harmonize_melody` with default options, K=5.
- Print the K progressions to stdout in a readable format (note names + per-step movement + total movement).
- Include a second example with `DualityScope::QuartalOnly` to show the duality-scope option in action.

Run via `cargo run -p music-comp-mt --example harmonize_melody`. Verify it produces sensible output and the output is stable across runs (or document that ordering between equal-cost progressions is unspecified).

**Update the top-level README** with a `### Melody Harmonization` subsection between "Open Tone Harmony Mode Analysis" and "Open Tone Harmony CLI Commands". Include:

- One-paragraph description of the feature.
- A 10–15 line code example mirroring the simplest doctest.
- A pointer to the `examples/harmonize_melody.rs` file and the CLI subcommand.

**Update the README's "Quintal/Quartal Voice-Leading Geometry" feature list** with a bullet for "Melody harmonization — top-K voice-led OTH chord progressions ranked by least total semitone movement, drawing from quintal and/or quartal voicings of the full 228-chord space."

**Bump version to `0.6.0`.** Per SemVer, adding new public types and a new module is a minor version bump. Update `crates/mt/Cargo.toml` from `0.5.2` to `0.6.0`. Verify version constraints in `crates/mt-cli/Cargo.toml`.

**No CHANGELOG.** This project does not maintain a `CHANGELOG.md` going forward. Release notes live in git history and tagged release notes; per Duncan's policy (2026-05-02), no manual changelog file is maintained. The legacy `CHANGELOG.md` at the project root is left untouched (or removed in a separate housekeeping pass — out of scope here).

### Acceptance criteria

`cargo run -p music-comp-mt --example harmonize_melody` succeeds and prints readable output. `cargo build --all-features` succeeds at the new version. README renders correctly (visual check on GitHub). `cargo doc --open` opens to the v0.6 docs.

---

## Phase 2.6 — Property and robustness tests

### Goal

Move beyond "the example tests pass" to "the algorithm satisfies its specification on adversarial inputs."

### Deliverables

**Add property tests** in `crates/mt/tests/harmonize/test_properties.rs` (new file). Use `proptest` (add to `dev-dependencies`). Properties:

- For any valid melody and any K ≥ 1: every returned `Harmonization`'s `total_movement` equals the sum of its `per_step_movements`.
- For any valid melody: every returned chord's `pitches[3]` equals the canonicalized target for its position.
- For any valid melody and K=10: returned progressions are non-decreasing in `total_movement`.
- For any valid 1-note melody: every returned chord's pitch class set is a member of `BaseSpace`, and the chord is voiced ascending.
- For any valid melody and `DualityScope::Both`: per-position candidate count = 152 (verifiable via the public API by running with K=200 on a 1-note melody).

**Brute-force comparison test for top-K Viterbi.** For tiny inputs (N ≤ 3 melody notes, K ≤ 3), enumerate all paths through the layered DAG by full Cartesian product, compute total cost for each, sort, and assert the top K matches what `harmonize_melody` returns. Lives in `tests/harmonize/test_viterbi_brute_force.rs`. Hand-construct a 2-position melody where the ground-truth top-3 paths can be verified by hand.

**Edge-case tests**:

- Melody at MIDI minimum (target = 0..=23): expect `TargetMidiOutOfRange` for some/all candidates; document expected behavior.
- Melody at MIDI maximum (target = 120..=127): no underflow but verify behavior near the top of range.
- Very long melody (N = 50): verify performance stays under 100ms; verify result correctness on a known-monotonic melody.
- Very large K (K = 500): verify no panic, returns at most `min(K, available_paths)` results.

**Determinism test.** Two consecutive calls with identical inputs must produce identical outputs. (`Vec` ordering, `BTreeMap` traversal, no-`HashSet`-in-the-pipeline — verify.) If determinism is hard to guarantee due to internal hashing, document the source of nondeterminism.

### Acceptance criteria

All property tests pass with at least 256 generated cases each. Brute-force comparison passes for all enumerated inputs. Edge-case tests pass with documented behavior. `cargo test` total runtime stays under ~30s.

---

## Phase 2.7 — Performance characterization

### Goal

Document expected performance and detect regressions. Not optimize — Phase 1 deliberately deferred optimization.

### Deliverables

**Add `criterion` benchmarks** in `crates/mt/benches/harmonize.rs`:

- `bench_single_note_melody` — 1-note melody, default options (~152 candidates, no Viterbi).
- `bench_typical_4_note_melody` — 4-note melody, default options (152 × 4 candidates, K=10 Viterbi).
- `bench_long_melody` — 20-note melody, K=10.
- `bench_long_melody_high_k` — 20-note melody, K=100.

Run via `cargo bench`. Document the observed timings on the development machine in the module-level rustdoc:

> # Performance
>
> On a typical modern laptop (2024 reference), `harmonize_melody` completes in
> approximately:
>
> - `<1ms` for melodies up to 4 notes with K=10.
> - `<10ms` for melodies up to 20 notes with K=10.
> - `<50ms` for melodies up to 50 notes with K=100.
>
> The algorithm is `O(N · M² · K · log K)` where N is melody length, M is
> candidate count per position (76 for single duality, 152 for `Both`), and K
> is requested progressions. Memory is `O(N · M · K)`.

**Profile and identify bottlenecks** if benchmarks reveal anything surprising. Likely candidates: `min_voiced_chord_l1` (24 perms × 4 voices per call) — could memoize per `(c_prev_idx, c_new_idx)` pair within Viterbi to halve total invocations. Defer the actual optimization unless benchmarks show it's needed; capture the analysis in the rustdoc.

**Add a `cargo test --release` line to CI** to catch perf regressions on the integration tests (debug builds can mask 10× slowdowns).

### Acceptance criteria

Benchmarks compile and run. Observed timings are documented in module rustdoc. The rustdoc claims match observed reality within 2×.

---

## Phase 2.8 — CLI integration

### Goal

`mt harmonize ...` is a first-class CLI subcommand alongside `mt scale`, `mt chord`, `mt oth modes`, etc.

### Deliverables

**Add `Commands::Harmonize` variant** to the CLI's `Commands` enum in `crates/mt-cli/src/cli.rs`, following the pattern of `Commands::Oth { action: OthAction }`:

```rust
Harmonize {
    /// Melody as comma-separated note names (e.g., "C5,E5,G5") or pitch
    /// classes ("0,4,7" with --pcs flag).
    melody: String,
    #[arg(long, default_value_t = 10)]
    k: usize,
    #[arg(long, default_value_t = -12)]
    top_voice_offset: i8,
    #[arg(long, default_value_t = 5)]
    melody_octave: i8,
    #[arg(long, value_enum, default_value_t = CliDualityScope::Both)]
    duality: CliDualityScope,
    #[arg(long)]
    pcs: bool,
    #[arg(long, value_enum, default_value_t = CliFormat::Pretty)]
    format: CliFormat,
}
```

Where `CliDualityScope` mirrors `DualityScope` and `CliFormat` is one of `Pretty | Json | Markdown`.

**Input parsing.** Accept melodies as:

- Comma-separated note names: `"C5,E5,G5"` (default).
- Comma-separated MIDI integers: `"60,64,67"` (with `--midi` flag, optional).
- Comma-separated pitch classes: `"0,4,7"` (with `--pcs` flag).

Reuse the note-parsing helpers from existing CLI surfaces (`mt scale C Ionian` etc.) so naming is consistent.

**Output formatting** (three formats):

- *Pretty (default):* Each progression on its own block with chord-by-chord layout, note names, optional orbit classification per chord, total movement.
- *JSON:* Machine-readable. Schema mirrors the Rust types with note names alongside MIDI numbers.
- *Markdown:* Pasteable into a notebook or document. Tabular layout.

For pretty/markdown output, render each chord as both note names and OTH classification — e.g.:

```
Progression 1 (total movement: 8 semitones)
  Position 1: C-G-D-A    [Q777 Summit]    top=C5
  Position 2: D-G-D-A    [Q877 Plateau]   top=E5
  Position 3: D-G-D-A    [Q877 Plateau]   top=G5
```

Look up orbit classification via `classify_orbit` and orbit name via the existing `Orbit::name()` (or whichever method exposes the human-readable name — e.g. "Summit", "Saddle"). If those aren't on the public API today, add them in Phase 2.3.

**CLI tests** in `crates/mt-cli/src/cli.rs` `#[cfg(test)] mod tests`:

- Round-trip: a known input produces known output.
- Each format produces valid output (JSON parses, markdown table is well-formed).
- Error paths surface through CLI as nonzero exit and human-readable stderr.
- Position info from `TargetMidiOutOfRange` reaches the user.

**README update** with the CLI subsection mirroring the existing `mt oth modes` examples:

```sh
mt harmonize "C5,E5,G5"                                  # default options, pretty
mt harmonize "C,E,G" --pcs --melody-octave 5             # pitch-class input
mt harmonize "60,64,67" --midi --k 5                     # MIDI input, top 5
mt harmonize "C5,E5,G5" --duality quartal-only           # quartal voicings only
mt harmonize "C5,E5,G5" --format json                    # JSON output
```

### Acceptance criteria

`mt harmonize "C5,E5,G5"` produces sensible output. All three formats produce valid output. CLI tests pass. README examples match what the CLI actually accepts. The CLI surface is consistent with `mt oth *` — naming, flag conventions, error formatting.

---

## Phase 2.9 — MCP server integration

### Goal

A `harmonize_melody` MCP tool registered in `music-theory-mcp` that Claude can invoke from a conversation. The tool's input/output schema is JSON, but the rendered output should feel like a music-theorist talking to a musician — note names, orbit classifications, fiber class, parent scale where relevant.

### Deliverables

**Investigate the existing `music-theory-mcp` tool registration pattern.** The MCP server already exposes ~50 tools (visible in this conversation's tool list under `mcp__music-theory__*`). Read one of the OTH tools — `get_oth_chord_info`, `get_oth_neighbors`, `analyze_roman_numerals` — to understand:

- The trait/macro pattern for tool registration.
- Input schema declaration (JSON Schema or Rust struct + serde).
- Output serialization (likely `serde_json::Value`).
- Error mapping from library error to MCP error.

Document the pattern so the harmonize tool follows it.

**Tool definition.** Register `harmonize_melody` as an MCP tool with:

- *Input schema:*
  - `melody`: array of pitches (string note names like `"C5"` or numeric MIDI like `60`) or pitch classes (`0..11`).
  - `melody_format`: enum `"note_names" | "midi" | "pitch_classes"`.
  - `k`: integer, default 10.
  - `top_voice_offset`: integer, default -12.
  - `melody_octave`: integer, default 5 (used only when `melody_format = "pitch_classes"`).
  - `duality`: enum `"quintal_only" | "quartal_only" | "both"`, default `"both"`.

- *Output schema:* an array of `Harmonization` objects, each containing:
  - `total_movement`: integer.
  - `per_step_movements`: array of integers.
  - `chords`: array of objects, each with:
    - `pitches`: array of MIDI integers.
    - `notes`: array of note names (e.g., `["C4", "G4", "D5", "A5"]`).
    - `pitch_classes`: array of integers.
    - `orbit`: orbit label (e.g., `"Q777"`).
    - `orbit_name`: terrain feature name (e.g., `"Summit"`, `"Saddle"`).
    - `fiber_class`: `"A"` or `"B"`.
    - `parent_scale`: scale name and root if in a recognizable scale family, else `null`.

The richer-than-strictly-needed output is the right call here — Claude will use the orbit name, fiber class, and parent scale to write a musically-literate explanation back to the user.

**Implementation.** New module `mcp-server/crates/server/src/harmonize.rs` (path matches the existing music-theory-mcp layout — confirm during implementation). The tool handler:

1. Validates input against the schema.
2. Constructs `MelodyInput` and `HarmonizeOptions` from the JSON.
3. Calls `harmonize_melody`.
4. Maps the result to the rich JSON output (looking up orbit/fiber/scale info via the existing `quintal::classify_orbit`, `quintal::FiberClass`, parent-scale helpers).
5. Maps `HarmonizeError` to an MCP error response with the same field detail (position, target_midi).

**Note-name conversion helper.** MIDI integer → note name (e.g., 60 → "C4") needs to live somewhere. If `crates/mt` doesn't already have a public helper, add one in this phase — it's useful for the CLI and the MCP layer both. Recommend `Note::from_midi(midi: u8) -> Note` (already exists?) plus `impl Display for Note` rendering to `"C4"`. Verify during implementation; add what's missing to the underlying crate, not in the MCP wrapper.

**MCP tool tests.** In `mcp-server/crates/server/tests/harmonize_tool.rs`:

- Happy path: 4-note melody, K=5, expected count and shape of returned objects.
- Note-name input: `"C5,E5,G5"` produces same result as `[72, 76, 79]` MIDI.
- Pitch-class input: `[0, 4, 7]` with `melody_octave=5` produces same canonical targets as MIDI `[72, 76, 79]`.
- Error mapping: invalid pitch class returns the MCP error with the right code/message.
- Schema validation: malformed input is rejected with a useful error before reaching the library.

**Tool documentation.** Update the MCP server's tool index (whatever README or docs file lists the tools) with a `harmonize_melody` entry: one-paragraph description, input/output schema link, example invocation. Match the existing entries' structure.

### Acceptance criteria

The tool appears in the MCP server's tool list and can be invoked end-to-end. A round-trip test from JSON input through `harmonize_melody` to JSON output produces results consistent with calling the library directly. Error paths surface to the MCP client with sufficient detail. The output shape includes orbit name and fiber class for every chord. Documentation lands alongside the implementation.

---

## Cross-phase concerns

### Backward compatibility

Phase 1 didn't make a SemVer commitment because v0.5.x is a development line. Phase 2 ships under v0.6.0 with the Phase 1 API stabilized. The `#[non_exhaustive]` attribute on enums lets us add variants in v0.6.x without a breaking change; field additions on structs require care (use struct-update syntax in examples to be additive-friendly).

### Documentation conventions

Match the existing crate conventions:

- Three-section rustdoc on functions: summary, `# Errors`, `# Examples`.
- Per-variant doc paragraphs on enums.
- Intra-doc links via `[`Type`]` and `[`crate::module::function`]`.
- Module-level rustdoc as a real introduction, not a one-line summary.

### Testing conventions

Match Phase 0/0.5/1 conventions:

- Integration tests in `crates/mt/tests/harmonize/`.
- Unit tests in `#[cfg(test)] mod tests` blocks within source files for internals.
- Property tests in a dedicated file using `proptest`.
- Brute-force comparison tests for algorithmic claims.
- `eprintln!` debug output that's available with `--nocapture`.

### CI

After Phase 2 lands, CI should run, in order: `cargo build --all-features`, `cargo test --all-features`, `cargo test --doc`, `cargo clippy --all-features -- -D warnings`, `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`, `cargo bench --no-run` (compile only — running benches in CI is too slow), and `cargo test --release --test harmonize` for performance regression detection.

### Workspace layout

No new crates. CLI and library work lands in `mt-rs/`. MCP integration lands in `ai-music-theory/mcp-server/crates/server/` (verify path during Phase 2.9).

---

## Open questions

1. **Re-export at crate root.** Should `harmonize_melody` and friends be re-exported from `music_comp_mt::*` directly, or always accessed via `music_comp_mt::harmonize::*`? The existing convention is mixed — `quintal::quintal_root` is *not* re-exported; `note::Note` and `chord::Chord` are commonly imported as `use music_comp_mt::chord::Chord`. Recommend: do not re-export at crate root; require `music_comp_mt::harmonize::harmonize_melody`. Confirm.

2. **`TargetMidiOutOfRange.target_midi` field type.** Recommend `i16` for faithful reporting (Phase 2.2). Confirm.

3. **`CliFormat` for `mt harmonize`.** Three formats (Pretty, JSON, Markdown) is what the existing `mt oth geodesic-distribution` exposes. Should `mt harmonize` match that exactly, or just Pretty + JSON for v0.6 and add Markdown later? Recommend match the existing pattern (all three).

4. **MCP output verbosity.** The proposed output includes orbit name, fiber class, parent scale per chord. This is rich but increases payload size. For long melodies (N=50, K=10) the response could be 50–100KB. Acceptable? Recommend yes — Claude can filter on its end if needed; the rich data is the point.

5. **Property-test complexity.** Should `proptest` be a hard dependency or behind a feature flag? Recommend `dev-dependencies` only; property tests are part of the test suite, not the published crate.

6. **Builder pattern for `HarmonizeOptions`.** Recommend defer (Phase 2.3 explanation). Confirm.

---

## Summary table

| Phase | Goal | Deliverable | Acceptance |
|-------|------|------------|------------|
| 2.1 | Cleanup from review | Test fixes, removed dead code, renamed tests | All tests pass; clippy clean |
| 2.2 | Error refinement | `i16` field, dropped variant, full Display audit | Error messages accurate; doc complete |
| 2.3 | API stability | Visibility tightening, serde derives, version bump | `pub` surface stable; serde works |
| 2.4 | Documentation | Module doc, per-type doc, intra-doc links | `cargo doc -D warnings` clean |
| 2.5 | Examples + README | Runnable example, README sections, version bump | `cargo run --example` works; README complete |
| 2.6 | Property tests | Property suite, brute-force comparison, edge cases | All properties hold over 256+ cases |
| 2.7 | Performance | Benchmarks, documented timings | Bench numbers match doc claims within 2× |
| 2.8 | CLI | `mt harmonize` subcommand with three formats | CLI matches `mt oth *` conventions |
| 2.9 | MCP | `harmonize_melody` tool with rich output | End-to-end JSON round-trip works |

## Related guidelines

- TD-07 `#[non_exhaustive]` on public enums (apply across all error and config types)
- EH-04 `thiserror` for library errors (already in place)
- API-12 / API-14 Owned returns; sealed extension surfaces (no extension points exposed in v0.6)
- DC-02 / DC-04 / DC-14 Crate-level orientation, `# Errors`, `# Examples` (Phase 2.4)
- AP-03 / AP-06 No `unwrap` / `expect` outside test code (verify; allow only with documented invariant rationale)
- The Phase 0 / 0.5 / 1 implementation plan (`0007-melody-harmonization-feature-implementation-plan-phases-0-0.5-1.md`)
- The Phase 1 review feedback (`workbench/0007-phase-1-dev-plan-review-feedback.md`)
- The implementation workflow supplement (`workbench/0007-phase-1-implementation-workflow-supplement.md`)
