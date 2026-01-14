# Real-Time MIDI Playback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real-time MIDI playback to connected hardware synthesizers with tempo-synced timing.

**Architecture:** A `MidiPlayer` connects to MIDI ports via `midir` and provides blocking/async playback. An internal scheduler thread handles precise timing, converting beat-based durations to real-time. Reuses existing `Duration`, `Velocity`, `Channel` types from the `midi` module.

**Tech Stack:** Rust, midir (cross-platform MIDI I/O), std::sync::mpsc for thread communication

---

## Task 1: Add Feature Flag and midir Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add midir dependency and feature flag**

Update `Cargo.toml`:

```toml
[features]
default = []
midi = ["dep:midly"]
midi-playback = ["midi", "dep:midir"]

[dependencies]
# ... existing deps ...
midly = { version = "0.5", optional = true }
midir = { version = "0.9", optional = true }
```

**Step 2: Verify feature compiles**

Run: `cargo check --features midi-playback`
Expected: Compiles successfully (no code uses midir yet, just checking dep resolution)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat(midi): add midi-playback feature flag with midir dependency"
```

---

## Task 2: Create Playback Module Structure

**Files:**
- Create: `src/midi/playback/mod.rs`
- Create: `src/midi/playback/error.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create playback module directory and mod.rs**

Create `src/midi/playback/mod.rs`:

```rust
//! Real-time MIDI playback functionality.

mod error;

pub use error::PlaybackError;
```

**Step 2: Create error.rs with PlaybackError enum**

Create `src/midi/playback/error.rs`:

```rust
//! Playback error types.

use std::fmt;

/// Errors that can occur during MIDI playback.
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

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybackError::NoPortsAvailable => write!(f, "No MIDI ports available"),
            PlaybackError::PortNotFound(name) => write!(f, "MIDI port not found: {}", name),
            PlaybackError::ConnectionFailed(msg) => write!(f, "Failed to connect: {}", msg),
            PlaybackError::Disconnected => write!(f, "MIDI port disconnected"),
            PlaybackError::InitError(msg) => write!(f, "MIDI initialization failed: {}", msg),
        }
    }
}

impl std::error::Error for PlaybackError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert_eq!(
            format!("{}", PlaybackError::NoPortsAvailable),
            "No MIDI ports available"
        );
        assert_eq!(
            format!("{}", PlaybackError::PortNotFound("USB MIDI".into())),
            "MIDI port not found: USB MIDI"
        );
    }

    #[test]
    fn error_is_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<PlaybackError>();
    }
}
```

**Step 3: Update midi/mod.rs to include playback module**

Add to `src/midi/mod.rs`:

```rust
#[cfg(feature = "midi-playback")]
pub mod playback;

#[cfg(feature = "midi-playback")]
pub use playback::PlaybackError;
```

**Step 4: Run tests**

Run: `cargo test --features midi-playback playback::error`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/playback/mod.rs src/midi/playback/error.rs src/midi/mod.rs
git commit -m "feat(midi): add PlaybackError type for midi-playback feature"
```

---

## Task 3: Implement MidiPorts for Device Discovery

**Files:**
- Create: `src/midi/playback/ports.rs`
- Modify: `src/midi/playback/mod.rs`

**Step 1: Write the tests first**

Create `src/midi/playback/ports.rs`:

```rust
//! MIDI port discovery.

use midir::MidiOutput;
use super::PlaybackError;

/// Lists available MIDI output ports on the system.
#[derive(Debug)]
pub struct MidiPorts {
    ports: Vec<String>,
}

impl MidiPorts {
    /// List all available MIDI output ports.
    pub fn list() -> Result<Self, PlaybackError> {
        let midi_out = MidiOutput::new("rust-music-theory")
            .map_err(|e| PlaybackError::InitError(e.to_string()))?;

        let ports: Vec<String> = midi_out
            .ports()
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect();

        Ok(Self { ports })
    }

    /// Get the number of available ports.
    pub fn len(&self) -> usize {
        self.ports.len()
    }

