# MIDI Export Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add MIDI export capability to rust-music-theory, enabling `Chord` and `Scale` to export to `.mid` files.

**Architecture:** Feature-gated `midi` module using `midly` crate for MIDI encoding. Type-safe newtypes for Velocity/Channel, musical Duration enum, MidiBuilder for track construction, MidiFile for multi-track export, ToMidi trait for simple one-shot exports.

**Tech Stack:** Rust, midly 0.5, feature flags

---

## Task 1: Add Feature Flag and Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add midly dependency and midi feature flag**

Add to `Cargo.toml`:

```toml
[features]
default = []
midi = ["dep:midly"]

[dependencies]
midly = { version = "0.5", optional = true }
```

Insert the `[features]` section after line 11 (after `keywords`), and add the midly dependency to the `[dependencies]` section.

**Step 2: Verify the feature compiles**

Run: `cargo check --features midi`
Expected: Compiles successfully (warning about unused dependency is OK)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat(midi): add optional midly dependency with feature flag"
```

---

## Task 2: Create MIDI Module Structure

**Files:**
- Create: `src/midi/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Create the midi module directory and mod.rs**

Create `src/midi/mod.rs`:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod duration;
mod types;

pub use duration::Duration;
pub use types::{Channel, Velocity};
```

**Step 2: Create placeholder files**

Create `src/midi/duration.rs`:

```rust
//! Musical duration values for MIDI export.
```

Create `src/midi/types.rs`:

```rust
//! MIDI type newtypes (Velocity, Channel).
```

**Step 3: Add conditional module export to lib.rs**

Add after line 50 in `src/lib.rs` (after the wasm module):

```rust
#[cfg(feature = "midi")]
pub mod midi;
```

**Step 4: Verify compilation**

Run: `cargo check --features midi`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/midi src/lib.rs
git commit -m "feat(midi): create midi module structure"
```

---

## Task 3: Implement Velocity Newtype

**Files:**
- Modify: `src/midi/types.rs`

**Step 1: Write the failing test**

Add to `src/midi/types.rs`:

```rust
//! MIDI type newtypes (Velocity, Channel).

/// MIDI velocity (0-127). Controls note loudness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity(u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_valid_values() {
        assert!(Velocity::new(0).is_some());
        assert!(Velocity::new(64).is_some());
        assert!(Velocity::new(127).is_some());
    }

    #[test]
    fn velocity_invalid_values() {
        assert!(Velocity::new(128).is_none());
        assert!(Velocity::new(255).is_none());
    }

    #[test]
    fn velocity_max() {
        assert_eq!(Velocity::max(), Velocity(127));
    }

    #[test]
    fn velocity_inner_value() {
        let vel = Velocity::new(100).unwrap();
        assert_eq!(vel.value(), 100);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi velocity`
Expected: FAIL - `new`, `max`, `value` methods not found

**Step 3: Implement Velocity**

Update `src/midi/types.rs`, add implementation before tests:

