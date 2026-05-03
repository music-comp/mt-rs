# Functional Pathway Detection — Implementation Plan

## Context

The v0.6.0 `harmonize_melody` output carries orbit classifications per chord but doesn't surface OTH functional regions or detect named pathways (Cadence, Departure). This enhancement adds automatic detection and optional re-ranking by functional interest.

---

## Commit Structure

Two commits:
1. `feat(quintal,harmonize): functional regions, pathways, and trajectory classification`
2. `feat(example): --sort= flag with functional pathway annotations and re-ranking`

---

## Commit 1: Library Additions

### 1.1 — New file: `crates/mt/src/quintal/functional.rs`

```rust
//! OTH functional regions and canonical pathways.
//!
//! The 14 T/I orbits map to seven functional regions named after natural
//! landforms. Two canonical pathways (Cadence and Departure) define
//! recognized harmonic motions through this terrain.

use super::Orbit;

/// The seven OTH functional regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FunctionalRegion {
    /// Q777. Maximum stability.
    Summit,
    /// Q787, Q877. Stable but not the apex.
    Plateau,
    /// Q786, Q776, Q876, Q867. Transitional terrain.
    Slope,
    /// Q767, Q868, Q878. Maximum tension, minimum connectivity.
    Valley,
    /// Q686. Structural dominant, T6-symmetric pivot.
    Saddle,
    /// Q688, Q788. Edge of stability.
    Precipice,
    /// Q676. Most constrained orbit.
    Narrows,
}

impl std::fmt::Display for FunctionalRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FunctionalRegion::Summit => "Summit",
            FunctionalRegion::Plateau => "Plateau",
            FunctionalRegion::Slope => "Slope",
            FunctionalRegion::Valley => "Valley",
            FunctionalRegion::Saddle => "Saddle",
            FunctionalRegion::Precipice => "Precipice",
            FunctionalRegion::Narrows => "Narrows",
        };
        f.write_str(name)
    }
}

impl Orbit {
    /// Return the functional region this orbit belongs to.
    pub fn functional_region(&self) -> FunctionalRegion {
        match self {
            Orbit::Q777 => FunctionalRegion::Summit,
            Orbit::Q787 | Orbit::Q877 => FunctionalRegion::Plateau,
            Orbit::Q786 | Orbit::Q776 | Orbit::Q876 | Orbit::Q867 => FunctionalRegion::Slope,
            Orbit::Q767 | Orbit::Q868 | Orbit::Q878 => FunctionalRegion::Valley,
            Orbit::Q686 => FunctionalRegion::Saddle,
            Orbit::Q688 | Orbit::Q788 => FunctionalRegion::Precipice,
            Orbit::Q676 => FunctionalRegion::Narrows,
        }
    }
}

/// A canonical OTH functional pathway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Pathway {
    /// Saddle → Slope → Summit.
    Cadence,
    /// Summit → Plateau → Slope → Saddle.
    Departure,
}

impl Pathway {
    /// The functional-region sequence this pathway represents.
    pub fn region_sequence(&self) -> &'static [FunctionalRegion] {
        match self {
            Pathway::Cadence => &[
                FunctionalRegion::Saddle,
                FunctionalRegion::Slope,
                FunctionalRegion::Summit,
            ],
            Pathway::Departure => &[
                FunctionalRegion::Summit,
                FunctionalRegion::Plateau,
                FunctionalRegion::Slope,
                FunctionalRegion::Saddle,
            ],
        }
    }

    /// All canonical pathways.
    pub fn all() -> &'static [Pathway] {
        &[Pathway::Cadence, Pathway::Departure]
    }
}

impl std::fmt::Display for Pathway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pathway::Cadence => f.write_str("Cadence"),
            Pathway::Departure => f.write_str("Departure"),
        }
    }
}
```

Unit tests at bottom of file:
- All 14 orbits map to the correct region (1+2+4+3+1+2+1 = 14 check).
- `Pathway::region_sequence` returns correct sequences.
- `Pathway::all()` returns both variants.

