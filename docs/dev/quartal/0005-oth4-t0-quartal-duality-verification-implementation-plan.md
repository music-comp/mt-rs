# OTH4 — T0 Quartal Duality + Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a quartal-perspective `duality` module and `verification` module to `crates/mt/src/quartal/`, mirroring the organisational shape of `crates/mt/src/quintal/duality.rs` and `crates/mt/src/quintal/verification.rs`. No new mathematics; both modules wrap or delegate to the existing quintal apparatus.

**Architecture:** Two thin perspective-layer modules under `quartal/`. `duality.rs` exposes quartal-native readings (taking `&QuartalVoicedChord`), a quartal-direction fiber-reversal verifier, plus `pub use` re-exports of three perspective-independent quintal helpers. `verification.rs` exposes two structural-law verifiers (Universal L1 Law, fiber classes) computed via the quartal traversal infrastructure (`quartal_l1_distances`, `quartal_inversion_cycle`).

**Tech Stack:** Rust 2024 (already on `edition = "2024"`), `crate::quintal` for shared mathematics, `super::types` / `super::voicing` / `super::interval` for quartal scaffolding. No new dependencies. Adheres to the project's Rust skill (`assets/ai/rust/SKILL.md`): rustdoc on every public item, intra-doc links to the dual quintal symbols, `Result` propagation where applicable, no `unwrap`/`expect` outside test code.

**Source spec:** `docs/design/05-active/0008-oth4-t0-implementation-spec.md` (oth4-T0-spec).

**Revision history:**
- v1 (initial): drafted by CC against oth4-T0-spec.
- v2 (current): revised by Claude per `oth4/oth4-T0-plan-review.md`. Three concrete changes: (1) `quartal_reading` body uses `chord.quartal_interval_structure()` (the previous composition was buggy on asymmetric chords — it reverse-and-complemented a top-down IS, producing the reversal of the correct quartal label); (2) asymmetric-chord tests added to catch any future regression; (3) `test_quartal_verifiers_agree_with_quintal` strengthened to `.expect()` form; saddle fixture renamed to its quartal label for naming consistency.

---

## 1. Context

The `mt-rs` codebase has a comprehensive `quintal/` module covering the entire topological/algebraic apparatus (base space, distance, fiber bundle, duality, verification, functional grammar, …). The `quartal/` module is currently a thin perspective layer — it has its own types, constructors, conversions, traversal, and orbits, but is missing symmetric counterparts to two analytical quintal modules: `duality` and `verification`.

T0 closes that gap purely for codebase symmetry: a quartal-minded reader should find quartal-perspective entry points without grepping the quintal namespace. T0 lays groundwork for T1 (centrality + display), T2 (functional regions), T3 (MCP tooling), and T4 (walkthrough). It must be small, mechanical, and well-tested — the spec author estimates ~150 lines source + ~150 lines tests.

The two modules being mirrored (`crates/mt/src/quintal/duality.rs`, `crates/mt/src/quintal/verification.rs`) are already implemented and exercised by `crates/mt/tests/quintal/test_duality.rs` (15 tests) and `crates/mt/tests/quintal/test_verification.rs` (8 tests). The new quartal-side tests should produce equivalent numeric/boolean results on the same chords — that is the regression check.

---

## 2. Spec Deviations & Judgment Calls (for CDC review)

The spec is internally consistent in intent but contains three concrete inconsistencies between its body code and the actual state of `crates/mt/src/quintal/`. Calling them out here so CDC can adjudicate before the implementer commits:

### D-T0-001 — `verify_universal_l1_law` signature mismatch

**Spec §5.2 body code:** `pub fn verify_quartal_universal_l1_law() -> bool` (no args, bool return).

**Reality (`crates/mt/src/quintal/verification.rs:99`):**
```rust
pub fn verify_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>
```

**Spec §5.3 implementation notes:** "match that signature unless the existing quintal version returns a richer type. Inspect `quintal/verification.rs` and follow its style."

**Recommended resolution:** match quintal's signature exactly. New signature:
```rust
pub fn verify_quartal_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>
```

This keeps the two perspectives' verifiers structurally identical (so a future reader can diff them line-for-line) and preserves the rich failure information for debugging. Tests assert `.is_ok()` rather than `== true`.

### D-T0-002 — `verify_fiber_classes` signature mismatch

**Spec §5.2 body code:** `pub fn verify_quartal_fiber_classes() -> bool`.

**Reality (`crates/mt/src/quintal/verification.rs:123`):**
```rust
pub fn verify_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>
```

**Recommended resolution:** match quintal's signature. New signature:
```rust
pub fn verify_quartal_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>
```

The "verify" semantic is preserved by the test asserting the map contains 14 orbits with 11 ClassA + 3 ClassB, matching `test_verify_fiber_classes_count` in `tests/quintal/test_verification.rs`. The quartal verifier additionally cross-checks that `quartal_inversion_cycle` visits the same pc-chord set as `inversion_cycle` — this is the *added* value of running it separately.

### D-T0-003 — Test-chord construction wording

**Spec §7.1:** Tests reference "a `QuartalVoicedChord` constructed via `pure_quartal_stack(0)`" and assert `quartal_reading(&chord) == QuartalIntervalStructure(5, 5, 5)`.

**Reality:** `pure_quartal_stack(0)` returns a `PcChord` (pcs `[0, 3, 5, 10]`), not a `QuartalVoicedChord`. There are *two* natural ways to voice this PcChord:

| Voicing | MIDI pitches | Bottom-up quintal IS | `quartal_reading` per spec body code |
|---|---|---|---|
| Quartal-rooted (C-F-Bb-Eb)         | `[60, 65, 70, 75]` | `IS(5, 5, 5)` (illegal quintal) | `QIS(7, 7, 7)` ❌ |
| Quintal-rooted (Eb-Bb-F-C)         | `[51, 58, 65, 72]` | `IS(7, 7, 7)` (legal quintal)   | `QIS(5, 5, 5)` ✓ |

The spec assertion `QIS(5, 5, 5)` only matches when the QVC is the **quintal-stacked** voicing of the Q555 Summit PcChord, wrapped via `to_quartal`. Likewise the saddle `(6, 4, 6)` assertion only matches the quintal-stacked saddle voicing.

**Recommended resolution:** read "constructed via `pure_quartal_stack(0)`" as shorthand for "the PcChord from `pure_quartal_stack(0)`, voiced via its legal quintal stacking and then wrapped via `to_quartal`". The plan's test code below uses explicit `VoicedChord::new([51, 58, 65, 72])` (and an analogous saddle chord) rather than the ambiguous `pure_quartal_stack`-then-voice path, so the intent is unambiguous in the source.

---

## 3. File Structure

```
crates/mt/src/quartal/
├── mod.rs                  # MODIFY: declare two new mods, two new pub-use blocks
├── duality.rs              # CREATE: ~85 lines, 4 fns + 3 re-exports
└── verification.rs         # CREATE: ~70 lines, 2 verifier fns + 1 private helper

crates/mt/tests/quartal/
├── mod.rs                  # MODIFY: declare two new test modules
├── test_duality.rs         # CREATE: ~110 lines, 10 tests (8 spec + 2 asymmetric regression)
└── test_verification.rs    # CREATE: ~95 lines, 5 tests
```

Total: ~360 lines added across 5 files. The spec's ~400-line ceiling (§12) is safely respected.

### Critical existing files / utilities to reuse (do not reinvent)

| Path | What | Why we use it |
|---|---|---|
| `crates/mt/src/quintal/duality.rs` | `quartal_reading`, `quintal_reading`, `t1_reversal_equivalence`, `reverse_interval_structure`, `orbit_self_duality`, `verify_all_orbits_self_dual` | The mathematical content; quartal-side delegates here |
| `crates/mt/src/quintal/verification.rs` | `verify_universal_l1_law`, `verify_fiber_classes`, `inversion_cycle`-based helpers | Canonical quintal-direction verifiers; quartal verifiers cross-check against these |
| `crates/mt/src/quartal/interval.rs:27` | `quintal_to_quartal_structure` | Reverse-and-complement IS conversion |
| `crates/mt/src/quartal/voicing.rs:46` | `quartal_inversion_cycle` | Quartal-direction traversal |
| `crates/mt/src/quartal/voicing.rs:61` | `quartal_l1_distances` | Quartal-direction L1 pattern |
| `crates/mt/src/quartal/conversion.rs:7` | `to_quartal(&VoicedChord) -> QuartalVoicedChord` | Wrapping after quintal voicing |
| `crates/mt/src/quartal/types.rs:38` | `QuartalVoicedChord(pub VoicedChord)` | Public field `.0` for delegation |
| `crates/mt/src/quintal/base_space.rs` (via `quintal::BaseSpace`) | `BaseSpace::new()`, `space.chords()` | Source of all 228 PcChords |
| `crate::quintal::enumerate_all` | The 228 PcChords | Iteration target in verifiers |
| `crate::quintal::classify_orbit` | `PcChord → Option<Orbit>` | Used by `verify_quartal_fiber_classes` |

---

## 4. Tasks

### Task 1: Baseline check + register new (empty) modules

**Files:**
- Create: `crates/mt/src/quartal/duality.rs` (initially empty stub)
- Create: `crates/mt/src/quartal/verification.rs` (initially empty stub)
- Modify: `crates/mt/src/quartal/mod.rs` (add two `mod` lines)
- Create: `crates/mt/tests/quartal/test_duality.rs` (empty stub)
- Create: `crates/mt/tests/quartal/test_verification.rs` (empty stub)
- Modify: `crates/mt/tests/quartal/mod.rs` (add two `mod` lines)

This task wires up empty modules so subsequent compile/test cycles are fast and incremental. We add the test mod declarations now too; empty test files compile cleanly.

- [ ] **Step 1.1: Capture baseline test count**

Run: `cargo test 2>&1 | tail -20`
Record the exact "running N tests" / "test result: ok. X passed" lines for both unit and integration test runs. CLAUDE.md says 530+ tests; capture the actual number — the ledger §6 needs it.

Run: `cargo test --features midi 2>&1 | tail -10`
Capture the `--features midi` count too.

Run: `cargo clippy 2>&1 | tail -30`
CLAUDE.md notes 4 expected `module_inception` warnings. Capture the exact count and any other warnings; the ledger §6 promises "0 new" beyond these.

- [ ] **Step 1.2: Create empty `duality.rs` stub**

Create `crates/mt/src/quartal/duality.rs` with only the module docstring (so a future grep confirms intent even before functions land):

```rust
//! Quartal-perspective duality — symmetric counterpart to [`crate::quintal::duality`].
//!
//! The mathematical content of the quartal/quintal duality lives in
//! [`crate::quintal::duality`]. This module provides a quartal-native interface
//! for the same operations: functions taking [`super::types::QuartalVoicedChord`]
//! and returning [`super::types::QuartalIntervalStructure`] where natural, plus
//! a quartal-direction fiber-reversal verifier.
//!
//! See [`crate::quintal::duality`] for the underlying theorems and proofs.
```

- [ ] **Step 1.3: Create empty `verification.rs` stub**

