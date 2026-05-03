---
number: 8
title: "Quartal Implementation Spec"
author: "Ozan Kasikci"
component: All
tags: [change-me]
created: 2026-05-03
updated: 2026-05-03
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# Quartal Implementation Spec

**Status:** Ready for handoff to Claude Code, 2026-05-03
**Phase:** T0 (first of five tooling phases for the OTH4 programme)
**Anchor docs:** [PLAN.md](./PLAN.md) §6, [oth4-00-tooling.md](./oth4-00-tooling.md) §3.4 and §3.5
**Repository:** `mt-rs` (the Rust music-theory library)

---

## 1. Context (just enough)

The mt-rs codebase has a comprehensive `quintal/` module containing the full topological apparatus (base space, distance, centrality, geodesics, fiber bundle, duality, functional grammar, verification). The `quartal/` module is currently a thin perspective layer — it defines its own types (`QuartalVoicedChord`, `QuartalIntervalStructure`, `QuartalOrbit`), constructors, conversions, and voicing operations, but lacks symmetric counterparts to several quintal analytical modules.

T0 closes the smallest two gaps: a quartal-side **duality** module (organisationally symmetric to `quintal/duality.rs`) and a quartal-side **verification** module (mirroring `quintal/verification.rs`). Both wrap or delegate to the underlying quintal mathematics — there is no new mathematical content. The point is codebase symmetry: a quartal-minded reader should find quartal-perspective entry points without grepping the quintal namespace.

After T0 lands, T1 (centrality + display), T2 (functional), T3 (MCP), and T4 (begin walkthrough) follow. T0 should be small, mechanical, and well-tested.

---

## 2. Files to Create

| Path | Purpose |
|---|---|
| `crates/mt/src/quartal/duality.rs` | Quartal-perspective duality functions and re-exports |
| `crates/mt/src/quartal/verification.rs` | Quartal-perspective verification of structural laws |
| `crates/mt/tests/quartal/test_duality.rs` | Integration tests for the new duality module |
| `crates/mt/tests/quartal/test_verification.rs` | Integration tests for the new verification module |

## 3. Files to Modify

| Path | Change |
|---|---|
| `crates/mt/src/quartal/mod.rs` | Declare `mod duality;` and `mod verification;`; add `pub use` re-exports |
| `crates/mt/tests/quartal/mod.rs` | Add `mod test_duality;` and `mod test_verification;` |

No other files in the workspace should change.

---

## 4. Spec — `crates/mt/src/quartal/duality.rs`

### 4.1 Module-level intent

Symmetric organisation with `quintal/duality.rs`. The mathematical content of duality already lives in quintal — this module gives quartal-minded readers a natural home for the same operations, accepting `QuartalVoicedChord` and returning `QuartalIntervalStructure` where natural, plus a quartal-direction fiber-reversal verifier.

Module docstring should make this purpose explicit: this is a *perspective layer*, not new math.

### 4.2 Public surface

```rust
//! Quartal-perspective duality — symmetric counterpart to [`crate::quintal::duality`].
//!
//! The mathematical content of the quartal/quintal duality lives in
//! [`crate::quintal::duality`]. This module provides a quartal-native interface
//! for the same operations: functions taking [`QuartalVoicedChord`] and
//! returning [`QuartalIntervalStructure`] where natural, plus a quartal-direction
//! fiber-reversal verifier.
//!
//! See [`crate::quintal::duality`] for the underlying theorems and proofs.

use super::interval::quintal_to_quartal_structure;
use super::types::{QuartalIntervalStructure, QuartalVoicedChord};
use crate::quintal;

/// Read a quartal voicing's intervals top-to-bottom — the natural quartal reading.
///
/// Equivalent to applying [`crate::quintal::quartal_reading`] to the inner
/// [`crate::quintal::VoicedChord`] and converting the result to a
/// [`QuartalIntervalStructure`] (which already represents fourths-complemented
/// values in {4, 5, 6}).
pub fn quartal_reading(chord: &QuartalVoicedChord) -> QuartalIntervalStructure {
    let q5_is = quintal::quartal_reading(&chord.0);
    quintal_to_quartal_structure(&q5_is)
}

/// Read a quartal voicing's intervals bottom-to-top — the dual (quintal) reading.
///
/// Returns the underlying [`crate::quintal::IntervalStructure`] in {6, 7, 8}.
/// This is the quintal interpretation of the same pitch collection.
pub fn quintal_reading(chord: &QuartalVoicedChord) -> quintal::IntervalStructure {
    quintal::quintal_reading(&chord.0)
}

/// Verify that the quartal traversal `t_quartal` traverses the same fiber
/// as `t1` in reversed order.
///
/// Symmetric counterpart to [`crate::quintal::t1_reversal_equivalence`].
/// Because `t_quartal == t_minus1` and `t_minus1` is the inverse of `t1`,
/// the quartal inversion cycle visits the four members of the fiber in
/// the reverse pitch-class order.
///
/// Returns `true` if the reversal equivalence holds for the given chord.
pub fn t_quartal_reversal_equivalence(chord: &QuartalVoicedChord) -> bool {
    quintal::t1_reversal_equivalence(&chord.0)
}

// Symmetric re-exports from the quintal-side duality module.
// These functions operate on quintal types but are conceptually shared;
// re-exporting here lets quartal-minded readers find them.

/// Reverse an [`crate::quintal::IntervalStructure`]: `(a, b, c) → (c, b, a)`.
///
/// Re-exported from [`crate::quintal::reverse_interval_structure`].
pub use crate::quintal::reverse_interval_structure;

/// Check whether a single [`crate::quintal::Orbit`] is self-dual under
/// interval-structure reversal.
///
/// Re-exported from [`crate::quintal::orbit_self_duality`].
pub use crate::quintal::orbit_self_duality;

/// Verify that all 14 orbits are self-dual.
///
/// Re-exported from [`crate::quintal::verify_all_orbits_self_dual`].
pub use crate::quintal::verify_all_orbits_self_dual;
```

