//! MIDI type newtypes (Velocity, Channel).

/// MIDI velocity (0-127). Controls note loudness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity(u8);

impl Velocity {
    /// Create a new velocity value. Returns None if value > 127.
    pub fn new(v: u8) -> Option<Self> {
        (v <= 127).then_some(Self(v))
    }

    /// Maximum velocity (127).
    pub fn max() -> Self {
        Self(127)
    }

    /// Get the inner velocity value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// MIDI channel (0-15). Channel 9 = drums by convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(u8);

impl Channel {
    /// Create a new channel. Returns None if value > 15.
    pub fn new(c: u8) -> Option<Self> {
        (c <= 15).then_some(Self(c))
    }

    /// Drum channel (9, which is channel 10 in 1-indexed MIDI).
    pub fn drums() -> Self {
        Self(9)
    }

    /// Get the inner channel value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_valid_values() {
        assert!(Velocity::new(0).is_some());
        assert!(Velocity::new(64).is_some());
        assert!(Velocity::new(127).is_some());
    }

    #[test]
    fn velocity_invalid_values() {
        assert!(Velocity::new(128).is_none());
        assert!(Velocity::new(255).is_none());
    }

    #[test]
    fn velocity_max() {
        assert_eq!(Velocity::max(), Velocity(127));
    }

    #[test]
    fn velocity_inner_value() {
        let vel = Velocity::new(100).unwrap();
        assert_eq!(vel.value(), 100);
    }

    #[test]
    fn channel_valid_values() {
        assert!(Channel::new(0).is_some());
        assert!(Channel::new(9).is_some());
        assert!(Channel::new(15).is_some());
    }

    #[test]
    fn channel_invalid_values() {
        assert!(Channel::new(16).is_none());
        assert!(Channel::new(255).is_none());
    }

    #[test]
    fn channel_drums() {
        assert_eq!(Channel::drums(), Channel(9));
    }

    #[test]
    fn channel_inner_value() {
        let ch = Channel::new(5).unwrap();
        assert_eq!(ch.value(), 5);
    }
}
