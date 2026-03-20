# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rust Skill & Guides

For Rust code quality, load these resources in priority order:

1. **`assets/ai/ai-rust/skills/claude/SKILL.md`** - Advanced Rust programming skill (**use this**)
2. **`assets/ai/ai-rust/guides/*.md`** - Comprehensive Rust guidelines referenced by the skill
3. **`assets/ai/CLAUDE-CODE-COVERAGE.md`** - Comprehensive test coverage guide

**Important:** `assets/ai/ai-rust` may be a symlink; if so, look in `assets/ai/ai-rust/` (note the trailing slash). The actual directory may be at `~/lab/oxur/ai-rust`, `~/lab/oxur/ai-rust-skill`, etc. If it does not exist on the file system in any form, ask permission to clone it:

```bash
git clone https://github.com/oxur/ai-rust assets/ai/ai-rust
```

## Build & Test Commands

```bash
cargo build                          # build library + CLI
cargo build --features midi          # build with MIDI file export
cargo build --features midi-playback # build with real-time MIDI playback (implies midi)

cargo test                           # run core tests (114 tests)
cargo test --features midi           # include MIDI export tests (+40 tests)
cargo test --features midi-playback  # include playback tests
cargo test chord::test_chord         # run a specific test module
cargo test --test tests              # run only integration tests
cargo test --test midi_integration --features midi  # MIDI integration tests

cargo run -- scale C Ionian          # CLI: generate scale
cargo run -- chord C# "Dominant Eleventh"  # CLI: generate chord
cargo run -- scale list              # CLI: list available scales
cargo run -- chord list              # CLI: list available chords

cargo run --example nocturne --features midi-playback  # run an example
```

## Architecture

### Core Trait: `Notes`

The `Notes` trait (`src/note/note.rs`) is the central abstraction. Both `Chord` and `Scale` implement it, producing `Vec<Note>`. The MIDI layer uses a blanket impl (`ToMidi` for all `Notes` types) so any theory type automatically gains MIDI export.

```
Notes trait ← Chord, Scale
    ↓ (blanket impl)
ToMidi trait → MidiExport → MidiFile
```

### Module Dependency Flow

```
note (Pitch, Note, NoteLetter, PitchSymbol, KeySignature)
  ↑
interval (semitone counting, note generation from intervals)
  ↑
chord + scale (both build on intervals, both impl Notes)
  ↑
midi (optional: export/playback, consumes Notes)
```

### Enharmonic Spelling System

`KeySignature` (`src/note/key_signature.rs`) maps tonic + mode to preferred note spellings. Both `Chord::notes()` and `Scale::notes()` apply key signatures to avoid double-sharps/flats and choose contextually correct enharmonic spellings (e.g., C# vs Db).

### Regex Parsing Pattern

Each theory type has a `from_regex()` constructor that parses natural-language music notation strings (e.g., "C# dominant seventh", "D Locrian"). These use `lazy_static` compiled regexes. The CLI binary delegates to these parsers.

### Feature Flags

- **No default features**: Core library has zero optional deps
- **`midi`**: Enables `midly` for MIDI file export (`ToMidi` trait, `MidiBuilder`, `MidiFile`)
- **`midi-playback`**: Implies `midi`, adds `midir` for real-time MIDI device I/O (`MidiPlayer`, `MidiPorts`)

### Test Organization

Integration tests live in `tests/` organized by module (`tests/chord/`, `tests/scale/`, `tests/note/`, `tests/interval/`). The entry point is `tests/tests.rs` which declares submodules. MIDI tests are in separate feature-gated files (`tests/midi_integration.rs`, `tests/midi_playback_integration.rs`).