### 4.3 Implementation notes

- The two reading functions do **not** return early on illegal chords. The underlying quintal functions don't either; they return whatever the IS algebra produces. This matches existing quintal-side behaviour.
- `t_quartal_reversal_equivalence` delegates to the inner `VoicedChord` because the fiber-reversal property is independent of perspective — it's a statement about pitch-class sets, which both perspectives share.
- Use the `quintal_to_quartal_structure` function from `super::interval` rather than reconstructing the reversal-and-complementation manually.

---

## 5. Spec — `crates/mt/src/quartal/verification.rs`

### 5.1 Module-level intent

Quartal-perspective verifications of structural laws. Mirrors `quintal/verification.rs` in shape and intent. Each verifier returns a boolean; failure should be debuggable via per-chord inspection (consider returning richer types if the existing quintal verifiers do — match their style).

### 5.2 Public surface

```rust
//! Quartal-perspective verification of structural laws — symmetric
//! counterpart to [`crate::quintal::verification`].
//!
//! The Universal L1 Law and fiber-class assignment are perspective-independent
//! statements about the shared base space, but having a quartal-direction
//! verifier is useful both as documentation and as a check that the quartal
//! traversal (`t_quartal`, `quartal_inversion_cycle`, `quartal_l1_distances`)
//! produces the expected results.

use super::types::QuartalVoicedChord;
use super::voicing::quartal_l1_distances;
use crate::quintal;

/// Verify the Universal L1 Law in the quartal direction.
///
/// For every legal [`QuartalVoicedChord`] derived from the 228 PcChords,
/// [`crate::quartal::quartal_l1_distances`] must return `[12, 12, 12, 36]`.
///
/// Returns `true` iff the law holds for every chord.
///
/// This is a quartal-direction restatement of
/// [`crate::quintal::verify_universal_l1_law`]; both should return `true`
/// on a correct implementation.
pub fn verify_quartal_universal_l1_law() -> bool {
    let chords = quintal::enumerate_all();
    for pc in &chords {
        // Convert each PcChord to a QuartalVoicedChord using the same
        // strategy as quintal::verification (default register starting at MIDI 48).
        // If a chord has no legal voicing, skip it — it contributes nothing
        // to the law's claim.
        let Some(qvc) = pc_chord_to_quartal_voiced(pc) else {
            continue;
        };
        if quartal_l1_distances(&qvc) != [12, 12, 12, 36] {
            return false;
        }
    }
    true
}

/// Verify all 14 quartal-orbit fiber classes via quartal traversal.
///
/// Symmetric counterpart to [`crate::quintal::verify_fiber_classes`].
/// Returns `true` iff every chord's quartal-traversal fiber class
/// agrees with its quintal-traversal fiber class.
pub fn verify_quartal_fiber_classes() -> bool {
    // Implementation strategy:
    // For each PcChord:
    //   1. Compute the quintal fiber_class via crate::quintal::fiber_class
    //   2. Compute the quartal-traversal fiber via quartal_inversion_cycle
    //   3. Verify that the quartal cycle visits the same set of pitch-class
    //      chords as the quintal inversion_cycle (which is guaranteed by
    //      the duality, but pinned here as a regression test).
    //
    // This function should agree with crate::quintal::verify_fiber_classes()
    // on every chord; the value of running it separately is to verify that
    // the quartal traversal infrastructure produces consistent results.
    //
    // Implementer: choose a clear implementation; the docstring above is
    // intent, not literal pseudocode.
    todo!()
}

/// Helper: construct a [`QuartalVoicedChord`] from a [`crate::quintal::PcChord`]
/// in a default register starting at MIDI 48 (C3).
///
/// Returns `None` if the chord has no legal quintal interval structure.
/// Mirrors the helper in [`crate::quintal::verification`] (private there;
/// reproduced here rather than exported, to keep quintal's API surface stable).
fn pc_chord_to_quartal_voiced(chord: &quintal::PcChord) -> Option<QuartalVoicedChord> {
    // Suggested implementation: delegate to a quintal helper if one is
    // public, or replicate the quintal::verification::pc_chord_to_voiced
    // logic and wrap the result via to_quartal.
    //
    // Note: quintal::verification::pc_chord_to_voiced is private. The
    // implementer should either:
    //   (a) add a pub(crate) wrapper in quintal::verification and use it here, OR
    //   (b) reproduce the logic locally (8 lines).
    // Prefer (b) to keep quintal's API surface unchanged.
    todo!()
}
```

