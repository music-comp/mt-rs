# Centrality Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a quartal-perspective `display` module and `centrality` module to `crates/mt/src/quartal/`, mirroring the organisational shape of `crates/mt/src/quintal/display.rs` and `crates/mt/src/quintal/centrality.rs`. No new mathematics; both modules wrap, relabel, or re-export existing quintal output.

**Architecture:** Two thin perspective-layer modules under `quartal/`. `display.rs` exposes (a) `render_quartal_chord_dashed` — a copy of quintal's `render_chord_dashed` with the legality predicate `(6..=8)` swapped for `(4..=6)`; (b) `render_quartal_is` — formats a `QuartalIntervalStructure` as `"a–b–c"`; (c) `pub use` re-exports of `pc_to_note_name` and `render_pcset_dashed` (perspective-invariant). `centrality.rs` exposes (a) `quartal_saddle_chords` — pairs each saddle PcChord with its `QuartalIntervalStructure`; (b) `quartal_orbits_by_betweenness` — all 14 `QuartalOrbit` variants ranked by max betweenness centrality of any member chord.

**Tech Stack:** Rust 2024 (already on `edition = "2024"`), `crate::quintal` for shared math (`betweenness_centrality`, `saddle_chords`, `classify_orbit`, `pc_to_note_name`, `render_pcset_dashed`), `super::types` / `super::orbit` for quartal labels. No new dependencies. Adheres to the project's Rust skill (`assets/ai/rust/SKILL.md`): rustdoc + doctests on every public item, `#[must_use]` on rendering helpers, intra-doc links to the dual quintal symbols, no `unwrap` outside test code (spec authorizes one `expect` in `quartal_saddle_chords` for an invariant condition).

**Source spec:** `docs/design/05-active/0009-oth4-t1-implementation-spec.md` (oth4-T1-spec).

**Predecessor:** `docs/dev/quartal/0005-oth4-t0-quartal-duality-verification-implementation-plan.md` — T0 merged 2026-05-04, post-merge baseline is **809 tests / 0 clippy warnings / 0 doc warnings**.

---

## 1. Context

The `mt-rs` codebase has a comprehensive `quintal/` module covering the entire topological/algebraic apparatus. T0 (merged) added `quartal/duality.rs` and `quartal/verification.rs` to bring the quartal module to organisational symmetry with quintal on those two analytical surfaces. T1 closes two more gaps: a quartal-side `display` module (mirroring `quintal/display.rs`) and a quartal-side `centrality` module (giving quartal-labelled views over the perspective-invariant betweenness values that already live in quintal).

After T1, two gaps remain on the quartal side: `quartal/functional.rs` (T2) and the MCP tool surface (T3). The walkthrough begins after T3 (T4).

T1 must be small, mechanical, and well-tested — the spec author estimates ~100 lines source + ~150 lines tests, similar in scale to T0. Both T1 modules wrap or relabel existing quintal output. The display module reproduces the [6,8]-legal-walk algorithm with a [4,5,6] predicate (justified duplication; D-quartal-T1-001 in the spec). The centrality module aggregates betweenness values up to the `QuartalOrbit` level and pairs the saddle PcChords with their quartal IS for narrative readability.

---

## 2. Spec Deviations & Findings (for CDC review)

The spec is well-prepared and pre-records four decisions in §14. The following are minor refinements / corrections I noticed during exploration:

### F-T1-001 — `pc_to_note_name` / `render_pcset_dashed` are not currently re-exported from `crate::quartal`

**Spec §6.3 expects:** "Note that `pc_to_note_name` and `render_pcset_dashed` are now exposed via *both* the quartal display module and the existing quintal re-exports."

**Reality (`crates/mt/src/quartal/mod.rs:43-58`):** the bottom-of-file `pub use crate::quintal::{...}` block only re-exports `crossroads_chords`, `all_distances_from`, `center`, `diameter`, `distance`, `eccentricity`, `betweenness_centrality`, `saddle_chords`, `count_geodesics`, `geodesics`, `passing_chords`, `enumerate_all`, `is_adjacent`, `BaseSpace`, `FiberClass`, `PcChord`, and `min_voiced_chord_l1`. **Neither `pc_to_note_name` nor `render_pcset_dashed` is currently in that block.**

**Resolution:** D-quartal-T1-003 simplifies — add the re-exports through `display::` only; there is no existing `pub use` to compete with, so no duplicate-symbol warning will arise. No action needed in the bottom-of-file block.

### F-T1-002 — Doctest discipline reaffirmation

**Spec §12 asserts:** "every public function in T0 had at least one doctest. Maintain that."

**Reality:** T0's public functions (`quartal_reading`, `quintal_reading`, `t_quartal_reversal_equivalence`, `verify_quartal_universal_l1_law`, `verify_quartal_fiber_classes`) were rustdoc-only; they did not include `# Examples` doctest blocks. `cargo doc --no-deps` was clean nonetheless. The §12 claim is aspirational rather than descriptive of T0.

**Resolution:** for T1, follow the spirit of §12. The spec body §5.2 already provides doctests for the centrality functions — I will preserve them. The spec body §4.2 does **not** provide doctests for the display functions; I will **add** runnable doctests to `render_quartal_chord_dashed`, `render_quartal_is`, and (transitively) the re-exports' rustdoc to bring T1 into compliance. This is a small additive item, not a deviation, but flagging so CDC sees it.

### F-T1-003 — Hand-verification of test expectations

The four `render_quartal_chord_dashed` expectations in spec §7.1 were verified by hand against the algorithm (PERMS_4 + `(4..=6)` predicate + smallest-start-pc tiebreak):