```rust
impl Velocity {
    /// Create a new velocity value. Returns None if value > 127.
    pub fn new(v: u8) -> Option<Self> {
        (v <= 127).then(|| Self(v))
    }

    /// Maximum velocity (127).
    pub fn max() -> Self {
        Self(127)
    }

    /// Get the inner velocity value.
    pub fn value(&self) -> u8 {
        self.0
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi velocity`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/types.rs
git commit -m "feat(midi): implement Velocity newtype"
```

---

## Task 4: Implement Channel Newtype

**Files:**
- Modify: `src/midi/types.rs`

**Step 1: Write the failing test**

Add to the test module in `src/midi/types.rs`:

```rust
    #[test]
    fn channel_valid_values() {
        assert!(Channel::new(0).is_some());
        assert!(Channel::new(9).is_some());
        assert!(Channel::new(15).is_some());
    }

    #[test]
    fn channel_invalid_values() {
        assert!(Channel::new(16).is_none());
        assert!(Channel::new(255).is_none());
    }

    #[test]
    fn channel_drums() {
        assert_eq!(Channel::drums(), Channel(9));
    }

    #[test]
    fn channel_inner_value() {
        let ch = Channel::new(5).unwrap();
        assert_eq!(ch.value(), 5);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi channel`
Expected: FAIL - `Channel` not found

**Step 3: Implement Channel**

Add after Velocity in `src/midi/types.rs`:

```rust
/// MIDI channel (0-15). Channel 9 = drums by convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(u8);

impl Channel {
    /// Create a new channel. Returns None if value > 15.
    pub fn new(c: u8) -> Option<Self> {
        (c <= 15).then(|| Self(c))
    }

    /// Drum channel (9, which is channel 10 in 1-indexed MIDI).
    pub fn drums() -> Self {
        Self(9)
    }

    /// Get the inner channel value.
    pub fn value(&self) -> u8 {
        self.0
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi channel`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/types.rs
git commit -m "feat(midi): implement Channel newtype"
```

---

## Task 5: Implement Duration Enum - Basic Values

**Files:**
- Modify: `src/midi/duration.rs`

**Step 1: Write the failing test**

Replace contents of `src/midi/duration.rs`:

```rust
//! Musical duration values for MIDI export.

/// Musical duration values.
///
/// Standard PPQ (Pulses Per Quarter Note) is 480.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Duration {
    /// Whole note (4 beats)
    Whole,
    /// Half note (2 beats)
    Half,
    /// Quarter note (1 beat)
    Quarter,
    /// Eighth note (1/2 beat)
    Eighth,
    /// Sixteenth note (1/4 beat)
    Sixteenth,
    /// Thirty-second note (1/8 beat)
    ThirtySecond,
    /// Dotted duration (1.5x length)
    Dotted(Box<Duration>),
    /// Triplet duration (2/3 length)
    Triplet(Box<Duration>),
    /// Raw ticks for precise control
    Ticks(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    const PPQ: u16 = 480;

    #[test]
    fn basic_durations_to_ticks() {
        assert_eq!(Duration::Whole.to_ticks(PPQ), 1920);
        assert_eq!(Duration::Half.to_ticks(PPQ), 960);
        assert_eq!(Duration::Quarter.to_ticks(PPQ), 480);
        assert_eq!(Duration::Eighth.to_ticks(PPQ), 240);
        assert_eq!(Duration::Sixteenth.to_ticks(PPQ), 120);
        assert_eq!(Duration::ThirtySecond.to_ticks(PPQ), 60);
    }

    #[test]
    fn ticks_passthrough() {
        assert_eq!(Duration::Ticks(123).to_ticks(PPQ), 123);
        assert_eq!(Duration::Ticks(999).to_ticks(PPQ), 999);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi duration`
Expected: FAIL - `to_ticks` method not found

**Step 3: Implement to_ticks for basic values**

Add implementation after the enum definition:

```rust
impl Duration {
    /// Convert duration to ticks at the given PPQ (Pulses Per Quarter Note).
    /// Standard PPQ is 480.
    pub fn to_ticks(&self, ppq: u16) -> u32 {
        let ppq = ppq as u32;
        match self {
            Duration::Whole => ppq * 4,
            Duration::Half => ppq * 2,
            Duration::Quarter => ppq,
            Duration::Eighth => ppq / 2,
            Duration::Sixteenth => ppq / 4,
            Duration::ThirtySecond => ppq / 8,
            Duration::Dotted(inner) => inner.to_ticks(ppq as u16) * 3 / 2,
            Duration::Triplet(inner) => inner.to_ticks(ppq as u16) * 2 / 3,
            Duration::Ticks(t) => *t,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi duration`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/duration.rs
git commit -m "feat(midi): implement Duration enum with basic values"
```

---

## Task 6: Implement Duration - Dotted and Triplet

**Files:**
- Modify: `src/midi/duration.rs`

**Step 1: Write the failing test**

Add to test module in `src/midi/duration.rs`:

```rust
    #[test]
    fn dotted_durations() {
        // Dotted quarter = 1.5 * 480 = 720
        assert_eq!(Duration::Dotted(Box::new(Duration::Quarter)).to_ticks(PPQ), 720);
        // Dotted half = 1.5 * 960 = 1440
        assert_eq!(Duration::Dotted(Box::new(Duration::Half)).to_ticks(PPQ), 1440);
        // Dotted eighth = 1.5 * 240 = 360
        assert_eq!(Duration::Dotted(Box::new(Duration::Eighth)).to_ticks(PPQ), 360);
    }

    #[test]
    fn triplet_durations() {
        // Triplet quarter = 2/3 * 480 = 320
        assert_eq!(Duration::Triplet(Box::new(Duration::Quarter)).to_ticks(PPQ), 320);
        // Triplet eighth = 2/3 * 240 = 160
        assert_eq!(Duration::Triplet(Box::new(Duration::Eighth)).to_ticks(PPQ), 160);
    }

    #[test]
    fn nested_modifiers() {
        // Dotted triplet quarter = 1.5 * (2/3 * 480) = 1.5 * 320 = 480
        let dotted_triplet = Duration::Dotted(Box::new(
            Duration::Triplet(Box::new(Duration::Quarter))
        ));
        assert_eq!(dotted_triplet.to_ticks(PPQ), 480);
    }
```

**Step 2: Run test to verify it passes**

Run: `cargo test --features midi duration`
Expected: All tests PASS (implementation already handles these cases)

**Step 3: Add helper constructors**

Add to impl Duration:

```rust
    /// Create a dotted duration (1.5x length).
    pub fn dotted(base: Duration) -> Self {
        Duration::Dotted(Box::new(base))
    }

    /// Create a triplet duration (2/3 length).
    pub fn triplet(base: Duration) -> Self {
        Duration::Triplet(Box::new(base))
    }
```

**Step 4: Add test for helpers**

Add to test module:

```rust
    #[test]
    fn helper_constructors() {
        assert_eq!(
            Duration::dotted(Duration::Quarter).to_ticks(PPQ),
            Duration::Dotted(Box::new(Duration::Quarter)).to_ticks(PPQ)
        );
        assert_eq!(
            Duration::triplet(Duration::Quarter).to_ticks(PPQ),
            Duration::Triplet(Box::new(Duration::Quarter)).to_ticks(PPQ)
        );
    }
```

**Step 5: Run all duration tests**

Run: `cargo test --features midi duration`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add src/midi/duration.rs
git commit -m "feat(midi): add Duration helper constructors for dotted/triplet"
```

---

## Task 7: Add midi_pitch() to Note

**Files:**
- Modify: `src/note/note.rs`

**Step 1: Write the failing test**

Add test module at end of `src/note/note.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::PitchSymbol::*;

    #[test]
    fn midi_pitch_middle_c() {
        let note = Note::new(Pitch::from(C), 4);
        assert_eq!(note.midi_pitch(), 60);
    }

    #[test]
    fn midi_pitch_a440() {
        let note = Note::new(Pitch::from(A), 4);
        assert_eq!(note.midi_pitch(), 69);
    }

    #[test]
    fn midi_pitch_octaves() {
        assert_eq!(Note::new(Pitch::from(C), 0).midi_pitch(), 12);
        assert_eq!(Note::new(Pitch::from(C), 1).midi_pitch(), 24);
        assert_eq!(Note::new(Pitch::from(C), 2).midi_pitch(), 36);
        assert_eq!(Note::new(Pitch::from(C), 3).midi_pitch(), 48);
        assert_eq!(Note::new(Pitch::from(C), 5).midi_pitch(), 72);
    }

    #[test]
    fn midi_pitch_accidentals() {
        assert_eq!(Note::new(Pitch::from(Cs), 4).midi_pitch(), 61);
        assert_eq!(Note::new(Pitch::from(Db), 4).midi_pitch(), 61);
        assert_eq!(Note::new(Pitch::from(Fs), 4).midi_pitch(), 66);
    }

    #[test]
    fn midi_pitch_clamps_to_127() {
        // Very high octave should clamp
        let note = Note::new(Pitch::from(G), 10);
        assert!(note.midi_pitch() <= 127);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test midi_pitch`
Expected: FAIL - `midi_pitch` method not found

**Step 3: Implement midi_pitch**

Add to `impl Note` in `src/note/note.rs` after the `new` function:

```rust
    /// Convert to MIDI pitch number (0-127).
    ///
    /// Middle C (C4) = 60, A4 (440Hz) = 69.
    /// Uses standard MIDI octave convention where octave -1 starts at 0.
    pub fn midi_pitch(&self) -> u8 {
        let semitone = self.pitch.into_u8();
        let midi_value = (self.octave as u16 + 1) * 12 + semitone as u16;
        midi_value.min(127) as u8
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test midi_pitch`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/note/note.rs
git commit -m "feat(note): add midi_pitch() method for MIDI export"
```

---

## Task 8: Implement MidiEvent Internal Type

**Files:**
- Create: `src/midi/event.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create event.rs with MidiEvent enum**

Create `src/midi/event.rs`:

```rust
//! Internal MIDI event representation.

use crate::midi::{Velocity, Channel};

/// Internal representation of a MIDI event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiEvent {
    /// Note on event
    NoteOn {
        tick: u32,
        channel: Channel,
        pitch: u8,
        velocity: Velocity,
    },
    /// Note off event
    NoteOff {
        tick: u32,
        channel: Channel,
        pitch: u8,
    },
    /// Tempo change (microseconds per beat)
    Tempo {
        tick: u32,
        microseconds_per_beat: u32,
    },
    /// Time signature change
    TimeSignature {
        tick: u32,
        numerator: u8,
        denominator: u8,
    },
}

impl MidiEvent {
    /// Get the tick position of this event.
    pub fn tick(&self) -> u32 {
        match self {
            MidiEvent::NoteOn { tick, .. } => *tick,
            MidiEvent::NoteOff { tick, .. } => *tick,
            MidiEvent::Tempo { tick, .. } => *tick,
            MidiEvent::TimeSignature { tick, .. } => *tick,
        }
    }

    /// Convert BPM to microseconds per beat for tempo events.
    pub fn bpm_to_microseconds(bpm: u16) -> u32 {
        60_000_000 / bpm as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpm_conversion() {
        // 120 BPM = 500,000 microseconds per beat
        assert_eq!(MidiEvent::bpm_to_microseconds(120), 500_000);
        // 60 BPM = 1,000,000 microseconds per beat
        assert_eq!(MidiEvent::bpm_to_microseconds(60), 1_000_000);
    }
}
```

**Step 2: Update mod.rs to include event module**

Update `src/midi/mod.rs`:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod duration;
mod event;
mod types;

pub use duration::Duration;
pub use types::{Channel, Velocity};

// event is internal, not re-exported
```

**Step 3: Run tests**

Run: `cargo test --features midi event`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/event.rs src/midi/mod.rs
git commit -m "feat(midi): add internal MidiEvent type"
```

---

## Task 9: Implement MidiBuilder - Basic Structure

**Files:**
- Create: `src/midi/builder.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create builder.rs with basic structure**

Create `src/midi/builder.rs`:

```rust
//! MIDI track builder with sequential and absolute positioning.

use crate::midi::event::MidiEvent;
use crate::midi::{Duration, Velocity, Channel};
use crate::note::Notes;

/// Default PPQ (Pulses Per Quarter Note).
pub const DEFAULT_PPQ: u16 = 480;

/// Builds a single MIDI track with sequential/absolute positioning.
#[derive(Debug, Clone)]
pub struct MidiBuilder {
    pub(crate) events: Vec<MidiEvent>,
    pub(crate) cursor: u32,
    pub(crate) ppq: u16,
    pub(crate) channel: Channel,
}

impl MidiBuilder {
    /// Create a new MIDI builder with default settings.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            cursor: 0,
            ppq: DEFAULT_PPQ,
            channel: Channel::new(0).unwrap(),
        }
    }

    /// Create a new MIDI builder with a specific PPQ.
    pub fn with_ppq(ppq: u16) -> Self {
        Self {
            events: Vec::new(),
            cursor: 0,
            ppq,
            channel: Channel::new(0).unwrap(),
        }
    }

    /// Get the current cursor position in ticks.
    pub fn cursor(&self) -> u32 {
        self.cursor
    }

    /// Get the PPQ setting.
    pub fn ppq(&self) -> u16 {
        self.ppq
    }
}

impl Default for MidiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builder_defaults() {
        let builder = MidiBuilder::new();
        assert_eq!(builder.cursor(), 0);
        assert_eq!(builder.ppq(), 480);
        assert!(builder.events.is_empty());
    }

    #[test]
    fn builder_with_custom_ppq() {
        let builder = MidiBuilder::with_ppq(960);
        assert_eq!(builder.ppq(), 960);
    }
}
```

**Step 2: Update mod.rs**

Update `src/midi/mod.rs`:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod builder;
mod duration;
mod event;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use types::{Channel, Velocity};
```

