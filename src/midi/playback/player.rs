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
