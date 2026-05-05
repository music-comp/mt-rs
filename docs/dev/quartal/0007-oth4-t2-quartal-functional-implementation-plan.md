# OTH4 — T2 Quartal Functional Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a quartal-perspective `functional` module to `crates/mt/src/quartal/`, mirroring `crates/mt/src/quintal/functional.rs`. Re-exports two perspective-invariant types from quintal (`FunctionalRegion`, `Pathway`) and adds three quartal-specific ergonomics: `QuartalOrbit::functional_region` (extension method) plus `quartal_orbits_in` and `quartal_pathway_stops` (free functions for narrative-document orbit-list queries).

**Architecture:** One thin perspective-layer module, ~80 lines. The mathematical content (the orbit-to-region mapping and the two canonical pathways) lives in quintal — `QuartalOrbit::functional_region` delegates to `self.to_quintal().functional_region()` (D-quartal-T2-001, single source of truth). The two free functions compose the orbit list and the pathway data already canonical in quintal.

**Tech Stack:** Rust 2024 (already on `edition = "2024"`), `crate::quintal::{FunctionalRegion, Pathway}` for the re-exports, `super::orbit::QuartalOrbit` for the extension method's receiver. No new dependencies. Adheres to the project's Rust skill (`assets/ai/rust/SKILL.md`): `#[must_use]` on every new public item, runnable doctest on every public item, intra-doc links to the dual quintal symbols.

**Source spec:** `docs/design/05-active/0010-oth4-t2-implementation-spec.md` (oth4-T2-spec).

**Predecessors:**
- T0 plan: `docs/dev/quartal/0005-oth4-t0-quartal-duality-verification-implementation-plan.md` (merged 2026-05-04)
- T1 plan: `docs/dev/quartal/0006-centrality-implementation-plan.md` (merged 2026-05-04)
- Post-T1 baseline: **829 tests / 0 clippy warnings / 0 doc warnings**.

---

## 1. Context

T0 added `quartal/duality.rs` and `quartal/verification.rs`. T1 added `quartal/display.rs` and `quartal/centrality.rs`. T2 closes the last analytical-symmetry gap: a quartal-side `functional` module mirroring `quintal/functional.rs`. After T2, the quartal module has full analytical symmetry with quintal on five surfaces (`duality`, `verification`, `display`, `centrality`, `functional`). T3 (MCP tool exposure) and T4 (the walkthrough) follow.

The quintal `functional.rs` already defines the two perspective-invariant types — `FunctionalRegion` (Summit, Plateau, Slope, Valley, Saddle, Precipice, Narrows) and `Pathway` (Cadence, Departure) — plus an inherent `Orbit::functional_region()` method and `Pathway::region_sequence()` / `Pathway::all()` accessors. T2 re-exports the two types through `quartal::functional`, adds the symmetric extension method `QuartalOrbit::functional_region()`, and adds two quartal-labelled orbit-list queries: `quartal_orbits_in` and `quartal_pathway_stops`.

Spec author estimates ~80 source lines + ~80 test lines (smallest of T0/T1/T2). Pre-recorded D-quartal-T2 decisions in spec §11; all four are direct adoptions of T0/T1 lessons.

---

## 2. Spec Findings (for CDC review)

The spec is well-prepared and pre-records four decisions in §11. The only finding worth flagging:

### F-T2-001 — `FunctionalRegion` / `Pathway` not currently re-exported from `crate::quartal`

**Spec §5.3 anticipates:** "There is no existing `pub use crate::quintal::FunctionalRegion` to compete with; no deduplication is required."

**Reality (`crates/mt/src/quartal/mod.rs`):** confirmed. The post-T1 bottom-of-file `pub use crate::quintal::{...}` block re-exports `BaseSpace, FiberClass, PcChord`, plus various betweenness / distance / geodesic / saddle helpers, but **not** `FunctionalRegion` or `Pathway`. Neither is currently reachable via `crate::quartal::*`.

**Resolution:** the spec's anticipated state matches reality. D-quartal-T2-002 (re-expose through `quartal::functional`) is straightforward — just add the new `pub use functional::{FunctionalRegion, Pathway, ...}` block; no cleanup of the bottom-of-file block. This finding is mostly a confirmation rather than a course-correction; it parallels the T1 plan's F-T1-001.