**Step 3: Run tests**

Run: `cargo test --features midi builder`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/builder.rs src/midi/mod.rs
git commit -m "feat(midi): add MidiBuilder basic structure"
```

---

## Task 10: Implement MidiBuilder - add() Method

**Files:**
- Modify: `src/midi/builder.rs`

**Step 1: Write the failing test**

Add to test module in `src/midi/builder.rs`:

```rust
    use crate::chord::{Chord, Quality, Number};
    use crate::note::{Pitch, PitchSymbol::*};

    #[test]
    fn add_chord_creates_note_events() {
        let mut builder = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let velocity = Velocity::new(100).unwrap();

        builder.add(&chord, Duration::Quarter, velocity);

        // C major triad = 3 notes, each needs NoteOn and NoteOff
        assert_eq!(builder.events.len(), 6);
    }

    #[test]
    fn add_advances_cursor() {
        let mut builder = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let velocity = Velocity::new(100).unwrap();

        assert_eq!(builder.cursor(), 0);
        builder.add(&chord, Duration::Quarter, velocity);
        assert_eq!(builder.cursor(), 480); // Quarter note at 480 PPQ
    }

    #[test]
    fn add_sequential_timing() {
        let mut builder = MidiBuilder::new();
        let c_chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let g_chord = Chord::new(Pitch::from(G), Quality::Major, Number::Triad);
        let velocity = Velocity::new(100).unwrap();

        builder.add(&c_chord, Duration::Quarter, velocity);
        builder.add(&g_chord, Duration::Quarter, velocity);

        // First chord events at tick 0, second chord at tick 480
        let first_note_on = &builder.events[0];
        let second_chord_note_on = &builder.events[6];

        assert_eq!(first_note_on.tick(), 0);
        assert_eq!(second_chord_note_on.tick(), 480);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi add_chord`
Expected: FAIL - `add` method not found

**Step 3: Implement add() method**

Add to `impl MidiBuilder` in `src/midi/builder.rs`:

```rust
    /// Add notes from anything implementing the Notes trait.
    ///
    /// All notes are played simultaneously (chord-style) for the given duration.
    /// The cursor advances by the duration after adding.
    pub fn add<N: Notes>(&mut self, notes: &N, duration: Duration, velocity: Velocity) -> &mut Self {
        let ticks = duration.to_ticks(self.ppq);
        let note_off_tick = self.cursor + ticks;

        for note in notes.notes() {
            let pitch = note.midi_pitch();

            self.events.push(MidiEvent::NoteOn {
                tick: self.cursor,
                channel: self.channel,
                pitch,
                velocity,
            });

            self.events.push(MidiEvent::NoteOff {
                tick: note_off_tick,
                channel: self.channel,
                pitch,
            });
        }

        self.cursor = note_off_tick;
        self
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi add_`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/builder.rs
git commit -m "feat(midi): implement MidiBuilder::add() for Notes"
```

---

## Task 11: Implement MidiBuilder - rest() and at_beat()

**Files:**
- Modify: `src/midi/builder.rs`

**Step 1: Write the failing tests**

Add to test module:

```rust
    #[test]
    fn rest_advances_cursor() {
        let mut builder = MidiBuilder::new();

        builder.rest(Duration::Quarter);
        assert_eq!(builder.cursor(), 480);

        builder.rest(Duration::Half);
        assert_eq!(builder.cursor(), 1440); // 480 + 960
    }

    #[test]
    fn rest_creates_no_events() {
        let mut builder = MidiBuilder::new();
        builder.rest(Duration::Quarter);
        assert!(builder.events.is_empty());
    }

    #[test]
    fn at_beat_sets_absolute_position() {
        let mut builder = MidiBuilder::new();

        builder.at_beat(2.0);
        assert_eq!(builder.cursor(), 960); // 2 beats * 480 PPQ

        builder.at_beat(0.5);
        assert_eq!(builder.cursor(), 240); // 0.5 beats * 480 PPQ
    }

    #[test]
    fn at_beat_allows_layering() {
        let mut builder = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let velocity = Velocity::new(100).unwrap();

        // Add chord at beat 0
        builder.add(&chord, Duration::Whole, velocity);
        assert_eq!(builder.cursor(), 1920);

        // Jump back to beat 2 to layer something
        builder.at_beat(2.0);
        assert_eq!(builder.cursor(), 960);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi rest`
Expected: FAIL - `rest` method not found

**Step 3: Implement rest() and at_beat()**

Add to `impl MidiBuilder`:

```rust
    /// Insert silence (rest) of the given duration.
    ///
    /// Advances the cursor without creating any note events.
    pub fn rest(&mut self, duration: Duration) -> &mut Self {
        self.cursor += duration.to_ticks(self.ppq);
        self
    }

    /// Jump to an absolute beat position.
    ///
    /// Useful for layering multiple parts or jumping to specific points.
    /// Beat 0.0 is the start, 1.0 is one quarter note in, etc.
    pub fn at_beat(&mut self, beat: f32) -> &mut Self {
        self.cursor = (beat * self.ppq as f32) as u32;
        self
    }

    /// Jump to an absolute tick position.
    pub fn at_tick(&mut self, tick: u32) -> &mut Self {
        self.cursor = tick;
        self
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi rest at_beat`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/builder.rs
git commit -m "feat(midi): implement MidiBuilder rest() and at_beat()"
```

---

## Task 12: Implement MidiBuilder - tempo() and time_signature()

**Files:**
- Modify: `src/midi/builder.rs`

**Step 1: Write the failing tests**

Add to test module:

```rust
    #[test]
    fn tempo_creates_event() {
        let mut builder = MidiBuilder::new();
        builder.tempo(120);

        assert_eq!(builder.events.len(), 1);
        match &builder.events[0] {
            MidiEvent::Tempo { tick, microseconds_per_beat } => {
                assert_eq!(*tick, 0);
                assert_eq!(*microseconds_per_beat, 500_000); // 120 BPM
            }
            _ => panic!("Expected Tempo event"),
        }
    }

    #[test]
    fn time_signature_creates_event() {
        let mut builder = MidiBuilder::new();
        builder.time_signature(6, 8);

        assert_eq!(builder.events.len(), 1);
        match &builder.events[0] {
            MidiEvent::TimeSignature { tick, numerator, denominator } => {
                assert_eq!(*tick, 0);
                assert_eq!(*numerator, 6);
                assert_eq!(*denominator, 8);
            }
            _ => panic!("Expected TimeSignature event"),
        }
    }

    #[test]
    fn tempo_change_mid_track() {
        let mut builder = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let velocity = Velocity::new(100).unwrap();

        builder.tempo(120);
        builder.add(&chord, Duration::Whole, velocity);
        builder.tempo(140); // Tempo change after first chord

        // Find tempo events
        let tempo_events: Vec<_> = builder.events.iter()
            .filter(|e| matches!(e, MidiEvent::Tempo { .. }))
            .collect();

        assert_eq!(tempo_events.len(), 2);
        assert_eq!(tempo_events[0].tick(), 0);
        assert_eq!(tempo_events[1].tick(), 1920); // After whole note
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi tempo`
Expected: FAIL - `tempo` method not found

**Step 3: Implement tempo() and time_signature()**

Add to `impl MidiBuilder`:

```rust
    /// Insert a tempo change at the current position.
    ///
    /// Tempo is specified in BPM (beats per minute).
    pub fn tempo(&mut self, bpm: u16) -> &mut Self {
        self.events.push(MidiEvent::Tempo {
            tick: self.cursor,
            microseconds_per_beat: MidiEvent::bpm_to_microseconds(bpm),
        });
        self
    }

    /// Insert a time signature change at the current position.
    ///
    /// For example, `time_signature(6, 8)` sets 6/8 time.
    pub fn time_signature(&mut self, numerator: u8, denominator: u8) -> &mut Self {
        self.events.push(MidiEvent::TimeSignature {
            tick: self.cursor,
            numerator,
            denominator,
        });
        self
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi tempo time_sig`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/builder.rs
git commit -m "feat(midi): implement MidiBuilder tempo() and time_signature()"
```

---

## Task 13: Implement MidiFile - Basic Structure

**Files:**
- Create: `src/midi/file.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create file.rs with basic structure**

Create `src/midi/file.rs`:

```rust
//! MIDI file export combining multiple tracks.

use std::io;
use std::path::Path;

use crate::midi::{MidiBuilder, Channel};
use crate::midi::event::MidiEvent;

/// Combines multiple tracks into a complete MIDI file.
#[derive(Debug, Clone)]
pub struct MidiFile {
    tracks: Vec<(MidiBuilder, Channel)>,
    default_tempo: u16,
    default_time_sig: (u8, u8),
    ppq: u16,
}

impl MidiFile {
    /// Create a new MIDI file with default settings.
    ///
    /// Defaults: 120 BPM, 4/4 time, 480 PPQ.
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            default_tempo: 120,
            default_time_sig: (4, 4),
            ppq: 480,
        }
    }

    /// Set the default tempo in BPM.
    pub fn tempo(mut self, bpm: u16) -> Self {
        self.default_tempo = bpm;
        self
    }

    /// Set the default time signature.
    pub fn time_signature(mut self, numerator: u8, denominator: u8) -> Self {
        self.default_time_sig = (numerator, denominator);
        self
    }

    /// Set the PPQ (Pulses Per Quarter Note).
    pub fn ppq(mut self, ppq: u16) -> Self {
        self.ppq = ppq;
        self
    }

    /// Add a track on the specified channel.
    pub fn track(mut self, builder: MidiBuilder, channel: Channel) -> Self {
        self.tracks.push((builder, channel));
        self
    }

    /// Get the number of tracks.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

impl Default for MidiFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::builder::MidiBuilder;

    #[test]
    fn new_file_defaults() {
        let file = MidiFile::new();
        assert_eq!(file.default_tempo, 120);
        assert_eq!(file.default_time_sig, (4, 4));
        assert_eq!(file.ppq, 480);
        assert_eq!(file.track_count(), 0);
    }

    #[test]
    fn builder_pattern() {
        let file = MidiFile::new()
            .tempo(90)
            .time_signature(6, 8)
            .ppq(960);

        assert_eq!(file.default_tempo, 90);
        assert_eq!(file.default_time_sig, (6, 8));
        assert_eq!(file.ppq, 960);
    }

    #[test]
    fn add_tracks() {
        let track1 = MidiBuilder::new();
        let track2 = MidiBuilder::new();

        let file = MidiFile::new()
            .track(track1, Channel::new(0).unwrap())
            .track(track2, Channel::new(1).unwrap());

        assert_eq!(file.track_count(), 2);
    }
}
```

**Step 2: Update mod.rs**

Update `src/midi/mod.rs`:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod builder;
mod duration;
mod event;
mod file;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use file::MidiFile;
pub use types::{Channel, Velocity};
```

**Step 3: Run tests**

Run: `cargo test --features midi file`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/file.rs src/midi/mod.rs
git commit -m "feat(midi): add MidiFile basic structure"
```

---

## Task 14: Implement MidiFile - to_bytes() with midly

**Files:**
- Modify: `src/midi/file.rs`

**Step 1: Write the failing test**

Add to test module in `src/midi/file.rs`:

```rust
    use crate::chord::{Chord, Quality, Number};
    use crate::note::{Pitch, PitchSymbol::*};
    use crate::midi::{Duration, Velocity};

    #[test]
    fn to_bytes_creates_valid_midi() {
        let mut track = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        track.add(&chord, Duration::Quarter, Velocity::new(100).unwrap());

        let file = MidiFile::new()
            .track(track, Channel::new(0).unwrap());

        let bytes = file.to_bytes();

        // MIDI files start with "MThd"
        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn to_bytes_includes_all_tracks() {
        let track1 = MidiBuilder::new();
        let track2 = MidiBuilder::new();

        let file = MidiFile::new()
            .track(track1, Channel::new(0).unwrap())
            .track(track2, Channel::new(1).unwrap());

        let bytes = file.to_bytes();

        // Count "MTrk" occurrences (track headers)
        // Should be 3: 1 tempo track + 2 note tracks
        let mtrk_count = bytes.windows(4)
            .filter(|w| w == b"MTrk")
            .count();
        assert_eq!(mtrk_count, 3);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features midi to_bytes`
Expected: FAIL - `to_bytes` method not found

**Step 3: Implement to_bytes() with midly conversion**

Add imports at top of `src/midi/file.rs`:

```rust
use midly::{Format, Header, Smf, Timing, Track, TrackEvent, TrackEventKind, MidiMessage};
use midly::num::{u4, u7, u15, u24, u28};
use midly::MetaMessage;
```

Add to `impl MidiFile`:

```rust
    /// Convert to MIDI bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let smf = self.to_smf();
        let mut bytes = Vec::new();
        smf.write(&mut bytes).expect("Failed to write MIDI to bytes");
        bytes
    }

    /// Convert to midly Smf structure.
    fn to_smf(&self) -> Smf<'static> {
        let header = Header::new(
            Format::Parallel,
            Timing::Metrical(u15::new(self.ppq)),
        );

        let mut smf = Smf::new(header);

        // Track 0: Tempo and time signature
        let mut tempo_track: Track = Vec::new();
        tempo_track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(
                u24::new(MidiEvent::bpm_to_microseconds(self.default_tempo))
            )),
        });
        tempo_track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                self.default_time_sig.0,
                self.denominator_to_power(self.default_time_sig.1),
                24, // MIDI clocks per metronome click
                8,  // 32nd notes per quarter note
            )),
        });
        tempo_track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        smf.tracks.push(tempo_track);

        // Add note tracks
        for (builder, channel) in &self.tracks {
            let track = self.builder_to_track(builder, *channel);
            smf.tracks.push(track);
        }

        smf
    }

    /// Convert a MidiBuilder to a midly Track.
    fn builder_to_track(&self, builder: &MidiBuilder, channel: Channel) -> Track<'static> {
        let mut track: Track = Vec::new();
        let mut events = builder.events.clone();

        // Sort events by tick
        events.sort_by_key(|e| e.tick());

        let mut last_tick = 0u32;

        for event in events {
            let delta = event.tick() - last_tick;
            last_tick = event.tick();

            let track_event = match event {
                MidiEvent::NoteOn { pitch, velocity, .. } => TrackEvent {
                    delta: u28::new(delta),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(channel.value()),
                        message: MidiMessage::NoteOn {
                            key: u7::new(pitch),
                            vel: u7::new(velocity.value()),
                        },
                    },
                },
                MidiEvent::NoteOff { pitch, .. } => TrackEvent {
                    delta: u28::new(delta),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(channel.value()),
                        message: MidiMessage::NoteOff {
                            key: u7::new(pitch),
                            vel: u7::new(0),
                        },
                    },
                },
                MidiEvent::Tempo { microseconds_per_beat, .. } => TrackEvent {
                    delta: u28::new(delta),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(
                        u24::new(microseconds_per_beat)
                    )),
                },
                MidiEvent::TimeSignature { numerator, denominator, .. } => TrackEvent {
                    delta: u28::new(delta),
                    kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                        numerator,
                        self.denominator_to_power(denominator),
                        24,
                        8,
                    )),
                },
            };
            track.push(track_event);
        }

        // End of track
        track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        track
    }

    /// Convert time signature denominator to MIDI power-of-2 format.
    /// MIDI stores denominator as power of 2 (e.g., 4 -> 2, 8 -> 3).
    fn denominator_to_power(&self, denom: u8) -> u8 {
        match denom {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            16 => 4,
            32 => 5,
            _ => 2, // Default to quarter note
        }
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi to_bytes`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/file.rs
git commit -m "feat(midi): implement MidiFile::to_bytes() with midly"
```

---

## Task 15: Implement MidiFile - save()

**Files:**
- Modify: `src/midi/file.rs`

**Step 1: Write the failing test**

Add to test module:

```rust
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn save_creates_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mid");

        let mut track = MidiBuilder::new();
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        track.add(&chord, Duration::Quarter, Velocity::new(100).unwrap());

        let midi_file = MidiFile::new()
            .track(track, Channel::new(0).unwrap());

        midi_file.save(&file_path).unwrap();

        assert!(file_path.exists());

        // Verify it's a valid MIDI file
        let contents = fs::read(&file_path).unwrap();
        assert_eq!(&contents[0..4], b"MThd");
    }
```

**Step 2: Add tempfile to dev-dependencies**

Add to `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Implement save()**

Add to `impl MidiFile`:

```rust
    /// Save to a MIDI file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let bytes = self.to_bytes();
        std::fs::write(path, bytes)
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --features midi save_creates`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/file.rs Cargo.toml
git commit -m "feat(midi): implement MidiFile::save()"
```

---

## Task 16: Implement ToMidi Trait and MidiExport

**Files:**
- Create: `src/midi/export.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create export.rs with ToMidi trait**

Create `src/midi/export.rs`:

```rust
//! ToMidi trait for simple one-shot MIDI export.

use std::io;
use std::path::Path;

use crate::note::{Note, Notes};
use crate::midi::{Duration, Velocity, Channel, MidiBuilder, MidiFile};

/// Quick MIDI export for Chord, Scale, and other Notes implementors.
pub trait ToMidi: Notes {
    /// Convert to a MidiExport for configuration and saving.
    fn to_midi(&self, duration: Duration, velocity: Velocity) -> MidiExport {
        MidiExport::new(self.notes(), duration, velocity)
    }
}

// Blanket implementation for anything with Notes
impl<T: Notes> ToMidi for T {}

/// Intermediate type for chaining configuration before export.
pub struct MidiExport {
    notes: Vec<Note>,
    duration: Duration,
    velocity: Velocity,
    tempo: u16,
    channel: Channel,
    time_sig: (u8, u8),
}

impl MidiExport {
    /// Create a new MidiExport with default settings.
    pub fn new(notes: Vec<Note>, duration: Duration, velocity: Velocity) -> Self {
        Self {
            notes,
            duration,
            velocity,
            tempo: 120,
            channel: Channel::new(0).unwrap(),
            time_sig: (4, 4),
        }
    }

    /// Set the tempo in BPM.
    pub fn tempo(mut self, bpm: u16) -> Self {
        self.tempo = bpm;
        self
    }

    /// Set the MIDI channel.
    pub fn channel(mut self, ch: Channel) -> Self {
        self.channel = ch;
        self
    }

    /// Set the time signature.
    pub fn time_signature(mut self, numerator: u8, denominator: u8) -> Self {
        self.time_sig = (numerator, denominator);
        self
    }

    /// Convert to MIDI bytes.
    pub fn to_bytes(self) -> Vec<u8> {
        self.to_midi_file().to_bytes()
    }

    /// Save to a MIDI file.
    pub fn save<P: AsRef<Path>>(self, path: P) -> io::Result<()> {
        self.to_midi_file().save(path)
    }

    /// Convert to a MidiFile.
    fn to_midi_file(self) -> MidiFile {
        // Create a simple wrapper that implements Notes
        struct NoteVec(Vec<Note>);
        impl Notes for NoteVec {
            fn notes(&self) -> Vec<Note> {
                self.0.clone()
            }
        }

        let mut builder = MidiBuilder::new();
        builder.add(&NoteVec(self.notes), self.duration, self.velocity);

        MidiFile::new()
            .tempo(self.tempo)
            .time_signature(self.time_sig.0, self.time_sig.1)
            .track(builder, self.channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord::{Chord, Quality, Number};
    use crate::note::{Pitch, PitchSymbol::*};

    #[test]
    fn chord_to_midi() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let bytes = chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
            .to_bytes();

        // Verify it's a valid MIDI file
        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn scale_to_midi() {
        use crate::scale::{Scale, ScaleType, Mode, Direction};

        let scale = Scale::new(
            ScaleType::Diatonic,
            Pitch::from(C),
            4,
            Some(Mode::Ionian),
            Direction::Ascending,
        ).unwrap();

        let bytes = scale.to_midi(Duration::Eighth, Velocity::new(80).unwrap())
            .to_bytes();

        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn to_midi_with_options() {
        let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
        let bytes = chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
            .tempo(90)
            .channel(Channel::new(5).unwrap())
            .time_signature(3, 4)
            .to_bytes();

        assert_eq!(&bytes[0..4], b"MThd");
    }
}
```

**Step 2: Update mod.rs**

Update `src/midi/mod.rs`:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```
//!
//! # Quick Export
//!
//! ```ignore
//! use rust_music_theory::midi::{ToMidi, Duration, Velocity};
//!
//! let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
//! chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
//!     .save("chord.mid")?;
//! ```

mod builder;
mod duration;
mod event;
mod export;
mod file;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use export::{MidiExport, ToMidi};
pub use file::MidiFile;
pub use types::{Channel, Velocity};
```

**Step 3: Run tests**

Run: `cargo test --features midi export`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/export.rs src/midi/mod.rs
git commit -m "feat(midi): implement ToMidi trait and MidiExport"
```

---

## Task 17: Integration Tests

**Files:**
- Create: `tests/midi_integration.rs`

**Step 1: Create comprehensive integration tests**

Create `tests/midi_integration.rs`:

```rust
#![cfg(feature = "midi")]

use rust_music_theory::chord::{Chord, Quality, Number};
use rust_music_theory::scale::{Scale, ScaleType, Mode, Direction};
use rust_music_theory::note::{Pitch, PitchSymbol::*};
use rust_music_theory::midi::{
    MidiBuilder, MidiFile, Duration, Velocity, Channel, ToMidi,
};

#[test]
fn full_composition_workflow() {
    // Create chord progression
    let mut chords = MidiBuilder::new();
    chords
        .tempo(120)
        .add(&Chord::new(Pitch::from(C), Quality::Major, Number::Triad),
             Duration::Whole, Velocity::new(90).unwrap())
        .add(&Chord::new(Pitch::from(G), Quality::Major, Number::Triad),
             Duration::Whole, Velocity::new(90).unwrap())
        .add(&Chord::new(Pitch::from(A), Quality::Minor, Number::Triad),
             Duration::Whole, Velocity::new(90).unwrap())
        .add(&Chord::new(Pitch::from(F), Quality::Major, Number::Triad),
             Duration::Whole, Velocity::new(90).unwrap());

    // Create melody
    let mut melody = MidiBuilder::new();
    melody
        .rest(Duration::Half)  // Start after half note rest
        .add(&Scale::new(ScaleType::PentatonicMajor, Pitch::from(C), 5, None, Direction::Ascending).unwrap(),
             Duration::Eighth, Velocity::new(100).unwrap());

    // Combine into file
    let file = MidiFile::new()
        .tempo(120)
        .time_signature(4, 4)
        .track(chords, Channel::new(0).unwrap())
        .track(melody, Channel::new(1).unwrap());

    let bytes = file.to_bytes();

    // Verify MIDI header
    assert_eq!(&bytes[0..4], b"MThd");

    // Verify we have 3 tracks (tempo + 2 note tracks)
    let mtrk_count = bytes.windows(4).filter(|w| w == b"MTrk").count();
    assert_eq!(mtrk_count, 3);
}

#[test]
fn simple_chord_export() {
    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Seventh);
    let bytes = chord.to_midi(Duration::Half, Velocity::new(100).unwrap())
        .tempo(90)
        .to_bytes();

    assert_eq!(&bytes[0..4], b"MThd");
}

#[test]
fn all_chord_qualities() {
    let qualities = [
        Quality::Major,
        Quality::Minor,
        Quality::Diminished,
        Quality::Augmented,
        Quality::HalfDiminished,
        Quality::Dominant,
    ];

    for quality in qualities {
        let chord = Chord::new(Pitch::from(C), quality, Number::Triad);
        let bytes = chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
            .to_bytes();
        assert_eq!(&bytes[0..4], b"MThd", "Failed for quality {:?}", quality);
    }
}

#[test]
fn all_scale_types() {
    let scale_configs = [
        (ScaleType::Diatonic, Some(Mode::Ionian)),
        (ScaleType::Diatonic, Some(Mode::Dorian)),
        (ScaleType::Diatonic, Some(Mode::Aeolian)),
        (ScaleType::HarmonicMinor, None),
        (ScaleType::MelodicMinor, None),
        (ScaleType::PentatonicMajor, None),
        (ScaleType::PentatonicMinor, None),
        (ScaleType::Blues, None),
    ];

    for (scale_type, mode) in scale_configs {
        let scale = Scale::new(scale_type, Pitch::from(C), 4, mode, Direction::Ascending).unwrap();
        let bytes = scale.to_midi(Duration::Eighth, Velocity::new(80).unwrap())
            .to_bytes();
        assert_eq!(&bytes[0..4], b"MThd", "Failed for scale {:?}", scale_type);
    }
}

#[test]
fn duration_varieties() {
    let durations = [
        Duration::Whole,
        Duration::Half,
        Duration::Quarter,
        Duration::Eighth,
        Duration::Sixteenth,
        Duration::dotted(Duration::Quarter),
        Duration::triplet(Duration::Quarter),
    ];

    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);

    for duration in durations {
        let bytes = chord.to_midi(duration.clone(), Velocity::new(100).unwrap())
            .to_bytes();
        assert_eq!(&bytes[0..4], b"MThd", "Failed for duration {:?}", duration);
    }
}

#[test]
fn builder_chaining() {
    let c = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
    let vel = Velocity::new(100).unwrap();

    let mut builder = MidiBuilder::new();
    builder
        .tempo(120)
        .time_signature(4, 4)
        .add(&c, Duration::Quarter, vel)
        .rest(Duration::Quarter)
        .add(&c, Duration::Quarter, vel)
        .at_beat(4.0)
        .add(&c, Duration::Whole, vel);

    let file = MidiFile::new()
        .track(builder, Channel::new(0).unwrap());

    let bytes = file.to_bytes();
    assert_eq!(&bytes[0..4], b"MThd");
}
```

**Step 2: Run integration tests**

Run: `cargo test --features midi --test midi_integration`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add tests/midi_integration.rs
git commit -m "test(midi): add comprehensive integration tests"
```

---

## Task 18: Documentation and Examples

**Files:**
- Update: `src/midi/mod.rs`
- Update: `src/lib.rs`

**Step 1: Add module-level documentation**

Update `src/midi/mod.rs` with comprehensive docs:

```rust
//! MIDI export functionality for rust-music-theory.
//!
//! This module provides MIDI file export capabilities, transforming music theory
//! constructs like Chords and Scales into playable MIDI files.
//!
//! # Feature Flag
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```
//!
//! # Quick Export
//!
//! The simplest way to export is using the [`ToMidi`] trait:
//!
//! ```ignore
//! use rust_music_theory::chord::{Chord, Quality, Number};
//! use rust_music_theory::note::{Pitch, PitchSymbol::*};
//! use rust_music_theory::midi::{ToMidi, Duration, Velocity};
//!
//! let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
//! chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
//!     .save("chord.mid")?;
//! ```
//!
//! # Programmatic Composition
//!
//! For more control, use [`MidiBuilder`] and [`MidiFile`]:
//!
//! ```ignore
//! use rust_music_theory::chord::{Chord, Quality, Number};
//! use rust_music_theory::note::{Pitch, PitchSymbol::*};
//! use rust_music_theory::midi::{MidiBuilder, MidiFile, Duration, Velocity, Channel};
//!
//! // Build a chord progression
//! let mut chords = MidiBuilder::new();
//! chords
//!     .add(&Chord::new(Pitch::from(C), Quality::Major, Number::Triad),
//!          Duration::Whole, Velocity::new(90).unwrap())
//!     .add(&Chord::new(Pitch::from(G), Quality::Major, Number::Triad),
//!          Duration::Whole, Velocity::new(90).unwrap());
//!
//! // Build a melody on a separate track
//! let mut melody = MidiBuilder::new();
//! melody
//!     .at_beat(2.0)  // Start at beat 2
//!     .add(&some_notes, Duration::Eighth, Velocity::new(100).unwrap());
//!
//! // Combine and export
//! MidiFile::new()
//!     .tempo(120)
//!     .track(chords, Channel::new(0).unwrap())
//!     .track(melody, Channel::new(1).unwrap())
//!     .save("song.mid")?;
//! ```

mod builder;
mod duration;
mod event;
mod export;
mod file;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use export::{MidiExport, ToMidi};
pub use file::MidiFile;
pub use types::{Channel, Velocity};
```

**Step 2: Update lib.rs documentation**

Add to the module docstring in `src/lib.rs` (after the existing quick example):

```rust
//!
//! ## MIDI Export (optional feature)
//!
//! With the `midi` feature enabled, you can export chords and scales to MIDI files:
//!
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```
//!
//! ```ignore
//! use rustmt::midi::{ToMidi, Duration, Velocity};
//!
//! chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
//!     .save("chord.mid")?;
//! ```
```

**Step 3: Run doc tests**

Run: `cargo test --features midi --doc`
Expected: No failures (doc examples are `ignore` so they won't run)

**Step 4: Build docs**

Run: `cargo doc --features midi --no-deps --open`
Expected: Documentation generates successfully

**Step 5: Commit**

```bash
git add src/midi/mod.rs src/lib.rs
git commit -m "docs(midi): add comprehensive documentation"
```

---

## Task 19: Final Verification

**Step 1: Run all tests**

Run: `cargo test --features midi`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cargo clippy --features midi -- -D warnings`
Expected: No warnings

**Step 3: Verify without feature**

Run: `cargo check`
Expected: Compiles without midi feature (no midly dependency pulled in)

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(midi): complete MIDI export feature implementation"
```

---

## Summary

This plan implements the MIDI export feature in 19 tasks:

1. **Setup** (Tasks 1-2): Feature flag, module structure
2. **Core Types** (Tasks 3-6): Velocity, Channel, Duration
3. **Note Integration** (Task 7): midi_pitch() method
4. **MidiBuilder** (Tasks 8-12): Event type, add/rest/at_beat/tempo/time_signature
5. **MidiFile** (Tasks 13-15): Structure, to_bytes with midly, save
6. **ToMidi Trait** (Task 16): Simple one-shot exports
7. **Testing & Docs** (Tasks 17-19): Integration tests, documentation, verification

Each task follows TDD: write failing test → implement → verify → commit.
