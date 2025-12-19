//! MIDI type newtypes (Velocity, Channel).

/// MIDI velocity (0-127). Controls note loudness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity(u8);

impl Velocity {
    /// Create a new velocity value. Returns None if value > 127.
    pub fn new(v: u8) -> Option<Self> {
        (v <= 127).then(|| Self(v))
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
}
