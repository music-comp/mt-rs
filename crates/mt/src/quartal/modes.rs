//! Quartal-native mode computation.
//!
//! Modes depend on pitch-class sets, not stacking direction, so quartal modes
//! are identical to quintal modes. This module provides a quartal-native API
//! that accepts `QuartalOrbit` and delegates to `quintal::modes`.

use super::QuartalOrbit;
use crate::quintal::modes::{orbit_modes, OrbitModes};

/// Compute modes for a quartal orbit, natively.
///
/// Returns the same modes as the quintal computation (because modes depend
/// on the PC set, not the stacking direction), but the API is quartal-native.
pub fn quartal_orbit_modes(orbit: &QuartalOrbit) -> OrbitModes {
    orbit_modes(&orbit.to_quintal())
}