### 5.3 Implementation notes

- `verify_fiber_classes` in quintal returns a bool — match that signature unless the existing quintal version returns a richer type. Inspect `quintal/verification.rs` and follow its style.
- The helper `pc_chord_to_quartal_voiced` should not duplicate ten lines of fragile parsing; if the quintal-side helper is private, prefer reproducing the small amount of logic in quartal/verification.rs over modifying quintal's API surface. (This is decision D-quartal-T0-001; record it in the rustdoc.)
- The two verifiers should both return `true` on a correct implementation. If either returns `false`, that's a bug — either in the new code or in the underlying quintal apparatus, and the tests will surface it.

---

## 6. Spec — `crates/mt/src/quartal/mod.rs` updates

### 6.1 Add module declarations

After the existing `mod` lines (currently `constructors, conversion, error, interval, modes, orbit, types, voicing`), add:

```rust
mod duality;
mod verification;
```

Keep alphabetical order if the existing list is alphabetical; otherwise match the existing convention.

### 6.2 Add public re-exports

After the existing `pub use` block, add:

```rust
pub use duality::{
    orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t_quartal_reversal_equivalence, verify_all_orbits_self_dual,
};
pub use verification::{verify_quartal_fiber_classes, verify_quartal_universal_l1_law};
```

(Order alphabetically within each group, matching existing conventions.)

### 6.3 Do not modify

- The existing re-exports from `crate::quintal::...` at the bottom of the file.
- The `base_space()` helper function.
- Any existing `mod` declaration or `pub use`.

---

## 7. Spec — Tests

### 7.1 `crates/mt/tests/quartal/test_duality.rs`

Test cases (all should pass):

| Test | Assertion |
|---|---|
| `test_quartal_reading_q555_summit` | For a `QuartalVoicedChord` constructed via `pure_quartal_stack(0)` (the C-rooted Q555 Summit chord), `quartal_reading(&chord) == QuartalIntervalStructure(5, 5, 5)`. |
| `test_quintal_reading_q555_summit` | Same chord, `quintal_reading(&chord)` returns the dual `IntervalStructure(7, 7, 7)`. |
| `test_quartal_reading_q646_saddle` | For a saddle chord (e.g., from `quintal::saddle_chords(&base_space())[0]` wrapped via `to_quartal`), `quartal_reading` returns `QuartalIntervalStructure(6, 4, 6)`. |
| `test_quintal_reading_q646_saddle` | Same chord, `quintal_reading` returns `IntervalStructure(6, 8, 6)`. |
| `test_t_quartal_reversal_equivalence_holds` | For a sample of chords (e.g., one chord from each of the 14 orbits, or three test chords as in `test_quartal_quintal_identity::test_fibers_same_chords`), `t_quartal_reversal_equivalence(chord) == true`. |
| `test_self_duality_re_export` | `verify_all_orbits_self_dual(&base_space())` is reachable from `crate::quartal::verify_all_orbits_self_dual` and returns `true`. |

Use `theory::quartal::*` imports paralleling the existing `tests/quartal/test_quartal_quintal_identity.rs`.

### 7.2 `crates/mt/tests/quartal/test_verification.rs`

Test cases:

| Test | Assertion |
|---|---|
| `test_quartal_universal_l1_law` | `verify_quartal_universal_l1_law() == true`. |
| `test_quartal_fiber_classes` | `verify_quartal_fiber_classes() == true`. |
| `test_quartal_l1_pattern_summit` | For the C-rooted Q555 Summit chord, `quartal_l1_distances(&chord) == [12, 12, 12, 36]`. |
| `test_quartal_l1_pattern_saddle` | For a saddle chord, `quartal_l1_distances(&chord) == [12, 12, 12, 36]`. |
| `test_quartal_verifiers_agree_with_quintal` | `verify_quartal_universal_l1_law() == quintal::verify_universal_l1_law()` (both `true`); `verify_quartal_fiber_classes() == quintal::verify_fiber_classes()`. |