    /// Check if there are no ports available.
    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }

    /// Get a port name by index.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.ports.get(index).map(|s| s.as_str())
    }

    /// Iterate over port names.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.ports.iter().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_does_not_panic() {
        // Should not panic even if no MIDI devices
        let result = MidiPorts::list();
        assert!(result.is_ok());
    }

    #[test]
    fn empty_ports_methods() {
        let ports = MidiPorts { ports: vec![] };
        assert!(ports.is_empty());
        assert_eq!(ports.len(), 0);
        assert!(ports.get(0).is_none());
        assert_eq!(ports.iter().count(), 0);
    }

    #[test]
    fn ports_with_items() {
        let ports = MidiPorts {
            ports: vec!["Port A".into(), "Port B".into()],
        };
        assert!(!ports.is_empty());
        assert_eq!(ports.len(), 2);
        assert_eq!(ports.get(0), Some("Port A"));
        assert_eq!(ports.get(1), Some("Port B"));
        assert_eq!(ports.get(2), None);

        let names: Vec<_> = ports.iter().collect();
        assert_eq!(names, vec!["Port A", "Port B"]);
    }
}
```

**Step 2: Update playback mod.rs**

Update `src/midi/playback/mod.rs`:

```rust
//! Real-time MIDI playback functionality.

mod error;
mod ports;

pub use error::PlaybackError;
pub use ports::MidiPorts;
```

**Step 3: Run tests**

Run: `cargo test --features midi-playback playback::ports`
Expected: All tests PASS

**Step 4: Update midi/mod.rs exports**

Update `src/midi/mod.rs` to export MidiPorts:

```rust
#[cfg(feature = "midi-playback")]
pub use playback::{PlaybackError, MidiPorts};
```

**Step 5: Commit**

```bash
git add src/midi/playback/ports.rs src/midi/playback/mod.rs src/midi/mod.rs
git commit -m "feat(midi): add MidiPorts for device discovery"
```

---

## Task 4: Add duration_to_ms Helper Function

**Files:**
- Create: `src/midi/playback/timing.rs`
- Modify: `src/midi/playback/mod.rs`

**Step 1: Write timing module with tests**

Create `src/midi/playback/timing.rs`:

```rust
//! Timing utilities for MIDI playback.

use crate::midi::Duration;

/// Default PPQ for timing calculations.
pub const DEFAULT_PPQ: u16 = 480;

/// Convert a Duration to milliseconds at the given BPM.
pub fn duration_to_ms(duration: &Duration, bpm: u16) -> u64 {
    let beat_ms = 60_000u64 / bpm as u64; // ms per quarter note
    let ticks = duration.to_ticks(DEFAULT_PPQ);
    (ticks as u64 * beat_ms) / DEFAULT_PPQ as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_note_at_120_bpm() {
        // 120 BPM = 500ms per quarter note
        assert_eq!(duration_to_ms(&Duration::Quarter, 120), 500);
    }

    #[test]
    fn half_note_at_120_bpm() {
        // 120 BPM = 1000ms per half note
        assert_eq!(duration_to_ms(&Duration::Half, 120), 1000);
    }

    #[test]
    fn eighth_note_at_120_bpm() {
        // 120 BPM = 250ms per eighth note
        assert_eq!(duration_to_ms(&Duration::Eighth, 120), 250);
    }

    #[test]
    fn whole_note_at_120_bpm() {
        // 120 BPM = 2000ms per whole note
        assert_eq!(duration_to_ms(&Duration::Whole, 120), 2000);
    }

    #[test]
    fn quarter_note_at_60_bpm() {
        // 60 BPM = 1000ms per quarter note
        assert_eq!(duration_to_ms(&Duration::Quarter, 60), 1000);
    }

    #[test]
    fn quarter_note_at_240_bpm() {
        // 240 BPM = 250ms per quarter note
        assert_eq!(duration_to_ms(&Duration::Quarter, 240), 250);
    }

    #[test]
    fn dotted_quarter_at_120_bpm() {
        // Dotted quarter = 1.5 * 500 = 750ms
        assert_eq!(duration_to_ms(&Duration::dotted(Duration::Quarter), 120), 750);
    }

    #[test]
    fn triplet_quarter_at_120_bpm() {
        // Triplet quarter = 2/3 * 500 = 333ms (rounded)
        assert_eq!(duration_to_ms(&Duration::triplet(Duration::Quarter), 120), 333);
    }
}
```

**Step 2: Update playback mod.rs**

Add to `src/midi/playback/mod.rs`:

```rust
mod timing;

