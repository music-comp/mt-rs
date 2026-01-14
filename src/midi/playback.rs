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
//! use rust_music_theory::midi::playback::{MidiPorts, MidiPlayer};
//! use rust_music_theory::midi::{Duration, Velocity};
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
