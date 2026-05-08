---
number: 10
title: "Quartal Implementation Spec (Phase T2)"
author: "the quintal"
component: All
tags: [change-me]
created: 2026-05-04
updated: 2026-05-05
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oth4 — T2 Implementation Spec

**Status:** Ready for handoff to Claude Code, 2026-05-04
**Phase:** T2 (third of five tooling phases for the OTH4 programme)
**Anchor docs:** [PLAN.md](./PLAN.md) §6, [oth4-00-tooling.md](./oth4-00-tooling.md) §3.1
**Predecessor:** [`oth4-T1-spec.md`](./oth4-T1-spec.md) — T1 merged 2026-05-04, branch `feat/oth4-t1-quartal-display-centrality`
**Repository:** `mt-rs` (Rust music-theory library)

---

## 1. Context (just enough)

T0 added `quartal/duality.rs` and `quartal/verification.rs`. T1 added `quartal/display.rs` and `quartal/centrality.rs`. T2 closes the last analytical-symmetry gap: a quartal-side **functional grammar** module, mirroring `crates/mt/src/quintal/functional.rs`.

After T2 lands, the quartal module has full analytical symmetry with quintal on five surfaces (`duality`, `verification`, `display`, `centrality`, `functional`). T3 (MCP tool exposure) and T4 (the walkthrough) follow. The walkthrough is the natural place to discover whether any further analytical helpers are needed; T2 deliberately ships a small, focused surface and defers speculative additions.

The quintal functional module defines two perspective-invariant types — `FunctionalRegion` (Summit, Plateau, Slope, Valley, Saddle, Precipice, Narrows) and `Pathway` (Cadence, Departure) — plus an `Orbit::functional_region()` extension method and `Pathway::region_sequence()` / `Pathway::all()` accessors. T2 re-exports the two types through `quartal::functional`, adds the symmetric extension method `QuartalOrbit::functional_region()`, and adds two free functions for quartal-labelled orbit-list queries that the walkthrough documents will need.

T2 is the smallest of the three tooling phases so far — estimated ~80 source lines + ~80 test lines, well under T1's footprint.

---

## 2. Files to Create

| Path | Purpose |
|---|---|
| `crates/mt/src/quartal/functional.rs` | Quartal-perspective functional grammar |
| `crates/mt/tests/quartal/test_functional.rs` | Integration tests for the new module |

## 3. Files to Modify

| Path | Change |
|---|---|
| `crates/mt/src/quartal/mod.rs` | Declare `mod functional;`; add one `pub use` block |
| `crates/mt/tests/quartal/mod.rs` | Add `mod test_functional;` |

The `mod` declarations are alphabetical; insert as appropriate. **No other files** should change.

---

## 4. Spec — `crates/mt/src/quartal/functional.rs`

### 4.1 Module-level intent

Quartal-perspective functional grammar. The mathematical content — the seven landform regions, the orbit-to-region mapping, the two canonical pathways — is perspective-invariant and lives in `crate::quintal::functional`. This module re-exports both enums and adds quartal-specific ergonomics: an extension method on `QuartalOrbit` and two free helpers for narrative-document orbit-list queries.

Module docstring should make the symmetry with `crate::quintal::functional` explicit and explain that regions and pathways are perspective-invariant by design.

### 4.2 Public surface