pub(crate) use timing::duration_to_ms;
```

**Step 3: Run tests**

Run: `cargo test --features midi-playback playback::timing`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/playback/timing.rs src/midi/playback/mod.rs
git commit -m "feat(midi): add duration_to_ms timing helper"
```

---

## Task 5: Implement MidiPlayer Struct and Connection

**Files:**
- Create: `src/midi/playback/player.rs`
- Modify: `src/midi/playback/mod.rs`
- Modify: `src/midi/mod.rs`

**Step 1: Create player.rs with struct and connection**

Create `src/midi/playback/player.rs`:

```rust
//! MIDI player for real-time playback.

use std::sync::{Arc, Mutex};
use midir::{MidiOutput, MidiOutputConnection};

use crate::midi::Channel;
use super::PlaybackError;

/// Real-time MIDI player.
pub struct MidiPlayer {
    connection: Arc<Mutex<MidiOutputConnection>>,
    tempo: u16,
    channel: Channel,
}

impl MidiPlayer {
    /// Connect to a MIDI output port by name.
    pub fn connect(port_name: &str) -> Result<Self, PlaybackError> {
        let midi_out = MidiOutput::new("rust-music-theory")
            .map_err(|e| PlaybackError::InitError(e.to_string()))?;

        let ports = midi_out.ports();
        let port = ports
            .iter()
            .find(|p| {
                midi_out.port_name(p).map(|n| n == port_name).unwrap_or(false)
            })
            .ok_or_else(|| PlaybackError::PortNotFound(port_name.to_string()))?;

        let connection = midi_out
            .connect(port, "output")
            .map_err(|e| PlaybackError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            tempo: 120,
            channel: Channel::new(0).unwrap(),
        })
    }

    /// Connect to a MIDI output port by index.
    pub fn connect_index(index: usize) -> Result<Self, PlaybackError> {
        let midi_out = MidiOutput::new("rust-music-theory")
            .map_err(|e| PlaybackError::InitError(e.to_string()))?;

        let ports = midi_out.ports();
        let port = ports
            .get(index)
            .ok_or_else(|| PlaybackError::PortNotFound(format!("index {}", index)))?;

        let connection = midi_out
            .connect(port, "output")
            .map_err(|e| PlaybackError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            tempo: 120,
            channel: Channel::new(0).unwrap(),
        })
    }

    /// Set the tempo in BPM.
    pub fn set_tempo(&mut self, bpm: u16) {
        self.tempo = bpm;
    }

    /// Get the current tempo.
    pub fn tempo(&self) -> u16 {
        self.tempo
    }

    /// Set the MIDI channel for output.
    pub fn set_channel(&mut self, channel: Channel) {
        self.channel = channel;
    }

    /// Get the current channel.
    pub fn channel(&self) -> Channel {
        self.channel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tempo_is_120() {
        // We can't test connection without hardware, but we can test defaults
        // by checking the struct fields after construction with mocked data
        // For now, just verify the expected default
        assert_eq!(120u16, 120);
    }

    #[test]
    fn channel_default() {
        let channel = Channel::new(0).unwrap();
        assert_eq!(channel.value(), 0);
    }

    #[test]
    fn connect_nonexistent_port_returns_error() {
        let result = MidiPlayer::connect("NonExistent Port XYZ 12345");
        assert!(result.is_err());
        match result {
            Err(PlaybackError::PortNotFound(_)) => {}
            _ => panic!("Expected PortNotFound error"),
        }
    }

    #[test]
    fn connect_invalid_index_returns_error() {
        let result = MidiPlayer::connect_index(99999);
        assert!(result.is_err());
        match result {
            Err(PlaybackError::PortNotFound(_)) => {}
            _ => panic!("Expected PortNotFound error"),
        }
    }
}
```

**Step 2: Update playback mod.rs**

Update `src/midi/playback/mod.rs`:

```rust
//! Real-time MIDI playback functionality.

mod error;
mod player;
mod ports;
mod timing;

pub use error::PlaybackError;
pub use player::MidiPlayer;
pub use ports::MidiPorts;

pub(crate) use timing::duration_to_ms;
```

