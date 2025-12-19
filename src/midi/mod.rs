//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod builder;
mod duration;
pub(crate) mod event;
mod file;
mod types;

pub use builder::{MidiBuilder, DEFAULT_PPQ};
pub use duration::Duration;
pub use file::MidiFile;
pub use types::{Channel, Velocity};