| PcChord | Quartal orbit | [4,5,6]-legal walks | Smallest-start walk | Expected output |
|---|---|---|---|---|
| `[0, 2, 7, 9]` | Q555 (Summit) | one: `[9, 2, 7, 0]` (5,5,5) | `[9, 2, 7, 0]` | `"A–D–G–C"` ✓ |
| `[0, 2, 6, 8]` | Q646 (Saddle) | **two:** `[2, 8, 0, 6]` (6,4,6) **and** `[8, 2, 6, 0]` (6,4,6) | `[2, 8, 0, 6]` | `"D–G#–C–F#"` ✓ |
| `[0, 3, 8, 10]` | Q554 (asymmetric) | one: `[10, 3, 8, 0]` (5,5,4) | `[10, 3, 8, 0]` | `"A#–D#–G#–C"` ✓ |

The asymmetric Q554 case is a critical regression test: if a future refactor accidentally reverses the predicate direction, the resulting walk would be the reversed-and-complemented one (different output), and this test catches it.

The Q646 (Saddle) row admits **two** legal quartal walks, not one, because the orbit has a non-trivial T6 stabilizer — matching the quintal-side docstring on `render_chord_dashed` ("every chord with a non-trivial stabilizer (e.g. Q686, Q676) admits two"). The two walks differ only in starting pc (2 vs 8); both have identical (6, 4, 6) interval structure. The smallest-start tiebreak resolves to `[2, 8, 0, 6]` regardless, so the test outcome is unaffected — but the walk count is documented correctly here so the analysis matches the quintal precedent.

For Q555 (Summit) and Q554 (asymmetric), an exhaustive search confirmed exactly one legal walk each, so the smallest-start tiebreak is trivially unambiguous.

---

## 3. File Structure

```
crates/mt/src/quartal/
├── mod.rs                  # MODIFY: declare two new mods, two new pub-use blocks
├── display.rs              # CREATE: ~50 lines, 2 fns + 2 re-exports + PERMS_4 const
└── centrality.rs           # CREATE: ~55 lines, 2 fns + doctests

crates/mt/tests/quartal/
├── mod.rs                  # MODIFY: declare two new test modules
├── test_display.rs         # CREATE: ~110 lines, 9 tests
└── test_centrality.rs      # CREATE: ~70 lines, 6 tests
```

Total: ~310 lines added across 5 files (excluding the ledger doc). Spec §13 caps at ~350; safely respected.

### Critical existing files / utilities to reuse (do not reinvent)

| Path | What | Why we use it |
|---|---|---|
| `crates/mt/src/quintal/display.rs:25-50` | `PERMS_4: [[usize; 4]; 24]` constant | Duplicated locally per D-quartal-T1-001 (private in quintal) |
| `crates/mt/src/quintal/display.rs:160-186` | `render_chord_dashed` algorithm | Reference for the walk-finding loop; copy with `(6..=8)` → `(4..=6)` |
| `crates/mt/src/quintal/display.rs:69-87` | `pc_to_note_name` | Re-exported via `quartal/display` |
| `crates/mt/src/quintal/display.rs:105-112` | `render_pcset_dashed` | Re-exported via `quartal/display`; also fallback path |
| `crates/mt/src/quintal/centrality.rs:28-80` | `betweenness_centrality(&BaseSpace) -> HashMap<PcChord, f64>` | One-shot betweenness compute |
| `crates/mt/src/quintal/centrality.rs:98-103` | `saddle_chords(&BaseSpace) -> Vec<PcChord>` | Source of the saddle-six |
| `crates/mt/src/quintal/orbit.rs` (re-exported as `quintal::classify_orbit`) | `classify_orbit(&PcChord) -> Option<Orbit>` | Used by `quartal_orbits_by_betweenness` |
| `crates/mt/src/quartal/orbit.rs:131-148` | `QuartalOrbit::from_quintal(&Orbit) -> QuartalOrbit` | Quintal→quartal orbit conversion |
| `crates/mt/src/quartal/conversion.rs:19-22` | `pc_chord_quartal_intervals(&PcChord) -> Option<QuartalIntervalStructure>` | Used by `quartal_saddle_chords` |
| `crates/mt/src/quartal/types.rs:12` | `QuartalIntervalStructure(pub u8, pub u8, pub u8)` | Public-field tuple-struct; `.0/.1/.2` accessible directly |
| `crates/mt/src/quartal/orbit.rs:55-70` | `QuartalOrbit::all() -> &[QuartalOrbit; 14]` | Iteration + presence checks in tests |

---

## 4. Tasks

### Task 1: Baseline check + register new (empty) modules

**Files:**

- Create: `crates/mt/src/quartal/centrality.rs` (initially empty stub)
- Create: `crates/mt/src/quartal/display.rs` (initially empty stub)
- Modify: `crates/mt/src/quartal/mod.rs` (add two `mod` lines)
- Create: `crates/mt/tests/quartal/test_centrality.rs` (empty stub)
- Create: `crates/mt/tests/quartal/test_display.rs` (empty stub)
- Modify: `crates/mt/tests/quartal/mod.rs` (add two `mod` lines)

This task wires up empty modules so subsequent compile/test cycles are fast and incremental. We add the test mod declarations now too; empty test files compile cleanly.

- [ ] **Step 1.1: Confirm working tree + branch state**

Run: `git status -sb`
Expected: clean working tree; branch is a fresh feature branch (e.g. `feat/oth4-t1-quartal-display-centrality`). If still on `main`, create the branch:
`git checkout -b feat/oth4-t1-quartal-display-centrality`

- [ ] **Step 1.2: Capture baseline test count**

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected (post-T0 merge): six "test result" lines summing to **809 passed**, 0 failed (58 lib unit + 86 + 603 + 34 + 6 + 22 doc).

Run: `cargo test --features midi 2>&1 | grep -E "^test result:"`
Expected: 814 passed (63 lib unit + 86 + 603 + 34 + 6 + 22 doc).