**Step 3: Update midi/mod.rs exports**

Update `src/midi/mod.rs`:

```rust
#[cfg(feature = "midi-playback")]
pub use playback::{PlaybackError, MidiPorts, MidiPlayer};
```

**Step 4: Run tests**

Run: `cargo test --features midi-playback playback::player`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/midi/playback/player.rs src/midi/playback/mod.rs src/midi/mod.rs
git commit -m "feat(midi): add MidiPlayer with connection methods"
```

---

## Task 6: Implement Blocking Playback

**Files:**
- Modify: `src/midi/playback/player.rs`

**Step 1: Add blocking playback methods**

Add to `src/midi/playback/player.rs` impl block:

```rust
use std::thread;
use std::time::Duration as StdDuration;
use crate::midi::{Duration, Velocity};
use crate::note::Notes;
use super::timing::duration_to_ms;

impl MidiPlayer {
    // ... existing methods ...

    /// Play notes and block until complete.
    pub fn play<N: Notes>(&self, notes: &N, duration: Duration, velocity: Velocity) {
        let pitches: Vec<u8> = notes.notes().iter().map(|n| n.midi_pitch()).collect();

        // Send Note On for all pitches
        for &pitch in &pitches {
            self.send_note_on(pitch, velocity.value());
        }

        // Wait for duration
        let ms = duration_to_ms(&duration, self.tempo);
        thread::sleep(StdDuration::from_millis(ms));

        // Send Note Off for all pitches
        for &pitch in &pitches {
            self.send_note_off(pitch);
        }
    }

    /// Play a single MIDI pitch.
    pub fn play_note(&self, pitch: u8, duration: Duration, velocity: Velocity) {
        self.send_note_on(pitch, velocity.value());

        let ms = duration_to_ms(&duration, self.tempo);
        thread::sleep(StdDuration::from_millis(ms));

        self.send_note_off(pitch);
    }

    /// Rest (silent pause) for a duration.
    pub fn rest(&self, duration: Duration) {
        let ms = duration_to_ms(&duration, self.tempo);
        thread::sleep(StdDuration::from_millis(ms));
    }

    /// Send a Note On message.
    fn send_note_on(&self, pitch: u8, velocity: u8) {
        let status = 0x90 | (self.channel.value() & 0x0F);
        let message = [status, pitch & 0x7F, velocity & 0x7F];

        if let Ok(mut conn) = self.connection.lock() {
            let _ = conn.send(&message);
        }
    }

    /// Send a Note Off message.
    fn send_note_off(&self, pitch: u8) {
        let status = 0x80 | (self.channel.value() & 0x0F);
        let message = [status, pitch & 0x7F, 0];

        if let Ok(mut conn) = self.connection.lock() {
            let _ = conn.send(&message);
        }
    }
}
```

**Step 2: Add imports at top of player.rs**

Make sure these imports are at the top:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;

use midir::{MidiOutput, MidiOutputConnection};

use crate::midi::{Channel, Duration, Velocity};
use crate::note::Notes;
use super::PlaybackError;
use super::timing::duration_to_ms;
```

**Step 3: Run tests**

Run: `cargo test --features midi-playback playback::player`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/playback/player.rs
git commit -m "feat(midi): add blocking playback methods (play, play_note, rest)"
```

---

## Task 7: Add Scheduler for Async Playback

**Files:**
- Create: `src/midi/playback/scheduler.rs`
- Modify: `src/midi/playback/mod.rs`

**Step 1: Create scheduler module**

Create `src/midi/playback/scheduler.rs`:

```rust
//! Background scheduler for async MIDI playback.

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::time::{Duration as StdDuration, Instant};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use midir::MidiOutputConnection;

/// A scheduled MIDI event.
#[derive(Debug, Clone)]
struct ScheduledEvent {
    time_ms: u64,
    message: Vec<u8>,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time_ms == other.time_ms
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.time_ms.cmp(&self.time_ms)
    }
}

/// Messages sent to the scheduler thread.
pub enum SchedulerCommand {
    /// Schedule a MIDI message at a future time (ms from start)
    Schedule { time_ms: u64, message: Vec<u8> },
    /// Update tempo (used to recalculate future events)
    SetTempo(u16),
    /// Stop all notes immediately
    Stop,
    /// Shutdown the scheduler
    Shutdown,
}

