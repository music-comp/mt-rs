# oth4 — T0 Ledger Report

**Implementer:** Claude Code
**Date:** 2026-05-04
**Branch / commit:** `feat/oth4-t0-quartal-duality-verification` @ `b8e54ff`
**Spec followed:** `docs/design/05-active/0008-oth4-t0-implementation-spec.md`
**Plan followed:** `docs/dev/quartal/0005-oth4-t0-quartal-duality-verification-implementation-plan.md` (v2)

## 1. Summary

Added a quartal-perspective `duality` module and `verification` module to `crates/mt/src/quartal/`, mirroring the organisational shape of `crates/mt/src/quintal/duality.rs` and `crates/mt/src/quintal/verification.rs`. No new mathematics: the new modules wrap or delegate to the underlying quintal apparatus. Implementation followed the v2 plan task-by-task with no surprises. Three judgment calls were pre-flagged in the plan and adopted as recommended: (D-T0-001/002) `verify_*` signatures match quintal's richer `Result<...>` / `BTreeMap<...>` shapes rather than the spec body's sketched `bool`; (D-T0-003) test fixtures voice the chord PcChord via legal quintal stacking, then wrap via `to_quartal`; (D-T0-004) `quartal_reading` delegates to `chord.quartal_interval_structure()` to avoid the directional pitfall in the spec body's composition. Two asymmetric-chord regression tests (Q554) added per the v2 revision history.

## 2. Files Created

- `crates/mt/src/quartal/duality.rs` — 74 lines. Three new fns (`quartal_reading`, `quintal_reading`, `t_quartal_reversal_equivalence`) plus three `pub use` re-exports from `crate::quintal` (`reverse_interval_structure`, `orbit_self_duality`, `verify_all_orbits_self_dual`).
- `crates/mt/src/quartal/verification.rs` — 133 lines. Two public verifier fns (`verify_quartal_universal_l1_law`, `verify_quartal_fiber_classes`), one private helper (`pc_chord_to_quartal_voiced`), and one `#[cfg(test)]` unit test.
- `crates/mt/tests/quartal/test_duality.rs` — 112 lines. Three fixture helpers + 10 integration tests.
- `crates/mt/tests/quartal/test_verification.rs` — 69 lines. 5 integration tests.
- `oth4/oth4-T0-ledger.md` — this file.

## 3. Files Modified

- `crates/mt/src/quartal/mod.rs` — added two `mod` declarations (`duality`, `verification`) and two `pub use` blocks. No other lines changed.
- `crates/mt/tests/quartal/mod.rs` — added two `mod` declarations (`test_duality`, `test_verification`) in alphabetical position. No other lines changed.

## 4. Public API Surface Added

New functions:

- `quartal::quartal_reading(chord: &QuartalVoicedChord) -> QuartalIntervalStructure`
- `quartal::quintal_reading(chord: &QuartalVoicedChord) -> IntervalStructure`
- `quartal::t_quartal_reversal_equivalence(chord: &QuartalVoicedChord) -> bool`
- `quartal::verify_quartal_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>`
- `quartal::verify_quartal_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>`

New re-exports (from `crate::quintal`, surfaced through `crate::quartal::duality`):

- `quartal::reverse_interval_structure`
- `quartal::orbit_self_duality`
- `quartal::verify_all_orbits_self_dual`

No new public types. No changes to existing public types or signatures.

## 5. Tests Added

### `crates/mt/tests/quartal/test_duality.rs` (10 tests)

