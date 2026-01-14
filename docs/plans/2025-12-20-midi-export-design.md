# MIDI Export Feature Design

## Overview

Add MIDI export capability to rust-music-theory, transforming it from a reference library into a productivity tool for composers. The feature enables programmatic composition by exporting `Chord`, `Scale`, and custom sequences to `.mid` files.

## Goals

- **Programmatic composition**: Build full MIDI sequences from music theory primitives
- **Simple one-shot exports**: Quick export of a single chord or scale
- **Complex compositions**: Multi-track songs with tempo/time signature changes
- **Rust-idiomatic API**: Type-safe, no runtime errors for invalid MIDI values

## Non-Goals

- MIDI playback (use external libraries)
- MIDI file parsing/reading
- Real-time MIDI output to devices

## Dependencies

- `midly` (optional, feature-gated) - battle-tested MIDI encoder/decoder

## API Design

### Core Types

```rust
/// MIDI velocity (0-127). Controls note loudness.
pub struct Velocity(u8);
impl Velocity {
    pub fn new(v: u8) -> Option<Self> { (v <= 127).then(|| Self(v)) }
    pub fn max() -> Self { Self(127) }
}

/// MIDI channel (0-15). Channel 9 = drums by convention.
pub struct Channel(u8);
impl Channel {
    pub fn new(c: u8) -> Option<Self> { (c <= 15).then(|| Self(c)) }
    pub fn drums() -> Self { Self(9) }
}

/// Musical duration values
pub enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    Dotted(Box<Duration>),      // 1.5x length
    Triplet(Box<Duration>),     // 2/3 length
    Ticks(u32),                 // escape hatch for precise control
}

impl Duration {
    /// Convert to ticks at given PPQ (default 480)
    pub fn to_ticks(&self, ppq: u16) -> u32;
}
```

### MidiBuilder (Track Construction)

```rust
/// Builds a single MIDI track with sequential/absolute positioning
pub struct MidiBuilder {
    events: Vec<MidiEvent>,
    cursor: u32,              // current position in ticks
    ppq: u16,                 // pulses per quarter note (default 480)
}

impl MidiBuilder {
    pub fn new() -> Self;

    /// Add notes from anything implementing Notes trait (Chord, Scale)
    pub fn add<N: Notes>(&mut self, notes: &N, duration: Duration, velocity: Velocity) -> &mut Self;

    /// Insert silence
    pub fn rest(&mut self, duration: Duration) -> &mut Self;

    /// Jump to absolute beat position
    pub fn at_beat(&mut self, beat: f32) -> &mut Self;

    /// Insert tempo change at current position
    pub fn tempo(&mut self, bpm: u16) -> &mut Self;

    /// Insert time signature change
    pub fn time_signature(&mut self, numerator: u8, denominator: u8) -> &mut Self;
}
```

### MidiFile (Combining Tracks & Export)

```rust
/// Combines multiple tracks into a complete MIDI file
pub struct MidiFile {
    tracks: Vec<(MidiBuilder, Channel)>,
    default_tempo: u16,
    default_time_sig: (u8, u8),
    ppq: u16,
}

impl MidiFile {
    pub fn new() -> Self;
    pub fn tempo(mut self, bpm: u16) -> Self;
    pub fn time_signature(mut self, num: u8, denom: u8) -> Self;
    pub fn track(mut self, builder: MidiBuilder, channel: Channel) -> Self;
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()>;
    pub fn to_bytes(&self) -> Vec<u8>;
}
```

### ToMidi Trait (Simple One-Shot Export)

```rust
/// Quick MIDI export for Chord, Scale, and other Notes implementors
pub trait ToMidi: Notes {
    fn to_midi(&self, duration: Duration, velocity: Velocity) -> MidiExport;
}

// Blanket implementation for anything with Notes
impl<T: Notes> ToMidi for T {}

/// Intermediate type for chaining configuration
pub struct MidiExport {
    notes: Vec<Note>,
    duration: Duration,
    velocity: Velocity,
    tempo: u16,
    channel: Channel,
    time_sig: (u8, u8),
}

impl MidiExport {
    pub fn tempo(mut self, bpm: u16) -> Self;
    pub fn channel(mut self, ch: Channel) -> Self;
    pub fn time_signature(mut self, num: u8, denom: u8) -> Self;
    pub fn save<P: AsRef<Path>>(self, path: P) -> io::Result<()>;
    pub fn to_bytes(self) -> Vec<u8>;
}
```