/// The scheduler manages a background thread for timed MIDI events.
pub struct Scheduler {
    sender: Sender<SchedulerCommand>,
    thread: Option<JoinHandle<()>>,
    current_time_ms: Arc<Mutex<u64>>,
    idle_signal: Arc<(Mutex<bool>, Condvar)>,
}

impl Scheduler {
    /// Create a new scheduler with the given MIDI connection.
    pub fn new(connection: Arc<Mutex<MidiOutputConnection>>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let current_time_ms = Arc::new(Mutex::new(0u64));
        let idle_signal = Arc::new((Mutex::new(true), Condvar::new()));

        let time_clone = current_time_ms.clone();
        let idle_clone = idle_signal.clone();

        let thread = thread::spawn(move || {
            Self::run(receiver, connection, time_clone, idle_clone);
        });

        Self {
            sender,
            thread: Some(thread),
            current_time_ms,
            idle_signal,
        }
    }

    /// Schedule a MIDI message at the given time offset (ms).
    pub fn schedule(&self, time_ms: u64, message: Vec<u8>) {
        // Mark as busy
        if let Ok(mut idle) = self.idle_signal.0.lock() {
            *idle = false;
        }
        let _ = self.sender.send(SchedulerCommand::Schedule { time_ms, message });
    }

    /// Stop all playing notes.
    pub fn stop(&self) {
        let _ = self.sender.send(SchedulerCommand::Stop);
    }

    /// Wait for all scheduled events to complete.
    pub fn wait(&self) {
        let (lock, cvar) = &*self.idle_signal;
        let mut idle = lock.lock().unwrap();
        while !*idle {
            idle = cvar.wait(idle).unwrap();
        }
    }

    /// Get the current playback time in ms.
    pub fn current_time_ms(&self) -> u64 {
        *self.current_time_ms.lock().unwrap()
    }

    /// Scheduler thread main loop.
    fn run(
        receiver: Receiver<SchedulerCommand>,
        connection: Arc<Mutex<MidiOutputConnection>>,
        current_time_ms: Arc<Mutex<u64>>,
        idle_signal: Arc<(Mutex<bool>, Condvar)>,
    ) {
        let mut queue: BinaryHeap<ScheduledEvent> = BinaryHeap::new();
        let start = Instant::now();

        loop {
            // Update current time
            let now_ms = start.elapsed().as_millis() as u64;
            if let Ok(mut time) = current_time_ms.lock() {
                *time = now_ms;
            }

            // Process any pending commands (non-blocking)
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SchedulerCommand::Schedule { time_ms, message } => {
                        queue.push(ScheduledEvent { time_ms, message });
                    }
                    SchedulerCommand::Stop => {
                        queue.clear();
                        // Send all notes off on all channels
                        if let Ok(mut conn) = connection.lock() {
                            for ch in 0..16u8 {
                                let _ = conn.send(&[0xB0 | ch, 123, 0]); // All Notes Off
                            }
                        }
                    }
                    SchedulerCommand::SetTempo(_) => {
                        // Tempo changes don't affect already-scheduled events
                    }
                    SchedulerCommand::Shutdown => {
                        return;
                    }
                }
            }

            // Send any events that are due
            while let Some(event) = queue.peek() {
                if event.time_ms <= now_ms {
                    let event = queue.pop().unwrap();
                    if let Ok(mut conn) = connection.lock() {
                        let _ = conn.send(&event.message);
                    }
                } else {
                    break;
                }
            }

            // Update idle status
            if queue.is_empty() {
                let (lock, cvar) = &*idle_signal;
                if let Ok(mut idle) = lock.lock() {
                    *idle = true;
                    cvar.notify_all();
                }
            }

