//! Real-time MIDI playback functionality.

mod error;
mod ports;
mod timing;

pub use error::PlaybackError;
pub use ports::MidiPorts;

pub(crate) use timing::duration_to_ms;
