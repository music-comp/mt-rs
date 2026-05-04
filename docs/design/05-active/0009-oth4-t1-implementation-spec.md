---
number: 9
title: "Quartal Implementation Spec (Phase T1)"
author: "the chord"
component: All
tags: [change-me]
created: 2026-05-04
updated: 2026-05-04
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# oth4 — T1 Implementation Spec

**Status:** Ready for handoff to Claude Code, 2026-05-04
**Phase:** T1 (second of five tooling phases for the OTH4 programme)
**Anchor docs:** [PLAN.md](./PLAN.md) §6, [oth4-00-tooling.md](./oth4-00-tooling.md) §3.2 and §3.3
**Predecessor:** [`oth4-T0-spec.md`](./oth4-T0-spec.md) — T0 merged 2026-05-04, branch `feat/oth4-t0-quartal-duality-verification`
**Repository:** `mt-rs` (Rust music-theory library)

---

## 1. Context (just enough)

T0 added `quartal/duality.rs` and `quartal/verification.rs` to bring the quartal module to organisational symmetry with quintal on those two analytical surfaces. T1 closes two more gaps: a quartal-side **display** module (mirroring `quintal/display.rs`) and a quartal-side **centrality** module (giving quartal-labelled views over the perspective-invariant betweenness values that already live in quintal).

After T1 lands, two gaps remain on the quartal side: `quartal/functional.rs` (T2) and the MCP tool surface (T3). After all four are in, the walkthrough begins (T4).

Both T1 modules wrap or relabel existing quintal output — there is no new mathematical content. The display module reproduces the [6,8]-legal-walk algorithm with a [4,5,6] predicate (justified duplication; documented in §10 below). The centrality module aggregates betweenness values up to the `QuartalOrbit` level and pairs the saddle PcChords with their quartal IS for narrative readability.

T1 should be small, mechanical, and well-tested. Estimated: ~100 lines source + ~150 lines tests, similar in scale to T0.

---

## 2. Files to Create

| Path | Purpose |
|---|---|
| `crates/mt/src/quartal/centrality.rs` | Quartal-labelled centrality views |
| `crates/mt/src/quartal/display.rs` | Quartal-perspective chord and IS rendering |
| `crates/mt/tests/quartal/test_centrality.rs` | Integration tests for the new centrality module |
| `crates/mt/tests/quartal/test_display.rs` | Integration tests for the new display module |

## 3. Files to Modify

| Path | Change |
|---|---|
| `crates/mt/src/quartal/mod.rs` | Declare `mod centrality;` and `mod display;`; add `pub use` re-exports |
| `crates/mt/tests/quartal/mod.rs` | Add `mod test_centrality;` and `mod test_display;` |

The `mod.rs` declarations are alphabetical; insert as appropriate. **No other files** should change.

---

## 4. Spec — `crates/mt/src/quartal/display.rs`

### 4.1 Module-level intent

Quartal-perspective rendering. The mathematical content of "render a chord as a dashed note-name string" is identical across perspectives; what differs is the **walk algorithm** that determines which pc starts the rendering and which order the rest follow. Quintal walks find the [6,8]-legal stacking; quartal walks find the [4,5,6]-legal stacking. Same chord, different reading.

Module docstring should make the symmetry with `crate::quintal::display` explicit and explain the algorithmic relationship.

### 4.2 Public surface