Run: `cargo clippy --all-targets 2>&1 | tee /tmp/baseline-clippy.txt | grep -cE "^warning"`
Expected: **0**. (CLAUDE.md mentions "4 module_inception" but per T0 ledger §10 / spec §11 the actual baseline is 0; T1 gates on "no new warnings beyond 0".)

Run: `cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"`
Expected: 0.

Record all four numbers — the ledger §6 needs them.

- [ ] **Step 1.3: Create empty `centrality.rs` stub**

Create `crates/mt/src/quartal/centrality.rs` with only the module docstring:

```rust
//! Quartal-labelled centrality views — symmetric counterpart to
//! [`crate::quintal::centrality`].
//!
//! The mathematical content (Brandes' algorithm, betweenness centrality
//! values) lives in quintal. This module surfaces those values keyed by
//! [`super::orbit::QuartalOrbit`] and [`super::types::QuartalIntervalStructure`]
//! so quartal-minded callers can read centrality in their native vocabulary
//! without quintal-side conversion at every call site.
```

- [ ] **Step 1.4: Create empty `display.rs` stub**

Create `crates/mt/src/quartal/display.rs` with only the module docstring:

```rust
//! Quartal-perspective rendering helpers — symmetric counterpart to
//! [`crate::quintal::display`].
//!
//! Two perspectives, two stack walks. [`render_quartal_chord_dashed`] walks
//! a [`crate::quintal::PcChord`]'s pcs by [4,5,6]-legal forward intervals;
//! the quintal counterpart [`crate::quintal::render_chord_dashed`] uses
//! [6,7,8]. Both render the same chord, but in different stacking orders.
//!
//! Set-class identity (ascending-pc dashed form) is perspective-invariant;
//! [`render_pcset_dashed`] is re-exported as-is, alongside the spelling
//! helper [`pc_to_note_name`].
```

- [ ] **Step 1.5: Wire `mod` declarations in `crates/mt/src/quartal/mod.rs`**

Current `mod` block (lines 8–17):

```rust
mod constructors;
mod conversion;
mod duality;
mod error;
mod interval;
mod modes;
mod orbit;
mod types;
mod verification;
mod voicing;
```

Insert `centrality` (before `constructors`) and `display` (after `conversion`) to preserve alphabetical order:

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

Do NOT add `pub use` lines yet — that comes after the module bodies are written. Adding empty re-export blocks now would emit "unused import" warnings.

- [ ] **Step 1.6: Create empty test files**

Create `crates/mt/tests/quartal/test_centrality.rs` with only:

```rust
extern crate music_comp_mt as theory;
```

Create `crates/mt/tests/quartal/test_display.rs` with only:

```rust
extern crate music_comp_mt as theory;
```

- [ ] **Step 1.7: Wire test `mod` declarations**

`crates/mt/tests/quartal/mod.rs` currently has eight entries (post-T0):

```rust
mod test_constructors;
mod test_duality;
mod test_modes;
mod test_quartal_quintal_identity;
mod test_root_constructors;
mod test_types;
mod test_verification;
mod test_voicing;
```

Insert `test_centrality` (before `test_constructors`) and `test_display` (after `test_duality`) for alphabetical order:

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

- [ ] **Step 1.8: Verify the empty wiring compiles cleanly**

Run: `cargo build`
Expected: succeeds, no new warnings.

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected: same baseline (809 passed) — the new test files declare zero tests.

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: 0.

- [ ] **Step 1.9: Commit**

```bash
git add crates/mt/src/quartal/centrality.rs \
        crates/mt/src/quartal/display.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_centrality.rs \
        crates/mt/tests/quartal/test_display.rs \
        crates/mt/tests/quartal/mod.rs
git commit -m "feat(quartal): scaffold display and centrality module stubs (oth4-T1)"
```

---

### Task 2: Implement `quartal::display` (TDD)

**Files:**

- Modify: `crates/mt/src/quartal/display.rs`
- Modify: `crates/mt/src/quartal/mod.rs` (add `pub use display::{...}`)
- Modify: `crates/mt/tests/quartal/test_display.rs`

The module exposes:

| Symbol | Origin | Notes |
|---|---|---|
| `render_quartal_chord_dashed(&PcChord) -> String` | New (D-quartal-T1-001) | Copy of `quintal::render_chord_dashed` with `(6..=8)` → `(4..=6)` |
| `render_quartal_is(&QuartalIntervalStructure) -> String` | New | Trivial format helper |
| `pub use crate::quintal::pc_to_note_name` | Re-export | Perspective-invariant |
| `pub use crate::quintal::render_pcset_dashed` | Re-export | Perspective-invariant; also the fallback path |

- [ ] **Step 2.1: Write failing tests in `tests/quartal/test_display.rs`**

Replace the file contents with:

```rust
extern crate music_comp_mt as theory;

use theory::quartal::{
    pc_to_note_name, render_pcset_dashed, render_quartal_chord_dashed, render_quartal_is,
    QuartalIntervalStructure,
};
use theory::quintal::{render_chord_dashed as quintal_render_chord_dashed, PcChord};

// ───────────────────────────── render_quartal_chord_dashed ──────────────────

/// Q555 Summit case: pcs `[0, 2, 7, 9]` admits exactly one [4,5,6]-legal walk
/// — `9 → 2 → 7 → 0` with intervals `(5, 5, 5)`. Renders as `"A–D–G–C"`.
#[test]
fn test_render_quartal_chord_q555_summit() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
}

/// Pin BOTH renderers on the same chord so a regression in either one
/// surfaces immediately and the perspective-difference invariant is
/// preserved.
#[test]
fn test_render_quintal_chord_q555_summit_for_comparison() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(quintal_render_chord_dashed(&cgda), "C–G–D–A");
    assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
    assert_ne!(
        quintal_render_chord_dashed(&cgda),
        render_quartal_chord_dashed(&cgda),
        "the two perspectives must produce different walks"
    );
}

/// Q646 Saddle case: pcs `[0, 2, 6, 8]` admits one [4,5,6]-legal walk
/// — `2 → 8 → 0 → 6` with intervals `(6, 4, 6)`. Renders as `"D–G#–C–F#"`.
#[test]
fn test_render_quartal_chord_q646_saddle() {
    let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&saddle), "D–G#–C–F#");
}

/// ASYMMETRIC Q554 case: pcs `[0, 3, 8, 10]` admits one [4,5,6]-legal walk
/// — `10 → 3 → 8 → 0` with intervals `(5, 5, 4)`. Renders as `"A#–D#–G#–C"`.
/// This is the regression test for any future predicate-direction bug:
/// a buggy implementation that reverses the predicate would produce the
/// reversed-and-complemented walk and fail this assertion.
#[test]
fn test_render_quartal_chord_q554_asymmetric() {
    let q554 = PcChord::new([0, 3, 8, 10]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&q554), "A#–D#–G#–C");
}

/// Non-[4,5,6]-legal chord falls back to ascending-pc rendering, equivalent
/// to `render_pcset_dashed`. Pins the documented total-function behaviour.
#[test]
fn test_render_quartal_chord_falls_back_for_non_legal_chord() {
    let bogus = PcChord::new([0, 1, 2, 3]).unwrap();
    let walk = render_quartal_chord_dashed(&bogus);
    let pcset = render_pcset_dashed(&bogus);
    assert_eq!(walk, pcset);
    assert_eq!(walk, "C–C#–D–D#");
}

// ─────────────────────────── render_pcset_dashed re-export ──────────────────

/// `render_pcset_dashed` is reachable via `crate::quartal::*` and behaves
/// identically to the quintal counterpart (perspective-invariant).
#[test]
fn test_render_quartal_pcset_re_export() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_pcset_dashed(&cgda), "C–D–G–A");
}

// ─────────────────────────── pc_to_note_name re-export ──────────────────────

#[test]
fn test_pc_to_note_name_re_export() {
    assert_eq!(pc_to_note_name(0), "C");
    assert_eq!(pc_to_note_name(8), "G#");
    // mod-12 wrap: pc=13 normalises to pc=1 → "C#"
    assert_eq!(pc_to_note_name(13), "C#");
}

// ─────────────────────────── render_quartal_is ──────────────────────────────

#[test]
fn test_render_quartal_is_summit() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(5, 5, 5)),
        "5–5–5"
    );
}

#[test]
fn test_render_quartal_is_saddle() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(6, 4, 6)),
        "6–4–6"
    );
}

/// ASYMMETRIC IS (5, 5, 4) — pins component-ordering invariance: the
/// renderer must NOT sort or rearrange components.
#[test]
fn test_render_quartal_is_asymmetric() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(5, 5, 4)),
        "5–5–4"
    );
}
```