### F-T2-002 — Per-region orbit ordering is deterministic; tests should pin it exactly

**Spec §6.1 wording:** several tests use "Returns 2 orbits *including* Q545 and Q554" — "including" suggests the spec author wasn't certain of the deterministic order.

**Reality:** `quartal_orbits_in(region)` is implemented as `QuartalOrbit::all().iter().filter(...).copied().collect()`. `QuartalOrbit::all()` returns a fixed-order static slice (declaration order: `Q555, Q565, Q545, Q656, Q646, Q454, Q464, Q655, Q554, Q564, Q654, Q445, Q645, Q446`). So the per-region output is fully determined.

**Hand-derived per-region outputs** (filtering declaration order):

| Region | Output (declaration order) | Length |
|---|---|---|
| `Summit` | `[Q555]` | 1 |
| `Plateau` | `[Q545, Q554]` | 2 |
| `Slope` | `[Q655, Q564, Q654, Q645]` | 4 |
| `Valley` | `[Q565, Q454, Q464]` | 3 |
| `Saddle` | `[Q646]` | 1 |
| `Precipice` | `[Q445, Q446]` | 2 |
| `Narrows` | `[Q656]` | 1 |

Total: 14 ✓.

**Resolution:** the plan's test bodies use **exact equality** (`assert_eq!(quartal_orbits_in(region), vec![...])`) for every region, including the orderings derived above. Tighter than spec wording; catches any future regression in `QuartalOrbit::all()` ordering as a side-effect.

### F-T2-003 — Pathway stop counts and orderings (sanity check)

`Pathway::region_sequence` (defined in `quintal/functional.rs:73-87`):

- `Cadence`: `[Saddle, Slope, Summit]` → 3 stops
- `Departure`: `[Summit, Plateau, Slope, Saddle]` → 4 stops

So `quartal_pathway_stops(Cadence)` returns:

```
[
    (Saddle,   [Q646]),
    (Slope,    [Q655, Q564, Q654, Q645]),
    (Summit,   [Q555]),
]
```

And `quartal_pathway_stops(Departure)` returns:

```
[
    (Summit,   [Q555]),
    (Plateau,  [Q545, Q554]),
    (Slope,    [Q655, Q564, Q654, Q645]),
    (Saddle,   [Q646]),
]
```

These outputs flow directly from `quartal_orbits_in` per F-T2-002; the plan's pathway tests assert the structural shape (length, region ordering, first/last stops) plus a cross-check against `Pathway::region_sequence()`.

---

## 3. File Structure

```
crates/mt/src/quartal/
├── mod.rs                  # MODIFY: declare one new mod, add one pub-use block
└── functional.rs           # CREATE: ~80 lines, 2 re-exports + 1 inherent method + 2 free fns

crates/mt/tests/quartal/
├── mod.rs                  # MODIFY: declare one new test module
└── test_functional.rs      # CREATE: ~120 lines, 14 tests
```

Total: ~205 lines added across 4 files (excluding the ledger doc). Spec §12 caps at ~250; safely respected.

### Critical existing files / utilities to reuse (do not reinvent)

| Path | What | Why we use it |
|---|---|---|
| `crates/mt/src/quintal/functional.rs:7-43` | `FunctionalRegion` enum + `Display` impl | Re-exported via `quartal::functional` |
| `crates/mt/src/quintal/functional.rs:45-58` | `impl Orbit { fn functional_region(&self) -> FunctionalRegion }` | Single source of truth for the orbit-to-region mapping; quartal delegates here |
| `crates/mt/src/quintal/functional.rs:60-93` | `Pathway` enum + `region_sequence()` + `all()` | Re-exported + used by `quartal_pathway_stops` |
| `crates/mt/src/quartal/orbit.rs:131-148` | `QuartalOrbit::to_quintal(&self) -> Orbit` | The 1-line bridge for `QuartalOrbit::functional_region` (D-quartal-T2-001) |
| `crates/mt/src/quartal/orbit.rs:74-76` | `QuartalOrbit::all() -> &'static [QuartalOrbit; 14]` | Iteration source for `quartal_orbits_in` |

---

## 4. Tasks

### Task 1: Baseline check + register new (empty) module

