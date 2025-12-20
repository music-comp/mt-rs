//! Real-time MIDI playback functionality.

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