(10 tests rather than the spec's 9 — added `test_render_quartal_chord_falls_back_for_non_legal_chord` to mirror the analogous quintal test in `tests/quintal/test_display.rs`. Adds ~5 lines of source budget. Documented in §8 of the ledger.)

- [ ] **Step 2.2: Run failing tests to confirm they fail to compile**

Run: `cargo test --test tests quartal::test_display 2>&1 | tail -25`
Expected: `error[E0432]: unresolved imports` for `pc_to_note_name`, `render_pcset_dashed`, `render_quartal_chord_dashed`, `render_quartal_is` (none yet exported from `crate::quartal`).

If errors are different, stop and reconcile.

- [ ] **Step 2.3: Write the display module body**

Replace `crates/mt/src/quartal/display.rs` with:

```rust
//! Quartal-perspective rendering helpers — symmetric counterpart to
//! [`crate::quintal::display`].
//!
//! Two perspectives, two stack walks. [`render_quartal_chord_dashed`] walks
//! a [`PcChord`]'s pcs by [4,5,6]-legal forward intervals; the quintal
//! counterpart [`crate::quintal::render_chord_dashed`] uses [6,7,8]. Both
//! render the same chord, but in different stacking orders.
//!
//! Set-class identity (ascending-pc dashed form) is perspective-invariant;
//! [`render_pcset_dashed`] is re-exported as-is, alongside the spelling
//! helper [`pc_to_note_name`].

use crate::quintal::PcChord;

use super::types::QuartalIntervalStructure;

// Re-exports of perspective-invariant utilities.

/// Spell a pitch class as a sharps-only note name.
///
/// Re-exported from [`crate::quintal::pc_to_note_name`].
pub use crate::quintal::pc_to_note_name;

/// Render a chord as ascending dashed note names.
///
/// Re-exported from [`crate::quintal::render_pcset_dashed`].
/// The pitch-class-set form is perspective-invariant — both quartal and
/// quintal callers see the same string for the same chord.
pub use crate::quintal::render_pcset_dashed;

/// All 24 permutations of `[0, 1, 2, 3]` — used to scan for a `[4, 6]`-legal
/// walk through a chord's four pcs.
///
/// **Justified duplication (D-quartal-T1-001):** a verbatim copy of the
/// `PERMS_4` constant in [`crate::quintal::display`] (which is private).
/// Extracting a shared helper would touch quintal and is out of T1 scope.
const PERMS_4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// Render a [`PcChord`] in **quartal root form** — the four pcs ordered by
/// the chord's canonical `[4, 6]` stack walk, dashed by en-dash.
///
/// For pcs `[0, 2, 7, 9]` (Q777 / Q555) this yields `"A–D–G–C"` — the same
/// chord that [`crate::quintal::render_chord_dashed`] renders as
/// `"C–G–D–A"`, read here as a stack of fourths bottom-up. For pcs
/// `[0, 2, 6, 8]` (Q686 / Q646 Saddle) this yields `"D–G#–C–F#"`.
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
/// For chords with no `[4, 6]`-legal walk (possible only for `PcChord`
/// values constructed outside [`crate::quintal::BaseSpace`]), falls back
/// to ascending-pc rendering — equivalent to [`render_pcset_dashed`].
/// Total over all `PcChord` values; never panics.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::render_quartal_chord_dashed;
/// use music_comp_mt::quintal::PcChord;
///
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
///
/// let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
/// assert_eq!(render_quartal_chord_dashed(&saddle), "D–G#–C–F#");
/// ```
#[must_use]
pub fn render_quartal_chord_dashed(chord: &PcChord) -> String {
    let pcs = chord.pcs;
    let mut legal_walks: Vec<[u8; 4]> = PERMS_4
        .iter()
        .map(|perm| [pcs[perm[0]], pcs[perm[1]], pcs[perm[2]], pcs[perm[3]]])
        .filter(|walk| {
            let i1 = (walk[1] as i16 - walk[0] as i16).rem_euclid(12);
            let i2 = (walk[2] as i16 - walk[1] as i16).rem_euclid(12);
            let i3 = (walk[3] as i16 - walk[2] as i16).rem_euclid(12);
            (4..=6).contains(&i1) && (4..=6).contains(&i2) && (4..=6).contains(&i3)
        })
        .collect();

    // Fallback for non-[4,6]-legal chords (PcChord allows them; BaseSpace
    // does not). Documented above; mirrors the quintal-side fallback.
    if legal_walks.is_empty() {
        return render_pcset_dashed(chord);
    }

    // Tiebreak: smallest starting pc, then lex-smallest full walk.
    legal_walks.sort();
    let walk = &legal_walks[0];
    walk.iter()
        .map(|&p| pc_to_note_name(p))
        .collect::<Vec<_>>()
        .join("–")
}

/// Render a [`QuartalIntervalStructure`] as dashed semitone counts.
///
/// `(5, 5, 5) → "5–5–5"`, `(6, 4, 6) → "6–4–6"`, `(5, 5, 4) → "5–5–4"`.
/// Used in narrative documents and table outputs where the quartal IS
/// should appear as a single readable cell.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{render_quartal_is, QuartalIntervalStructure};
///
/// assert_eq!(
///     render_quartal_is(&QuartalIntervalStructure(5, 5, 5)),
///     "5–5–5"
/// );
/// assert_eq!(
///     render_quartal_is(&QuartalIntervalStructure(5, 5, 4)),
///     "5–5–4"
/// );
/// ```
#[must_use]
pub fn render_quartal_is(is: &QuartalIntervalStructure) -> String {
    format!("{}–{}–{}", is.0, is.1, is.2)
}
```

- [ ] **Step 2.4: Add `pub use` block to `crates/mt/src/quartal/mod.rs`**

Insert after the `pub use conversion::{...}` line and before `pub use duality::{...}` (alphabetical order):

```rust
pub use display::{
    pc_to_note_name, render_pcset_dashed, render_quartal_chord_dashed, render_quartal_is,
};
```

(Four symbols, alphabetised within the brace.)

- [ ] **Step 2.5: Run tests; confirm display tests pass**

Run: `cargo test --test tests quartal::test_display 2>&1 | tail -20`
Expected: 10 tests pass.

If the asymmetric Q554 test fails, the most likely cause is a predicate direction or PERMS_4 transcription bug. Compare the body byte-for-byte against `crates/mt/src/quintal/display.rs` (only the predicate range should differ).

- [ ] **Step 2.6: Run full test suite + doctests**

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected after Task 2:

- lib unit: **58** (unchanged from baseline)
- the unrelated three suites (86, 34, 6): unchanged
- integration `tests` target: **613** (baseline 603 + 10 new display tests)
- doctests: **24** (baseline 22 + 2 new in `display.rs`)
- total: **821 passed** (baseline 809 + 12)

Run: `cargo test --features midi 2>&1 | grep -E "^test result:"`
Expected: total **826 passed** (baseline 814 + 12).

- [ ] **Step 2.7: Lint + doc check**

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: 0.

Run: `cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"`
Expected: 0.

If clippy complains about `#[must_use]` on a function returning a non-`Result` type, follow its suggestion. If `cargo doc` flags a broken intra-doc link (`[PcChord]` resolving incorrectly, etc.), fix the path inline by writing `[crate::quintal::PcChord]`.

- [ ] **Step 2.8: Commit**

```bash
git add crates/mt/src/quartal/display.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_display.rs
git commit -m "feat(quartal): add display module — quartal-perspective rendering (oth4-T1)"
```

---

### Task 3: Implement `quartal::centrality` (TDD)

**Files:**

- Modify: `crates/mt/src/quartal/centrality.rs`
- Modify: `crates/mt/src/quartal/mod.rs` (add `pub use centrality::{...}`)
- Modify: `crates/mt/tests/quartal/test_centrality.rs`

The module exposes:

| Symbol | Signature | Source |
|---|---|---|
| `quartal_saddle_chords` | `fn(&BaseSpace) -> Vec<(PcChord, QuartalIntervalStructure)>` | New — pairs `quintal::saddle_chords` output with `pc_chord_quartal_intervals` |
| `quartal_orbits_by_betweenness` | `fn(&BaseSpace) -> Vec<(QuartalOrbit, f64)>` | New — aggregates `betweenness_centrality` by orbit, sorted descending |

- [ ] **Step 3.1: Write failing tests in `tests/quartal/test_centrality.rs`**

Replace the file contents with:

```rust
extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, quartal_orbits_by_betweenness, quartal_saddle_chords, QuartalIntervalStructure,
    QuartalOrbit,
};
use theory::quintal::{saddle_chords as quintal_saddle_chords, PcChord};

#[test]
fn test_quartal_saddle_chords_count_and_is() {
    let space = base_space();
    let saddle = quartal_saddle_chords(&space);
    assert_eq!(saddle.len(), 6);
    // Saddle orbit Q646 is palindromic: every pair has the same IS (6, 4, 6).
    for (chord, is) in &saddle {
        assert_eq!(
            *is,
            QuartalIntervalStructure(6, 4, 6),
            "chord {:?} did not produce QuartalIntervalStructure(6, 4, 6)",
            chord
        );
    }
}

#[test]
fn test_quartal_saddle_chords_match_quintal() {
    // The PcChord set returned by quartal_saddle_chords (drop the IS) must
    // equal the Vec<PcChord> returned by quintal::saddle_chords.
    let space = base_space();
    let quartal_pcs: Vec<PcChord> = quartal_saddle_chords(&space)
        .into_iter()
        .map(|(chord, _)| chord)
        .collect();
    let quintal_pcs = quintal_saddle_chords(&space);
    assert_eq!(quartal_pcs, quintal_pcs);
}

#[test]
fn test_quartal_orbits_by_betweenness_count() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    assert_eq!(ranked.len(), 14);
}

#[test]
fn test_quartal_orbits_by_betweenness_top_is_saddle() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    assert_eq!(ranked[0].0, QuartalOrbit::Q646);
    // The saddle's max betweenness is documented as ~0.139; be generous and
    // assert it's noticeably above the next-highest orbit's value.
    assert!(
        ranked[0].1 > ranked[1].1,
        "Saddle (Q646) betweenness should strictly exceed the next orbit's"
    );
}

#[test]
fn test_quartal_orbits_by_betweenness_descending() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    for window in ranked.windows(2) {
        assert!(
            window[0].1 >= window[1].1,
            "betweenness must be non-strictly descending: {:?} < {:?}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn test_quartal_orbits_by_betweenness_no_duplicates() {
    let space = base_space();
    let ranked = quartal_orbits_by_betweenness(&space);
    let mut seen = std::collections::BTreeSet::new();
    for (orbit, _) in &ranked {
        assert!(
            seen.insert(*orbit),
            "duplicate QuartalOrbit in ranking: {:?}",
            orbit
        );
    }
    // Every variant must appear.
    for variant in QuartalOrbit::all() {
        assert!(
            seen.contains(variant),
            "QuartalOrbit::{:?} missing from ranking",
            variant
        );
    }
}
```

- [ ] **Step 3.2: Run failing tests to confirm they don't compile**

Run: `cargo test --test tests quartal::test_centrality 2>&1 | tail -15`
Expected: `error[E0432]` on imports of `quartal_orbits_by_betweenness` and `quartal_saddle_chords`.

- [ ] **Step 3.3: Write the centrality module body**

Replace `crates/mt/src/quartal/centrality.rs` with:

```rust
//! Quartal-labelled centrality views — symmetric counterpart to
//! [`crate::quintal::centrality`].
//!
//! The mathematical content (Brandes' algorithm, betweenness centrality
//! values) lives in quintal. This module surfaces those values keyed by
//! [`QuartalOrbit`] and [`QuartalIntervalStructure`] so quartal-minded
//! callers can read centrality in their native vocabulary without
//! quintal-side conversion at every call site.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::quintal::{betweenness_centrality, classify_orbit, saddle_chords, BaseSpace, PcChord};

use super::conversion::pc_chord_quartal_intervals;
use super::orbit::QuartalOrbit;
use super::types::QuartalIntervalStructure;

/// Saddle chords paired with their quartal interval structures.
///
/// Identical PcChord set to [`crate::quintal::saddle_chords`] (the top six
/// chords by betweenness centrality — all members of the Saddle orbit
/// `Q686` / `Q646`), paired with each chord's quartal IS for direct
/// quartal-perspective inspection.
///
/// All six pairs return `QuartalIntervalStructure(6, 4, 6)` — the canonical
/// `Q646` quartal IS — because the Saddle orbit is palindromic.
///
/// # Panics
///
/// Panics if any saddle chord lacks a legal quintal interval structure (and
/// therefore no quartal IS). Saddle chords are always members of `Q686`,
/// which has a legal IS, so this should never happen on the canonical base
/// space; the `expect` exists to surface a regression elsewhere in the
/// codebase rather than silently mis-pair the IS.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{base_space, quartal_saddle_chords};
///
/// let space = base_space();
/// let saddle = quartal_saddle_chords(&space);
/// assert_eq!(saddle.len(), 6);
/// ```
#[must_use]
pub fn quartal_saddle_chords(space: &BaseSpace) -> Vec<(PcChord, QuartalIntervalStructure)> {
    saddle_chords(space)
        .into_iter()
        .map(|chord| {
            let is = pc_chord_quartal_intervals(&chord)
                .expect("saddle chord must have a legal quartal interval structure");
            (chord, is)
        })
        .collect()
}

/// All 14 quartal orbits ranked by maximum betweenness centrality of any
/// member chord (descending).
///
/// Ties are broken by the natural ordering of [`QuartalOrbit`] (the
/// declaration order in [`super::orbit`]).
///
/// # Algorithm
///
/// 1. Compute [`crate::quintal::betweenness_centrality`] once — `O(V·E)`.
/// 2. For each `PcChord` in the result, look up its quintal `Orbit` via
///    [`crate::quintal::classify_orbit`] and convert to [`QuartalOrbit`]
///    via [`QuartalOrbit::from_quintal`].
/// 3. Aggregate per `QuartalOrbit`: keep the maximum betweenness across
///    all members.
/// 4. Sort by descending betweenness, with ties broken by the natural
///    `QuartalOrbit` ordering.
///
/// `f64` ordering uses `partial_cmp(...).unwrap_or(Ordering::Equal)`;
/// `betweenness_centrality` produces non-negative finite values
/// (Brandes' algorithm on a non-empty connected graph) so `NaN` never
/// arises.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{base_space, quartal_orbits_by_betweenness, QuartalOrbit};
///
/// let space = base_space();
/// let ranked = quartal_orbits_by_betweenness(&space);
/// assert_eq!(ranked.len(), 14);
/// // The Saddle (Q646) is at the top.
/// assert_eq!(ranked[0].0, QuartalOrbit::Q646);
/// ```
#[must_use]
pub fn quartal_orbits_by_betweenness(space: &BaseSpace) -> Vec<(QuartalOrbit, f64)> {
    let bc = betweenness_centrality(space);

    let mut max_per_orbit: HashMap<QuartalOrbit, f64> = HashMap::new();
    for (chord, value) in &bc {
        if let Some(orb) = classify_orbit(chord) {
            let q_orb = QuartalOrbit::from_quintal(&orb);
            max_per_orbit
                .entry(q_orb)
                .and_modify(|cur| {
                    if *value > *cur {
                        *cur = *value;
                    }
                })
                .or_insert(*value);
        }
    }

    let mut ranked: Vec<(QuartalOrbit, f64)> = max_per_orbit.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
}
```

**Implementation note:** `QuartalOrbit` derives `PartialOrd` + `Ord` (see `quartal/orbit.rs:21`), so `a.0.cmp(&b.0)` for the secondary sort is well-defined and follows declaration order.

- [ ] **Step 3.4: Add `pub use` block to `crates/mt/src/quartal/mod.rs`**

Insert at the very top of the `pub use` blocks (alphabetical order — `centrality` precedes `constructors`):

```rust
pub use centrality::{quartal_orbits_by_betweenness, quartal_saddle_chords};
```

- [ ] **Step 3.5: Run the centrality tests**

Run: `cargo test --test tests quartal::test_centrality 2>&1 | tail -15`
Expected: 6 tests pass.

- [ ] **Step 3.6: Run full test suite + doctests**

Run: `cargo test 2>&1 | grep -E "^test result:"`
Expected after Task 3 (cumulative with Task 2):

- lib unit: **58** (unchanged)
- integration `tests` target: **619** (baseline 603 + 10 display + 6 centrality = +16)
- doctests: **26** (baseline 22 + 2 display + 2 centrality = +4)
- total: **829 passed** (baseline 809 + 20)

Run: `cargo test --features midi 2>&1 | grep -E "^test result:"`
Expected: total **834 passed** (baseline 814 + 20).

- [ ] **Step 3.7: Lint + doc check**

Run: `cargo clippy --all-targets 2>&1 | grep -cE "^warning"`
Expected: 0.

Run: `cargo clippy --all-targets --features midi 2>&1 | grep -cE "^warning"`
Expected: 0.

Run: `cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"`
Expected: 0.

Likely clippy warning candidates and fixes:

- `clippy::float_cmp` on the `*value > *cur` comparison — this is an intentional comparison for max-aggregation; if flagged, add a `#[allow(clippy::float_cmp)]` with a comment, or use `f64::max(*cur, *value)`.
- Missing `# Errors` / `# Panics` section on `quartal_saddle_chords` — the body above already has `# Panics`.
- `clippy::redundant_closure` on the `.map(...)` — refactor if flagged.

- [ ] **Step 3.8: Commit**

```bash
git add crates/mt/src/quartal/centrality.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_centrality.rs
git commit -m "feat(quartal): add centrality module — quartal-labelled betweenness views (oth4-T1)"
```

---

### Task 4: Final acceptance verification + ledger report

- [ ] **Step 4.1: Walk the spec §8 acceptance criteria one by one**

| Criterion | Command | Pass condition |
|---|---|---|
| `cargo build` succeeds | `cargo build` | exit 0, no new warnings |
| `cargo build --features midi` succeeds | `cargo build --features midi` | exit 0, no new warnings |
| `cargo test` passes | `cargo test 2>&1 \| grep -E "^test result:"` | total 829 passing, 0 failing |
| `cargo test --features midi` passes | `cargo test --features midi 2>&1 \| grep -E "^test result:"` | total 834 passing, 0 failing |
| `cargo clippy --all-targets` clean | `cargo clippy --all-targets 2>&1 \| grep -cE '^warning'` | 0 |
| `cargo clippy --all-targets --features midi` clean | `cargo clippy --all-targets --features midi 2>&1 \| grep -cE '^warning'` | 0 |
| All public API has rustdoc + intra-doc link | `cargo doc --no-deps 2>&1 \| grep -ciE 'missing\|broken'` | 0 |
| `cargo fmt --check` clean | `cargo fmt --check` | empty output |
| No changes outside `quartal/` | `git diff $(git merge-base HEAD main) --stat \| grep -vE 'crates/mt/(src\|tests)/quartal/\|oth4/'` | empty |
| Serde feature flag honoured | (no new public types) | manual confirmation — no new `#[derive(Serialize, ...)]` |

- [ ] **Step 4.2: Confirm scope compliance (spec §10)**

Run: `git diff $(git merge-base HEAD main) --stat`
Expected output should mention only:

- `crates/mt/src/quartal/centrality.rs` (new)
- `crates/mt/src/quartal/display.rs` (new)
- `crates/mt/src/quartal/mod.rs` (modified)
- `crates/mt/tests/quartal/test_centrality.rs` (new)
- `crates/mt/tests/quartal/test_display.rs` (new)
- `crates/mt/tests/quartal/mod.rs` (modified)
- `oth4/oth4-T1-ledger.md` (new — explicit deliverable per spec §9)

If anything else appears, revert it (or document why in the ledger §7/§9).

- [ ] **Step 4.3: Generate the ledger report**

Create `oth4/oth4-T1-ledger.md` per spec §9. Section-by-section content guidance:

- **§1 Summary:** one paragraph; mention F-T1-001 (no `pc_to_note_name` re-export conflict) and F-T1-002 (T0 doctest discipline informally relaxed; T1 brings it back) plus the four pre-recorded D-quartal-T1 decisions adopted as-is.
- **§2 Files Created:** five files with line counts (`wc -l <path>`).
- **§3 Files Modified:** the two `mod.rs` files; one sentence each.
- **§4 Public API Surface Added:** `quartal::render_quartal_chord_dashed`, `quartal::render_quartal_is`, `quartal::quartal_saddle_chords`, `quartal::quartal_orbits_by_betweenness`. Re-exports separately: `quartal::pc_to_note_name`, `quartal::render_pcset_dashed`.
- **§5 Tests Added:** 10 in `test_display.rs` (9 spec + 1 fallback regression) + 6 in `test_centrality.rs` + 4 doctests (2 in display + 2 in centrality) = 20 net test additions.
- **§6 Test Results:** plug in actual numbers from §4.1.
- **§7 Specced but Skipped:** none expected.
- **§8 Added Beyond Spec:**
  - `test_render_quartal_chord_falls_back_for_non_legal_chord` — mirrors the analogous quintal test; pins the documented total-function behaviour and is a regression guard for the fallback path.
  - Doctests on `render_quartal_chord_dashed` and `render_quartal_is` — added per F-T1-002 (spec §12 doctest discipline).
- **§9 Judgment Calls:**
  - **D-quartal-T1-001 (spec):** `render_quartal_chord_dashed` reproduces the walk algorithm + `PERMS_4` constant locally rather than widening quintal's API. Adopted as-is.
  - **D-quartal-T1-002 (spec):** `QuartalCentralitySummary` deferred. T1 ships the two functions only; no struct.
  - **D-quartal-T1-003 (spec, refined):** `pc_to_note_name` and `render_pcset_dashed` re-exposed via `quartal/display`. Per F-T1-001 there is no existing `pub use crate::quintal::pc_to_note_name` to remove, so no `mod.rs` cleanup needed.
  - **D-quartal-T1-004 (spec):** T1 does not edit `CLAUDE.md`. Adopted as-is.
  - **F-T1-002 resolution:** added doctests to all four new public functions. Aligns T1 with the spec §12 stance even though T0 did not.
- **§10 Open Questions:** any signatures or design points CDC may want to revisit before T2.

- [ ] **Step 4.4: Commit the ledger**

```bash
git add oth4/oth4-T1-ledger.md
git commit -m "docs(oth4): add T1 ledger report"
```

---

## 5. Verification (end-to-end smoke)

After all four tasks land, the following sequence should pass on a clean checkout of the implementation branch:

```bash
git status                                                    # clean tree, on the impl branch
cargo fmt --check                                             # exit 0, empty output
cargo build                                                   # exit 0
cargo build --features midi                                   # exit 0
cargo test 2>&1 | grep -E "^test result:"                     # all six lines, 829 total passed
cargo test --features midi 2>&1 | grep -E "^test result:"     # 834 total passed
cargo clippy --all-targets 2>&1 | grep -cE "^warning"         # 0
cargo clippy --all-targets --features midi 2>&1 | grep -cE "^warning"  # 0
cargo doc --no-deps 2>&1 | grep -ciE "missing|broken"         # 0
git diff $(git merge-base HEAD main) --stat | wc -l           # 7 file changes
```

Manual eyeball check:

- Open `target/doc/music_comp_mt/quartal/index.html` — the new `display` and `centrality` submodules render with their docstrings, and every public symbol has a one-line summary plus an intra-doc link to its quintal counterpart.
- Open `target/doc/music_comp_mt/quartal/centrality/fn.quartal_orbits_by_betweenness.html` — the doctest example renders and is runnable.
- Confirm `Cargo.toml` MSRV (`rust-version`) is unchanged.
- Confirm `Cargo.lock` is unchanged (no new dependencies).

---

## 6. Out of Scope (do not do — spec §10)

- ❌ Adding any module other than `centrality.rs` and `display.rs`.
- ❌ Modifying `crates/mt/src/quintal/` in any way (including making `quintal::display::PERMS_4` public for sharing).
- ❌ Adding MCP tools (T3).
- ❌ Adding a `quartal/functional.rs` module (T2).
- ❌ Adding a `QuartalCentralitySummary` struct (deferred per D-quartal-T1-002).
- ❌ Refactoring `quintal/display`'s `render_chord_dashed` to extract a shared walk-finding helper.
- ❌ Updating `CLAUDE.md` to correct the stale "4 module_inception warnings" expectation (D-quartal-T1-004).
- ❌ Changing existing tests.
- ❌ Refactoring existing quartal modules (constructors, types, voicing, …).
- ❌ Adding new dependencies to `Cargo.toml`.

If a spec point is unclear during implementation, the spec instruction (§10 last line) is: prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 7. Self-review checklist (run this after Task 4 completes)

- [ ] Every spec §2 / §3 file appears in the diff.
- [ ] Every spec §4.2 public function (`render_quartal_chord_dashed`, `render_quartal_is`) is present in `display.rs` with `#[must_use]` and at least one doctest.
- [ ] Every spec §5.2 public function (`quartal_saddle_chords`, `quartal_orbits_by_betweenness`) is present in `centrality.rs` with `#[must_use]` and a doctest.
- [ ] Every spec §7.1 / §7.2 test is present (with the additional fallback regression test from §8).
- [ ] No `todo!()` or placeholder remains in shipped code.
- [ ] Every `pub fn` / `pub use` in the new modules has a rustdoc comment containing at least one intra-doc link to its quintal dual where one exists.
- [ ] `cargo fmt --check` is clean.
- [ ] `cargo clippy --all-targets` emits 0 warnings.
- [ ] `cargo doc --no-deps` emits 0 warnings (missing-docs + broken intra-doc).
- [ ] No file outside `crates/mt/src/quartal/`, `crates/mt/tests/quartal/`, or the new `oth4/oth4-T1-ledger.md` is touched.

---

*End of plan.*