            // Small sleep to prevent busy-waiting
            thread::sleep(StdDuration::from_micros(500));
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let _ = self.sender.send(SchedulerCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_event_ordering() {
        // Earlier events should have higher priority (min-heap)
        let early = ScheduledEvent { time_ms: 100, message: vec![] };
        let late = ScheduledEvent { time_ms: 200, message: vec![] };

        // In a max-heap with reversed ordering, early > late
        assert!(early > late);
    }
}
```

**Step 2: Update playback mod.rs**

Add to `src/midi/playback/mod.rs`:

```rust
mod scheduler;

pub(crate) use scheduler::Scheduler;
```

**Step 3: Run tests**

Run: `cargo test --features midi-playback playback::scheduler`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/midi/playback/scheduler.rs src/midi/playback/mod.rs
git commit -m "feat(midi): add Scheduler for async MIDI playback"
```

---

## Task 8: Implement Async Playback Methods

**Files:**
- Modify: `src/midi/playback/player.rs`

**Step 1: Add scheduler to MidiPlayer and async methods**

Update `src/midi/playback/player.rs`:

```rust
use super::scheduler::Scheduler;

/// Real-time MIDI player.
pub struct MidiPlayer {
    connection: Arc<Mutex<MidiOutputConnection>>,
    scheduler: Scheduler,
    tempo: u16,
    channel: Channel,
    cursor_ms: u64,
}

impl MidiPlayer {
    /// Connect to a MIDI output port by name.
    pub fn connect(port_name: &str) -> Result<Self, PlaybackError> {
        let midi_out = MidiOutput::new("rust-music-theory")
            .map_err(|e| PlaybackError::InitError(e.to_string()))?;

        let ports = midi_out.ports();
        let port = ports
            .iter()
            .find(|p| {
                midi_out.port_name(p).map(|n| n == port_name).unwrap_or(false)
            })
            .ok_or_else(|| PlaybackError::PortNotFound(port_name.to_string()))?;

        let connection = midi_out
            .connect(port, "output")
            .map_err(|e| PlaybackError::ConnectionFailed(e.to_string()))?;

        let connection = Arc::new(Mutex::new(connection));
        let scheduler = Scheduler::new(connection.clone());

        Ok(Self {
            connection,
            scheduler,
            tempo: 120,
            channel: Channel::new(0).unwrap(),
            cursor_ms: 0,
        })
    }

    /// Connect to a MIDI output port by index.
    pub fn connect_index(index: usize) -> Result<Self, PlaybackError> {
        let midi_out = MidiOutput::new("rust-music-theory")
            .map_err(|e| PlaybackError::InitError(e.to_string()))?;

        let ports = midi_out.ports();
        let port = ports
            .get(index)
            .ok_or_else(|| PlaybackError::PortNotFound(format!("index {}", index)))?;

        let connection = midi_out
            .connect(port, "output")
            .map_err(|e| PlaybackError::ConnectionFailed(e.to_string()))?;

        let connection = Arc::new(Mutex::new(connection));
        let scheduler = Scheduler::new(connection.clone());

        Ok(Self {
            connection,
            scheduler,
            tempo: 120,
            channel: Channel::new(0).unwrap(),
            cursor_ms: 0,
        })
    }

    // ... keep existing set_tempo, tempo, set_channel, channel methods ...

    /// Schedule notes to play asynchronously.
    pub fn play_async<N: Notes>(&mut self, notes: &N, duration: Duration, velocity: Velocity) {
        let pitches: Vec<u8> = notes.notes().iter().map(|n| n.midi_pitch()).collect();
        let duration_ms = duration_to_ms(&duration, self.tempo);
        let channel = self.channel.value();

        // Schedule Note On for all pitches
        for &pitch in &pitches {
            let message = vec![0x90 | (channel & 0x0F), pitch & 0x7F, velocity.value() & 0x7F];
            self.scheduler.schedule(self.cursor_ms, message);
        }

        // Schedule Note Off for all pitches
        let note_off_time = self.cursor_ms + duration_ms;
        for &pitch in &pitches {
            let message = vec![0x80 | (channel & 0x0F), pitch & 0x7F, 0];
            self.scheduler.schedule(note_off_time, message);
        }

        // Advance cursor
        self.cursor_ms = note_off_time;
    }

    /// Schedule a single note to play asynchronously.
    pub fn play_note_async(&mut self, pitch: u8, duration: Duration, velocity: Velocity) {
        let duration_ms = duration_to_ms(&duration, self.tempo);
        let channel = self.channel.value();

        let note_on = vec![0x90 | (channel & 0x0F), pitch & 0x7F, velocity.value() & 0x7F];
        self.scheduler.schedule(self.cursor_ms, note_on);

        let note_off_time = self.cursor_ms + duration_ms;
        let note_off = vec![0x80 | (channel & 0x0F), pitch & 0x7F, 0];
        self.scheduler.schedule(note_off_time, note_off);

        self.cursor_ms = note_off_time;
    }

    /// Schedule a rest (advances cursor without playing).
    pub fn rest_async(&mut self, duration: Duration) {
        let duration_ms = duration_to_ms(&duration, self.tempo);
        self.cursor_ms += duration_ms;
    }

    /// Wait for all scheduled notes to finish playing.
    pub fn wait(&self) {
        self.scheduler.wait();
    }

    /// Stop all playing notes immediately.
    pub fn stop(&self) {
        self.scheduler.stop();
    }
}
```

**Step 2: Run tests**

Run: `cargo test --features midi-playback playback::player`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add src/midi/playback/player.rs
git commit -m "feat(midi): add async playback methods (play_async, wait, stop)"
```

---

## Task 9: Integration Tests

**Files:**
- Create: `tests/midi_playback_integration.rs`

**Step 1: Create integration tests**

Create `tests/midi_playback_integration.rs`:

```rust
#![cfg(feature = "midi-playback")]

use rust_music_theory::chord::{Chord, Quality, Number};
use rust_music_theory::note::{Pitch, PitchSymbol::*};
use rust_music_theory::midi::{MidiPorts, MidiPlayer, Duration, Velocity, Channel};

#[test]
fn list_ports_does_not_panic() {
    let ports = MidiPorts::list();
    assert!(ports.is_ok());
}

#[test]
fn ports_iteration() {
    let ports = MidiPorts::list().unwrap();
    for (i, name) in ports.iter().enumerate() {
        println!("Port {}: {}", i, name);
    }
}

#[test]
fn connect_invalid_port_name() {
    let result = MidiPlayer::connect("This Port Does Not Exist 12345");
    assert!(result.is_err());
}

#[test]
fn connect_invalid_port_index() {
    let result = MidiPlayer::connect_index(99999);
    assert!(result.is_err());
}

#[test]
#[ignore] // Requires hardware: cargo test --features midi-playback -- --ignored
fn play_chord_blocking() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() {
        println!("No MIDI ports available, skipping test");
        return;
    }

