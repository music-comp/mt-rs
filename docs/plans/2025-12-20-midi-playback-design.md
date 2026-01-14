# Real-Time MIDI Playback Design

## Overview

Extend rust-music-theory with real-time MIDI playback to connected hardware synthesizers. This enables algorithmic composition - generating and playing musical patterns programmatically with tempo-synced timing.

## Design Decisions

| Aspect | Decision |
|--------|----------|
| Target | Hardware synthesizers over USB |
| Use case | Algorithmic composition |
| Timing | Tempo-synced (BPM, beats/bars) |
| API style | Both blocking and non-blocking |
| Device discovery | Explicit (list ports, pick one) |
| Feature flag | Separate `midi-playback` feature |

## Feature Flag

```toml
[features]
midi = ["dep:midly"]
midi-playback = ["midi", "dep:midir"]
```

The `midi-playback` feature depends on `midi` (reuses `Duration`, `Velocity`, `Channel`, `Notes` trait) and adds `midir` for cross-platform MIDI I/O.

## Components

### MidiPorts

Lists available MIDI output ports on the system.

```rust
pub struct MidiPorts {
    ports: Vec<String>,
}

impl MidiPorts {
    /// List all available MIDI output ports
    pub fn list() -> Result<Self, PlaybackError>;

    /// Get port count
    pub fn len(&self) -> usize;

    /// Check if empty
    pub fn is_empty(&self) -> bool;

    /// Get port name by index
    pub fn get(&self, index: usize) -> Option<&str>;

    /// Iterate over port names
    pub fn iter(&self) -> impl Iterator<Item = &str>;
}
```

### MidiPlayer

Main interface for real-time playback.

**Construction & Configuration:**

```rust
impl MidiPlayer {
    /// Connect to a MIDI output port by name
    pub fn connect(port_name: &str) -> Result<Self, PlaybackError>;

    /// Connect to a port by index (from MidiPorts::list())
    pub fn connect_index(index: usize) -> Result<Self, PlaybackError>;

    /// Set tempo in BPM (default: 120)
    pub fn set_tempo(&mut self, bpm: u16);

    /// Set MIDI channel for output (default: 0)
    pub fn set_channel(&mut self, channel: Channel);

    /// Get current tempo
    pub fn tempo(&self) -> u16;
}
```

**Blocking Playback:**

```rust
impl MidiPlayer {
    /// Play notes and block until complete
    pub fn play<N: Notes>(&self, notes: &N, duration: Duration, velocity: Velocity);

    /// Rest (silent pause) for a duration
    pub fn rest(&self, duration: Duration);

    /// Play a raw MIDI note by pitch number
    pub fn play_note(&self, pitch: u8, duration: Duration, velocity: Velocity);
}
```

**Non-blocking Playback:**

```rust
impl MidiPlayer {
    /// Schedule notes to play, returns immediately
    pub fn play_async<N: Notes>(&self, notes: &N, duration: Duration, velocity: Velocity);

    /// Schedule a rest
    pub fn rest_async(&self, duration: Duration);

    /// Wait for all scheduled notes to finish
    pub fn wait(&self);

    /// Stop all playing notes immediately
    pub fn stop(&self);
}
```

### Scheduler (Internal)

Background thread that handles precise timing.

**Responsibilities:**
1. Maintains a queue of scheduled MIDI events (note on/off with beat timestamps)
2. Tracks current playback position in beats
3. Converts beats to real-time using the current tempo
4. Sends MIDI messages at the precise moment via `midir`

**Timing Conversion:**

```rust
fn duration_to_ms(duration: Duration, bpm: u16) -> u64 {
    let beat_ms = 60_000 / bpm as u64;  // ms per quarter note
    let ticks = duration.to_ticks(480);  // reuse existing Duration logic
    (ticks as u64 * beat_ms) / 480
}
```

**Thread Communication:**

```rust
enum SchedulerMessage {
    NoteOn { pitch: u8, velocity: u8, channel: u8 },
    NoteOff { pitch: u8, channel: u8 },
    ScheduleAt { beat: f64, message: Box<SchedulerMessage> },
    SetTempo(u16),
    Stop,
    Shutdown,
}
```

Main thread communicates via `std::sync::mpsc::channel`.

## Error Handling

```rust
#[derive(Debug)]
pub enum PlaybackError {
    /// No MIDI ports available on the system
    NoPortsAvailable,

    /// Port not found by name or index
    PortNotFound(String),

    /// Failed to connect to port
    ConnectionFailed(String),

    /// Port disconnected during playback
    Disconnected,

    /// MIDI system initialization failed
    InitError(String),
}

impl std::error::Error for PlaybackError {}
impl std::fmt::Display for PlaybackError { ... }
```

**Disconnect Handling:**

If the USB device disconnects mid-playback:
- Scheduler thread detects send failure
- Sets an internal "disconnected" flag
- Subsequent `play()` calls return `Err(PlaybackError::Disconnected)`
- User can attempt reconnection with a new `MidiPlayer::connect()`

## Usage Examples

**Simple Playback:**

```rust
use rust_music_theory::chord::{Chord, Quality, Number};
use rust_music_theory::note::{Pitch, PitchSymbol::*};
use rust_music_theory::midi::{MidiPorts, MidiPlayer, Duration, Velocity};

// List available ports
let ports = MidiPorts::list()?;
for (i, name) in ports.iter().enumerate() {
    println!("{}: {}", i, name);
}

// Connect to first port
let mut player = MidiPlayer::connect_index(0)?;
player.set_tempo(120);

// Play a chord
let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
```

**Algorithmic Composition:**

```rust
let mut player = MidiPlayer::connect_index(0)?;
player.set_tempo(140);

// Generate a pattern
let roots = [C, G, Am, F];
for root in roots.iter().cycle().take(16) {
    let chord = Chord::new(Pitch::from(*root), Quality::Major, Number::Triad);
    player.play_async(&chord, Duration::Quarter, Velocity::new(90).unwrap());
}

// Wait for pattern to complete
player.wait();
```

**Layered Voices:**

```rust
let mut player = MidiPlayer::connect_index(0)?;
player.set_tempo(120);

// Queue bass notes
player.set_channel(Channel::new(0).unwrap());
player.play_async(&bass_line, Duration::Whole, Velocity::new(80).unwrap());

// Queue melody on different channel
player.set_channel(Channel::new(1).unwrap());
player.play_async(&melody, Duration::Quarter, Velocity::new(100).unwrap());

player.wait();
```

## Testing Strategy

**Unit tests (no hardware):**

```rust
#[test]
fn duration_to_ms_conversion() {
    assert_eq!(duration_to_ms(Duration::Quarter, 120), 500);
    assert_eq!(duration_to_ms(Duration::Half, 120), 1000);
    assert_eq!(duration_to_ms(Duration::Eighth, 120), 250);
}

#[test]
fn scheduler_message_ordering() {
    // Test that messages are processed in correct order
}

#[test]
fn port_listing_handles_no_devices() {
    // Should return empty list, not panic
}
```

**Integration tests (hardware required):**

```rust
#[test]
#[ignore] // cargo test --features midi-playback -- --ignored
fn play_chord_on_hardware() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let player = MidiPlayer::connect_index(0).unwrap();
    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
    player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
}
```

## Dependencies

- `midir` (cross-platform MIDI I/O)
  - macOS: CoreMIDI
  - Linux: ALSA
  - Windows: WinMM