```rust
//! Quartal-perspective rendering helpers — symmetric counterpart to
//! [`crate::quintal::display`].
//!
//! Two perspectives, two stack walks. [`render_quartal_chord_dashed`] walks
//! a [`PcChord`]'s pcs by [4,5,6]-legal forward intervals; the quintal
//! counterpart [`crate::quintal::render_chord_dashed`] uses [6,7,8]. Both
//! render the same chord, but in different stacking orders.
//!
//! Set-class identity is perspective-invariant; for that, re-export
//! [`crate::quintal::render_pcset_dashed`] as-is.

use super::types::QuartalIntervalStructure;
use crate::quintal::PcChord;

// Re-exports of perspective-invariant utilities.
pub use crate::quintal::pc_to_note_name;
pub use crate::quintal::render_pcset_dashed;

/// Render a [`PcChord`] in quartal root-form — pcs ordered by the chord's
/// canonical [4,5,6]-legal stack walk, dashed by en-dash.
///
/// For Q777 / Q555 (pcs `[0, 2, 7, 9]`) this yields `"A–D–G–C"` — the
/// same chord that [`crate::quintal::render_chord_dashed`] renders as
/// `"C–G–D–A"`, read here as a stack of fourths bottom-up.
///
/// For the Saddle (pcs `[0, 2, 6, 8]`) this yields `"D–G#–C–F#"`, with
/// forward intervals `(6, 4, 6)` — the canonical Q646 quartal IS.
///
/// # Algorithm
///
/// Identical in shape to [`crate::quintal::render_chord_dashed`] but with
/// the legal-interval predicate `(4..=6)` instead of `(6..=8)`. Scans all
/// 24 permutations of the chord's four pcs, retains those whose three
/// forward intervals are all in `{4, 5, 6}`, and returns the walk with
/// the smallest starting pc (lex tiebreak on the full `[p0, p1, p2, p3]`
/// array).
///
/// # Fallback
///
/// For chords with no [4,5,6]-legal walk (possible only for `PcChord`
/// values constructed outside [`crate::quintal::BaseSpace`]), falls back
/// to ascending-pc rendering — equivalent to [`render_pcset_dashed`].
/// Total over all `PcChord` values; never panics.
pub fn render_quartal_chord_dashed(chord: &PcChord) -> String {
    // Implementation: copy the algorithm from quintal/display::render_chord_dashed
    // verbatim, changing only the legal-interval predicate from `(6..=8)` to
    // `(4..=6)`. The PERMS_4 constant table is duplicated locally — extracting
    // a shared helper would touch quintal and is out of scope for T1
    // (justified duplication; D-quartal-T1-001 in §10 below).
    todo!()
}

/// Render a [`QuartalIntervalStructure`] as dashed semitone counts.
///
/// `(5, 5, 5) → "5–5–5"`, `(6, 4, 6) → "6–4–6"`, `(5, 5, 4) → "5–5–4"`.
///
/// Used in narrative documents and table outputs where the quartal IS
/// should appear as a single readable cell.
pub fn render_quartal_is(is: &QuartalIntervalStructure) -> String {
    format!("{}–{}–{}", is.0, is.1, is.2)
}
```

### 4.3 Implementation notes

- Use `–` (Unicode en-dash, U+2013) as the separator, matching `quintal::display`. Not a hyphen-minus.
- The `PERMS_4` permutation table is duplicated locally inside `render_quartal_chord_dashed`; do **not** widen quintal's API surface to expose it (decision **D-quartal-T1-001**: justified duplication; document in rustdoc).
- The fallback to `render_pcset_dashed` for non-[4,5,6]-legal chords mirrors quintal's fallback to ascending-pc rendering. Document the symmetry.
- Match quintal's `#[must_use]` attribute on the new `pub fn` items.

---

## 5. Spec — `crates/mt/src/quartal/centrality.rs`

### 5.1 Module-level intent