**Files:**
- Create: `crates/mt/src/quartal/functional.rs` (initially empty stub)
- Modify: `crates/mt/src/quartal/mod.rs` (add one `mod` line)
- Create: `crates/mt/tests/quartal/test_functional.rs` (empty stub)
- Modify: `crates/mt/tests/quartal/mod.rs` (add one `mod` line)

This task wires up an empty module so subsequent compile/test cycles are fast and incremental.

- [ ] **Step 1.1: Confirm working tree + branch state**

Run: `git status -sb`
Expected: clean working tree on a fresh feature branch (e.g. `feat/oth4-t2-quartal-functional`). If still on `main`, create the branch:
`git checkout -b feat/oth4-t2-quartal-functional`

- [ ] **Step 1.2: Capture baseline test count**

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected (post-T1 merge): six "test result" lines summing to **829 passed**, 0 failed (58 lib unit + 86 + 619 + 34 + 6 + 26 doc).

Run: `cargo test --features midi 2>&1 | grep -E "^test result:"`
Expected: **834 passed** (63 lib unit + 86 + 619 + 34 + 6 + 26 doc).

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: **0**.

Run: `cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"`
Expected: **0**.

Record all four numbers — the ledger §6 needs them.

- [ ] **Step 1.3: Create empty `functional.rs` stub**

Create `crates/mt/src/quartal/functional.rs` with only the module docstring:

```rust
//! Quartal-perspective functional grammar — symmetric counterpart to
//! [`crate::quintal::functional`].
//!
//! The seven [`FunctionalRegion`] variants and two [`Pathway`] variants are
//! perspective-invariant — they describe the *topographic role* of an
//! orbit (Summit, Saddle, Slope, …), not its interval structure. Both
//! types are re-exported as-is from quintal. This module adds quartal-side
//! ergonomics: [`super::orbit::QuartalOrbit::functional_region`] as an
//! extension method, and two free functions returning quartal-labelled
//! orbit lists.
//!
//! Region-level queries (e.g. "which quartal orbits live on the Slope?")
//! and pathway-level queries (e.g. "what is the quartal stop list for the
//! Cadence pathway?") are the primary use cases the walkthrough documents
//! will exercise.
```

- [ ] **Step 1.4: Wire `mod` declaration in `crates/mt/src/quartal/mod.rs`**

Current `mod` block (post-T1):
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

Insert `functional` between `error` and `interval` (alphabetical):

```rust
mod centrality;
mod constructors;
mod conversion;
mod display;
mod duality;
mod error;
mod functional;
mod interval;
mod modes;
mod orbit;
mod types;
mod verification;
mod voicing;
```

Do NOT add the `pub use` line yet — adding an empty re-export block now would emit "unused import" warnings.

- [ ] **Step 1.5: Create empty test stub**

Create `crates/mt/tests/quartal/test_functional.rs` with only:
```rust
extern crate music_comp_mt as theory;
```

- [ ] **Step 1.6: Wire test `mod` declaration**

`crates/mt/tests/quartal/mod.rs` post-T1 has ten entries. Insert `test_functional` between `test_duality` and `test_modes` (alphabetical):

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

- [ ] **Step 1.7: Verify the empty wiring compiles cleanly**

Run: `cargo build`
Expected: succeeds, 0 new warnings.

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected: same baseline (**829 passed**) — the new test file declares zero tests.

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: 0.

- [ ] **Step 1.8: Commit**

```bash
git add crates/mt/src/quartal/functional.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_functional.rs \
        crates/mt/tests/quartal/mod.rs
git commit -m "feat(quartal): scaffold functional module stub (oth4-T2)"
```

---

### Task 2: Implement `quartal::functional` (TDD)

**Files:**
- Modify: `crates/mt/src/quartal/functional.rs`
- Modify: `crates/mt/src/quartal/mod.rs` (add `pub use functional::{...}`)
- Modify: `crates/mt/tests/quartal/test_functional.rs`

The module exposes:

| Symbol | Origin | Notes |
|---|---|---|
| `pub use crate::quintal::FunctionalRegion` | Re-export | Perspective-invariant; carries `Display` and serde impls |
| `pub use crate::quintal::Pathway` | Re-export | Perspective-invariant; carries `Display`, `region_sequence`, `all` |
| `impl QuartalOrbit { pub fn functional_region(&self) -> FunctionalRegion }` | New extension method | Delegates to `self.to_quintal().functional_region()` (D-quartal-T2-001) |
| `pub fn quartal_orbits_in(region: FunctionalRegion) -> Vec<QuartalOrbit>` | New free fn | Filters `QuartalOrbit::all()` |
| `pub fn quartal_pathway_stops(pathway: Pathway) -> Vec<(FunctionalRegion, Vec<QuartalOrbit>)>` | New free fn | Composes `Pathway::region_sequence()` with `quartal_orbits_in` |

- [ ] **Step 2.1: Write failing tests in `tests/quartal/test_functional.rs`**

Replace the file contents with:

```rust
extern crate music_comp_mt as theory;

use theory::quartal::{
    quartal_orbits_in, quartal_pathway_stops, FunctionalRegion, Pathway, QuartalOrbit,
};

// ────────────────────────── QuartalOrbit::functional_region ─────────────────

/// Table-driven: every QuartalOrbit variant maps to the expected
/// FunctionalRegion. Includes asymmetric orbits (Q554, Q655, Q654, Q564,
/// Q645, Q445, Q446, Q565, Q464, Q454) — analog of the asymmetric-chord
/// regression discipline established in T0/T1.
#[test]
fn test_all_14_quartal_orbits_map_to_correct_region() {
    use FunctionalRegion::*;
    let expected: [(QuartalOrbit, FunctionalRegion); 14] = [
        (QuartalOrbit::Q555, Summit),
        // Plateau (palindromic + asymmetric):
        (QuartalOrbit::Q545, Plateau),
        (QuartalOrbit::Q554, Plateau),
        // Slope (all asymmetric):
        (QuartalOrbit::Q655, Slope),
        (QuartalOrbit::Q564, Slope),
        (QuartalOrbit::Q654, Slope),
        (QuartalOrbit::Q645, Slope),
        // Valley (palindromic):
        (QuartalOrbit::Q565, Valley),
        (QuartalOrbit::Q454, Valley),
        (QuartalOrbit::Q464, Valley),
        // Saddle:
        (QuartalOrbit::Q646, Saddle),
        // Precipice (asymmetric):
        (QuartalOrbit::Q445, Precipice),
        (QuartalOrbit::Q446, Precipice),
        // Narrows:
        (QuartalOrbit::Q656, Narrows),
    ];
    assert_eq!(expected.len(), 14);
    for (orbit, region) in expected {
        assert_eq!(
            orbit.functional_region(),
            region,
            "{:?} should map to {:?}",
            orbit,
            region
        );
    }
}

// ────────────────────────── quartal_orbits_in ───────────────────────────────

#[test]
fn test_quartal_orbits_in_summit() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Summit),
        vec![QuartalOrbit::Q555]
    );
}

#[test]
fn test_quartal_orbits_in_plateau() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Plateau),
        vec![QuartalOrbit::Q545, QuartalOrbit::Q554]
    );
}

#[test]
fn test_quartal_orbits_in_slope() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Slope),
        vec![
            QuartalOrbit::Q655,
            QuartalOrbit::Q564,
            QuartalOrbit::Q654,
            QuartalOrbit::Q645,
        ]
    );
}

#[test]
fn test_quartal_orbits_in_valley() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Valley),
        vec![QuartalOrbit::Q565, QuartalOrbit::Q454, QuartalOrbit::Q464]
    );
}

#[test]
fn test_quartal_orbits_in_saddle() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Saddle),
        vec![QuartalOrbit::Q646]
    );
}

#[test]
fn test_quartal_orbits_in_precipice() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Precipice),
        vec![QuartalOrbit::Q445, QuartalOrbit::Q446]
    );
}

#[test]
fn test_quartal_orbits_in_narrows() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Narrows),
        vec![QuartalOrbit::Q656]
    );
}

#[test]
fn test_quartal_orbits_in_partition_is_complete() {
    // Across all 7 regions, every QuartalOrbit variant must appear exactly
    // once; total cardinality is 14 (the size of QuartalOrbit::all()).
    use std::collections::BTreeSet;
    let regions = [
        FunctionalRegion::Summit,
        FunctionalRegion::Plateau,
        FunctionalRegion::Slope,
        FunctionalRegion::Valley,
        FunctionalRegion::Saddle,
        FunctionalRegion::Precipice,
        FunctionalRegion::Narrows,
    ];
    let mut seen: BTreeSet<QuartalOrbit> = BTreeSet::new();
    let mut total = 0usize;
    for region in regions {
        let orbits = quartal_orbits_in(region);
        total += orbits.len();
        for orbit in orbits {
            assert!(
                seen.insert(orbit),
                "{:?} appears in more than one region",
                orbit
            );
        }
    }
    assert_eq!(total, 14, "total orbit count across all regions should be 14");
    assert_eq!(seen.len(), 14, "every QuartalOrbit variant should appear exactly once");
    for variant in QuartalOrbit::all() {
        assert!(
            seen.contains(variant),
            "QuartalOrbit::{:?} missing from partition",
            variant
        );
    }
}

// ────────────────────────── quartal_pathway_stops ───────────────────────────

#[test]
fn test_quartal_pathway_stops_cadence() {
    let stops = quartal_pathway_stops(Pathway::Cadence);
    assert_eq!(stops.len(), 3);
    assert_eq!(stops[0].0, FunctionalRegion::Saddle);
    assert_eq!(stops[0].1, vec![QuartalOrbit::Q646]);
    assert_eq!(stops[1].0, FunctionalRegion::Slope);
    assert_eq!(stops[1].1.len(), 4);
    assert_eq!(stops[2].0, FunctionalRegion::Summit);
    assert_eq!(stops[2].1, vec![QuartalOrbit::Q555]);
}

#[test]
fn test_quartal_pathway_stops_departure() {
    let stops = quartal_pathway_stops(Pathway::Departure);
    assert_eq!(stops.len(), 4);
    assert_eq!(stops[0].0, FunctionalRegion::Summit);
    assert_eq!(stops[0].1, vec![QuartalOrbit::Q555]);
    assert_eq!(stops[1].0, FunctionalRegion::Plateau);
    assert_eq!(stops[1].1, vec![QuartalOrbit::Q545, QuartalOrbit::Q554]);
    assert_eq!(stops[2].0, FunctionalRegion::Slope);
    assert_eq!(stops[2].1.len(), 4);
    assert_eq!(stops[3].0, FunctionalRegion::Saddle);
    assert_eq!(stops[3].1, vec![QuartalOrbit::Q646]);
}

#[test]
fn test_quartal_pathway_stops_match_region_sequence() {
    // For every canonical pathway: stop count matches region_sequence; each
    // stop's region matches; each stop's orbit list equals quartal_orbits_in.
    for &pathway in Pathway::all() {
        let stops = quartal_pathway_stops(pathway);
        let regions = pathway.region_sequence();
        assert_eq!(
            stops.len(),
            regions.len(),
            "stop count for {:?} must match region_sequence length",
            pathway
        );
        for (i, &expected_region) in regions.iter().enumerate() {
            assert_eq!(
                stops[i].0, expected_region,
                "stop {} region for {:?} mismatch",
                i, pathway
            );
            assert_eq!(
                stops[i].1,
                quartal_orbits_in(expected_region),
                "stop {} orbits for {:?} must equal quartal_orbits_in({:?})",
                i,
                pathway,
                expected_region
            );
        }
    }
}

// ────────────────────────── Re-export reachability ──────────────────────────

#[test]
fn test_functional_region_re_export() {
    // Reachable via crate::quartal::*; Display impl renders the variant name.
    let region = FunctionalRegion::Summit;
    assert_eq!(format!("{}", region), "Summit");
}

#[test]
fn test_pathway_re_export() {
    let pathway = Pathway::Cadence;
    assert_eq!(format!("{}", pathway), "Cadence");
}
```

