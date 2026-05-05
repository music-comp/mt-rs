# oth4 — T2 Ledger Report

**Implementer:** Claude Code
**Date:** 2026-05-05
**Branch / commit:** `feat/oth4-t2-quartal-functional` @ `513b109`
**Spec followed:** `docs/design/05-active/0010-oth4-t2-implementation-spec.md`
**Plan followed:** `docs/dev/quartal/0007-oth4-t2-quartal-functional-implementation-plan.md`

## 1. Summary

Added a quartal-perspective `functional` module to `crates/mt/src/quartal/`, mirroring `crates/mt/src/quintal/functional.rs`. After T2, the quartal module has full analytical symmetry with quintal on five surfaces (`duality`, `verification`, `display`, `centrality`, `functional`). The module re-exports two perspective-invariant types from quintal (`FunctionalRegion`, `Pathway`) and adds three quartal-specific ergonomics: `QuartalOrbit::functional_region` (delegating extension method per D-quartal-T2-001), plus two free functions (`quartal_orbits_in`, `quartal_pathway_stops`) for narrative-document orbit-list queries. All four pre-recorded D-quartal-T2 spec decisions adopted as-is. Implementation proceeded through three TDD tasks with no surprises; the only mid-task adjustments were a `cargo fmt` normalisation of the `pub use functional::{...}` block onto a single line and a tightening of one `assert_eq!` formatting.

## 2. Files Created

- `crates/mt/src/quartal/functional.rs` — 114 lines. Two `pub use` re-exports (`FunctionalRegion`, `Pathway`), one inherent-method extension (`QuartalOrbit::functional_region`), two free fns (`quartal_orbits_in`, `quartal_pathway_stops`).
- `crates/mt/tests/quartal/test_functional.rs` — 230 lines. 14 integration tests (table-driven mapping + 7 region partitions + 3 pathway tests + 2 re-export reachability tests + 1 partition completeness test).
- `oth4/oth4-T2-ledger.md` — this file.

## 3. Files Modified

- `crates/mt/src/quartal/mod.rs` — added `mod functional;` (alphabetical, between `error` and `interval`) and a `pub use functional::{...}` block. No other lines changed.
- `crates/mt/tests/quartal/mod.rs` — added `mod test_functional;` (alphabetical, between `test_duality` and `test_modes`). No other lines changed.

## 4. Public API Surface Added

New inherent method:

- `QuartalOrbit::functional_region(&self) -> FunctionalRegion`

New free fns:

- `quartal::quartal_orbits_in(region: FunctionalRegion) -> Vec<QuartalOrbit>`
- `quartal::quartal_pathway_stops(pathway: Pathway) -> Vec<(FunctionalRegion, Vec<QuartalOrbit>)>`

All three carry `#[must_use]` and at least one runnable doctest (with at least one asymmetric orbit example per F-T2-002 / spec §10 discipline).

New re-exports (from `crate::quintal`, surfaced through `crate::quartal::functional`):

- `quartal::FunctionalRegion`
- `quartal::Pathway`

No new public types. No changes to existing public types or signatures. Cargo.toml / Cargo.lock unchanged (no new dependencies).

## 5. Tests Added

### `crates/mt/tests/quartal/test_functional.rs` (14 integration tests)

| Test | Description |
|---|---|
| `test_all_14_quartal_orbits_map_to_correct_region` | Table-driven: every QuartalOrbit variant → expected region. Asymmetric coverage: Q554, Q655, Q654, Q564, Q645, Q445, Q446 (and the palindromic-asymmetric mix in Plateau / Valley / Precipice). |
| `test_quartal_orbits_in_summit` | `[Q555]` — exact equality (F-T2-002 strengthening). |
| `test_quartal_orbits_in_plateau` | `[Q545, Q554]` — exact. |
| `test_quartal_orbits_in_slope` | `[Q655, Q564, Q654, Q645]` — exact (declaration-order regression guard). |
| `test_quartal_orbits_in_valley` | `[Q565, Q454, Q464]` — exact. |
| `test_quartal_orbits_in_saddle` | `[Q646]` — exact. |
| `test_quartal_orbits_in_precipice` | `[Q445, Q446]` — exact. |
| `test_quartal_orbits_in_narrows` | `[Q656]` — exact. |
| `test_quartal_orbits_in_partition_is_complete` | All 14 variants present exactly once across the 7 regions; total cardinality 14. |
| `test_quartal_pathway_stops_cadence` | 3 stops: (Saddle, [Q646]) → (Slope, 4 orbits) → (Summit, [Q555]). |
| `test_quartal_pathway_stops_departure` | 4 stops: Summit → Plateau → Slope → Saddle, with full orbit lists pinned for the singleton stops. |
| `test_quartal_pathway_stops_match_region_sequence` | For both pathways: stop count matches `Pathway::region_sequence()`, each stop region matches, each stop's orbit list equals `quartal_orbits_in(region)`. |
| `test_functional_region_re_export` | `FunctionalRegion::Summit` reachable via `crate::quartal::*`; `Display` renders `"Summit"`. |
| `test_pathway_re_export` | `Pathway::Cadence` reachable via `crate::quartal::*`; `Display` renders `"Cadence"`. |

### Doctests added (3 new)

- `QuartalOrbit::functional_region` — Q555/Q646 (palindromic) + Q554 (asymmetric) examples.
- `quartal_orbits_in` — Summit / Saddle exact equality + Slope length check.
- `quartal_pathway_stops` — Cadence shape + first/last stop assertions.

**Total tests added:** 14 integration + 3 doctests = 17.

## 6. Test Results