```rust
//! Quartal-perspective functional grammar — symmetric counterpart to
//! [`crate::quintal::functional`].
//!
//! The seven [`FunctionalRegion`] variants and two [`Pathway`] variants are
//! perspective-invariant — they describe the *topographic role* of an
//! orbit (Summit, Saddle, Slope, …), not its interval structure. Both
//! types are re-exported as-is from quintal. This module adds quartal-side
//! ergonomics: [`QuartalOrbit::functional_region`] as an extension method,
//! and two free functions returning quartal-labelled orbit lists.
//!
//! Region-level queries (e.g. "which quartal orbits live on the Slope?")
//! and pathway-level queries (e.g. "what is the quartal stop list for the
//! Cadence pathway?") are the primary use cases the walkthrough documents
//! will exercise.

use super::orbit::QuartalOrbit;

// Re-exports of perspective-invariant types from quintal.

/// The seven OTH functional regions.
///
/// Re-exported from [`crate::quintal::functional::FunctionalRegion`].
/// The regions describe topographic role on the shared base-space graph —
/// they are perspective-invariant. The same orbit, read in either quintal
/// or quartal vocabulary, lives in the same region.
pub use crate::quintal::FunctionalRegion;

/// A canonical OTH functional pathway.
///
/// Re-exported from [`crate::quintal::functional::Pathway`].
/// Pathway region sequences are perspective-invariant.
pub use crate::quintal::Pathway;

impl QuartalOrbit {
    /// Return the [`FunctionalRegion`] this orbit belongs to.
    ///
    /// Symmetric counterpart to [`crate::quintal::Orbit::functional_region`].
    /// Delegates via [`QuartalOrbit::to_quintal`] to maintain a single source
    /// of truth for the orbit-to-region mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use music_comp_mt::quartal::{FunctionalRegion, QuartalOrbit};
    ///
    /// assert_eq!(QuartalOrbit::Q555.functional_region(), FunctionalRegion::Summit);
    /// assert_eq!(QuartalOrbit::Q646.functional_region(), FunctionalRegion::Saddle);
    /// // Asymmetric orbits map correctly too:
    /// assert_eq!(QuartalOrbit::Q554.functional_region(), FunctionalRegion::Plateau);
    /// ```
    #[must_use]
    pub fn functional_region(&self) -> FunctionalRegion {
        self.to_quintal().functional_region()
    }
}

/// All quartal orbits that belong to a given functional region.
///
/// Returns the orbits in [`QuartalOrbit::all`] declaration order. The
/// returned `Vec` length matches the canonical region sizes:
/// Summit (1), Plateau (2), Slope (4), Valley (3), Saddle (1),
/// Precipice (2), Narrows (1) — totalling 14.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_orbits_in, FunctionalRegion, QuartalOrbit};
///
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Summit), vec![QuartalOrbit::Q555]);
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Saddle), vec![QuartalOrbit::Q646]);
/// assert_eq!(quartal_orbits_in(FunctionalRegion::Slope).len(), 4);
/// ```
#[must_use]
pub fn quartal_orbits_in(region: FunctionalRegion) -> Vec<QuartalOrbit> {
    QuartalOrbit::all()
        .iter()
        .filter(|orbit| orbit.functional_region() == region)
        .copied()
        .collect()
}

/// The quartal-labelled stop list for a canonical [`Pathway`].
///
/// Returns one entry per region in [`Pathway::region_sequence`], pairing
/// the region with all quartal orbits in that region (in
/// [`QuartalOrbit::all`] declaration order). Used directly by the
/// walkthrough documents to render pathway tables in quartal vocabulary.
///
/// For [`Pathway::Cadence`] the returned slice has length 3 (Saddle →
/// Slope → Summit). For [`Pathway::Departure`] the length is 4
/// (Summit → Plateau → Slope → Saddle).
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{quartal_pathway_stops, FunctionalRegion, Pathway, QuartalOrbit};
///
/// let stops = quartal_pathway_stops(Pathway::Cadence);
/// assert_eq!(stops.len(), 3);
/// assert_eq!(stops[0].0, FunctionalRegion::Saddle);
/// assert_eq!(stops[0].1, vec![QuartalOrbit::Q646]);
/// assert_eq!(stops[2].0, FunctionalRegion::Summit);
/// assert_eq!(stops[2].1, vec![QuartalOrbit::Q555]);
/// ```
#[must_use]
pub fn quartal_pathway_stops(pathway: Pathway) -> Vec<(FunctionalRegion, Vec<QuartalOrbit>)> {
    pathway
        .region_sequence()
        .iter()
        .map(|&region| (region, quartal_orbits_in(region)))
        .collect()
}
```

### 4.3 Implementation notes

- The `QuartalOrbit::functional_region` body delegates to `self.to_quintal().functional_region()` — a one-line bridge. Decision **D-quartal-T2-001**: prefer delegation over a direct 14-arm `match` block, on the grounds that no new mathematics is introduced and the orbit-to-region mapping has a single source of truth in `quintal/functional.rs`. Document this in the rustdoc.
- `quartal_orbits_in` filters `QuartalOrbit::all()` by region. The order is `QuartalOrbit::all()` declaration order — match the convention used by the quintal tests' tables (which iterate orbits in `Orbit::all()` order).
- `quartal_pathway_stops` composes `Pathway::region_sequence()` with `quartal_orbits_in` — a five-line implementation. No additional state; pathway data is already canonical in quintal.
- Both free functions get `#[must_use]`. The extension method on `QuartalOrbit` also gets `#[must_use]` (matches `quintal::Orbit::functional_region`'s implicit pattern even though that method doesn't carry the attribute — we are *adding* the convention here per the T1 lesson).
- Every public item gets a runnable doctest (T1 lesson; spec §12 reaffirmed). Docstring examples should include at least one *asymmetric* orbit (Q554 or similar) to keep the asymmetric-coverage discipline visible even though the mapping is direct lookup.
- No new types, no new dependencies.