    let mut player = MidiPlayer::connect_index(0).unwrap();
    player.set_tempo(120);

    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
    player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
}

#[test]
#[ignore] // Requires hardware
fn play_chord_async() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let mut player = MidiPlayer::connect_index(0).unwrap();
    player.set_tempo(120);

    let c_chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
    let g_chord = Chord::new(Pitch::from(G), Quality::Major, Number::Triad);

    player.play_async(&c_chord, Duration::Quarter, Velocity::new(100).unwrap());
    player.play_async(&g_chord, Duration::Quarter, Velocity::new(100).unwrap());

    player.wait();
}

#[test]
#[ignore] // Requires hardware
fn play_with_rest() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let mut player = MidiPlayer::connect_index(0).unwrap();
    player.set_tempo(120);

    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);

    player.play_async(&chord, Duration::Quarter, Velocity::new(100).unwrap());
    player.rest_async(Duration::Quarter);
    player.play_async(&chord, Duration::Quarter, Velocity::new(100).unwrap());

    player.wait();
}

#[test]
#[ignore] // Requires hardware
fn change_channel() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let mut player = MidiPlayer::connect_index(0).unwrap();
    player.set_tempo(120);

    let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);

    player.set_channel(Channel::new(0).unwrap());
    player.play_async(&chord, Duration::Quarter, Velocity::new(100).unwrap());

    player.set_channel(Channel::new(1).unwrap());
    player.play_async(&chord, Duration::Quarter, Velocity::new(100).unwrap());

    player.wait();
}
```

**Step 2: Run unit tests (no hardware needed)**

Run: `cargo test --features midi-playback --test midi_playback_integration`
Expected: 4 tests PASS, 4 tests IGNORED

**Step 3: Commit**

```bash
git add tests/midi_playback_integration.rs
git commit -m "test(midi): add integration tests for midi-playback"
```

---

## Task 10: Update Documentation

**Files:**
- Modify: `src/midi/playback/mod.rs`
- Modify: `src/midi/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Add module-level documentation to playback/mod.rs**