Create `crates/mt/src/quartal/verification.rs` with only the module docstring:

```rust
//! Quartal-perspective verification of structural laws — symmetric
//! counterpart to [`crate::quintal::verification`].
//!
//! The Universal L1 Law and fiber-class assignment are perspective-independent
//! statements about the shared base space, but having a quartal-direction
//! verifier is useful both as documentation and as a check that the quartal
//! traversal ([`super::voicing::t_quartal`], [`super::voicing::quartal_inversion_cycle`],
//! [`super::voicing::quartal_l1_distances`]) produces the expected results.
```

- [ ] **Step 1.4: Wire `mod` declarations in `crates/mt/src/quartal/mod.rs`**

The current `mod` block (lines 8–15) is alphabetical. Insert `duality` after `conversion`, and `verification` after `voicing` to preserve order:

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

Do NOT add `pub use` lines yet — that comes after the module bodies are written. Adding empty re-export blocks now would emit "unused import" warnings.

- [ ] **Step 1.5: Create empty test files**

Create `crates/mt/tests/quartal/test_duality.rs` containing:
```rust
extern crate music_comp_mt as theory;
```

Create `crates/mt/tests/quartal/test_verification.rs` containing:
```rust
extern crate music_comp_mt as theory;
```

- [ ] **Step 1.6: Wire test `mod` declarations**

`crates/mt/tests/quartal/mod.rs` currently has six entries in alphabetical order. Insert `test_duality` after `test_constructors` and `test_verification` after `test_types` to preserve alphabetisation:

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

- [ ] **Step 1.7: Verify the empty wiring compiles cleanly**

Run: `cargo build`
Expected: build succeeds, no new warnings (empty module bodies are fine — Rust does not warn on empty modules).

Run: `cargo test 2>&1 | tail -20`
Expected: same baseline test count as step 1.1 (the new test files declare zero tests). Should pass.

Run: `cargo clippy 2>&1 | tail -30`
Expected: same 4 `module_inception` warnings as baseline; no new ones.

- [ ] **Step 1.8: Commit**

```bash
git add crates/mt/src/quartal/duality.rs \
        crates/mt/src/quartal/verification.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_duality.rs \
        crates/mt/tests/quartal/test_verification.rs \
        crates/mt/tests/quartal/mod.rs
git commit -m "feat(quartal): scaffold duality and verification module stubs (oth4-T0)"
```

---

### Task 2: Implement `quartal::duality` (TDD)

**Files:**
- Modify: `crates/mt/src/quartal/duality.rs`
- Modify: `crates/mt/src/quartal/mod.rs` (add `pub use duality::{...}`)
- Modify: `crates/mt/tests/quartal/test_duality.rs`

The module exposes:

| Symbol | Origin |
|---|---|
| `quartal_reading(&QuartalVoicedChord) -> QuartalIntervalStructure` | New — delegates to `chord.quartal_interval_structure()` (see plan v2 D-T0-004 for why this avoids the directional pitfall) |
| `quintal_reading(&QuartalVoicedChord) -> IntervalStructure`        | New — delegates to `quintal::quintal_reading(&chord.0)` |
| `t_quartal_reversal_equivalence(&QuartalVoicedChord) -> bool`      | New — delegates to `quintal::t1_reversal_equivalence(&chord.0)` |
| `pub use crate::quintal::reverse_interval_structure`                | Re-export |
| `pub use crate::quintal::orbit_self_duality`                        | Re-export |
| `pub use crate::quintal::verify_all_orbits_self_dual`               | Re-export |

- [ ] **Step 2.1: Write failing tests in `tests/quartal/test_duality.rs`**

Replace the file contents with:

```rust
extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, orbit_self_duality, quartal_reading, quintal_reading,
    reverse_interval_structure, t_quartal_reversal_equivalence,
    to_quartal, verify_all_orbits_self_dual, QuartalIntervalStructure,
};
use theory::quintal::{IntervalStructure, Orbit, VoicedChord};

// Q555 (= quintal Q777) Summit, voiced quintally as Eb-Bb-F-C — the
// legal-quintal stacking of pcs {0,3,5,10}. Palindromic IS.
// See plan §2 D-T0-003 for why "constructed via pure_quartal_stack" reads
// as "PcChord wrapped via to_quartal of its legal quintal voicing."
fn q555_summit_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([51, 58, 65, 72]).unwrap(); // Eb3-Bb3-F4-C5
    to_quartal(&vc)
}

// Q646 (= quintal Q686) Saddle, voiced quintally as C-F#-D-G# — the
// legal-quintal stacking of pcs {0,2,6,8}. Palindromic IS.
fn q646_saddle_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([48, 54, 62, 68]).unwrap(); // C3-F#3-D4-G#4
    to_quartal(&vc)
}

// Q554 (= quintal Q877), voiced quintally as C-G#-D#-A# — the
// legal-quintal stacking of pcs {0,3,8,10}. Bottom-up IS (8,7,7) — ASYMMETRIC.
// This fixture is the asymmetric-chord case that catches reversal bugs in
// quartal_reading; see plan v2 revision history.
fn q554_asymmetric_qvc() -> theory::quartal::QuartalVoicedChord {
    let vc = VoicedChord::new([60, 68, 75, 82]).unwrap(); // C4-G#4-D#5-A#5
    to_quartal(&vc)
}

#[test]
fn test_quartal_reading_q555_summit() {
    let chord = q555_summit_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(5, 5, 5));
}

#[test]
fn test_quintal_reading_q555_summit() {
    let chord = q555_summit_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(7, 7, 7));
}

#[test]
fn test_quartal_reading_q646_saddle() {
    let chord = q646_saddle_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(6, 4, 6));
}

#[test]
fn test_quintal_reading_q646_saddle() {
    let chord = q646_saddle_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(6, 8, 6));
}

#[test]
fn test_quartal_reading_q554_asymmetric() {
    // Asymmetric chord: bottom-up IS (8,7,7) → quartal IS (5,5,4).
    // If quartal_reading is implemented by composing quintal::quartal_reading
    // (top-down) with quintal_to_quartal_structure (which expects bottom-up),
    // the result will be (4,5,5) instead of (5,5,4) — this test catches that.
    let chord = q554_asymmetric_qvc();
    assert_eq!(quartal_reading(&chord), QuartalIntervalStructure(5, 5, 4));
}

#[test]
fn test_quintal_reading_q554_asymmetric() {
    let chord = q554_asymmetric_qvc();
    assert_eq!(quintal_reading(&chord), IntervalStructure(8, 7, 7));
}

#[test]
fn test_t_quartal_reversal_equivalence_holds() {
    // Sample one chord per orbit class; mirrors the spec §7.1 intent.
    let test_pitches: [[u8; 4]; 4] = [
        [48, 55, 62, 69], // Q777
        [48, 54, 62, 68], // Q686 saddle
        [48, 55, 62, 68], // Q776
        [48, 56, 63, 70], // Q877
    ];
    for pitches in &test_pitches {
        let qvc = to_quartal(&VoicedChord::new(*pitches).unwrap());
        assert!(
            t_quartal_reversal_equivalence(&qvc),
            "t_quartal reversal failed for {:?}",
            pitches
        );
    }
}

#[test]
fn test_self_duality_re_export() {
    // verify_all_orbits_self_dual must be reachable via crate::quartal::* and
    // return true on the canonical base space.
    let space = base_space();
    assert!(verify_all_orbits_self_dual(&space));
}

#[test]
fn test_orbit_self_duality_re_export() {
    let space = base_space();
    assert!(orbit_self_duality(&Orbit::Q777, &space));
}

#[test]
fn test_reverse_interval_structure_re_export() {
    let is = IntervalStructure(7, 7, 6);
    assert_eq!(reverse_interval_structure(&is), IntervalStructure(6, 7, 7));
}

```

- [ ] **Step 2.2: Run failing tests to confirm they fail to compile**

Run: `cargo test --test tests quartal::test_duality 2>&1 | tail -40`
Expected: `error[E0432]: unresolved imports` for `quartal_reading`, `quintal_reading`, `t_quartal_reversal_equivalence` (the three new symbols not yet exported). The re-exported symbols (`reverse_interval_structure`, `orbit_self_duality`, `verify_all_orbits_self_dual`) are also unresolved at the `quartal::` path because we haven't added the `pub use` block yet.

If errors are different, stop and reconcile before proceeding.

- [ ] **Step 2.3: Write the duality module body**

Replace `crates/mt/src/quartal/duality.rs` with:

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

use super::types::{QuartalIntervalStructure, QuartalVoicedChord};
use crate::quintal;

/// Read a quartal voicing's intervals top-to-bottom, complemented to fourths —
/// the natural quartal reading.
///
/// Delegates to [`QuartalVoicedChord::quartal_interval_structure`], which
/// composes `chord.0.interval_structure()` (bottom-up quintal IS) with
/// [`crate::quartal::quintal_to_quartal_structure`] (reverse-and-complement).
///
/// For a chord with bottom-up quintal IS `(i1, i2, i3)`, this returns
/// `QuartalIntervalStructure((12 - i3) % 12, (12 - i2) % 12, (12 - i1) % 12)` —
/// the canonical quartal label for that orbit.
///
/// Note: a naive composition of [`crate::quintal::quartal_reading`] (top-down)
/// with [`crate::quartal::quintal_to_quartal_structure`] (bottom-up input)
/// silently inverts the result for asymmetric IS. This function avoids that
/// pitfall by delegating to the existing method on `QuartalVoicedChord`.
pub fn quartal_reading(chord: &QuartalVoicedChord) -> QuartalIntervalStructure {
    chord.quartal_interval_structure()
}

/// Read a quartal voicing's intervals bottom-to-top — the dual (quintal) reading.
///
/// Returns the underlying [`crate::quintal::IntervalStructure`]. For a legal
/// quintal voicing the components are in `{6, 7, 8}`. This is the quintal
/// interpretation of the same pitch collection.
pub fn quintal_reading(chord: &QuartalVoicedChord) -> quintal::IntervalStructure {
    quintal::quintal_reading(&chord.0)
}

/// Verify that the quartal traversal `t_quartal` traverses the same fiber
/// as `t1` in reversed order.
///
/// Symmetric counterpart to [`crate::quintal::t1_reversal_equivalence`].
/// Because `t_quartal == t_minus1` (the inverse of `t1`), the quartal
/// inversion cycle visits the four members of the fiber in the reverse
/// pitch-class order.
///
/// Returns `true` if the reversal equivalence holds for the given chord.
pub fn t_quartal_reversal_equivalence(chord: &QuartalVoicedChord) -> bool {
    quintal::t1_reversal_equivalence(&chord.0)
}

// --- Symmetric re-exports from the quintal-side duality module ---
// These functions operate on quintal types but are conceptually shared;
// re-exporting here lets quartal-minded readers find them without
// reaching into the quintal namespace.

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

- [ ] **Step 2.4: Add `pub use` block to `crates/mt/src/quartal/mod.rs`**

Insert after the `pub use conversion::{...}` line (and before `pub use error::...`), preserving alphabetical order of the `pub use` blocks already in place:

```rust
pub use duality::{
    orbit_self_duality, quartal_reading, quintal_reading, reverse_interval_structure,
    t_quartal_reversal_equivalence, verify_all_orbits_self_dual,
};
```

(Six symbols: three new + three re-exported; alphabetised within the brace.)

- [ ] **Step 2.5: Run tests; confirm duality tests pass**

Run: `cargo test --test tests quartal::test_duality 2>&1 | tail -30`
Expected: 10 tests pass (`test_quartal_reading_q555_summit`, `test_quintal_reading_q555_summit`, `test_quartal_reading_q646_saddle`, `test_quintal_reading_q646_saddle`, `test_quartal_reading_q554_asymmetric`, `test_quintal_reading_q554_asymmetric`, `test_t_quartal_reversal_equivalence_holds`, `test_self_duality_re_export`, `test_orbit_self_duality_re_export`, `test_reverse_interval_structure_re_export`).

If any fail, stop and reconcile against §2 (D-T0-003 check the chord construction first). The asymmetric tests are the most likely to fail if `quartal_reading` regresses to the buggy composition documented in plan v2 history.

- [ ] **Step 2.6: Run full test suite to confirm no regression**

Run: `cargo test 2>&1 | tail -10`
Expected: `(baseline + 10) passed`. No previous tests fail.

Run: `cargo test --features midi 2>&1 | tail -10`
Expected: same delta on the midi feature flag.

- [ ] **Step 2.7: Lint check**

Run: `cargo clippy 2>&1 | tail -30`
Expected: still only 4 `module_inception` warnings, no new warnings.

Run: `cargo clippy --features midi 2>&1 | tail -30`
Expected: same.

If new warnings appear, fix them before commit. Likely candidates: unused `quartal` import (use it in the helper function), missing rustdoc on a new public symbol (every `pub fn` and `pub use` must have at least a one-line doc comment per the spec acceptance criteria §8).

- [ ] **Step 2.8: Commit**

```bash
git add crates/mt/src/quartal/duality.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_duality.rs
git commit -m "feat(quartal): add duality module — quartal-native readings + re-exports (oth4-T0)"
```

---

### Task 3: Implement `quartal::verification` (TDD)

**Files:**
- Modify: `crates/mt/src/quartal/verification.rs`
- Modify: `crates/mt/src/quartal/mod.rs` (add `pub use verification::{...}`)
- Modify: `crates/mt/tests/quartal/test_verification.rs`

**Resolution of D-T0-001 / D-T0-002 (see §2):** signatures match quintal exactly.

| Symbol | Signature |
|---|---|
| `verify_quartal_universal_l1_law` | `fn(space: &BaseSpace) -> Result<(), Vec<PcChord>>` |
| `verify_quartal_fiber_classes`    | `fn(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>` |
| `pc_chord_to_quartal_voiced` (private)  | `fn(chord: &PcChord) -> Option<QuartalVoicedChord>` |

- [ ] **Step 3.1: Write failing tests in `tests/quartal/test_verification.rs`**

Replace the file contents with:

```rust
extern crate music_comp_mt as theory;

use theory::quartal::{
    base_space, quartal_l1_distances, to_quartal, verify_quartal_fiber_classes,
    verify_quartal_universal_l1_law,
};
use theory::quintal::{self, FiberClass, Orbit, VoicedChord};

#[test]
fn test_quartal_universal_l1_law() {
    let space = base_space();
    assert!(verify_quartal_universal_l1_law(&space).is_ok());
}

#[test]
fn test_quartal_fiber_classes_count() {
    // Mirrors quintal::test_verify_fiber_classes_count: 14 orbits,
    // 11 ClassA + 3 ClassB (Q676, Q686, Q688).
    let space = base_space();
    let classes = verify_quartal_fiber_classes(&space);
    assert_eq!(classes.len(), 14);
    let class_a = classes.values().filter(|&&fc| fc == FiberClass::ClassA).count();
    let class_b = classes.values().filter(|&&fc| fc == FiberClass::ClassB).count();
    assert_eq!(class_a, 11);
    assert_eq!(class_b, 3);
    let class_b_orbits: Vec<Orbit> = classes
        .iter()
        .filter(|(_, &fc)| fc == FiberClass::ClassB)
        .map(|(&orb, _)| orb)
        .collect();
    assert!(class_b_orbits.contains(&Orbit::Q676));
    assert!(class_b_orbits.contains(&Orbit::Q686));
    assert!(class_b_orbits.contains(&Orbit::Q688));
}

#[test]
fn test_quartal_l1_pattern_summit() {
    // Q555 Summit voiced quintally as Eb-Bb-F-C ([51,58,65,72]).
    let qvc = to_quartal(&VoicedChord::new([51, 58, 65, 72]).unwrap());
    assert_eq!(quartal_l1_distances(&qvc), [12, 12, 12, 36]);
}

#[test]
fn test_quartal_l1_pattern_saddle() {
    // Q686 saddle voiced quintally as C-F#-D-G# ([48,54,62,68]).
    let qvc = to_quartal(&VoicedChord::new([48, 54, 62, 68]).unwrap());
    assert_eq!(quartal_l1_distances(&qvc), [12, 12, 12, 36]);
}

#[test]
fn test_quartal_verifiers_agree_with_quintal() {
    let space = base_space();

    // Universal L1 law: both directions must succeed (and we want the
    // expect message to surface the failing chord list if not). Equality of
    // .is_ok() alone would pass even if the two perspectives flagged
    // different chords as failures.
    verify_quartal_universal_l1_law(&space)
        .expect("quartal Universal L1 Law must hold on the canonical base space");
    quintal::verify_universal_l1_law(&space)
        .expect("quintal Universal L1 Law must hold on the canonical base space");

    // Fiber classes: identical maps. PartialEq on BTreeMap<Orbit, FiberClass>
    // gives us a strict pointwise comparison.
    assert_eq!(
        verify_quartal_fiber_classes(&space),
        quintal::verify_fiber_classes(&space)
    );
}
```

