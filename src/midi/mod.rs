//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod duration;
pub(crate) mod event;
mod types;

pub use duration::Duration;
pub use types::{Channel, Velocity};
