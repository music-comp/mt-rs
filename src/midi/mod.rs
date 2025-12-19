//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```
//!
//! # Quick Export
//!
//! ```ignore
//! use rust_music_theory::midi::{ToMidi, Duration, Velocity};
//!
//! let chord = Chord::new(Pitch::from(C), Quality::Major, Number::Triad);
//! chord.to_midi(Duration::Quarter, Velocity::new(100).unwrap())
//!     .save("chord.mid")?;
//! ```

mod builder;
mod duration;
pub(crate) mod event;
mod export;
mod file;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use export::{MidiExport, ToMidi};
pub use file::MidiFile;
pub use types::{Channel, Velocity};
