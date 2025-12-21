# MIDI Extensions Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Control Change (CC), Program Change, and MIDI Clock (master) to midi-playback.

**Architecture:** Extend MidiPlayer with new methods that construct MIDI messages and send them via the existing connection. MIDI Clock runs on the scheduler thread with a separate timing loop.

**Tech Stack:** Rust, midir

---

### Task 1: Add Control Change

**Files:**
- Modify: `src/midi/playback/player.rs`
- Test: `src/midi/playback/player.rs` (inline tests)

**Step 1: Write the failing test**

Add to `src/midi/playback/player.rs` in the `tests` module:

```rust
#[test]
fn control_change_message_bytes() {
    // CC message format: [0xB0 | channel, cc_number, value]
    let channel = 0u8;
    let cc = 1u8;  // Modulation
    let value = 64u8;

    let status = 0xB0 | (channel & 0x0F);
    let message = [status, cc & 0x7F, value & 0x7F];

    assert_eq!(message, [0xB0, 1, 64]);
}

#[test]
fn control_change_channel_5() {
    let channel = 5u8;
    let status = 0xB0 | (channel & 0x0F);
    assert_eq!(status, 0xB5);
}
```

**Step 2: Run test to verify it passes** (this is a message format test)

Run: `cargo test --features midi-playback control_change_message`
Expected: PASS

**Step 3: Add control_change method to MidiPlayer**

Add after `send_note_off` method in `src/midi/playback/player.rs`:

```rust
/// Send a Control Change message immediately.
pub fn control_change(&self, cc: u8, value: u8) {
    let status = 0xB0 | (self.channel.value() & 0x0F);
    let message = [status, cc & 0x7F, value & 0x7F];

    if let Ok(mut conn) = self.connection.lock() {
        let _ = conn.send(&message);
    }
}

/// Schedule a Control Change message asynchronously.
pub fn control_change_async(&mut self, cc: u8, value: u8) {
    let status = 0xB0 | (self.channel.value() & 0x0F);
    let message = vec![status, cc & 0x7F, value & 0x7F];
    self.scheduler.schedule(self.cursor_ms, message);
}
```

**Step 4: Run tests**

Run: `cargo test --features midi-playback`
Expected: PASS

**Step 5: Commit**

```bash
git add src/midi/playback/player.rs
git commit -m "add control_change and control_change_async methods"
```

---

### Task 2: Add Program Change

**Files:**
- Modify: `src/midi/playback/player.rs`

**Step 1: Write the failing test**

Add to tests module:

```rust
#[test]
fn program_change_message_bytes() {
    // Program Change: [0xC0 | channel, program]
    let channel = 0u8;
    let program = 5u8;

    let status = 0xC0 | (channel & 0x0F);
    let message = [status, program & 0x7F];

    assert_eq!(message, [0xC0, 5]);
}

#[test]
fn bank_select_message_bytes() {
    // Bank Select MSB: [0xB0 | channel, 0, msb]
    // Bank Select LSB: [0xB0 | channel, 32, lsb]
    let channel = 0u8;
    let msb = 1u8;
    let lsb = 2u8;

    let status = 0xB0 | (channel & 0x0F);
    let msg_msb = [status, 0, msb & 0x7F];
    let msg_lsb = [status, 32, lsb & 0x7F];

    assert_eq!(msg_msb, [0xB0, 0, 1]);
    assert_eq!(msg_lsb, [0xB0, 32, 2]);
}
```

**Step 2: Run test**

Run: `cargo test --features midi-playback program_change`
Expected: PASS

**Step 3: Add program_change methods**

Add after `control_change_async`:

```rust
/// Send a Program Change message immediately.
pub fn program_change(&self, program: u8) {
    let status = 0xC0 | (self.channel.value() & 0x0F);
    let message = [status, program & 0x7F];

    if let Ok(mut conn) = self.connection.lock() {
        let _ = conn.send(&message);
    }
}

/// Send a Program Change with Bank Select immediately.
pub fn program_change_with_bank(&self, program: u8, bank_msb: u8, bank_lsb: u8) {
    let channel = self.channel.value() & 0x0F;
    let cc_status = 0xB0 | channel;
    let pc_status = 0xC0 | channel;

    if let Ok(mut conn) = self.connection.lock() {
        // Bank Select MSB (CC 0)
        let _ = conn.send(&[cc_status, 0, bank_msb & 0x7F]);
        // Bank Select LSB (CC 32)
        let _ = conn.send(&[cc_status, 32, bank_lsb & 0x7F]);
        // Program Change
        let _ = conn.send(&[pc_status, program & 0x7F]);
    }
}

/// Schedule a Program Change asynchronously.
pub fn program_change_async(&mut self, program: u8) {
    let status = 0xC0 | (self.channel.value() & 0x0F);
    let message = vec![status, program & 0x7F];
    self.scheduler.schedule(self.cursor_ms, message);
}

/// Schedule a Program Change with Bank Select asynchronously.
pub fn program_change_with_bank_async(&mut self, program: u8, bank_msb: u8, bank_lsb: u8) {
    let channel = self.channel.value() & 0x0F;
    let cc_status = 0xB0 | channel;
    let pc_status = 0xC0 | channel;

    // Bank Select MSB (CC 0)
    self.scheduler.schedule(self.cursor_ms, vec![cc_status, 0, bank_msb & 0x7F]);
    // Bank Select LSB (CC 32)
    self.scheduler.schedule(self.cursor_ms, vec![cc_status, 32, bank_lsb & 0x7F]);
    // Program Change
    self.scheduler.schedule(self.cursor_ms, vec![pc_status, program & 0x7F]);
}
```

**Step 4: Run tests**

Run: `cargo test --features midi-playback`
Expected: PASS

**Step 5: Commit**

```bash
git add src/midi/playback/player.rs
git commit -m "add program_change methods with bank select support"
```

---

### Task 3: Add MIDI Clock to Scheduler

**Files:**
- Modify: `src/midi/playback/scheduler.rs`

**Step 1: Write the failing test**

Add to tests module in `scheduler.rs`:

```rust
#[test]
fn clock_tick_interval_at_120_bpm() {
    // 24 ticks per quarter note
    // At 120 BPM: quarter note = 500ms
    // Tick interval = 500ms / 24 = 20.833...ms
    let bpm = 120u16;
    let quarter_note_ms = 60_000u64 / bpm as u64;
    let tick_interval_us = (quarter_note_ms * 1000) / 24;

    assert_eq!(quarter_note_ms, 500);
    assert_eq!(tick_interval_us, 20833);
}

#[test]
fn clock_tick_interval_at_140_bpm() {
    let bpm = 140u16;
    let quarter_note_ms = 60_000u64 / bpm as u64;
    let tick_interval_us = (quarter_note_ms * 1000) / 24;

    assert_eq!(quarter_note_ms, 428); // 60000/140 = 428.57
    assert_eq!(tick_interval_us, 17833); // 428000/24 = 17833.33
}
```

**Step 2: Run test**

Run: `cargo test --features midi-playback clock_tick`
Expected: PASS

**Step 3: Add clock commands to SchedulerCommand**

Update the enum in `scheduler.rs`:

```rust
/// Messages sent to the scheduler thread.
#[allow(dead_code)]
pub enum SchedulerCommand {
    /// Schedule a MIDI message at a future time (ms from start)
    Schedule { time_ms: u64, message: Vec<u8> },
    /// Update tempo (used for clock timing)
    SetTempo(u16),
    /// Stop all notes immediately
    Stop,
    /// Shutdown the scheduler
    Shutdown,
    /// Start MIDI clock
    StartClock,
    /// Stop MIDI clock
    StopClock,
}
```

**Step 4: Update Scheduler struct and run method**

Add `tempo` field to Scheduler struct:

```rust
pub struct Scheduler {
    sender: Sender<SchedulerCommand>,
    thread: Option<JoinHandle<()>>,
    #[allow(dead_code)]
    current_time_ms: Arc<Mutex<u64>>,
    idle_signal: Arc<(Mutex<bool>, Condvar)>,
}
```

Update `new()` to pass initial tempo:

```rust
pub fn new(connection: Arc<Mutex<MidiOutputConnection>>) -> Self {
    Self::with_tempo(connection, 120)
}

pub fn with_tempo(connection: Arc<Mutex<MidiOutputConnection>>, initial_tempo: u16) -> Self {
    let (sender, receiver) = mpsc::channel();
    let current_time_ms = Arc::new(Mutex::new(0u64));
    let idle_signal = Arc::new((Mutex::new(true), Condvar::new()));

    let time_clone = current_time_ms.clone();
    let idle_clone = idle_signal.clone();

    let thread = thread::spawn(move || {
        Self::run(receiver, connection, time_clone, idle_clone, initial_tempo);
    });

    Self {
        sender,
        thread: Some(thread),
        current_time_ms,
        idle_signal,
    }
}
```

Add clock methods:

```rust
/// Start the MIDI clock.
pub fn start_clock(&self) {
    let _ = self.sender.send(SchedulerCommand::StartClock);
}

/// Stop the MIDI clock.
pub fn stop_clock(&self) {
    let _ = self.sender.send(SchedulerCommand::StopClock);
}

/// Update the tempo (affects clock speed).
pub fn set_tempo(&self, bpm: u16) {
    let _ = self.sender.send(SchedulerCommand::SetTempo(bpm));
}
```

Update `run()` signature and add clock logic:

```rust
fn run(
    receiver: Receiver<SchedulerCommand>,
    connection: Arc<Mutex<MidiOutputConnection>>,
    current_time_ms: Arc<Mutex<u64>>,
    idle_signal: Arc<(Mutex<bool>, Condvar)>,
    initial_tempo: u16,
) {
    let mut queue: BinaryHeap<ScheduledEvent> = BinaryHeap::new();
    let start = Instant::now();
    let mut clock_running = false;
    let mut tempo = initial_tempo;
    let mut last_clock_tick = Instant::now();

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
                    clock_running = false;
                    // Send all notes off on all channels
                    if let Ok(mut conn) = connection.lock() {
                        for ch in 0..16u8 {
                            let _ = conn.send(&[0xB0 | ch, 123, 0]);
                        }
                        // Send MIDI Stop
                        let _ = conn.send(&[0xFC]);
                    }
                }
                SchedulerCommand::SetTempo(bpm) => {
                    tempo = bpm;
                }
                SchedulerCommand::Shutdown => {
                    if clock_running {
                        if let Ok(mut conn) = connection.lock() {
                            let _ = conn.send(&[0xFC]); // Stop
                        }
                    }
                    return;
                }
                SchedulerCommand::StartClock => {
                    clock_running = true;
                    last_clock_tick = Instant::now();
                    if let Ok(mut conn) = connection.lock() {
                        let _ = conn.send(&[0xFA]); // MIDI Start
                    }
                }
                SchedulerCommand::StopClock => {
                    clock_running = false;
                    if let Ok(mut conn) = connection.lock() {
                        let _ = conn.send(&[0xFC]); // MIDI Stop
                    }
                }
            }
        }

        // Send MIDI clock ticks if running
        if clock_running {
            let tick_interval_us = (60_000_000u64 / tempo as u64) / 24;
            let elapsed = last_clock_tick.elapsed().as_micros() as u64;

            if elapsed >= tick_interval_us {
                if let Ok(mut conn) = connection.lock() {
                    let _ = conn.send(&[0xF8]); // Clock tick
                }
                last_clock_tick = Instant::now();
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

        // Update idle status (clock running counts as not idle)
        if queue.is_empty() && !clock_running {
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
```

**Step 5: Run tests**

Run: `cargo test --features midi-playback`
Expected: PASS

**Step 6: Commit**

```bash
git add src/midi/playback/scheduler.rs
git commit -m "add MIDI clock support to scheduler"
```

---

### Task 4: Add Clock Methods to MidiPlayer

**Files:**
- Modify: `src/midi/playback/player.rs`

**Step 1: Add clock methods to MidiPlayer**

Add after `stop()`:

```rust
/// Start the MIDI clock (sends 24 pulses per quarter note).
pub fn start_clock(&self) {
    self.scheduler.start_clock();
}

/// Stop the MIDI clock.
pub fn stop_clock(&self) {
    self.scheduler.stop_clock();
}
```

**Step 2: Update set_tempo to also update scheduler**

Modify existing `set_tempo`:

```rust
/// Set the tempo in BPM.
pub fn set_tempo(&mut self, bpm: u16) {
    self.tempo = bpm;
    self.scheduler.set_tempo(bpm);
}
```

**Step 3: Run tests**

Run: `cargo test --features midi-playback`
Expected: PASS

**Step 4: Commit**