| Run | Result | Time |
|---|---|---|
| `cargo test` (default features) | 58 + 86 + 633 + 34 + 6 + 29 = **846 passed**, 0 failed | ~4s wall (full suite) |
| `cargo test --features midi` | 63 + 86 + 633 + 34 + 6 + 29 = **851 passed**, 0 failed | ~4s wall |
| `cargo build` (default) | clean, 0 new warnings | <1s incremental |
| `cargo build --features midi` | clean, 0 new warnings | ~1s incremental |
| `cargo clippy --all-targets` (default) | **0 warnings** | <1s incremental |
| `cargo clippy --all-targets --features midi` | **0 warnings** | <1s incremental |
| `cargo doc --no-deps` | **0 warnings**, 0 missing-docs, 0 broken intra-doc links | ~1s incremental |
| `cargo fmt --check` | clean (post-fmt-normalisation; see §9 below) | <1s |

Baseline (pre-T2, post-T1 merge): 829 default / 834 midi / 0 clippy / 0 doc. Delta: **+17** in both feature flags (matching the 14 integration + 3 doctest additions).

## 7. Specced but Skipped

None. Every spec §2/§3 file was created or modified. Every spec §4.2 public item landed with the documented signatures. Every spec §6.1 test landed (with the F-T2-002 strengthening from "including X, Y, Z" wording to exact-equality assertions). Spec §8 ledger template followed verbatim. Spec §9 out-of-scope rules respected — see §9 below for the one-file in-scope-by-exception note.

## 8. Added Beyond Spec

- Tests use **exact-equality** assertions on `quartal_orbits_in` outputs (F-T2-002 strengthening) where the spec §6.1 used "including X, Y, Z" wording. Catches any future regression in `QuartalOrbit::all()` declaration order as a side effect. This is a strengthening of test rigour, not a deviation in scope.
- `test_quartal_orbits_in_partition_is_complete` cross-checks against `QuartalOrbit::all()` (every variant present exactly once); spec asked only for "every variant in exactly one region's list".
- `test_quartal_pathway_stops_match_region_sequence` cross-checks against the canonical `Pathway::region_sequence()` and `Pathway::all()` accessors — a regression guard tying the quartal helpers to the canonical quintal data.
- `#[must_use]` attribute on all three new public items — matches the convention.

## 9. Judgment Calls

- **D-quartal-T2-001 (spec):** `QuartalOrbit::functional_region` delegates to `self.to_quintal().functional_region()` — a one-line bridge. Adopted as-is. No 14-arm match. Documented inline in the rustdoc.
- **D-quartal-T2-002 (spec):** `FunctionalRegion` and `Pathway` re-exposed via `quartal::functional`. Per **F-T2-001** (confirmed during exploration), there is no existing `pub use crate::quintal::FunctionalRegion` to compete with, so no `mod.rs` cleanup was needed. The bottom-of-file `pub use crate::quintal::{...}` block was left untouched.
- **D-quartal-T2-003 (spec):** `QuartalFunctionalSummary`-style speculative struct deferred. T2 ships only the three new public items.
- **D-quartal-T2-004 (spec):** T2 does not edit `CLAUDE.md`. The `module_inception` staleness is now flagged in three consecutive ledger reports.
- **F-T2-002 strengthening:** tests use exact-equality on `quartal_orbits_in` outputs — see §8. Picked up at plan-writing time; locked into the plan.
- **Plan-doc commit on feature branch (in-scope-by-exception):** the T2 implementation plan (`docs/dev/quartal/0007-oth4-t2-quartal-functional-implementation-plan.md`) was committed as the first commit on this feature branch (commit `6b92ef3`), since it was an untracked file in the worktree when implementation started. T0 and T1's plans were pre-committed to `main` before their respective feature branches began; T2's plan was created during the same plan-mode session that handed off to implementation, so it landed on the feature branch instead. This adds one file to the diff vs `main` (5 quartal + 1 plan + 1 ledger = 7 files vs the spec's expected 5). The plan file's content is unchanged from approval; the user can squash, rebase, or relocate the plan commit at merge time. Documented here per spec §9 last-line guidance ("do less, document the question in §9 of the ledger").
- **`cargo fmt` normalisation (mid-task 2):** rustfmt collapsed the `pub use functional::{quartal_orbits_in, quartal_pathway_stops, FunctionalRegion, Pathway};` block onto a single line (it fits within the configured line limit) and tightened one `assert_eq!(total, 14, "...")` formatting in the test file. Both are intentional rustfmt outputs; the post-fmt code is what was committed.

## 10. Open Questions

- **CLAUDE.md staleness:** the "4 module_inception warnings expected" claim is now flagged in T0 §10, T1 §10, and T2 §10. The fix is a one-line edit; flagging again so it doesn't get lost across a fourth ledger.
- **Plan-doc location convention:** for T0 and T1 the user committed the plan to `main` before starting impl; for T2 the plan landed on the feature branch (per the in-scope-by-exception note in §9). If the user prefers the T0/T1 convention going forward, a quick stance on whether to (a) cherry-pick the plan commit to `main` before merging, (b) leave it as a single squash commit with the impl, or (c) accept the T2 status quo — would be helpful for T3.
- **`QuartalFunctionalSummary` design:** D-quartal-T2-003 deferred this struct. The walkthrough (T4) is the natural place to evaluate whether such a struct is needed; flagging for the T3/T4 specs to revisit.
- **Topical-submodule re-export convention:** locked in T1 §6 of the plan review (D-quartal-T1-003) and reaffirmed in T2 §11 (D-quartal-T2-002). Three consecutive phases have applied this rule successfully; consider promoting it from "T-phase convention" to a permanent project-style note.