### 1.2 — Wire into `quintal/mod.rs`

Add `pub mod functional;` and re-export:
```rust
pub use functional::{FunctionalRegion, Pathway};
```

### 1.3 — New file: `crates/mt/src/harmonize/functional.rs`

Note on rustdoc: per the DC-04/DC-14 conventions established in Phase 2.4, every public type and function gets the full "summary + semantics + `# Examples`" treatment. Sketches below show the required rustdoc level. Each public item also gets per-field/per-variant doc paragraphs.

```rust
//! Functional pathway matching and trajectory classification for harmonizations.
//!
//! Given a [`Harmonization`] produced by [`crate::harmonize::harmonize_melody`],
//! these helpers detect canonical OTH pathways (Cadence, Departure) and
//! classify the progression's degree trajectory (Ascent / Descent / Traverse
//! / Mixed). Useful for re-ranking results by functional interest rather
//! than by least-movement only.

use crate::quintal::{classify_orbit, FunctionalRegion, Orbit, Pathway};
use super::Harmonization;

/// A canonical pathway found within a progression.
///
/// Returned by [`match_functional_pathways`]. Multiple matches per
/// progression are possible — the same pathway type can appear at
/// different positions, and different pathways can overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchedPathway {
    /// Which canonical pathway matched (e.g. [`Pathway::Cadence`]).
    pub pathway: Pathway,
    /// 0-indexed position in the progression's `chords` array where the
    /// match starts. Inclusive.
    pub start_position: usize,
    /// 0-indexed position where the match ends. Inclusive.
    pub end_position: usize,
}

/// Find all canonical OTH functional pathways that occur as contiguous
/// subsequences in a harmonization's functional-region sequence.
///
/// Each chord in the input is projected to its [`FunctionalRegion`] (via
/// [`Orbit::functional_region`]). The resulting region sequence is then
/// scanned for occurrences of every canonical pathway returned by
/// [`Pathway::all`]. Matches are reported as [`MatchedPathway`] entries
/// with inclusive `start_position` and `end_position` indices.
///
/// Matching is **contiguous-only** in this iteration. Overlapping
/// matches and multiple matches of the same pathway are returned
/// independently. Chords whose orbit cannot be classified are treated
/// as non-matching positions; this should never occur for output of
/// `harmonize_melody`, but is handled defensively rather than via
/// `expect` because `to_pc_chord` is fallible.
///
/// # Examples
///
/// ```
/// // Build a 3-chord harmonization that traces a Cadence
/// // (Saddle → Slope → Summit) and verify exactly one match is found.
/// // [doctest body here — construct via VoicedChord::new with chord
/// //  pitches that classify to Q686, then a Slope orbit, then Q777]
/// ```
pub fn match_functional_pathways(harmonization: &Harmonization) -> Vec<MatchedPathway> {
    let regions: Vec<Option<FunctionalRegion>> = harmonization
        .chords
        .iter()
        .map(|vc| {
            vc.to_pc_chord()
                .ok()
                .and_then(|pc| classify_orbit(&pc))
                .map(|orb| orb.functional_region())
        })
        .collect();

    let mut matches = Vec::new();
    for &pathway in Pathway::all() {
        let pattern = pathway.region_sequence();
        if pattern.len() > regions.len() {
            continue;
        }
        for i in 0..=regions.len() - pattern.len() {
            let window = &regions[i..i + pattern.len()];
            let matched = window
                .iter()
                .zip(pattern.iter())
                .all(|(actual, expected)| *actual == Some(*expected));
            if matched {
                matches.push(MatchedPathway {
                    pathway,
                    start_position: i,
                    end_position: i + pattern.len() - 1,
                });
            }
        }
    }
    matches
}