### 7.3 Test module wiring

Update `crates/mt/tests/quartal/mod.rs` to add:

```rust
mod test_duality;
mod test_verification;
```

(Match existing style — e.g., if other test modules are in alphabetical order, place these alphabetically.)

---

## 8. Acceptance Criteria

T0 is complete when **all** of the following hold:

- [ ] `cargo build` succeeds with no new warnings.
- [ ] `cargo build --features midi` succeeds with no new warnings.
- [ ] `cargo test` passes — all existing 530+ tests plus the new tests in `test_duality.rs` and `test_verification.rs`.
- [ ] `cargo test --features midi` passes.
- [ ] `cargo clippy` produces no new warnings beyond the four expected `module_inception` warnings noted in `CLAUDE.md`.
- [ ] `cargo clippy --features midi` likewise.
- [ ] All public API on the new modules has rustdoc with at least a one-line summary and at least one cross-reference (`[link]`) where a related quintal function exists.
- [ ] No changes to files outside `crates/mt/src/quartal/` and `crates/mt/tests/quartal/` (and the two `mod.rs` files in those directories), except where strictly necessary — and any exception is documented in the ledger report (§9) with rationale.
- [ ] The serde feature flag is honoured on any new public types (none expected in T0; if added, they need `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`).

---

## 9. Ledger Report Template

After implementation, CC produces a report at `oth4/oth4-T0-ledger.md` with the following sections. Every claim should be checkable from the diff.

```markdown
# oth4 — T0 Ledger Report

**Implementer:** Claude Code
**Date:** YYYY-MM-DD
**Branch / commit:** <branch>@<sha>
**Spec followed:** oth4-T0-spec.md (this repo's oth4/ folder)

## 1. Summary
One paragraph: what was built, anything notable.

## 2. Files Created
- `path/to/file.rs` — N lines, brief description
- ...

## 3. Files Modified
- `path/to/file.rs` — what changed (1–2 sentences)
- ...

## 4. Public API Surface Added
List every new public symbol, by module:
- `quartal::duality::quartal_reading(chord: &QuartalVoicedChord) -> QuartalIntervalStructure`
- `quartal::duality::quintal_reading(chord: &QuartalVoicedChord) -> IntervalStructure`
- `quartal::duality::t_quartal_reversal_equivalence(chord: &QuartalVoicedChord) -> bool`
- (re-exports listed separately)
- ...

## 5. Tests Added
Per file, list test name and one-line description.

## 6. Test Results
- `cargo test`: <pass count>/<total>, time
- `cargo test --features midi`: <pass count>/<total>, time
- `cargo clippy`: <warning count> (expected 4 module_inception + 0 new)
- `cargo clippy --features midi`: <warning count>

## 7. Specced but Skipped
Any item from §2–§7 of the spec that was *not* delivered, with rationale.

## 8. Added Beyond Spec
Any code or test added that the spec did not request, with rationale.

## 9. Judgment Calls
Any decisions where the spec was ambiguous and CC made a choice. Document
the choice and why.

## 10. Open Questions
Anything the implementer wants Duncan / Claude to resolve before T1.
```

The ledger is the single artefact passed back for review. Diffs in `mt-rs` are the source of truth; the ledger is the index into them.

---

## 10. Out of Scope (do not do)

- ❌ Adding any module other than `duality.rs` and `verification.rs`.
- ❌ Modifying `quintal/` in any way (including making the private `pc_chord_to_voiced` helper public).
- ❌ Adding MCP tools — that's T3.
- ❌ Adding centrality, display, or functional modules — those are T1 and T2.
- ❌ Changing existing tests.
- ❌ Refactoring the existing quartal module (constructors, types, voicing, etc.).
- ❌ Adding new dependencies to `Cargo.toml`.

If the spec is unclear on a specific point, prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 11. Style Notes

- Match existing rustdoc conventions: triple-slash, intra-doc links via `[`Type`]`, examples where the function is non-obvious.
- Match existing error style: `Result<_, QuartalError>` for fallible operations; this T0 phase has no expected fallibility.
- Match existing naming: snake_case functions, UpperCamelCase types.
- Use `pub use` for re-exports; do not re-implement re-exported functions.
- Use `super::` for sibling modules within `quartal/`, and `crate::quintal` for cross-module references.

The Rust SKILL referenced in `CLAUDE.md` (`assets/ai/rust/SKILL.md`) is authoritative for general Rust style.

---

## 12. Estimated Effort

Spec author's estimate: **1–2 hours** of focused implementation + test work for an experienced Rust developer with the codebase loaded. The two modules together should be fewer than ~150 lines of source plus ~150 lines of tests.

If the implementation expands beyond ~400 lines total, that's a signal that scope has crept and §10 should be revisited.

---

*End of oth4-T0-spec.md*
