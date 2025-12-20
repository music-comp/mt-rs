//! Real-time MIDI playback functionality.

mod error;
mod player;
mod ports;
mod timing;

pub use error::PlaybackError;
pub use player::MidiPlayer;
pub use ports::MidiPorts;

pub(crate) use timing::duration_to_ms;