- [ ] **Step 3.2: Run failing tests to confirm they don't compile**

Run: `cargo test --test tests quartal::test_verification 2>&1 | tail -40`
Expected: `error[E0432]` on imports of `verify_quartal_universal_l1_law` and `verify_quartal_fiber_classes` (not yet defined or re-exported).

- [ ] **Step 3.3: Write the verification module body**

Replace `crates/mt/src/quartal/verification.rs` with:

```rust
//! Quartal-perspective verification of structural laws — symmetric
//! counterpart to [`crate::quintal::verification`].
//!
//! The Universal L1 Law and fiber-class assignment are perspective-independent
//! statements about the shared base space, but having a quartal-direction
//! verifier is useful both as documentation and as a check that the quartal
//! traversal ([`super::voicing::t_quartal`], [`super::voicing::quartal_inversion_cycle`],
//! [`super::voicing::quartal_l1_distances`]) produces the expected results.

use std::collections::BTreeMap;

use super::conversion::to_quartal;
use super::types::QuartalVoicedChord;
use super::voicing::{quartal_inversion_cycle, quartal_l1_distances};
use crate::quintal::{
    classify_orbit, BaseSpace, FiberClass, Orbit, PcChord, VoicedChord,
};

/// Verify the Universal L1 Law in the quartal direction.
///
/// For every legal [`QuartalVoicedChord`] derived from the 228 PcChords,
/// [`super::voicing::quartal_l1_distances`] must return `[12, 12, 12, 36]`.
///
/// Returns `Ok(())` if the law holds for every chord, otherwise `Err` with
/// the list of violators. Mirrors the signature of
/// [`crate::quintal::verify_universal_l1_law`].
///
/// See plan note D-T0-001 for the rationale on this richer signature.
pub fn verify_quartal_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>> {
    let mut failures = Vec::new();
    for &chord in space.chords() {
        match pc_chord_to_quartal_voiced(&chord) {
            Some(qvc) => {
                if quartal_l1_distances(&qvc) != [12, 12, 12, 36] {
                    failures.push(chord);
                }
            }
            None => failures.push(chord),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Verify all 14 fiber classes via quartal traversal.
///
/// Symmetric counterpart to [`crate::quintal::verify_fiber_classes`]. Returns
/// the orbit-to-class map computed by classifying a representative chord of
/// each orbit through the quartal inversion cycle. Should agree pointwise
/// with the quintal verifier (modulo identical Class A / Class B
/// assignments).
///
/// The cross-check that `quartal_inversion_cycle` and `inversion_cycle`
/// visit the same pitch-class set per chord is the *added* value of this
/// quartal-side verifier; it is asserted in
/// `tests/quartal/test_quartal_quintal_identity.rs::test_fibers_same_chords`.
pub fn verify_quartal_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass> {
    let mut result = BTreeMap::new();
    for &orb in Orbit::all() {
        for &chord in space.chords() {
            if classify_orbit(&chord) == Some(orb) {
                if let Some(qvc) = pc_chord_to_quartal_voiced(&chord) {
                    let cycle = quartal_inversion_cycle(&qvc);
                    let count = cycle
                        .iter()
                        .filter(|qinv| qinv.0.interval_structure().is_legal())
                        .count();
                    let fc = match count {
                        1 => FiberClass::ClassA,
                        2 => FiberClass::ClassB,
                        _ => continue,
                    };
                    result.insert(orb, fc);
                    break;
                }
            }
        }
    }
    result
}

/// Construct a [`QuartalVoicedChord`] from a [`PcChord`] in a default
/// register starting at MIDI 48 (C3), via the chord's legal quintal
/// stacking.
///
/// Returns `None` if the chord has no legal quintal interval structure.
///
/// This duplicates ~10 lines of logic from
/// `crate::quintal::verification::pc_chord_to_voiced` (which is private)
/// rather than widening quintal's API surface for a single internal use
/// (decision D-quartal-T0-001 — see plan §2).
fn pc_chord_to_quartal_voiced(chord: &PcChord) -> Option<QuartalVoicedChord> {
    let is = chord.interval_structure()?;

    for &start_pc in &chord.pcs {
        let base = 48 + start_pc;
        let pitches = [
            base,
            base + is.0,
            base + is.0 + is.1,
            base + is.0 + is.1 + is.2,
        ];

        if let Ok(vc) = VoicedChord::new(pitches) {
            if let Ok(pc) = vc.to_pc_chord() {
                if pc == *chord {
                    return Some(to_quartal(&vc));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quintal::{classify_orbit, Orbit};

    #[test]
    fn helper_round_trips_q777() {
        // pcs {0, 2, 7, 9} is C-D-G-A (= the Q777 Summit pitch-class set).
        let pc = PcChord::new([0, 2, 7, 9]).unwrap();
        let qvc = pc_chord_to_quartal_voiced(&pc).unwrap();
        // Round-trip preserves the chord through the quartal wrap.
        assert_eq!(qvc.0.to_pc_chord().unwrap(), pc);
        // And the chord lands in the expected orbit.
        assert_eq!(classify_orbit(&pc), Some(Orbit::Q777));
    }
}
```

**Implementation note:** the helper duplication is decision **D-quartal-T0-001** in the spec §5.3 and is recorded in the rustdoc.