## Usage Examples

### Simple Export

```rust
use rust_music_theory::{
    chord::{Chord, Quality, Number},
    note::Pitch,
    midi::{Duration, Velocity, ToMidi},
};

let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
    .save("c_major.mid")?;
```

### Programmatic Composition

```rust
use rust_music_theory::{
    chord::{Chord, Quality, Number},
    scale::{Scale, ScaleType, Mode, Direction},
    note::Pitch,
    midi::{MidiBuilder, MidiFile, Duration, Velocity, Channel},
};

let verse_chords = MidiBuilder::new()
    .add(&Chord::new(Pitch::from(A), Quality::Minor, Number::Triad),
         Duration::Whole, Velocity::new(90).unwrap())
    .add(&Chord::new(Pitch::from(F), Quality::Major, Number::Triad),
         Duration::Whole, Velocity::new(90).unwrap());

let melody = MidiBuilder::new()
    .add(&Scale::new(ScaleType::PentatonicMinor, Pitch::from(A), 4, None, Direction::Ascending)?,
         Duration::Eighth, Velocity::new(100).unwrap());

MidiFile::new()
    .tempo(90)
    .track(verse_chords, Channel::new(0).unwrap())
    .track(melody, Channel::new(1).unwrap())
    .save("song.mid")?;
```

## Module Organization

```
src/
├── lib.rs              # Add: #[cfg(feature = "midi")] pub mod midi;
└── midi/
    ├── mod.rs          # Public exports
    ├── duration.rs     # Duration enum
    ├── types.rs        # Velocity, Channel
    ├── builder.rs      # MidiBuilder
    ├── file.rs         # MidiFile
    └── export.rs       # ToMidi, MidiExport
```

## Feature Flag

```toml
# Cargo.toml
[features]
default = []
midi = ["dep:midly"]

[dependencies]
midly = { version = "0.5", optional = true }
```

Users enable with:
```toml
rust-music-theory = { version = "0.3", features = ["midi"] }
```

## Integration with Existing Types

Add to `Note`:
```rust
impl Note {
    /// Convert to MIDI pitch number (0-127)
    /// Middle C (C4) = 60
    pub fn midi_pitch(&self) -> u8 {
        let semitone = self.pitch.into_u8();
        let octave_offset = (self.octave + 1) * 12;
        (octave_offset + semitone).min(127)
    }
}
```

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| PPQ Resolution | 480 (default) | Industry standard, good divisibility for triplets |
| Duration API | Musical enum | Matches library philosophy ("music theory in code") |
| Velocity | Always explicit | No hidden defaults, predictable behavior |
| Timing | Sequential + absolute | Sequential for 90% of cases, `.at_beat()` for layering |
| Multi-track | Separate builders | Clean, composable, testable independently |
| Channels | Explicit | Users know what they want, no auto-assignment magic |
| Tempo | Global + inline changes | Simple default, flexibility for complex pieces |
| Errors | Validation at construction | Rust-idiomatic, invalid states unrepresentable |
| Dependency | Optional feature flag | Zero cost for users who don't need MIDI |

## Testing Strategy

1. **Unit tests**: Duration math, Velocity/Channel validation
2. **Integration tests**: Round-trip through midly to verify output
3. **Timing tests**: Sequential, absolute positioning, rests
4. **All in-memory**: No external files needed

## References

- [midly crate](https://lib.rs/crates/midly) - MIDI encoder/decoder
- [MIDI PPQ/timing](http://midi.teragonaudio.com/tech/midifile/ppqn.htm)
- [Compose (Scala)](https://underscore.io/blog/posts/2015/03/05/compositional-music-composition.html) - Sequential composition inspiration
- [music21](https://www.music21.org/music21docs/) - append/insert pattern inspiration
