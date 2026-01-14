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
