use crate::quintal::IntervalStructure;

use super::types::QuartalIntervalStructure;

/// Complement an interval mod 12.
///
/// Maps P5(7) to P4(5), A5(8) to d4(4), tritone(6) to tritone(6).
/// This is an involution: applying it twice returns the original value.
pub fn quintal_to_quartal_interval(semitones: u8) -> u8 {
    (12 - semitones) % 12
}

/// Complement an interval mod 12 (quartal to quintal direction).
///
/// This is the same function as [`quintal_to_quartal_interval`] because
/// interval complementation is an involution.
pub fn quartal_to_quintal_interval(semitones: u8) -> u8 {
    (12 - semitones) % 12
}

/// Convert a quintal interval structure to quartal.
///
/// Reverses the order AND complements each interval:
/// `(i1, i2, i3)` becomes `((12-i3)%12, (12-i2)%12, (12-i1)%12)`.
///
/// This is an involution: applying it twice returns the original structure.
pub fn quintal_to_quartal_structure(qs: &IntervalStructure) -> QuartalIntervalStructure {
    QuartalIntervalStructure((12 - qs.2) % 12, (12 - qs.1) % 12, (12 - qs.0) % 12)
}

/// Convert a quartal interval structure to quintal.
///
/// This is the same transformation as [`quintal_to_quartal_structure`]
/// because the combined reversal-and-complement operation is an involution.
pub fn quartal_to_quintal_structure(qs: &QuartalIntervalStructure) -> IntervalStructure {
    IntervalStructure((12 - qs.2) % 12, (12 - qs.1) % 12, (12 - qs.0) % 12)
}
