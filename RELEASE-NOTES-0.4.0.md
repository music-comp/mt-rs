# Release Notes — v0.4.0

**Date:** 2026-03-21
**Crates:** `music-comp-mt` (library), `music-comp-mt-cli` (binary)
**MSRV:** Rust 1.80+

This is a major feature release that transforms the library from a basic chord/scale generator into a comprehensive music theory computation engine. Every music theory fact was verified against 4,315 concept cards from 29 authoritative textbooks via the music-theory-skill MCP server.

## Highlights

- **12 modules** (was 4) covering fundamentals through graduate-level theory
- **349 tests** (was 114) with 96%+ code coverage
- **Correct enharmonic spelling** — G minor now produces Bb (not A#), dominant 7ths use Bb (not A#), blues scales use flats
- **Cargo workspace** with separate library and CLI crates
- **Serde serialization** on all public types behind a feature flag

## Breaking Changes

- **Crate renamed:** `rust-music-theory` → `music-comp-mt`
- **Binary renamed:** `rustmt` → `mt` (install via `cargo install music-comp-mt-cli`)
- **Workspace structure:** library at `mt/`, CLI at `mt-cli/`
- **Enharmonic spelling changes:** Chord and scale output now uses musically correct spellings. Code relying on the old spellings (e.g., `A#` instead of `Bb` for minor thirds) will need test updates.
- **`Notes` trait:** `print_notes()` refactored — new `format_notes()` returns `String` instead of printing directly
- **`PitchSymbol` enum:** Added `Fb` variant (may affect exhaustive matches)
- **`interval::Number` enum:** Added compound interval variants (`Ninth` through `Fifteenth`)
- **`NoteLetter`:** Now `#[repr(u8)]` with explicit discriminants

## Bug Fixes

- **Critical: Minor chord spelling** — G minor now correctly produces [G, Bb, D] instead of [G, A#, D]. Root cause: `KeySignature` was using the root's major key signature for all chord qualities. Fix: letter-based chord spelling that ensures each tone uses the correct diatonic letter name.
- **Critical: Dominant/augmented 7th spelling** — C7 now produces Bb instead of A#. Gb7 produces Fb instead of E.
- **Critical: Harmonic/melodic minor scale spelling** — C harmonic minor now gives Eb/Ab instead of D#/G#.
- **Critical: Blues scale spelling** — C blues now gives Eb/Gb/Bb instead of D#/F#/A#.
- **Critical: Dominant 11th interval** — Pattern fixed from [4,3,3,4,4] (augmented 11th) to [4,3,3,4,3] (perfect 11th). Verified against Hutchinson Ch. 31: "C11 = C-E-G-Bb-D-F".
- **Descending melodic minor** — Now uses natural minor intervals when descending (traditional practice).
- **`module_inception` warnings eliminated** — Restructured all 4 offending modules to use `mod.rs` pattern.

## New Features

### Tier 1 — Essential Computations

| Feature | API | Description |
|---------|-----|-------------|
| Enharmonic equivalence | `Pitch::is_enharmonic_to()` | Check if two pitches sound the same |
| Letter distance | `NoteLetter::distance_to()` | Count letter steps between note names |
| Interval calculation | `Interval::between()` | Letter-aware interval between two pitches — distinguishes augmented 4th from diminished 5th |
| Scale identification | `Scale::identify()` | Find all scales containing a set of pitch classes |
| Chord identification | `Chord::identify()` | Identify chords from unordered pitches with inversion detection |
| Diatonic chords | `harmony::diatonic_triads/sevenths()` | Build triads/7ths on each scale degree |
| Roman numerals | `analysis::roman_numeral()` | Label chords with Roman numerals (I, vi, V7, vii°, etc.) |
| Transposition | `Pitch::transpose_up/down()` | Transpose pitches by interval |
| Serde support | `--features serde` | Serialize/deserialize all public types |

### Tier 2 — Comprehensive Theory

| Feature | API | Description |
|---------|-----|-------------|
| Common tones | `harmony::common_tones()` | Pitch classes shared between two chord/scale voicings |
| Chord-scale compatibility | `harmony::compatible_scales()` | Find scales that contain all chord tones |
| Pivot chords | `harmony::pivot_chords()` | Find chords diatonic to two keys (for modulation analysis) |
| Voice leading | `voice_leading::minimal_movement()` | Optimal voice assignment minimizing total semitone movement |
| Compound intervals | `Interval::from_semitone(13..=24)` | Minor 9th through perfect 15th |
| Secondary dominants | `analysis::secondary_dominant()` | Detect V/x and V7/x chords |

### Tier 3 — Advanced/Specialized

| Feature | API | Description |
|---------|-----|-------------|
| Neo-Riemannian | `neo_riemannian::transform()` | P, R, L operations on triads with chaining |
| Pitch-class sets | `set_class::PitchClassSet` | Normal form, prime form, T_n, I_n, interval vector, Forte numbers |
| Counterpoint | `counterpoint::check_first_species()` | Rule checking for note-against-note counterpoint |
| Figured bass | `figured_bass::realize()` | Realize figured bass symbols into chord voicings |

### Other Additions

- **Suspended seventh chords** — C7sus4 = C-F-G-Bb now supported
- **`PitchSymbol::Fb`** — Enables correct spelling for Db minor, Gb major, etc.
- **`Interval::is_compound()` / `simple()`** — Query and convert compound intervals
- **`NoteLetter::from_index()`** — Convert 0-6 index back to letter name
- **`Notes::format_notes()`** — Returns formatted string instead of printing to stdout

## Infrastructure

- **Cargo workspace** — `mt/` (library) + `mt-cli/` (binary)
- **CI/CD** — GitHub Actions with 3-OS test matrix, feature flag matrix (12 configurations), coverage threshold gate, Codecov integration, tag-based publishing
- **`make` targets aligned with CI** — `make lint`, `make test`, `make docs`, `make coverage-check` run the exact same commands as CI
- **MSRV** — `rust-version = "1.80"` declared in both crate manifests
- **`rust-toolchain.toml`** — Pins stable channel with clippy, rustfmt, llvm-tools-preview
- **96%+ test coverage** — 349 tests across the workspace

## Stats

```
Commits:     39
Files:       98 changed
Lines:       +8,196 / -1,127
Tests:       114 → 349
Modules:     4 → 12
Coverage:    unknown → 96%
```

## Upgrading from 0.3.0

1. Update dependency: `music-comp-mt = "0.4"` (was `rust-music-theory = "0.3"`)
2. Update imports: `use music_comp_mt::` (was `use rust_music_theory::`)
3. Binary: `cargo install music-comp-mt-cli` (installs as `mt`)
4. Review any tests that assert specific enharmonic spellings — the library now produces musically correct spellings