Quartal-labelled views over the perspective-invariant betweenness values produced by `quintal::betweenness_centrality`. The mathematical content (Brandes' algorithm, the f64 values themselves) lives in quintal. This module aggregates by orbit (`QuartalOrbit`) and pairs saddle PcChords with their quartal IS for direct narrative use.

The `Vec<PcChord>` returned by `quintal::saddle_chords` is already accessible via the `crate::quartal::saddle_chords` re-export (in `quartal/mod.rs:54`); we are not duplicating it. We are adding **two** new functions that surface quartal vocabulary:

### 5.2 Public surface

```rust
//! Quartal-labelled centrality views — symmetric counterpart to
//! [`crate::quintal::centrality`].
//!
//! The mathematical content (Brandes' algorithm, betweenness centrality
//! values) lives in quintal. This module surfaces those values keyed by
//! [`QuartalOrbit`] and [`QuartalIntervalStructure`] so quartal-minded
//! callers can read centrality in their native vocabulary without
//! quintal-side conversion at every call site.

use crate::quintal::{betweenness_centrality, BaseSpace, PcChord, classify_orbit};
use super::orbit::QuartalOrbit;
use super::types::QuartalIntervalStructure;

/// Saddle chords paired with their quartal interval structures.
///
/// Identical PcChord set to [`crate::quintal::saddle_chords`] (the top 6
/// chords by betweenness centrality — all members of the Saddle orbit
/// Q686 / Q646), paired with each chord's quartal IS for direct
/// quartal-perspective inspection.
///
/// All six pairs return `QuartalIntervalStructure(6, 4, 6)` — the
/// canonical Q646 quartal IS — because the Saddle orbit is palindromic.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_saddle_chords, base_space};
///
/// let space = base_space();
/// let saddle = quartal_saddle_chords(&space);
/// assert_eq!(saddle.len(), 6);
/// ```
pub fn quartal_saddle_chords(space: &BaseSpace) -> Vec<(PcChord, QuartalIntervalStructure)> {
    // Implementation: call crate::quintal::saddle_chords(space), pair each
    // PcChord with the quartal IS computed via
    // crate::quartal::pc_chord_quartal_intervals(&pc).
    // Return None-IS PcChords are not expected (saddle chords are always
    // members of Q686, which has a legal IS); if the conversion fails for
    // any reason, the function should panic with a clear message because
    // that indicates a bug elsewhere in the codebase.
    todo!()
}

/// All 14 quartal orbits ranked by maximum betweenness centrality of
/// any member chord (descending).
///
/// Ties are broken by the natural ordering of [`QuartalOrbit`] (the
/// declaration order in [`super::orbit`]).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_orbits_by_betweenness, base_space, QuartalOrbit};
///
/// let space = base_space();
/// let ranked = quartal_orbits_by_betweenness(&space);
/// assert_eq!(ranked.len(), 14);
/// // The Saddle (Q646) is at the top.
/// assert_eq!(ranked[0].0, QuartalOrbit::Q646);
/// ```
pub fn quartal_orbits_by_betweenness(space: &BaseSpace) -> Vec<(QuartalOrbit, f64)> {
    // Implementation:
    //   1. Compute betweenness_centrality(space) — one call, O(VE).
    //   2. For each PcChord, look up its Orbit via classify_orbit.
    //   3. Convert Orbit → QuartalOrbit via QuartalOrbit::from_quintal.
    //   4. Aggregate per QuartalOrbit: max betweenness across all members.
    //   5. Sort descending by max betweenness; secondary by QuartalOrbit
    //      declaration order.
    //   6. Return Vec of length 14.
    todo!()
}
```

### 5.3 Implementation notes

- `betweenness_centrality(space)` is O(V·E) — call exactly once per top-level call; do not invoke it inside an inner loop.
- `QuartalOrbit::from_quintal(&Orbit)` is the right conversion (see `quartal/orbit.rs`).
- For `quartal_saddle_chords`, the IS conversion uses `pc_chord_quartal_intervals` (already exported from `crate::quartal`). All six saddle chords have a legal IS, so `unwrap` is safe — but use `expect` with a message identifying the unexpected case rather than a bare `unwrap`, so a future regression surfaces the failing chord.
- `f64` ordering: use `partial_cmp(...).unwrap_or(Ordering::Equal)` for the sort closure; document the assumption that `betweenness_centrality` never produces NaN (it doesn't — Brandes' algorithm produces non-negative finite values for non-empty connected graphs).
- Match quintal's `#[must_use]` attribute on the new `pub fn` items where appropriate.

---

## 6. Spec — `crates/mt/src/quartal/mod.rs` updates

### 6.1 Add module declarations

After the existing `mod` block (currently `constructors, conversion, duality, error, interval, modes, orbit, types, verification, voicing`), insert `centrality` and `display` alphabetically. Resulting block:

```rust
mod centrality;
mod constructors;
mod conversion;
mod display;
mod duality;
mod error;
mod interval;
mod modes;
mod orbit;
mod types;
mod verification;
mod voicing;
```

### 6.2 Add public re-exports

Insert these `pub use` blocks alphabetically among the existing ones:

```rust
pub use centrality::{quartal_orbits_by_betweenness, quartal_saddle_chords};
pub use display::{
    pc_to_note_name, render_pcset_dashed, render_quartal_chord_dashed, render_quartal_is,
};
```

(Each block alphabetised within the brace.)

### 6.3 Do not modify

- Existing module declarations or `pub use` blocks (T0 left them in good shape).
- The `base_space()` helper.
- The `pub use crate::quintal::...` lines at the bottom of the file. Note that `pc_to_note_name` and `render_pcset_dashed` are now exposed via *both* the quartal display module and the existing quintal re-exports. That is acceptable — Rust allows re-exporting the same symbol from two paths — but if rustc emits a duplicate-symbol warning, prefer to expose via `display::` and remove redundant lines from the bottom-of-file re-exports. Document any such adjustment in the ledger.

---

## 7. Spec — Tests

### 7.1 `crates/mt/tests/quartal/test_display.rs`

Test cases (all should pass):

| Test | Assertion |
|---|---|
| `test_render_quartal_chord_q555_summit` | For `PcChord::new([0, 2, 7, 9])` (Q777 / Q555), `render_quartal_chord_dashed` returns `"A–D–G–C"`. |
| `test_render_quintal_chord_q555_summit_for_comparison` | For the same chord, `crate::quartal::render_chord_dashed`-equivalent (use `crate::quintal::render_chord_dashed` directly via `theory::quintal::render_chord_dashed`) returns `"C–G–D–A"`. Pin both to confirm the perspectives differ. |
| `test_render_quartal_chord_q646_saddle` | For `PcChord::new([0, 2, 6, 8])` (Q686 / Q646 Saddle), `render_quartal_chord_dashed` returns `"D–G#–C–F#"`. |
| `test_render_quartal_chord_q554_asymmetric` | For `PcChord::new([0, 3, 8, 10])` (Q877 / Q554, ASYMMETRIC), `render_quartal_chord_dashed` returns `"A#–D#–G#–C"`. The asymmetric case is the regression test for any future predicate-direction bug. |
| `test_render_quartal_pcset_re_export` | `render_pcset_dashed(&PcChord::new([0, 2, 7, 9]))` returns `"C–D–G–A"` — the perspective-invariant ascending form. |
| `test_pc_to_note_name_re_export` | `pc_to_note_name(0) == "C"`, `pc_to_note_name(8) == "G#"`, `pc_to_note_name(13) == "C#"` (mod-12 wrap). |
| `test_render_quartal_is_summit` | `render_quartal_is(&QuartalIntervalStructure(5, 5, 5))` returns `"5–5–5"`. |
| `test_render_quartal_is_saddle` | `render_quartal_is(&QuartalIntervalStructure(6, 4, 6))` returns `"6–4–6"`. |
| `test_render_quartal_is_asymmetric` | `render_quartal_is(&QuartalIntervalStructure(5, 5, 4))` returns `"5–5–4"`. The asymmetric case confirms component ordering is preserved. |

### 7.2 `crates/mt/tests/quartal/test_centrality.rs`

| Test | Assertion |
|---|---|
| `test_quartal_saddle_chords_count_and_is` | `quartal_saddle_chords(&base_space()).len() == 6`; every pair's IS equals `QuartalIntervalStructure(6, 4, 6)` (the Q646 / Q686 palindromic IS). |
| `test_quartal_saddle_chords_match_quintal` | The PcChord set returned by `quartal_saddle_chords` (drop the IS) equals the `Vec<PcChord>` returned by `crate::quintal::saddle_chords` (use `theory::quintal::saddle_chords`). |
| `test_quartal_orbits_by_betweenness_count` | `quartal_orbits_by_betweenness(&base_space()).len() == 14`. All 14 quartal orbits appear. |
| `test_quartal_orbits_by_betweenness_top_is_saddle` | The first entry of `quartal_orbits_by_betweenness(&space)` is `QuartalOrbit::Q646` (the Saddle has maximum betweenness). |
| `test_quartal_orbits_by_betweenness_descending` | The f64 values are non-strictly descending: for every consecutive pair, `bc[i] >= bc[i+1]`. |
| `test_quartal_orbits_by_betweenness_no_duplicates` | All 14 `QuartalOrbit` variants are present exactly once. |

Use `theory::quartal::*` imports paralleling the existing `tests/quartal/test_*.rs` style.

### 7.3 Test module wiring

Update `crates/mt/tests/quartal/mod.rs` to add `mod test_centrality;` and `mod test_display;` in alphabetical position. After insertion, the block should be:

```rust
mod test_centrality;
mod test_constructors;
mod test_display;
mod test_duality;
mod test_modes;
mod test_quartal_quintal_identity;
mod test_root_constructors;
mod test_types;
mod test_verification;
mod test_voicing;
```

---

## 8. Acceptance Criteria

T1 is complete when **all** of the following hold:

- [ ] `cargo build` succeeds with no new warnings.
- [ ] `cargo build --features midi` succeeds with no new warnings.
- [ ] `cargo test` passes — current main is **809 passing** (post-T0); after T1 the count should be **809 + N** where N is the number of new tests (15 expected from §7).
- [ ] `cargo test --features midi` passes with the same delta from baseline 814.
- [ ] `cargo clippy --all-targets` produces **zero new warnings** (baseline is currently zero — see §11 on the CLAUDE.md staleness note).
- [ ] `cargo clippy --all-targets --features midi` likewise.
- [ ] `cargo doc --no-deps` produces zero missing-docs and zero broken intra-doc links on the new symbols.
- [ ] All public API on the new modules has rustdoc with at least a one-line summary and at least one cross-reference to the quintal counterpart where one exists.
- [ ] No changes to files outside `crates/mt/src/quartal/`, `crates/mt/tests/quartal/`, and the new ledger doc.
- [ ] Serde feature flag honoured on any new public types (none expected — T1 adds no new public types beyond the existing ones it imports).

---

## 9. Ledger Report Template

After implementation, CC produces `oth4/oth4-T1-ledger.md` following the same template as T0. Reproduced here for self-containment:

```markdown
# oth4 — T1 Ledger Report

**Implementer:** Claude Code
**Date:** YYYY-MM-DD
**Branch / commit:** <branch>@<sha>
**Spec followed:** oth4-T1-spec.md (this repo's oth4/ folder)
**Plan followed:** docs/dev/quartal/<NNNN>-oth4-t1-...-plan.md

## 1. Summary
One paragraph: what was built, anything notable.

## 2. Files Created
List each new file with line count.

## 3. Files Modified
List each modified file with a one-sentence change summary.

## 4. Public API Surface Added
Every new public symbol, by module. List re-exports separately.

## 5. Tests Added
Per file, list test name and one-line description.

## 6. Test Results
- `cargo test`: <pass count>/<total>, time
- `cargo test --features midi`: <pass count>/<total>, time
- `cargo clippy --all-targets`: <warning count> (expected 0, since current main is at 0)
- `cargo clippy --all-targets --features midi`: <warning count>
- `cargo doc --no-deps`: <missing-doc count> + <broken-link count> (both expected 0)
- `cargo fmt --check`: clean / not clean

## 7. Specced but Skipped
Any item from §2–§7 of the spec that was *not* delivered, with rationale.

## 8. Added Beyond Spec
Any code or test added that the spec did not request, with rationale.

## 9. Judgment Calls
Any decisions where the spec was ambiguous and CC made a choice. Document
the choice and why.

## 10. Open Questions
Anything the implementer wants Duncan / Claude to resolve before T2.
```

---

## 10. Out of Scope (do not do)

- ❌ Adding any module other than `centrality.rs` and `display.rs`.
- ❌ Modifying `quintal/` in any way (including making `quintal::display::PERMS_4` public for sharing).
- ❌ Adding MCP tools — that's T3.
- ❌ Adding a `quartal/functional.rs` module — that's T2.
- ❌ Adding a `QuartalCentralitySummary` struct or `quartal_orbit_centrality_summary` function — deferred from the v2 tooling spec to a later phase, on the grounds that T1's two functions cover the immediate analytical-narrative needs.
- ❌ Refactoring `quintal/display`'s `render_chord_dashed` to extract a shared walk-finding helper. The duplication in `quartal/display.rs` is justified (decision **D-quartal-T1-001**); the shared-helper refactor is a separate concern that touches both modules and is out of scope here.
- ❌ Updating `CLAUDE.md` to correct the stale "4 module_inception warnings" expectation — see §11.
- ❌ Changing existing tests.
- ❌ Refactoring the existing quartal modules.
- ❌ Adding new dependencies to `Cargo.toml`.

If a spec point is unclear during implementation, prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 11. Explicit Non-Goal — CLAUDE.md cleanup

The T0 ledger §10 noted that `crates/mt/CLAUDE.md` says four `module_inception` clippy warnings are expected, but the actual baseline is **zero**. This is stale — likely from one of the recent quintal cleanup commits. T1 should treat the current state ("zero clippy warnings") as ground truth and gate on "zero new warnings." It should **not** edit CLAUDE.md.

A separate one-line CLAUDE.md correction commit is the right way to land this fix, and it can happen in parallel to T1 or after, at Duncan's discretion. T1 just needs to not depend on the stale claim.

---

## 12. Style Notes

- Match existing rustdoc conventions: triple-slash, intra-doc links via `[`Type`]`, examples on each public function (mirror the doctest-rich style of `quintal/display.rs` and `quintal/centrality.rs`).
- Match existing error style: panics in this T1 phase are limited to the `expect(...)` in `quartal_saddle_chords` (saddle-IS-conversion failure); no `Result` types are introduced.
- Use `super::` for sibling modules within `quartal/`, and `crate::quintal` for cross-module references.
- The Rust SKILL referenced in `crates/mt/CLAUDE.md` (`assets/ai/rust/SKILL.md`) is authoritative for general Rust style. T0 produced clean clippy and clean docs against this style; T1 should match.
- Doctest discipline: every public function in T0 had at least one doctest. Maintain that. The doctest examples should be runnable (`cargo doc --no-deps` and `cargo test --doc` should both pass).

---

## 13. Estimated Effort

Spec author's estimate: **1.5–2 hours** of focused implementation + test work for an experienced Rust developer with the codebase loaded. Source: ~100 lines (45 display + 55 centrality, including doctests). Tests: ~150 lines (9 display + 6 centrality + fixtures).

If the implementation expands beyond ~350 lines total (excluding the ledger), that's a signal that scope has crept and §10 should be revisited.

---

## 14. Decisions Recorded in This Spec

For convenient reference in the ledger:

| ID | Decision | Rationale |
|---|---|---|
| D-quartal-T1-001 | `render_quartal_chord_dashed` reproduces the walk algorithm from `quintal/display::render_chord_dashed` with the predicate changed from `(6..=8)` to `(4..=6)`. The `PERMS_4` constant table is duplicated locally. | Extracting a shared helper would touch quintal and is out of scope. The duplication is ~25 lines and self-contained. |
| D-quartal-T1-002 | T1 ships `quartal_saddle_chords` and `quartal_orbits_by_betweenness` only; the `QuartalCentralitySummary` struct from v2 tooling spec §3.3 is deferred. | The two functions cover the immediate walkthrough-document needs; the summary struct should be designed against the actual queries the walkthrough produces, not speculatively. |
| D-quartal-T1-003 | `pc_to_note_name` and `render_pcset_dashed` are re-exposed from `quartal/display` (in addition to the existing `pub use` lines at the bottom of `quartal/mod.rs`). If rustc emits a duplicate-symbol warning, prefer the `display::` route. | Symmetric module organisation — a quartal-minded reader expects to find rendering helpers in `display`, not at the bottom of `mod.rs`. |
| D-quartal-T1-004 | T1 does not edit `CLAUDE.md`. | The CLAUDE.md staleness fix is a separate concern; coupling it to T1 expands scope and risks the symmetry-only nature of this phase. |

---

*End of oth4-T1-spec.md*