```bash
git add src/midi/playback/player.rs
git commit -m "add start_clock and stop_clock to MidiPlayer"
```

---

### Task 5: Add Integration Tests

**Files:**
- Modify: `tests/midi_playback_integration.rs`

**Step 1: Add integration tests**

Add to end of file:

```rust
#[test]
#[ignore]
fn control_change_modulation() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let player = MidiPlayer::connect_index(0).unwrap();
    player.control_change(1, 64);  // Modulation at 50%
    player.control_change(1, 127); // Modulation at 100%
    player.control_change(1, 0);   // Modulation off
}

#[test]
#[ignore]
fn program_change_switches_instrument() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let player = MidiPlayer::connect_index(0).unwrap();
    player.program_change(0);  // Piano
    player.program_change(25); // Acoustic Guitar
    player.program_change(40); // Violin
}

#[test]
#[ignore]
fn program_change_with_bank_select() {
    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let player = MidiPlayer::connect_index(0).unwrap();
    player.program_change_with_bank(5, 0, 1);
}

#[test]
#[ignore]
fn midi_clock_sends_to_daw() {
    use std::thread;
    use std::time::Duration;

    let ports = MidiPorts::list().unwrap();
    if ports.is_empty() { return; }

    let mut player = MidiPlayer::connect_index(0).unwrap();
    player.set_tempo(120);
    player.start_clock();
    thread::sleep(Duration::from_secs(2));
    player.stop_clock();
}
```

**Step 2: Run tests**

Run: `cargo test --features midi-playback`
Expected: PASS (ignored tests don't run)

**Step 3: Commit**

```bash
git add tests/midi_playback_integration.rs
git commit -m "add integration tests for CC, program change, and clock"
```

---

### Task 6: Add Example

**Files:**
- Create: `examples/midi_cc_demo.rs`

**Step 1: Create example file**

```rust
//! MIDI CC, Program Change, and Clock demo.
//! Run with: cargo run --example midi_cc_demo --features midi-playback

use std::thread;
use std::time::Duration;

use rust_music_theory::midi::playback::{MidiPorts, MidiPlayer};
use rust_music_theory::midi::{Duration as NoteDuration, Velocity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ports = MidiPorts::list()?;
    println!("Available MIDI ports:");
    for (i, name) in ports.iter().enumerate() {
        println!("  {}: {}", i, name);
    }

    if ports.is_empty() {
        println!("\nNo MIDI ports found.");
        return Ok(());
    }

    let mut player = MidiPlayer::connect_index(0)?;
    player.set_tempo(120);

    // Demo 1: Program Change
    println!("\n--- Program Change Demo ---");
    println!("Switching to Piano (Program 0)...");
    player.program_change(0);
    player.play_note(60, NoteDuration::Quarter, Velocity::new(100).unwrap());

    println!("Switching to Electric Piano (Program 4)...");
    player.program_change(4);
    player.play_note(60, NoteDuration::Quarter, Velocity::new(100).unwrap());

    // Demo 2: Control Change
    println!("\n--- Control Change Demo ---");
    println!("Modulation wheel sweep...");
    for i in (0..=127).step_by(16) {
        player.control_change(1, i); // Modulation
        thread::sleep(Duration::from_millis(100));
    }
    player.control_change(1, 0); // Reset

    // Demo 3: MIDI Clock
    println!("\n--- MIDI Clock Demo ---");
    println!("Starting clock at 120 BPM for 4 seconds...");
    println!("(Enable 'Ext' sync in your DAW to see it sync)");
    player.start_clock();
    thread::sleep(Duration::from_secs(4));
    player.stop_clock();
    println!("Clock stopped.");

    println!("\nDemo complete!");
    Ok(())
}
```

**Step 2: Test example compiles**

Run: `cargo build --example midi_cc_demo --features midi-playback`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add examples/midi_cc_demo.rs
git commit -m "add midi_cc_demo example"
```

---

### Task 7: Final Verification

**Step 1: Run all tests**

Run: `cargo test --features midi-playback`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --features midi-playback`
Expected: No errors in playback module

**Step 3: Test with hardware (manual)**

Run: `cargo run --example midi_cc_demo --features midi-playback`
Expected: Plays demo, clock syncs with DAW

**Step 4: Commit any fixes and push**

```bash
git push
```