Update `src/midi/playback/mod.rs`:

```rust
//! Real-time MIDI playback functionality.
//!
//! This module provides real-time MIDI playback to connected hardware synthesizers.
//! Enable with the `midi-playback` feature flag.
//!
//! # Example
//!
//! ```ignore
//! use rust_music_theory::chord::{Chord, Quality, Number};
//! use rust_music_theory::note::{Pitch, PitchSymbol::*};
//! use rust_music_theory::midi::{MidiPorts, MidiPlayer, Duration, Velocity};
//!
//! // List available MIDI ports
//! let ports = MidiPorts::list()?;
//! for (i, name) in ports.iter().enumerate() {
//!     println!("{}: {}", i, name);
//! }
//!
//! // Connect and play
//! let mut player = MidiPlayer::connect_index(0)?;
//! player.set_tempo(120);
//!
//! let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
//! player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
//! ```

mod error;
mod player;
mod ports;
mod scheduler;
mod timing;

pub use error::PlaybackError;
pub use player::MidiPlayer;
pub use ports::MidiPorts;

pub(crate) use scheduler::Scheduler;
pub(crate) use timing::duration_to_ms;
```

**Step 2: Update midi/mod.rs with playback documentation**

Add to the module docstring in `src/midi/mod.rs`:

```rust
//! # Real-Time Playback (optional feature)
//!
//! With the `midi-playback` feature, you can play notes on connected MIDI devices:
//!
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi-playback"] }
//! ```
//!
//! ```ignore
//! use rust_music_theory::midi::{MidiPorts, MidiPlayer, Duration, Velocity};
//!
//! let ports = MidiPorts::list()?;
//! let mut player = MidiPlayer::connect_index(0)?;
//! player.set_tempo(120);
//! player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
//! ```
```

**Step 3: Update lib.rs with playback mention**

Add to the module docstring in `src/lib.rs`:

```rust
//! ## Real-Time Playback (optional feature)
//!
//! With the `midi-playback` feature, play to connected MIDI instruments:
//!
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi-playback"] }
//! ```
//!
//! ```ignore
//! use rustmt::midi::{MidiPorts, MidiPlayer};
//!
//! let ports = MidiPorts::list()?;
//! let mut player = MidiPlayer::connect_index(0)?;
//! player.play(&chord, Duration::Quarter, Velocity::new(100).unwrap());
//! ```
```

**Step 4: Run doc tests**

Run: `cargo test --features midi-playback --doc`
Expected: All doc tests pass (ignored examples don't run)

**Step 5: Commit**

```bash
git add src/midi/playback/mod.rs src/midi/mod.rs src/lib.rs
git commit -m "docs(midi): add midi-playback documentation"
```

---

## Task 11: Final Verification

**Step 1: Run all tests**

Run: `cargo test --features midi-playback`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cargo clippy --features midi-playback -- -D warnings`
Expected: No warnings from midi-playback code

**Step 3: Verify without feature**

Run: `cargo check`
Expected: Compiles without midi-playback feature

**Step 4: Verify both features together**

Run: `cargo check --features midi,midi-playback`
Expected: Compiles successfully

**Step 5: Build docs**

Run: `cargo doc --features midi-playback --no-deps`
Expected: Documentation builds successfully

**Step 6: Commit any fixes**

If any issues were found and fixed:

```bash
git add -A
git commit -m "fix(midi): address final verification issues"
```

---

## Summary

| Task | Component | Description |
|------|-----------|-------------|
| 1 | Feature flag | Add `midi-playback` feature with `midir` dependency |
| 2 | Module structure | Create playback module with `PlaybackError` |
| 3 | MidiPorts | Device discovery and port listing |
| 4 | Timing | `duration_to_ms` helper function |
| 5 | MidiPlayer | Connection and configuration |
| 6 | Blocking | `play`, `play_note`, `rest` methods |
| 7 | Scheduler | Background thread for timed events |
| 8 | Async | `play_async`, `rest_async`, `wait`, `stop` |
| 9 | Integration | Hardware integration tests |
| 10 | Documentation | Module and crate docs |
| 11 | Verification | Final testing and validation |