---

## 5. Spec — `crates/mt/src/quartal/mod.rs` updates

### 5.1 Add module declaration

Insert `functional` alphabetically (after `error`, before `interval`) in the existing `mod` block:

```rust
mod centrality;
mod constructors;
mod conversion;
mod display;
mod duality;
mod error;
mod functional;          // <-- new
mod interval;
mod modes;
mod orbit;
mod types;
mod verification;
mod voicing;
```

### 5.2 Add public re-export block

Insert after the `pub use error::QuartalError;` line and before `pub use interval::{...}`:

```rust
pub use functional::{
    quartal_orbits_in, quartal_pathway_stops, FunctionalRegion, Pathway,
};
```

(Four symbols, alphabetised within the brace.)

### 5.3 Do not modify

- Existing module declarations or `pub use` blocks (T0 and T1 left them in good shape).
- The `base_space()` helper.
- The bottom-of-file `pub use crate::quintal::...` block. `FunctionalRegion` and `Pathway` are now reachable as `crate::quartal::FunctionalRegion` / `crate::quartal::Pathway` via the new functional re-export — and **also** indirectly via `crate::quintal::FunctionalRegion`. Per the T1-locked-in convention (perspective-specific views go through their topical submodule, perspective-invariant infrastructure stays in the bottom-of-file `pub use crate::quintal::*` block), `FunctionalRegion` lives in `quartal::functional` because it is a *functional-grammar* concern even though the values themselves are perspective-invariant. There is no existing `pub use crate::quintal::FunctionalRegion` to compete with; no deduplication is required.

---

## 6. Spec — Tests

### 6.1 `crates/mt/tests/quartal/test_functional.rs`

Test cases (all should pass):

| Test | Assertion |
|---|---|
| `test_all_14_quartal_orbits_map_to_correct_region` | Table-driven: every `QuartalOrbit` variant maps to the expected `FunctionalRegion`. Includes asymmetric variants (Q554 → Plateau, Q655 → Slope, Q654 → Slope, Q564 → Slope, Q645 → Slope, Q445 → Precipice) — this is the analog of the asymmetric-chord regression test from T0/T1, ensuring no asymmetric orbit is mis-classified. |
| `test_quartal_orbits_in_summit` | `quartal_orbits_in(Summit) == vec![Q555]`. |
| `test_quartal_orbits_in_plateau` | Returns 2 orbits including Q545 and Q554. |
| `test_quartal_orbits_in_slope` | Returns 4 orbits including Q645, Q655, Q654, Q564. |
| `test_quartal_orbits_in_valley` | Returns 3 orbits including Q565, Q464, Q454. |
| `test_quartal_orbits_in_saddle` | `quartal_orbits_in(Saddle) == vec![Q646]`. |
| `test_quartal_orbits_in_precipice` | Returns 2 orbits including Q446 and Q445. |
| `test_quartal_orbits_in_narrows` | `quartal_orbits_in(Narrows) == vec![Q656]`. |
| `test_quartal_orbits_in_partition_is_complete` | Across all 7 regions, every `QuartalOrbit` variant appears in exactly one region's list; total cardinality is 14. |
| `test_quartal_pathway_stops_cadence` | Cadence pathway has 3 stops: (Saddle, [Q646]) → (Slope, 4 orbits) → (Summit, [Q555]). |
| `test_quartal_pathway_stops_departure` | Departure pathway has 4 stops in order: Summit → Plateau → Slope → Saddle. |
| `test_quartal_pathway_stops_match_region_sequence` | For both pathways, the stop count and region sequence match `Pathway::region_sequence()`, and each stop's orbit list equals `quartal_orbits_in(region)`. |
| `test_functional_region_re_export` | `crate::quartal::FunctionalRegion::Summit` is reachable; the `Display` impl renders `"Summit"`. |
| `test_pathway_re_export` | `crate::quartal::Pathway::Cadence` is reachable; `Display` renders `"Cadence"`. |

