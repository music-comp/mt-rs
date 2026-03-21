## Rust Music Theory

[![Coverage](https://img.shields.io/badge/coverage-96%25-brightgreen)](https://github.com/music-comp/mt-rs)
[![Crates.io](https://img.shields.io/crates/v/rust-music-theory.svg?style=flat-square)](https://crates.io/crates/rust-music-theory)
[![Documentation](https://docs.rs/rust-music-theory/badge.svg)](https://docs.rs/rust-music-theory)

A library and executable that provides programmatic implementation of the basis of the music theory.

## Table of Contents

- [Overview](#overview)
- [Usage as a Library](#usage-as-a-library)
- [Usage as an Executable](#usage-as-an-executable)
- [Building From Source](#building-from-source)
- [Roadmap](#roadmap)

## Overview

`Rust Music Theory` is used to procedurally utilize music theory notions like Note, Chord, Scale,
Interval and more. The main purpose of this library is to let music theory be used in other programs and produce music/audio in a programmatic way.

## Usage as a Library

Add `rust-music-theory` as a dependency in your Cargo.toml.
```toml
[dependencies]
rust-music-theory = "0.3"
```

After installing the dependencies, you can use the library as follows.
```rust
extern crate rust_music_theory as rustmt;
use rustmt::note::{Note, Notes, Pitch, PitchSymbol::*};
use rustmt::scale::{Scale, ScaleType, Mode, Direction};
use rustmt::chord::{Chord, Number as ChordNumber, Quality as ChordQuality};

// to create a Note, specify a pitch and an octave;
let note = Note::new(Pitch::from(As), 4);

// Scale Example
let scale = Scale::new(
    ScaleType::Diatonic,
    Pitch::from(C),
    4,
    Some(Mode::Ionian),
    Direction::Ascending,
).unwrap();

let scale_notes = scale.notes();

// Chord Example
let chord = Chord::new(Pitch::from(C), ChordQuality::Major, ChordNumber::Triad);

let chord_notes = chord.notes();
```

For detailed examples, please see the tests folder.

## Usage as an Executable

`cargo install --git https://github.com/ozankasikci/rust-music-theory`

This lets cargo install the library as an executable called `rustmt`. Some usage examples;

`rustmt scale D Locrian`
```yaml
Notes:
  1: D
  2: D#
  3: F
  4: G
  5: G#
  6: A#
  7: C
  8: D
```
`rustmt chord C# Dominant Eleventh`
```yaml
Notes:
  1: C#
  2: F
  3: G#
  4: B
  5: D#
  6: G
```

`rustmt scale list`
```yaml
Available Scales:
 - Major|Ionian
 - Minor|Aeolian
 - Dorian
 - Phrygian
 - Lydian
 - Mixolydian
 - Locrian
 - Harmonic Minor
 - Melodic Minor
```


`rustmt chord list`
```yaml
Available chords:
 - Major Triad
 - Minor Triad
 - Suspended2 Triad
 - Suspended4 Triad
 - Augmented Triad
 - Diminished Triad
 - Major Seventh
 - Minor Seventh
 - Augmented Seventh
 - Augmented Major Seventh
 - Diminished Seventh
 - Half Diminished Seventh
 - Minor Major Seventh
 - Dominant Seventh
 - Dominant Ninth
 - Major Ninth
 - Dominant Eleventh
 - Major Eleventh
 - Minor Eleventh
 - Dominant Thirteenth
 - Major Thirteenth
 - Minor Thirteenth
```

## Building From Source

To quickly build and run the executable locally;

`git clone http://github.com/ozankasikci/rust-music-theory && cd rust-music-theory`

Then you can directly compile using cargo. An example;

`cargo run -- scale D Locrian`
```yaml
Notes:
  1: D
  2: D#
  3: F
  4: G
  5: G#
  6: A#
  7: C
  8: D
```

## Roadmap
- [x] Properly display enharmonic spelling
- [x] Add inversion support for chords
- [ ] Add missing modes for Melodic & Harmonic minor scales
- [ ] Add support for arbitrary accidentals
- [ ] Add a mechanism to find the chord from the given notes
