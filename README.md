# Music Theory for Rust

[![CI](https://github.com/music-comp/mt-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/music-comp/mt-rs/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/badge/coverage-96%25-brightgreen)](https://github.com/music-comp/mt-rs)
[![Crates.io](https://img.shields.io/crates/v/music-comp-mt.svg)](https://crates.io/crates/music-comp-mt)
[![Documentation](https://docs.rs/music-comp-mt/badge.svg)](https://docs.rs/music-comp-mt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[![][logo]][logo-large]

*A comprehensive music theory library and CLI for Rust*

This library covers music-theoretic fundamentals through graduate-level theory: notes, intervals, chords, scales, harmony analysis, voice leading, neo-Riemannian transformations, pitch-class set theory, counterpoint, and figured bass.

Every music theory fact in the library has been verified against 4,315 concept cards from 14 authoritative textbooks.

Note that this project started as a fork of Ozan Kaşıkçı's [excellent library](https://github.com/ozankasikci/rust-music-theory).

## Quick Start

### As a Library

```toml
[dependencies]
music-comp-mt = "0.4"
```

```rust
use music_comp_mt::note::{Notes, Pitch, PitchSymbol::*};
use music_comp_mt::chord::{Chord, Quality, Number};
use music_comp_mt::scale::{Scale, ScaleType, Mode, Direction};
use music_comp_mt::interval::Interval;

// Notes and chords
let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
assert_eq!(chord.format_notes(), "Notes:\n  1: C\n  2: E\n  3: G\n");

// Correct enharmonic spelling everywhere
let gm = Chord::new(Pitch::from(G), Quality::Minor, Number::Triad);
let notes = gm.notes();  // G, Bb, D — not G, A#, D

// Identify chords from notes
let matches = Chord::identify(&[Pitch::from(E), Pitch::from(G), Pitch::from(C)]);
// Finds: C major, first inversion

// Calculate intervals between pitches (letter-aware)
let interval = Interval::between(
    &Pitch::from(F),
    &Pitch::from(B),
).unwrap();
// Augmented 4th (not diminished 5th — letters matter)

// Find scales containing a set of notes
let scales = Scale::identify(&[
    Pitch::from(C), Pitch::from(D), Pitch::from(E),
    Pitch::from(Fs), Pitch::from(G), Pitch::from(A), Pitch::from(B),
]);
// Matches: C Lydian, G Ionian, ...
```

### As a CLI

```sh
cargo install music-comp-mt-cli
```

After building with `make build`, the binary is at `./bin/mt`:

```sh
$ ./bin/mt scale C Ionian
Notes:
  1: C
  2: D
  3: E
  4: F
  5: G
  6: A
  7: B
  8: C

$ ./bin/mt chord G "dominant seventh"
Notes:
  1: G
  2: B
  3: D
  4: F

$ ./bin/mt scale D Locrian
Notes:
  1: D
  2: Eb
  3: F
  4: G
  5: Ab
  6: Bb
  7: C
  8: D

$ ./bin/mt scale list
$ ./bin/mt chord list
```

## Modules

### Fundamentals

| Module | Description |
|--------|-------------|
| `note` | `Pitch`, `Note`, `NoteLetter`, `PitchSymbol`, `KeySignature`, enharmonic equivalence, transposition |
| `interval` | Simple and compound intervals (0-24 semitones), quality/number classification, letter-aware `between()`, inversion |
| `chord` | 22+ chord types, letter-based spelling, identification with inversion detection, regex parsing |
| `scale` | 8 scale types, 14 modes, identification from notes, ascending/descending support |

### Harmony & Analysis

| Module | Description |
|--------|-------------|
| `harmony` | Diatonic triads/sevenths, common tones, chord-scale compatibility, pivot chords |
| `analysis` | Roman numeral labeling (I, vi, V7, vii°, etc.), secondary dominant detection (V/x) |
| `voice_leading` | Optimal voice assignment minimizing total semitone movement |

### Advanced Theory

| Module | Description |
|--------|-------------|
| `neo_riemannian` | P, R, L operations on triads with chaining |
| `set_class` | `PitchClassSet` with normal/prime form, T_n, I_n, interval vector, Forte numbers |
| `counterpoint` | First-species rule checking (parallel 5ths/8ves, consonance, voice crossing) |
| `figured_bass` | Realize figured bass symbols into chord voicings |

## Feature Flags

| Flag | Description |
|------|-------------|
| `midi` | Enables `Note::midi_pitch()` for MIDI pitch number conversion |
| `serde` | Derives `Serialize`/`Deserialize` on all public types |

```toml
music-comp-mt = { version = "0.4", features = ["serde", "midi"] }
```

## Examples

### Harmony Analysis

```rust
use music_comp_mt::harmony;
use music_comp_mt::analysis;
use music_comp_mt::note::{Pitch, PitchSymbol::*};
use music_comp_mt::chord::{Chord, Quality, Number};
use music_comp_mt::scale::Mode;

// Diatonic chords in C major
let chords = harmony::diatonic_triads(Pitch::from(C), Mode::Ionian);
// I=C maj, ii=D min, iii=E min, IV=F maj, V=G maj, vi=A min, vii°=B dim

// Roman numeral analysis
let g7 = Chord::new(Pitch::from(G), Quality::Dominant, Number::Seventh);
let rn = analysis::roman_numeral(Pitch::from(C), Mode::Ionian, &g7).unwrap();
assert_eq!(rn.label, "V7");

// Secondary dominant detection
let d_major = Chord::new(Pitch::from(D), Quality::Major, Number::Triad);
let sd = analysis::secondary_dominant(Pitch::from(C), Mode::Ionian, &d_major).unwrap();
assert_eq!(sd.label, "V/V");

// Pivot chords between C major and G major
let pivots = harmony::pivot_chords(
    Pitch::from(C), Mode::Ionian,
    Pitch::from(G), Mode::Ionian,
);
// G major is V in C, I in G; C major is I in C, IV in G; etc.
```

### Neo-Riemannian Transformations

```rust
use music_comp_mt::neo_riemannian::{transform, transform_chain, NROperation};
use music_comp_mt::chord::{Chord, Quality, Number};
use music_comp_mt::note::{Pitch, PitchSymbol::*};

let c_major = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);

// P (Parallel): C major → C minor
let c_minor = transform(&c_major, NROperation::P).unwrap();

// R (Relative): C major → A minor
let a_minor = transform(&c_major, NROperation::R).unwrap();

// Chain operations: C major → P → R → L
let path = transform_chain(&c_major, &[NROperation::P, NROperation::R, NROperation::L]).unwrap();
```

### Pitch-Class Set Theory

```rust
use music_comp_mt::set_class::PitchClassSet;

let major_triad = PitchClassSet::new(&[0, 4, 7]);
assert_eq!(major_triad.prime_form(), vec![0, 3, 7]);
assert_eq!(major_triad.forte_number(), Some("3-11".to_string()));
assert_eq!(major_triad.interval_vector(), [0, 0, 1, 1, 1, 0]);

// Transpose and invert
let transposed = major_triad.transpose(5);  // T_5
let inverted = major_triad.invert(0);       // I_0
```

## Building From Source

```sh
git clone https://github.com/music-comp/mt-rs && cd mt-rs
make build        # Build library + CLI
make test         # Run all 349 tests
make lint         # Clippy + fmt (same checks as CI)
make coverage     # Generate coverage report (96%+)
make docs         # Build rustdoc (warnings as errors)
make check-all    # Build + lint + coverage + docs
```

## Project Structure

```
Cargo.toml              workspace root
mt/                     library crate (music-comp-mt)
  src/
    lib.rs
    note/, interval/, chord/, scale/
    harmony/, analysis/, voice_leading/
    neo_riemannian/, set_class/, counterpoint/, figured_bass/
  tests/
mt-cli/                 binary crate (music-comp-mt-cli)
  src/
    main.rs, cli.rs
  tests/
```

## License

MIT

[//]: ---Named-Links---

[logo]: https://avatars.githubusercontent.com/u/255628285?s=250
[logo-large]: https://avatars.githubusercontent.com/u/255628285