| Test | Description |
|---|---|
| `test_quartal_reading_q555_summit` | Summit Q555 (palindromic) → `QuartalIntervalStructure(5,5,5)`. |
| `test_quintal_reading_q555_summit` | Same chord, dual quintal reading → `IntervalStructure(7,7,7)`. |
| `test_quartal_reading_q646_saddle` | Saddle Q646 (palindromic) → `QuartalIntervalStructure(6,4,6)`. |
| `test_quintal_reading_q646_saddle` | Same chord, dual quintal reading → `IntervalStructure(6,8,6)`. |
| `test_quartal_reading_q554_asymmetric` | Q554 asymmetric (bottom-up `(8,7,7)`) → `QuartalIntervalStructure(5,5,4)`. Catches directional bugs. |
| `test_quintal_reading_q554_asymmetric` | Same chord → `IntervalStructure(8,7,7)`. |
| `test_t_quartal_reversal_equivalence_holds` | Sample of four orbit representatives all satisfy reversal equivalence. |
| `test_self_duality_re_export` | `verify_all_orbits_self_dual` reachable via `crate::quartal::*` and returns `true` on canonical base space. |
| `test_orbit_self_duality_re_export` | `orbit_self_duality(&Orbit::Q777, &space)` reachable via `crate::quartal::*` and returns `true`. |
| `test_reverse_interval_structure_re_export` | `reverse_interval_structure` reachable via `crate::quartal::*` (`(7,7,6) → (6,7,7)`). |

### `crates/mt/tests/quartal/test_verification.rs` (5 tests)

| Test | Description |
|---|---|
| `test_quartal_universal_l1_law` | `verify_quartal_universal_l1_law(&space).is_ok()`. |
| `test_quartal_fiber_classes_count` | Returns 14 orbits with 11 ClassA + 3 ClassB; Q676/Q686/Q688 are the ClassB members. |
| `test_quartal_l1_pattern_summit` | `quartal_l1_distances` of the Q555 Summit = `[12,12,12,36]`. |
| `test_quartal_l1_pattern_saddle` | `quartal_l1_distances` of the Q686 saddle = `[12,12,12,36]`. |
| `test_quartal_verifiers_agree_with_quintal` | Quartal and quintal verifiers agree pointwise (BTreeMap equality + both Universal L1 Law `.expect()` succeeds). |

### `crates/mt/src/quartal/verification.rs` (1 unit test)

| Test | Description |
|---|---|
| `tests::helper_round_trips_q777` | `pc_chord_to_quartal_voiced(&PcChord([0,2,7,9])).0.to_pc_chord()` round-trips, and the chord classifies as `Orbit::Q777`. |

**Total tests added:** 16 (10 duality integration + 5 verification integration + 1 verification unit).

## 6. Test Results

| Run | Result | Time |
|---|---|---|
| `cargo test` (default features) | 58 lib + 86 + 603 + 34 + 6 + 22 doc = **809 passed**, 0 failed | ~3.8s wall |
| `cargo test --features midi` | 63 lib + 86 + 603 + 34 + 6 + 22 doc = **814 passed**, 0 failed | ~3.8s wall |
| `cargo build` | clean exit, 0 new warnings | 1.94s incremental |
| `cargo build --features midi` | clean exit, 0 new warnings | <1s incremental |
| `cargo clippy --all-targets` | **0 warnings** | 2.20s |
| `cargo clippy --all-targets --features midi` | **0 warnings** | <1s incremental |
| `cargo doc --no-deps` | **0 warnings**, 0 missing-docs, 0 broken intra-doc-links | 0.73s |

Baseline (pre-change) test counts: 793 default / 798 midi. Delta = +16 in both runs (matching the 16 tests added).

**Note on the 4 expected `module_inception` warnings (CLAUDE.md):** the actual baseline was **0 warnings**, not 4. CLAUDE.md is out of date on this point — flagging for follow-up cleanup outside this T0 scope.

## 7. Specced but Skipped

None. Every spec §2/§3 file was created or modified. Every spec §4.2 and §5.2 public function landed (with the type-signature adaptations recorded in §9 below). Every spec §7.1 and §7.2 test landed (with the fixture-construction tightening recorded in §9). Spec §9 ledger template followed verbatim. Spec §10 out-of-scope rules respected.

## 8. Added Beyond Spec