### 6.2 Test module wiring

Update `crates/mt/tests/quartal/mod.rs` to add `mod test_functional;` in alphabetical position. After insertion, the block should be:

```rust
mod test_centrality;
mod test_constructors;
mod test_display;
mod test_duality;
mod test_functional;
mod test_modes;
mod test_quartal_quintal_identity;
mod test_root_constructors;
mod test_types;
mod test_verification;
mod test_voicing;
```

---

## 7. Acceptance Criteria

T2 is complete when **all** of the following hold:

- [ ] `cargo build` succeeds with no new warnings.
- [ ] `cargo build --features midi` succeeds with no new warnings.
- [ ] `cargo test` passes — current main is **829 passing** (post-T1); after T2 the count should be **829 + N** where N is the number of new tests added (at least 14 integration + 3 doctests = 17 expected).
- [ ] `cargo test --features midi` passes with the same delta from baseline 834.
- [ ] `cargo clippy --all-targets` produces **zero new warnings** (current baseline is zero).
- [ ] `cargo clippy --all-targets --features midi` likewise.
- [ ] `cargo doc --no-deps` produces zero missing-docs and zero broken intra-doc links on the new symbols.
- [ ] `cargo fmt --check` is clean.
- [ ] Every public function in `quartal/functional.rs` has `#[must_use]` and at least one runnable doctest.
- [ ] No changes to files outside `crates/mt/src/quartal/`, `crates/mt/tests/quartal/`, and the new ledger doc.
- [ ] Serde feature flag honoured on any new public types (none expected — T2 adds no new public types beyond the re-exports from quintal, which already carry their `cfg_attr` derives).

---

## 8. Ledger Report Template

After implementation, CC produces `oth4/oth4-T2-ledger.md` following the same template as T1. Reproduced here for self-containment:

```markdown
# oth4 — T2 Ledger Report

**Implementer:** Claude Code
**Date:** YYYY-MM-DD
**Branch / commit:** <branch>@<sha>
**Spec followed:** oth4-T2-spec.md
**Plan followed:** docs/dev/quartal/<NNNN>-oth4-t2-...-plan.md

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
- `cargo clippy --all-targets`: <warning count> (expected 0)
- `cargo clippy --all-targets --features midi`: <warning count>
- `cargo doc --no-deps`: <missing-doc count> + <broken-link count> (both expected 0)
- `cargo fmt --check`: clean / not clean

## 7. Specced but Skipped
Any item from §2–§6 of the spec that was *not* delivered, with rationale.

## 8. Added Beyond Spec
Any code or test added that the spec did not request, with rationale.

## 9. Judgment Calls
Any decisions where the spec was ambiguous and CC made a choice. Document
the choice and why.

## 10. Open Questions
Anything the implementer wants Duncan / Claude to resolve before T3.
```

---

## 9. Out of Scope (do not do)

- ❌ Adding any module other than `functional.rs`.
- ❌ Modifying `crates/mt/src/quintal/` in any way.
- ❌ Adding MCP tools — that's T3.
- ❌ Adding speculative narrative-helper structs (e.g. `QuartalFunctionalSummary`, `QuartalRegionDetail`, etc.). The walkthrough (T4) is the natural place to evaluate whether such structs are needed; designing them speculatively risks importing assumptions the walkthrough doesn't actually need.
- ❌ Implementing a direct 14-arm `match` for `QuartalOrbit::functional_region`. Use delegation (D-quartal-T2-001) — single source of truth for the orbit-to-region mapping.
- ❌ Adding new `Display` impls. The re-exported `FunctionalRegion` and `Pathway` already have their `Display` impls from quintal; those carry through transparently.
- ❌ Updating `CLAUDE.md` to correct the stale `module_inception` warning expectation (carried forward from T0 / T1 ledger §10).
- ❌ Refactoring existing quartal modules.
- ❌ Adding new dependencies to `Cargo.toml`.

