# MIDI Extensions Design

## Overview

Extend midi-playback with Control Change (CC), Program Change, and MIDI Clock (master mode).

## API

### Control Change

```rust
// Send any CC message (0-127 for both cc and value)
player.control_change(cc: u8, value: u8);

// Examples:
player.control_change(1, 64);   // Modulation at 50%
player.control_change(7, 100);  // Volume
player.control_change(64, 127); // Sustain pedal on
player.control_change(64, 0);   // Sustain pedal off
```

### Program Change

```rust
// Change instrument (program 0-127, optional bank)
player.program_change(program: u8);
player.program_change_with_bank(program: u8, bank_msb: u8, bank_lsb: u8);

// Examples:
player.program_change(0);                   // Piano (GM)
player.program_change_with_bank(5, 0, 1);   // Bank 1, program 5
```

### MIDI Clock (Master)

```rust
// Start sending clock (24 pulses per quarter note)
player.start_clock();

// Stop clock
player.stop_clock();

// Clock runs automatically based on player.tempo
```

## Implementation

### Control Change (CC)

- MIDI message: `[0xB0 | channel, cc_number, value]`
- Add `send_control_change()` to MidiPlayer
- Both blocking (immediate) and async (scheduled) variants

### Program Change

- Program: `[0xC0 | channel, program]`
- Bank Select MSB: `[0xB0 | channel, 0, msb]`
- Bank Select LSB: `[0xB0 | channel, 32, lsb]`
- Send bank CCs before program change

### MIDI Clock

- Runs on scheduler thread (already exists)
- Clock tick: `[0xF8]` - 24 times per quarter note
- Start: `[0xFA]`
- Stop: `[0xFC]`
- At 120 BPM: tick every 20.83ms (500ms / 24)
- Add `clock_running: bool` flag to Scheduler

### New Scheduler Commands

```rust
enum SchedulerCommand {
    // existing...
    ControlChange { cc: u8, value: u8 },
    ProgramChange { program: u8 },
    BankSelect { msb: u8, lsb: u8 },
    StartClock,
    StopClock,
}
```

## Testing

### Unit Tests (no hardware)

- CC message byte construction (status, cc, value)
- Program change message bytes
- Bank select + program sequence
- Clock tick interval calculation at various BPMs

### Integration Tests (hardware required)

```rust
#[test]
#[ignore]
fn control_change_modulation() {
    let player = MidiPlayer::connect_index(0)?;
    player.control_change(1, 127);
}

#[test]
#[ignore]
fn program_change_with_bank() {
    let player = MidiPlayer::connect_index(0)?;
    player.program_change_with_bank(5, 0, 1);
}

#[test]
#[ignore]
fn clock_sends_to_daw() {
    let mut player = MidiPlayer::connect_index(0)?;
    player.set_tempo(120);
    player.start_clock();
    thread::sleep(Duration::from_secs(2));
    player.stop_clock();
}
```

### Example

`examples/midi_cc_demo.rs` - demonstrates CC, program change, and clock with Ableton.