**Note on the `quintal` import:** the body uses only the destructured names (`classify_orbit`, `BaseSpace`, etc.). The `quintal::` path is not needed in code (only in doc comments via intra-doc links), so `self` is intentionally absent from the import list.

- [ ] **Step 3.4: Add `pub use` block to `crates/mt/src/quartal/mod.rs`**

After the `pub use types::{...}` line (and before `pub use voicing::{...}`), insert:

```rust
pub use verification::{verify_quartal_fiber_classes, verify_quartal_universal_l1_law};
```

(Alphabetised within the brace.)

- [ ] **Step 3.5: Run the verification tests**

Run: `cargo test --test tests quartal::test_verification 2>&1 | tail -30`
Expected: 5 tests pass.

- [ ] **Step 3.6: Run unit tests inside the new module**

Run: `cargo test --lib quartal::verification 2>&1 | tail -10`
Expected: 1 test passes (`helper_round_trips_q777`).

- [ ] **Step 3.7: Run full test suite to confirm no regression**

Run: `cargo test 2>&1 | tail -10`
Expected: baseline + 10 (duality) + 5 (verification integration) + 1 (verification unit) = baseline + 16, all passing.

Run: `cargo test --features midi 2>&1 | tail -10`
Expected: same delta.

- [ ] **Step 3.8: Lint check**

Run: `cargo clippy 2>&1 | tail -30`
Expected: still only 4 `module_inception` warnings.

Run: `cargo clippy --features midi 2>&1 | tail -30`
Expected: same.

If clippy complains about the `use crate::quintal::{self, ...}` self-import being unused, drop `self`. If it complains about a missing `# Errors` or `# Panics` section on a public function, add it.

- [ ] **Step 3.9: Commit**

```bash
git add crates/mt/src/quartal/verification.rs \
        crates/mt/src/quartal/mod.rs \
        crates/mt/tests/quartal/test_verification.rs
git commit -m "feat(quartal): add verification module — quartal Universal L1 + fiber classes (oth4-T0)"
```

---

### Task 4: Final acceptance verification + ledger report

- [ ] **Step 4.1: Walk the spec §8 acceptance criteria one by one**

| Criterion | Command | Pass condition |
|---|---|---|
| `cargo build` succeeds | `cargo build` | exit 0, no new warnings |
| `cargo build --features midi` succeeds | `cargo build --features midi` | exit 0, no new warnings |
| `cargo test` passes | `cargo test 2>&1 \| tail -5` | (baseline + 16) passing, 0 failing |
| `cargo test --features midi` passes | `cargo test --features midi 2>&1 \| tail -5` | same delta |
| `cargo clippy` clean | `cargo clippy 2>&1 \| grep warning \| wc -l` | exactly 4 (the existing `module_inception`) |
| `cargo clippy --features midi` clean | `cargo clippy --features midi 2>&1 \| grep warning \| wc -l` | exactly 4 |
| All public API has rustdoc + intra-doc link | `cargo doc --no-deps 2>&1 \| grep -i 'missing'` | empty |
| No changes outside `quartal/` | `git diff main --stat \| grep -v 'crates/mt/src/quartal/' \| grep -v 'crates/mt/tests/quartal/'` | empty |
| Serde feature flag honoured | (no new public types; manual confirmation by reading the new modules) | no `#[derive(Serialize, ...)]` without `cfg_attr(feature = "serde", ...)` |

- [ ] **Step 4.2: Confirm `cargo doc` is clean**

Run: `cargo doc --no-deps 2>&1 | tail -30`
Expected: no warnings about missing docs, broken intra-doc links, or unresolved references on the new symbols.

If a `[broken intra-doc link]` warning appears for `[QuartalVoicedChord]` or `[QuartalIntervalStructure]`, the issue is usually a missing `crate::` prefix in the doc — fix the path inline.

- [ ] **Step 4.3: Confirm scope compliance (spec §10)**