- `test_quartal_reading_q554_asymmetric` and `test_quintal_reading_q554_asymmetric` — asymmetric-chord regression tests added per plan v2 revision history. The spec's two test chords (Q555 Summit, Q646 Saddle) both have palindromic interval structures, so a buggy `quartal_reading` that reverses its result would still pass. Q554 (bottom-up `(8,7,7)`) catches that bug.
- `test_orbit_self_duality_re_export` and `test_reverse_interval_structure_re_export` — the spec §7.1 only explicitly tests one re-export (`verify_all_orbits_self_dual`); these add the same coverage to the other two. The plan v2 §3 file structure budgets ~110 lines for this file (8 spec + 2 asymmetric); the additional re-export tests fit within that budget.
- `verification::tests::helper_round_trips_q777` — `#[cfg(test)]` unit test inside `verification.rs` that pins the round-trip property of the private `pc_chord_to_quartal_voiced` helper. Useful regression coverage for an otherwise-private function with no other test reach.

## 9. Judgment Calls

- **D-T0-001:** `verify_quartal_universal_l1_law` returns `Result<(), Vec<PcChord>>` (matching `crate::quintal::verify_universal_l1_law`), not `bool` as the spec §5.2 body sketched. Spec §5.3 explicitly licenses this: "match that signature unless the existing quintal version returns a richer type. Inspect `quintal/verification.rs` and follow its style." Rich return preserves diagnostic value (the failing chord list) and lets the two perspectives' verifiers be diff'd line-for-line.
- **D-T0-002:** `verify_quartal_fiber_classes` returns `BTreeMap<Orbit, FiberClass>` (matching `crate::quintal::verify_fiber_classes`), not `bool`. Same rationale as D-T0-001. The "verify" semantic is preserved by `test_quartal_fiber_classes_count` asserting the map has 14 orbits with the expected 11 ClassA + 3 ClassB split.
- **D-T0-003:** Spec §7.1 wording "constructed via `pure_quartal_stack(0)`" interpreted as "the PcChord from `pure_quartal_stack(0)`, voiced via legal quintal stacking and wrapped via `to_quartal`" — the only construction under which `quartal_reading` of the Q555 Summit yields `(5,5,5)` and the saddle yields `(6,4,6)`. Tests use explicit `VoicedChord::new([51, 58, 65, 72])` / `VoicedChord::new([48, 54, 62, 68])` rather than the ambiguous `pure_quartal_stack`-then-voice path so the intent is unambiguous in the source.
- **D-T0-004 (plan v2):** `quartal_reading` body delegates to `chord.quartal_interval_structure()` rather than composing `quintal::quartal_reading` (top-down) with `quintal_to_quartal_structure` (bottom-up input). The composition was buggy on asymmetric chords — the directional mismatch silently inverted the result. Discovered during plan v2 review; both Q554 regression tests (above) explicitly catch the regression.
- **D-quartal-T0-001:** `pc_chord_to_quartal_voiced` reproduces ~10 lines of logic from the private `crate::quintal::verification::pc_chord_to_voiced` rather than widening quintal's API surface. Spec §10 explicitly forbids modifying `quintal/`; the helper duplication is a clean ~10-line copy with the additional `to_quartal(&vc)` wrap at the return.

## 10. Open Questions

- **`module_inception` warning count:** CLAUDE.md says 4 expected; actual is 0. Either the warnings have been silenced by recent cleanup commits and CLAUDE.md is stale, or the toolchain version is producing different lint output. Suggest a follow-up CLAUDE.md correction (out of T0 scope).
- **Quartal-rooted vs quintal-rooted QVC fixtures:** the test fixtures use the *quintal-stacked* voicing of each PcChord wrapped via `to_quartal`. An alternative interpretation of `pure_quartal_stack` would build the *quartal-stacked* voicing (e.g., `[60, 65, 70, 75]` for Q555). Both are legitimate `QuartalVoicedChord` instances with different MIDI pitches but the same pitch-class set. The plan locked in the quintal-stacked reading (D-T0-003); whether the spec author intended the quartal-stacked alternative is worth a quick CDC confirmation.
- **`verify_quartal_fiber_classes` vs `verify_fiber_classes`:** both return identical maps on the canonical base space (asserted by `test_quartal_verifiers_agree_with_quintal`). The *quartal-side* verifier's added value is exercising `quartal_inversion_cycle` end-to-end as a regression check; if T1 introduces additional quartal-direction state, this verifier is the natural place to extend.