If a spec point is unclear during implementation, prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 10. Style Notes (T2 conventions, locked from T0/T1)

- Match existing rustdoc conventions: triple-slash, intra-doc links via `[Type]`, examples on each public function (mirror `quintal/functional.rs` and the T1 doctest discipline).
- `#[must_use]` on all four new public items (the inherent method on `QuartalOrbit` plus the two free functions; the re-exports inherit their attributes from quintal).
- Use `super::` for sibling modules within `quartal/`, and `crate::quintal` for cross-module references.
- The Rust SKILL referenced in `crates/mt/CLAUDE.md` (`assets/ai/rust/SKILL.md`) is authoritative for general Rust style.
- Doctest examples in §4.2 include at least one asymmetric orbit (Q554) — keep that discipline. The implementation rustdoc's running examples should include all three flavours: a Summit (palindromic, single-orbit region), a Saddle (palindromic, single-orbit region), and an asymmetric orbit (Q554 → Plateau).

---

## 11. Decisions Recorded in This Spec

For convenient reference in the ledger:

| ID | Decision | Rationale |
|---|---|---|
| D-quartal-T2-001 | `QuartalOrbit::functional_region` delegates to `self.to_quintal().functional_region()` rather than re-implementing a 14-arm match. | No new mathematics; single source of truth for the orbit-to-region mapping; `to_quintal` is already public and free. |
| D-quartal-T2-002 | `FunctionalRegion` and `Pathway` are re-exposed from `quartal::functional`, **not** from the bottom-of-file `pub use crate::quintal::*` block. | Per T1-locked-in convention: topical-submodule wins for related symbols even when perspective-invariant. A reader looking for "quartal functional grammar" should find the types in `quartal::functional`, not by scanning the bottom of `mod.rs`. |
| D-quartal-T2-003 | T2 ships `QuartalOrbit::functional_region`, `quartal_orbits_in`, and `quartal_pathway_stops` only. No `QuartalFunctionalSummary`-style speculative struct. | Symmetric with T1's deferral of `QuartalCentralitySummary`. Walkthrough (T4) drives any such design. |
| D-quartal-T2-004 | T2 does not edit `CLAUDE.md`. The stale `module_inception` warning expectation is now flagged in three consecutive ledger reports; landing it as a separate one-line commit is preferred. | Same rationale as T1's D-quartal-T1-004; coupling unrelated cleanup to T2 expands scope. |

---

## 12. Estimated Effort

Spec author's estimate: **1–1.5 hours** of focused implementation + test work for an experienced Rust developer with the codebase loaded. This is the smallest of the three tooling phases so far.

- Source: ~80 lines (re-exports + 3 functions, including substantial rustdoc + doctests).
- Tests: ~80 lines (14 integration tests, mostly table-driven and small).
- Total ~160 lines, well under T1's ~310 source-and-test footprint.

If the implementation expands beyond ~250 lines total (excluding the ledger), that's a signal that scope has crept and §9 should be revisited.

---

## 13. T0/T1 Lessons Already Applied

For convenient reference — these are baked into the spec, no action required:

- **Baseline numbers cited live, not from CLAUDE.md.** Acceptance criteria reference 829/834, not "530+" or whatever CLAUDE.md says.
- **Asymmetric coverage discipline.** The 14-orbit table-driven test plus the explicit asymmetric examples in doctests (Q554 in `QuartalOrbit::functional_region` doctest) ensure asymmetric orbits aren't silently mis-classified. The orbit-to-region mapping is direct lookup so the bug surface is "wrong region for an orbit" — the table-driven test catches that.
- **Speculative-struct deferral.** Symmetric with T1's `QuartalCentralitySummary` deferral.
- **Topical-submodule re-export.** Locked in T1 §6 of the implementation review; D-quartal-T2-002 records the application here.
- **`#[must_use]` everywhere.** All four new items.
- **Doctests everywhere.** All public items.
- **`CLAUDE.md` cleanup explicitly out of scope.** Carried forward from T0/T1.

---

*End of oth4-T2-spec.md*