Run: `git diff $(git merge-base HEAD main) --stat`
(Using `merge-base` rather than `main` directly so the diff shows only this branch's changes regardless of how main has advanced.)

Expected output should mention only:
- `crates/mt/src/quartal/duality.rs` (new)
- `crates/mt/src/quartal/verification.rs` (new)
- `crates/mt/src/quartal/mod.rs` (modified)
- `crates/mt/tests/quartal/test_duality.rs` (new)
- `crates/mt/tests/quartal/test_verification.rs` (new)
- `crates/mt/tests/quartal/mod.rs` (modified)

If anything else appears, revert it (or document why in the ledger §7/§9).

- [ ] **Step 4.4: Generate the ledger report**

Create `oth4/oth4-T0-ledger.md` (path per spec §9 — note this is a NEW file outside `quartal/`, but it is the explicit deliverable per the spec, so it's in-scope by exception). If the `oth4/` directory doesn't exist at the repo root, create it.

Use the spec §9 template verbatim. Section-by-section content guidance:

- **§1 Summary:** one paragraph; mention D-T0-001/002 (signatures matched quintal style) and D-T0-003 (test chord construction) as the only judgment calls.
- **§2 Files Created:** five files with line counts (`wc -l <path>`).
- **§3 Files Modified:** the two `mod.rs` files; one sentence each.
- **§4 Public API Surface Added:** list `quartal::quartal_reading`, `quartal::quintal_reading`, `quartal::t_quartal_reversal_equivalence`, `quartal::verify_quartal_universal_l1_law`, `quartal::verify_quartal_fiber_classes`. List re-exports separately: `quartal::orbit_self_duality`, `quartal::reverse_interval_structure`, `quartal::verify_all_orbits_self_dual`.
- **§5 Tests Added:** 10 in `test_duality.rs` (8 spec-original + 2 asymmetric-chord regression tests added in plan v2) + 5 in `test_verification.rs` + 1 unit test in `verification.rs`.
- **§6 Test Results:** plug in the actual numbers from §4.1 of this plan.
- **§7 Specced but Skipped:** none expected.
- **§8 Added Beyond Spec:** the unit test `helper_round_trips_q777`; the `test_orbit_self_duality_re_export` and `test_reverse_interval_structure_re_export` integration tests (added because the spec §7.1 only explicitly tests one re-export — `verify_all_orbits_self_dual` — but acceptance §8 requires "All public API on the new modules has rustdoc"; we test the re-export plumbing for the same reason); and the `test_quartal_reading_q554_asymmetric` / `test_quintal_reading_q554_asymmetric` regression tests (added per plan v2 revision to catch the directional bug in `quartal_reading` that the spec's palindromic-only fixtures could not surface — these are non-negotiable per the v2 revision history).
- **§9 Judgment Calls:**
  - **D-T0-001:** `verify_quartal_universal_l1_law` returns `Result<(), Vec<PcChord>>` (matching `crate::quintal::verify_universal_l1_law`), not `bool` as the spec body sketched. Justified by spec §5.3.
  - **D-T0-002:** `verify_quartal_fiber_classes` returns `BTreeMap<Orbit, FiberClass>` (matching `crate::quintal::verify_fiber_classes`), not `bool`. Justified by spec §5.3.
  - **D-T0-003:** Spec §7.1 wording "constructed via `pure_quartal_stack(0)`" interpreted as "the PcChord from `pure_quartal_stack(0)`, voiced via legal quintal stacking and wrapped via `to_quartal`". The asserted intervals `(5,5,5)` and `(6,4,6)` only match this reading.
  - **D-T0-004 (plan v2):** `quartal_reading` body delegates to `chord.quartal_interval_structure()` rather than composing `quintal::quartal_reading` with `quintal_to_quartal_structure`. The composition was buggy on asymmetric chords (the directional mismatch between top-down and bottom-up IS conventions silently inverted the result). Discovered during plan review; documented in plan v2 revision history.
  - **D-quartal-T0-001:** `pc_chord_to_quartal_voiced` reproduces ~10 lines of logic from the private `crate::quintal::verification::pc_chord_to_voiced` rather than widening quintal's API surface. Justified by spec §10 (do not modify `quintal/`).
- **§10 Open Questions:** any signatures that CDC may want to revisit; any ambiguity discovered during implementation.

- [ ] **Step 4.5: Commit the ledger**

```bash
mkdir -p oth4
# (create the file via $EDITOR or via the implementation tool)
git add oth4/oth4-T0-ledger.md
git commit -m "docs(oth4): add T0 ledger report"
```

---

## 5. Verification (end-to-end smoke)

After all four tasks land, the following sequence should pass on a clean checkout of the implementation branch:

```bash
git status                                    # clean tree, on the implementation branch
cargo fmt --check                             # exit 0
cargo build                                   # exit 0
cargo build --features midi                   # exit 0
cargo test 2>&1 | tail -3                     # baseline + 16 tests pass
cargo test --features midi 2>&1 | tail -3     # same delta
cargo clippy 2>&1 | grep -c 'warning'         # exactly 4
cargo clippy --features midi 2>&1 | grep -c 'warning'  # exactly 4
cargo doc --no-deps 2>&1 | grep -ci 'missing\|broken'  # 0
git diff $(git merge-base HEAD main) --stat | wc -l    # 6 file changes (5 quartal + 1 ledger doc)
```

Manual eyeball check:
- Open `target/doc/music_comp_mt/quartal/index.html` — the new `duality` and `verification` submodules render with their docstrings, and every public symbol has a one-line summary plus an intra-doc link to its quintal counterpart.
- Confirm the `Cargo.toml` MSRV (`rust-version`) is unchanged.
- Confirm no new dependencies appear in `Cargo.lock`.

---

## 6. Out of Scope (do not do — spec §10)

- ❌ Adding any module other than `duality.rs` and `verification.rs`.
- ❌ Modifying `crates/mt/src/quintal/` in any way (including making `pc_chord_to_voiced` `pub(crate)`).
- ❌ Adding MCP tools (T3).
- ❌ Adding centrality / display / functional modules (T1, T2).
- ❌ Changing existing tests.
- ❌ Refactoring existing quartal modules (constructors, types, voicing, …).
- ❌ Adding new dependencies to `Cargo.toml`.

If a spec point is unclear during implementation, the spec instruction (§10 last line) is: prefer "do less, document the question in §9 of the ledger" over expanding scope.

---

## 7. Self-review checklist (run this after Task 4 completes)

- [ ] Every spec §2 / §3 file appears in the diff.
- [ ] Every spec §4.2 public function (or its agreed counterpart per §2 of this plan) is present in `duality.rs`.
- [ ] Every spec §5.2 public function (or its agreed counterpart) is present in `verification.rs`.
- [ ] Every spec §7.1 / §7.2 test is present (with possibly tighter chord construction per D-T0-003).
- [ ] No placeholder `todo!()` remains in shipped code.
- [ ] Every `pub fn`/`pub use` in the new modules has a rustdoc comment containing at least one intra-doc link to its quintal dual.
- [ ] `cargo clippy` emits no new warnings (4 `module_inception` only).
- [ ] No file outside `crates/mt/src/quartal/`, `crates/mt/tests/quartal/`, or the new `oth4/oth4-T0-ledger.md` is touched.

---

*End of plan.*
