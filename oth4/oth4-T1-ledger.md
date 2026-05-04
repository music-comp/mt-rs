# oth4 — T1 Ledger Report

**Implementer:** Claude Code
**Date:** 2026-05-04
**Branch / commit:** `feat/oth4-t1-quartal-display-centrality` @ `d091eb5`
**Spec followed:** `docs/design/05-active/0009-oth4-t1-implementation-spec.md`
**Plan followed:** `docs/dev/quartal/0006-centrality-implementation-plan.md` (v2 — user-edited from CC's draft)

## 1. Summary

Added quartal-perspective `display` and `centrality` modules to `crates/mt/src/quartal/`, mirroring `crates/mt/src/quintal/display.rs` and `crates/mt/src/quintal/centrality.rs`. No new mathematics: the new modules wrap, relabel, or re-export existing quintal output. Implementation followed the v2 plan task-by-task with no surprises — the four pre-recorded D-quartal-T1 spec decisions were adopted as recommended, plus the two T1-specific findings (F-T1-001 confirming no `pc_to_note_name` re-export conflict, F-T1-002 reaffirming doctest discipline). One additional integration test was added beyond spec count to mirror an analogous quintal test (the non-legal-chord fallback regression). All four new public functions ship with `#[must_use]` attributes and runnable doctests.

## 2. Files Created

- `crates/mt/src/quartal/display.rs` — 154 lines. Two new public fns (`render_quartal_chord_dashed`, `render_quartal_is`), two `pub use` re-exports from `crate::quintal` (`pc_to_note_name`, `render_pcset_dashed`), and a private `PERMS_4` const (D-quartal-T1-001 justified duplication).
- `crates/mt/src/quartal/centrality.rs` — 117 lines. Two new public fns (`quartal_saddle_chords`, `quartal_orbits_by_betweenness`).
- `crates/mt/tests/quartal/test_display.rs` — 111 lines. 10 integration tests.
- `crates/mt/tests/quartal/test_centrality.rs` — 92 lines. 6 integration tests.
- `oth4/oth4-T1-ledger.md` — this file.

## 3. Files Modified

- `crates/mt/src/quartal/mod.rs` — added two `mod` declarations (`centrality`, `display`) and two `pub use` blocks. No other lines changed.
- `crates/mt/tests/quartal/mod.rs` — added two `mod` declarations (`test_centrality`, `test_display`) in alphabetical position. No other lines changed.

## 4. Public API Surface Added

New functions:

- `quartal::render_quartal_chord_dashed(chord: &PcChord) -> String`
- `quartal::render_quartal_is(is: &QuartalIntervalStructure) -> String`
- `quartal::quartal_saddle_chords(space: &BaseSpace) -> Vec<(PcChord, QuartalIntervalStructure)>`
- `quartal::quartal_orbits_by_betweenness(space: &BaseSpace) -> Vec<(QuartalOrbit, f64)>`

All four carry `#[must_use]` and at least one runnable doctest.

New re-exports (from `crate::quintal`, surfaced through `crate::quartal::display`):

- `quartal::pc_to_note_name`
- `quartal::render_pcset_dashed`

No new public types. No changes to existing public types or signatures. Cargo.toml / Cargo.lock unchanged (no new dependencies).

## 5. Tests Added

### `crates/mt/tests/quartal/test_display.rs` (10 integration tests)

| Test | Description |
|---|---|
| `test_render_quartal_chord_q555_summit` | Q555 Summit (palindromic): pcs `[0,2,7,9]` → `"A–D–G–C"`. |
| `test_render_quintal_chord_q555_summit_for_comparison` | Pins both quartal and quintal renderers on the same chord; asserts they differ. |
| `test_render_quartal_chord_q646_saddle` | Q646 Saddle (palindromic, two legal walks, smallest-start tiebreak): pcs `[0,2,6,8]` → `"D–G#–C–F#"`. |
| `test_render_quartal_chord_q554_asymmetric` | Q554 ASYMMETRIC: pcs `[0,3,8,10]` → `"A#–D#–G#–C"`. Catches predicate-direction bugs. |
| `test_render_quartal_chord_falls_back_for_non_legal_chord` | Non-[4,5,6]-legal chord falls back to ascending-pc rendering (mirrors quintal-side test). |
| `test_render_quartal_pcset_re_export` | `render_pcset_dashed` reachable via `crate::quartal::*`. |
| `test_pc_to_note_name_re_export` | `pc_to_note_name` reachable via `crate::quartal::*`; mod-12 wrap pinned. |
| `test_render_quartal_is_summit` | `(5,5,5) → "5–5–5"`. |
| `test_render_quartal_is_saddle` | `(6,4,6) → "6–4–6"`. |
| `test_render_quartal_is_asymmetric` | `(5,5,4) → "5–5–4"`. Pins component-ordering invariance. |

### `crates/mt/tests/quartal/test_centrality.rs` (6 integration tests)

| Test | Description |
|---|---|
| `test_quartal_saddle_chords_count_and_is` | 6 pairs returned; every IS equals `QuartalIntervalStructure(6, 4, 6)`. |
| `test_quartal_saddle_chords_match_quintal` | Dropping the IS recovers `quintal::saddle_chords` exactly. |
| `test_quartal_orbits_by_betweenness_count` | Returns 14 orbits. |
| `test_quartal_orbits_by_betweenness_top_is_saddle` | First entry is `QuartalOrbit::Q646`; saddle strictly exceeds the next. |
| `test_quartal_orbits_by_betweenness_descending` | All consecutive pairs are non-strictly descending. |
| `test_quartal_orbits_by_betweenness_no_duplicates` | All 14 `QuartalOrbit` variants appear exactly once. |

### Doctests added (4 new)

- `display::render_quartal_chord_dashed` — Q555 Summit and Q646 Saddle examples.
- `display::render_quartal_is` — `(5,5,5)` and `(5,5,4)` examples.
- `centrality::quartal_saddle_chords` — basic length assertion.
- `centrality::quartal_orbits_by_betweenness` — top-orbit assertion.

**Total tests added:** 16 integration + 4 doctests = 20.

## 6. Test Results

| Run | Result | Time |
|---|---|---|
| `cargo test` (default features) | 58 + 86 + 619 + 34 + 6 + 26 = **829 passed**, 0 failed | ~3.8s wall (full suite) |
| `cargo test --features midi` | 63 + 86 + 619 + 34 + 6 + 26 = **834 passed**, 0 failed | ~3.8s wall |
| `cargo build` (default) | clean, 0 new warnings | <1s incremental |
| `cargo build --features midi` | clean, 0 new warnings | <1s incremental |
| `cargo clippy --all-targets` (default) | **0 warnings** | <1s incremental |
| `cargo clippy --all-targets --features midi` | **0 warnings** | <1s incremental |
| `cargo doc --no-deps` | **0 warnings**, 0 missing-docs, 0 broken intra-doc links | ~1s incremental |
| `cargo fmt --check` | clean (empty diff) | <1s |

Baseline (pre-T1, post-T0 merge): 809 default / 814 midi / 0 clippy / 0 doc. Delta: +20 in both feature flags (matching the 16 integration + 4 doctest additions).

## 7. Specced but Skipped

None. Every spec §2/§3 file was created or modified. Every spec §4.2 and §5.2 public function landed with the documented signatures. Every spec §7.1 and §7.2 test landed (with the additional non-legal-chord fallback test from §8 below). Spec §9 ledger template followed verbatim. Spec §10 out-of-scope rules respected.

## 8. Added Beyond Spec

- `test_render_quartal_chord_falls_back_for_non_legal_chord` — mirrors the analogous quintal test (`render_chord_dashed_falls_back_for_non_legal_chord` in `tests/quintal/test_display.rs`). Pins the documented total-function behaviour and is a regression guard for the fallback path. Adds ~5 lines.
- Doctests on `render_quartal_chord_dashed` and `render_quartal_is` — the spec body §4.2 did not include doctests for these, but spec §12 reaffirms doctest discipline. Added per F-T1-002 (plan §2). Adds ~22 lines of doc-comment block (counted toward `display.rs` line count).
- `#[must_use]` attribute on all four new public functions — matches the convention in `quintal/display.rs`. Spec §4.3 mentions this for `display`; applied to `centrality` too for symmetry.

No additional production code or signatures beyond what the spec requested.

## 9. Judgment Calls

- **D-quartal-T1-001 (spec):** `render_quartal_chord_dashed` reproduces the walk algorithm + `PERMS_4` constant locally rather than widening quintal's API. Adopted as-is. The duplication is documented inline in the rustdoc on `PERMS_4` and in the algorithm section of `render_quartal_chord_dashed`.
- **D-quartal-T1-002 (spec):** `QuartalCentralitySummary` struct deferred. T1 ships only `quartal_saddle_chords` and `quartal_orbits_by_betweenness` — no struct.
- **D-quartal-T1-003 (spec, refined):** `pc_to_note_name` and `render_pcset_dashed` re-exposed via `quartal/display`. Per F-T1-001 there is no existing `pub use crate::quintal::pc_to_note_name` to compete with, so no `mod.rs` cleanup was needed. The bottom-of-file `pub use crate::quintal::{...}` block was left untouched.
- **D-quartal-T1-004 (spec):** T1 does not edit `CLAUDE.md`. The "4 module_inception warnings" expectation is stale (the actual baseline is 0); a separate one-line correction commit can land outside T1 scope.
- **F-T1-002 resolution:** added doctests to all four new public functions. Aligns T1 with spec §12 stance even though T0 did not.
- **F-T1-003 (saddle walk count):** plan v2 noted that Q646 Saddle admits **two** legal quartal walks (not one — the orbit has a non-trivial T6 stabilizer). Both walks have identical `(6, 4, 6)` interval structure; the smallest-start tiebreak picks `[2, 8, 0, 6]` deterministically. Test outcomes are unaffected; the plan's analysis was simply tightened to match the quintal-side documentation precedent.

## 10. Open Questions

- **CLAUDE.md staleness:** the "4 module_inception warnings expected" claim is now flagged in two consecutive ledger reports (T0 §10 and T1 here). The fix is a one-line edit; flagging again so it doesn't get lost.
- **Re-export style (D-quartal-T1-003):** the spec body §6.3 anticipates a possible duplicate-symbol scenario. With `pc_to_note_name` and `render_pcset_dashed` now exposed from `quartal::display::*`, future symmetric phases (T2 functional, T3 MCP) might want to standardise on "always re-export through the topical submodule, never from the bottom of `mod.rs`". Worth a CDC stance before T2.
- **`QuartalCentralitySummary`:** D-quartal-T1-002 deferred this struct. The walkthrough (T4) will be the natural place to evaluate whether the two new functions cover the actual queries; the struct can be designed against those queries rather than speculatively. Flagging for the T2/T3 specs to revisit.