(14 integration tests total, exactly matching spec §6.1.)

- [ ] **Step 2.2: Run failing tests to confirm they fail to compile**

Run: `cargo test --test tests quartal::test_functional 2>&1 | tail -25`
Expected: `error[E0432]: unresolved imports` for `quartal_orbits_in`, `quartal_pathway_stops`, `FunctionalRegion`, `Pathway` (and possibly `QuartalOrbit::functional_region` as a missing method).

If errors are different, stop and reconcile.

- [ ] **Step 2.3: Write the functional module body**

Replace `crates/mt/src/quartal/functional.rs` with:

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
/// Re-exported from [`crate::quintal::FunctionalRegion`].
/// The regions describe topographic role on the shared base-space graph —
/// they are perspective-invariant. The same orbit, read in either quintal
/// or quartal vocabulary, lives in the same region.
pub use crate::quintal::FunctionalRegion;

/// A canonical OTH functional pathway.
///
/// Re-exported from [`crate::quintal::Pathway`].
/// Pathway region sequences are perspective-invariant.
pub use crate::quintal::Pathway;

impl QuartalOrbit {
    /// Return the [`FunctionalRegion`] this orbit belongs to.
    ///
    /// Symmetric counterpart to [`crate::quintal::Orbit::functional_region`].
    /// Delegates via [`QuartalOrbit::to_quintal`] to maintain a single source
    /// of truth for the orbit-to-region mapping (decision D-quartal-T2-001).
    ///
    /// # Examples
    ///
    /// ```
    /// use music_comp_mt::quartal::{FunctionalRegion, QuartalOrbit};
    ///
    /// // Palindromic, single-orbit region:
    /// assert_eq!(QuartalOrbit::Q555.functional_region(), FunctionalRegion::Summit);
    /// // Palindromic, single-orbit region:
    /// assert_eq!(QuartalOrbit::Q646.functional_region(), FunctionalRegion::Saddle);
    /// // Asymmetric orbit maps correctly too:
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
/// For [`Pathway::Cadence`] the returned `Vec` has length 3 (Saddle →
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

- [ ] **Step 2.4: Add `pub use` block to `crates/mt/src/quartal/mod.rs`**

Insert after `pub use error::QuartalError;` and before `pub use interval::{...}` (alphabetical):

```rust
pub use functional::{
    quartal_orbits_in, quartal_pathway_stops, FunctionalRegion, Pathway,
};
```

(Four symbols; alphabetised within the brace.)

- [ ] **Step 2.5: Run tests; confirm functional tests pass**

Run: `cargo test --test tests quartal::test_functional 2>&1 | tail -25`
Expected: 14 tests pass.

If `test_all_14_quartal_orbits_map_to_correct_region` fails, the most likely cause is an off-by-one in `to_quintal()` (or someone silently changed the orbit declaration order). Compare the failing variant's `.to_quintal()` output against the spec table.

- [ ] **Step 2.6: Run full test suite + doctests**

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected after Task 2:

- lib unit: **58** (unchanged)
- the unrelated three suites (86, 34, 6): unchanged
- integration `tests` target: **633** (baseline 619 + 14 new functional tests)
- doctests: **29** (baseline 26 + 3 new in `functional.rs`)
- total: **846 passed** (baseline 829 + 17)

Run: `cargo test --features midi 2>&1 | grep -E "^test result:"`
Expected: total **851 passed** (baseline 834 + 17).

- [ ] **Step 2.7: Lint + doc check**

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: 0.

Run: `cargo clippy --all-targets --features midi 2>&1 | grep -cE "^warning"`
Expected: 0.

Run: `cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"`
Expected: 0.

Likely clippy candidates and fixes:
- `clippy::needless_pass_by_value` on `quartal_orbits_in(region: FunctionalRegion)` — `FunctionalRegion` is `Copy` (size 1, single-byte enum); passing by value is fine. Suppress only if clippy actually flags it, with a one-line `#[allow]` and an inline comment.
- `clippy::redundant_closure` on the `.map(|&region| ...)` — leave as-is; the explicit closure clarifies the dereference.

- [ ] **Step 2.8: Commit**

```bash
git add crates/mt/src/quartal/functional.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_functional.rs
git commit -m "feat(quartal): add functional module — perspective-invariant region/pathway re-exports + quartal helpers (oth4-T2)"
```

---

### Task 3: Final acceptance verification + ledger report

- [ ] **Step 3.1: Walk the spec §7 acceptance criteria one by one**

| Criterion | Command | Pass condition |
|---|---|---|
| `cargo build` succeeds | `cargo build` | exit 0, no new warnings |
| `cargo build --features midi` succeeds | `cargo build --features midi` | exit 0, no new warnings |
| `cargo test` passes | `cargo test 2>&1 \| grep -E "^test result:"` | total **846** passing |
| `cargo test --features midi` passes | `cargo test --features midi 2>&1 \| grep -E "^test result:"` | total **851** passing |
| `cargo clippy --all-targets` clean | `cargo clippy --all-targets 2>&1 \| grep -cE '^warning'` | 0 |
| `cargo clippy --all-targets --features midi` clean | `cargo clippy --all-targets --features midi 2>&1 \| grep -cE '^warning'` | 0 |
| `cargo doc --no-deps` clean | `cargo doc --no-deps 2>&1 \| grep -ciE 'missing\|broken'` | 0 |
| `cargo fmt --check` clean | `cargo fmt --check` | empty output |
| Every public fn has `#[must_use]` + doctest | manual eyeball of `functional.rs` | three `#[must_use]` + three `# Examples` blocks |
| No changes outside `quartal/` | `git diff $(git merge-base HEAD main) --stat \| grep -vE 'crates/mt/(src\|tests)/quartal/\|oth4/'` | empty |
| Serde feature flag honoured | (no new public types) | manual confirmation — re-exports inherit cfg_attr from quintal |

- [ ] **Step 3.2: Confirm scope compliance (spec §9)**

Run: `git diff $(git merge-base HEAD main) --stat`
Expected output should mention only:
- `crates/mt/src/quartal/functional.rs` (new)
- `crates/mt/src/quartal/mod.rs` (modified)
- `crates/mt/tests/quartal/test_functional.rs` (new)
- `crates/mt/tests/quartal/mod.rs` (modified)
- `oth4/oth4-T2-ledger.md` (new — explicit deliverable per spec §8)

5 file changes total. If anything else appears, revert it (or document why in the ledger §7/§9).

- [ ] **Step 3.3: Generate the ledger report**

Create `oth4/oth4-T2-ledger.md` per spec §8. Section-by-section content guidance:

- **§1 Summary:** one paragraph; mention F-T2-001 (no `FunctionalRegion`/`Pathway` re-export conflict — direct adoption of D-quartal-T2-002), F-T2-002 (per-region orderings tightened to exact equality), F-T2-003 (pathway stop sanity check), plus the four pre-recorded D-quartal-T2 spec decisions adopted as-is.
- **§2 Files Created:** four files with line counts (`wc -l <path>`).
- **§3 Files Modified:** the two `mod.rs` files; one sentence each.
- **§4 Public API Surface Added:**
  - Inherent method: `QuartalOrbit::functional_region(&self) -> FunctionalRegion`
  - Free fns: `quartal::quartal_orbits_in(FunctionalRegion) -> Vec<QuartalOrbit>`, `quartal::quartal_pathway_stops(Pathway) -> Vec<(FunctionalRegion, Vec<QuartalOrbit>)>`
  - Re-exports: `quartal::FunctionalRegion`, `quartal::Pathway`
- **§5 Tests Added:** 14 integration tests in `test_functional.rs` (table-driven mapping + 7 region partitions + 3 pathway tests + 2 re-export reachability tests) + 3 doctests = **17 net test additions**.
- **§6 Test Results:** plug in actual numbers from §3.1 (expected: 846 default, 851 midi, 0 clippy / 0 doc / clean fmt).
- **§7 Specced but Skipped:** none expected.
- **§8 Added Beyond Spec:**
  - The plan tightened the spec's "Returns N orbits including X, Y, Z" wording to **exact-equality assertions** with the deterministic per-region orderings (F-T2-002). Tests are stricter than the spec required; this is a strengthening, not a deviation.
  - `test_quartal_orbits_in_partition_is_complete` cross-checks against `QuartalOrbit::all()` (every variant present exactly once); spec asked only for "every variant in exactly one region's list".
- **§9 Judgment Calls:**
  - **D-quartal-T2-001 (spec):** `QuartalOrbit::functional_region` delegates to `self.to_quintal().functional_region()`. Adopted as-is.
  - **D-quartal-T2-002 (spec):** `FunctionalRegion` and `Pathway` re-exposed from `quartal::functional`. Per F-T2-001, no existing `pub use crate::quintal::FunctionalRegion` to compete with, so no `mod.rs` cleanup needed.
  - **D-quartal-T2-003 (spec):** `QuartalFunctionalSummary`-style speculative struct deferred. T2 ships the three new public items only.
  - **D-quartal-T2-004 (spec):** T2 does not edit `CLAUDE.md`. The stale `module_inception` warning expectation is now flagged in three consecutive ledger reports (T0, T1, T2) — a one-line correction commit can land outside any T-phase scope.
  - **F-T2-002 strengthening:** tests use exact-equality on `quartal_orbits_in` outputs. Catches any future regression in `QuartalOrbit::all()` ordering as a side effect.
- **§10 Open Questions:** flagged repeatedly in T0/T1/T2 ledgers — the `CLAUDE.md` `module_inception` staleness fix; `QuartalFunctionalSummary` design (defer to walkthrough T4).

- [ ] **Step 3.4: Commit the ledger**

```bash
git add oth4/oth4-T2-ledger.md
git commit -m "docs(oth4): add T2 ledger report"
```

---

## 5. Verification (end-to-end smoke)

After all three tasks land, the following sequence should pass on a clean checkout of the implementation branch:

```bash
git status                                                    # clean tree, on the impl branch
cargo fmt --check                                             # exit 0, empty output
cargo build                                                   # exit 0
cargo build --features midi                                   # exit 0
cargo test 2>&1 | grep -E "^test result:"                     # all six lines, 846 total passed
cargo test --features midi 2>&1 | grep -E "^test result:"     # 851 total passed
cargo clippy --all-targets 2>&1 | grep -cE "^warning"         # 0
cargo clippy --all-targets --features midi 2>&1 | grep -cE "^warning"  # 0
cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"         # 0
git diff $(git merge-base HEAD main) --stat | wc -l           # 5 file changes
```

Manual eyeball check:
- Open `target/doc/music_comp_mt/quartal/functional/index.html` — module renders with docstring, the three new public items each have a one-line summary, intra-doc links to quintal counterparts, runnable doctests.
- Open `target/doc/music_comp_mt/quartal/struct.QuartalOrbit.html` (or wherever the inherent method appears) — `functional_region` shows up as an inherent method on `QuartalOrbit`, with its doctest visible.
- Confirm `Cargo.toml` MSRV (`rust-version`) is unchanged.
- Confirm `Cargo.lock` is unchanged (no new dependencies).

---

## 6. Out of Scope (do not do — spec §9)

- ❌ Adding any module other than `functional.rs`.
- ❌ Modifying `crates/mt/src/quintal/` in any way.
- ❌ Adding MCP tools (T3).
- ❌ Adding speculative narrative-helper structs (`QuartalFunctionalSummary`, `QuartalRegionDetail`, etc.).
- ❌ Implementing a direct 14-arm `match` for `QuartalOrbit::functional_region` — use delegation per D-quartal-T2-001.
- ❌ Adding new `Display` impls — re-exported types' `Display` carries through transparently.
- ❌ Updating `CLAUDE.md` (D-quartal-T2-004).
- ❌ Refactoring existing quartal modules (constructors, types, voicing, …).
- ❌ Adding new dependencies to `Cargo.toml`.

If a spec point is unclear during implementation, the spec instruction (§9 last line) is: prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 7. Self-review checklist (run this after Task 3 completes)

- [ ] Every spec §2 / §3 file appears in the diff.
- [ ] Every spec §4.2 public item is present in `functional.rs` with `#[must_use]` and at least one doctest.
- [ ] Every spec §6.1 test is present (with the F-T2-002 strengthening to exact equality).
- [ ] No `todo!()` or placeholder remains in shipped code.
- [ ] Every `pub fn` / `pub use` in `functional.rs` has a rustdoc comment containing at least one intra-doc link to its quintal dual.
- [ ] `cargo fmt --check` is clean.
- [ ] `cargo clippy --all-targets` emits 0 warnings.
- [ ] `cargo doc --no-deps` emits 0 warnings (missing-docs + broken intra-doc).
- [ ] No file outside `crates/mt/src/quartal/`, `crates/mt/tests/quartal/`, or the new `oth4/oth4-T2-ledger.md` is touched.

---

*End of plan.*
