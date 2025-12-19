//! MIDI export functionality for rust-music-theory.
//!
//! Enable with the `midi` feature flag:
//! ```toml
//! rust-music-theory = { version = "0.3", features = ["midi"] }
//! ```

mod types;

// Duration will be added in Task 5
// mod duration;
// pub use duration::Duration;

pub use types::{Channel, Velocity};
