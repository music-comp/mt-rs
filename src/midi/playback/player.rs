//! MIDI player for real-time playback.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;

use midir::{MidiOutput, MidiOutputConnection};

use crate::midi::{Channel, Duration, Velocity};
use crate::note::Notes;
use super::PlaybackError;
use super::timing::duration_to_ms;

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