/// The shape of a progression's degree trajectory across positions.
///
/// Degree is the chord-graph connectivity of each chord (4, 5, 6, or 8 —
/// see the README's degree distribution `{4:90, 5:48, 6:60, 8:30}`).
/// Each variant corresponds to a named harmonic-motion category from
/// OTH:
///
/// - [`Trajectory::Ascent`] — degrees non-decreasing across positions
///   AND not all equal. The "strong" direction in OTH grammar.
/// - [`Trajectory::Descent`] — degrees non-increasing AND not all equal.
///   Departure from stability toward tension.
/// - [`Trajectory::Traverse`] — all degrees equal. Lateral motion at
///   constant tension level.
/// - [`Trajectory::Mixed`] — neither monotonic nor constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Trajectory {
    /// Degrees non-decreasing and not all equal.
    Ascent,
    /// Degrees non-increasing and not all equal.
    Descent,
    /// All degrees equal.
    Traverse,
    /// Neither monotonic nor constant.
    Mixed,
}

impl std::fmt::Display for Trajectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trajectory::Ascent => f.write_str("Ascent"),
            Trajectory::Descent => f.write_str("Descent"),
            Trajectory::Traverse => f.write_str("Traverse"),
            Trajectory::Mixed => f.write_str("Mixed"),
        }
    }
}

/// Classify a harmonization's degree trajectory.
///
/// Computes the per-position degree sequence by classifying each chord's
/// orbit and looking up its [`Orbit::degree`], then categorizes the
/// resulting sequence per the [`Trajectory`] variants. Single-chord
/// harmonizations classify as [`Trajectory::Traverse`] (degenerately
/// constant).
///
/// Panics with a clear `expect` message if any chord in the input cannot
/// be classified — this is an invariant violation, not a runtime error,
/// because `harmonize_melody` only produces chords from the 228-chord
/// OTH base space.
///
/// # Examples
///
/// ```
/// // Build a 4-chord harmonization with degrees [4, 5, 6, 8] and verify
/// // it classifies as Ascent.
/// // [doctest body here]
/// ```
pub fn classify_trajectory(harmonization: &Harmonization) -> Trajectory {
    let degrees: Vec<usize> = harmonization
        .chords
        .iter()
        .map(|vc| {
            let pc = vc
                .to_pc_chord()
                .expect("invariant: harmonize_melody output is in BaseSpace (PcChord projection cannot fail)");
            let orb = classify_orbit(&pc)
                .expect("invariant: harmonize_melody output is in BaseSpace (orbit classification cannot fail)");
            orb.degree()
        })
        .collect();

    if degrees.len() <= 1 {
        return Trajectory::Traverse;
    }

    let all_equal = degrees.windows(2).all(|w| w[0] == w[1]);
    if all_equal {
        return Trajectory::Traverse;
    }

    let non_decreasing = degrees.windows(2).all(|w| w[0] <= w[1]);
    if non_decreasing {
        return Trajectory::Ascent;
    }

    let non_increasing = degrees.windows(2).all(|w| w[0] >= w[1]);
    if non_increasing {
        return Trajectory::Descent;
    }

    Trajectory::Mixed
}
```

Unit tests: pathway matching on hand-built harmonizations (one Cadence; one with both Departure and Cadence overlapping; one with no matches), trajectory classification on hand-built harmonizations whose degree sequences are [4, 5, 6, 8] (Ascent), [8, 6, 5, 4] (Descent), [6, 6, 6] (Traverse), [4, 4, 5] (Ascent — non-strict), [6, 4, 8] (Mixed), and a single-chord case (Traverse).

### 1.4 — Wire into `harmonize/mod.rs`

Add `pub mod functional;` and re-export:
```rust
pub use functional::{classify_trajectory, match_functional_pathways, MatchedPathway, Trajectory};
```

### 1.5 — Verification

```bash
cargo check
cargo test --all-features
cargo clippy --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

---

## Commit 2: Example Update

### 2.1 — Add `--sort=<mode>` flag

