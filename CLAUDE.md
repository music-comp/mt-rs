# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rust Skill & Guides

For Rust code quality, load these resources in priority order:

1. **`assets/ai/rust/SKILL.md`** - Advanced Rust programming skill (**use this**)
2. **`assets/ai/rust/guides/*`** - Comprehensive Rust guidelines referenced by the skill
3. **`assets/ai/CLAUDE-CODE-COVERAGE.md`** - Comprehensive test coverage guide

**Important:** `assets/ai/rust` may be a symlink; if so, look in `assets/ai/rust/` (note the trailing slash). The actual directory may be at `~/lab/billosys/ai-engineering/knowledge/rust` or similar. If it does not exist on the file system in any form, stop and discuss with the user.

## Build & Test Commands

```bash
cargo build                          # build library + CLI
cargo build --features midi          # build with midi_pitch() support
cargo test                           # run all tests (846+ default, 851+ midi)
cargo test --features midi           # include midi_pitch tests
cargo test chord::test_chord         # run a specific test module
cargo test --test tests              # run only integration tests
cargo clippy --all-targets           # lint (zero warnings expected; gate any new work on "0 new warnings")

cargo run -- scale C Ionian          # CLI: generate scale
cargo run -- chord C# "Dominant Eleventh"  # CLI: generate chord
cargo run -- scale list              # CLI: list available scales
cargo run -- chord list              # CLI: list available chords
```

## Architecture

### Core Trait: `Notes`

The `Notes` trait (`crates/mt/src/note/note.rs`) is the central abstraction. Both `Chord` and `Scale` implement it, producing `Vec<Note>`.

```
note (Pitch, Note, NoteLetter, PitchSymbol, KeySignature)
  ↑
interval (semitone counting, note generation from intervals)
  ↑
chord + scale (both build on intervals, both impl Notes)
```

### Enharmonic Spelling System

`KeySignature` (`crates/mt/src/note/key_signature.rs`) maps tonic + mode to preferred note spellings. Both `Chord::notes()` and `Scale::notes()` apply key signatures to avoid double-sharps/flats and choose contextually correct enharmonic spellings (e.g., C# vs Db).

### Regex Parsing Pattern

Each theory type has a `from_regex()` constructor that parses natural-language music notation strings (e.g., "C# dominant seventh", "D Locrian"). These use `std::sync::LazyLock` compiled regexes. The CLI binary delegates to these parsers.

### Feature Flags

- **No default features**: Core library has zero dependencies beyond strum, regex, and clap
- **`midi`**: Gates `Note::midi_pitch()` for MIDI pitch number conversion (no external deps)

### Test Organization

Integration tests live in `crates/mt/tests/` organized by module (`tests/chord/`, `tests/scale/`, `tests/note/`, `tests/interval/`, `tests/quintal/`, `tests/quartal/`). The entry point is `tests/tests.rs` which declares submodules.

### Module Organization Conventions

When adding a quartal-side analytical module that mirrors a quintal-side counterpart (e.g., `quartal/duality.rs` mirrors `quintal/duality.rs`), follow the **topical-submodule re-export convention** established during the OTH4 programme:

- **Perspective-specific views** (rendering, centrality narration, functional-grammar narration, etc.) live in their topical submodule and are re-exported via that submodule's `pub use` block in `quartal/mod.rs`. Example: `quartal::display::render_quartal_chord_dashed`, `quartal::centrality::quartal_saddle_chords`.
- **Perspective-invariant infrastructure** (`BaseSpace`, the bare `betweenness_centrality` function, `enumerate_all`, etc.) continues to live in the bottom-of-file `pub use crate::quintal::*` block in `quartal/mod.rs`.
- **Perspective-invariant *helpers* used by perspective-specific views** (like `pc_to_note_name`, `render_pcset_dashed`, `FunctionalRegion`, `Pathway`) go through the topical submodule that uses them.

The convention keeps a quartal-minded reader's mental model crisp: "where do I look for X?" maps to a single submodule for any X-shaped query. New analytical work on the quartal side should follow this pattern; perspective-independent infrastructure additions can continue to land at the bottom of `quartal/mod.rs`.