Replace the original `--functional-mode` boolean with a value-bearing flag `--sort=<mode>` that admits multiple ranking modes. The flag idiom is more idiomatic for clap-style CLIs and gives us room for future ranking modes (e.g. `--sort=key-changes` once the modulation iteration ships) without flag-name proliferation.

Add a small enum local to the example:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    /// Default. Rank progressions by ascending `total_movement`.
    Movement,
    /// Rank by descending count of canonical pathway matches, ties broken
    /// by ascending `total_movement`.
    Functional,
}

fn parse_sort_mode(args: &[String]) -> Result<SortMode, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix("--sort=") {
            return match value {
                "movement" => Ok(SortMode::Movement),
                "functional" => Ok(SortMode::Functional),
                other => Err(format!(
                    "unknown --sort value: {other:?} (expected one of: movement, functional)"
                )),
            };
        }
    }
    Ok(SortMode::Movement)
}
```

Parse via `env::args()` consistent with the existing `--export-midi <path>` pattern. An unknown `--sort=` value produces an error printed to stderr and exits non-zero; a missing flag defaults to `SortMode::Movement`.

### 2.2 — When `SortMode::Functional` is selected:

- After `harmonize_melody`, compute `match_functional_pathways` and `classify_trajectory` for each result.
- Re-sort: primary key = pathway match count (descending), secondary key = `total_movement` (ascending). Use `sort_by` (which is **stable** in Rust's standard library — equal-key results preserve their original relative order, which is the property we want).
- Annotate each chord line with `[Region]`.
- Add summary to progression header: matched pathway names with positions, trajectory label.

### 2.3 — When `SortMode::Movement` is selected (default, no flag):

Output is byte-identical to v0.6.0 today. No annotations, no re-sort, no trajectory line. Backward compatibility for the existing example invocation is non-negotiable.

### 2.4 — Example output format (`--sort=functional`):

```
Progression 1 (total movement: 26 semitones | Cadence@5–7, Mixed)
  Position 1: B1–F#2–G#2–C3 [Slope] (Orbit: Q(8,6,7), quartal inv 3)
  Position 2: C2–F#2–G#2–D3 [Saddle] (Orbit: Q(6,8,6) [augmented], quartal inv 3)
  ...
```

### 2.5 — Verification

```bash
# Default — unchanged output (byte-identical to v0.6.0)
cargo run -p music-comp-mt --example harmonize_melody

# Explicit movement mode — equivalent to default
cargo run -p music-comp-mt --example harmonize_melody -- --sort=movement

# Functional mode — annotated, re-sorted by pathway-match-count
cargo run -p music-comp-mt --example harmonize_melody -- --sort=functional

# Combined with existing flag
cargo run -p music-comp-mt --example harmonize_melody -- --sort=functional --export-midi out.mid

# Error path — unknown sort mode produces a clear error
cargo run -p music-comp-mt --example harmonize_melody -- --sort=bananas
```

---

## Acceptance Criteria

```bash
cargo build --all-features
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo run -p music-comp-mt --example harmonize_melody                       # default — backward compatible
cargo run -p music-comp-mt --example harmonize_melody -- --sort=movement    # explicit default
cargo run -p music-comp-mt --example harmonize_melody -- --sort=functional  # annotated, re-sorted
cargo run -p music-comp-mt --example harmonize_melody -- --sort=bananas     # exits non-zero with clear error
```

---

## Observations

1. `Orbit::degree()` already exists (returns `usize`). No need to add it.
2. The prompt specifies Q788 as degree 4 — confirmed in the source: `Q788 => 4`.
3. The `functional_region()` method is added via a second `impl Orbit` block in `quintal/functional.rs` — Rust allows multiple impl blocks for the same type within the same crate.
4. No `clap` needed for the example — continue with manual `env::args()` detection.
5. The `harmonize::functional` module must be `pub mod` (not `pub(crate)`) since the example is an external consumer that needs to reach `match_functional_pathways`, `classify_trajectory`, `MatchedPathway`, and `Trajectory`.
