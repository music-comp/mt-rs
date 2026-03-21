use crate::note::Note;

/// A counterpoint rule violation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Violation {
    /// Beat position (0-indexed).
    pub position: usize,
    /// Rule identifier.
    pub rule: String,
    /// Human-readable explanation.
    pub description: String,
}

/// Result of a counterpoint check.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CounterpointResult {
    pub valid: bool,
    pub violations: Vec<Violation>,
}

fn absolute_pitch(note: &Note) -> i16 {
    note.octave as i16 * 12 + note.pitch.as_u8() as i16
}

fn interval_class(a: &Note, b: &Note) -> u8 {
    let diff = (absolute_pitch(b) - absolute_pitch(a)).unsigned_abs() as u8;
    diff % 12
}

/// Consonant intervals in first species: P1(0), m3(3), M3(4), P5(7), m6(8), M6(9).
/// P4(5) is consonant when the counterpoint is above.
fn is_consonant(ic: u8) -> bool {
    matches!(ic, 0 | 3 | 4 | 5 | 7 | 8 | 9)
}

/// Perfect consonances: P1/P8(0), P5(7). P4(5) included for upper voices.
fn is_perfect_consonance(ic: u8) -> bool {
    matches!(ic, 0 | 5 | 7)
}

/// Check two voices against first-species (note-against-note) counterpoint rules.
///
/// Rules checked:
/// 1. All vertical intervals must be consonant
/// 2. No parallel perfect consonances (parallel 5ths, octaves, unisons)
/// 3. First interval must be a perfect consonance
/// 4. Last interval must be a perfect consonance
/// 5. No voice crossing (counterpoint should stay above or below cantus firmus)
pub fn check_first_species(cantus_firmus: &[Note], counterpoint: &[Note]) -> CounterpointResult {
    let n = cantus_firmus.len().min(counterpoint.len());
    if n == 0 {
        return CounterpointResult {
            valid: true,
            violations: vec![],
        };
    }

    let mut violations = Vec::new();

    // Determine if counterpoint starts above or below
    let cp_above = absolute_pitch(&counterpoint[0]) >= absolute_pitch(&cantus_firmus[0]);

    for i in 0..n {
        let ic = interval_class(&cantus_firmus[i], &counterpoint[i]);

        // Rule 1: consonance
        if !is_consonant(ic) {
            violations.push(Violation {
                position: i,
                rule: "dissonance".into(),
                description: format!("Dissonant interval (ic {}) at position {}", ic, i),
            });
        }

        // Rule 5: voice crossing
        let cp_pitch = absolute_pitch(&counterpoint[i]);
        let cf_pitch = absolute_pitch(&cantus_firmus[i]);
        let crossed = (cp_above && cp_pitch < cf_pitch) || (!cp_above && cp_pitch > cf_pitch);
        if crossed {
            violations.push(Violation {
                position: i,
                rule: "voice_crossing".into(),
                description: format!("Voice crossing at position {}", i),
            });
        }
    }

    // Rule 3: opening must be perfect consonance
    let first_ic = interval_class(&cantus_firmus[0], &counterpoint[0]);
    if !is_perfect_consonance(first_ic) {
        violations.push(Violation {
            position: 0,
            rule: "opening".into(),
            description: format!(
                "Opening interval (ic {}) must be a perfect consonance",
                first_ic
            ),
        });
    }

    // Rule 4: closing must be perfect consonance
    if n >= 2 {
        let last_ic = interval_class(&cantus_firmus[n - 1], &counterpoint[n - 1]);
        if !is_perfect_consonance(last_ic) {
            violations.push(Violation {
                position: n - 1,
                rule: "closing".into(),
                description: format!(
                    "Closing interval (ic {}) must be a perfect consonance",
                    last_ic
                ),
            });
        }
    }

    // Rule 2: no parallel perfect consonances
    for i in 1..n {
        let prev_ic = interval_class(&cantus_firmus[i - 1], &counterpoint[i - 1]);
        let curr_ic = interval_class(&cantus_firmus[i], &counterpoint[i]);

        if is_perfect_consonance(prev_ic) && prev_ic == curr_ic {
            violations.push(Violation {
                position: i,
                rule: "parallel_perfect".into(),
                description: format!(
                    "Parallel perfect consonance (ic {}) at positions {}-{}",
                    curr_ic,
                    i - 1,
                    i
                ),
            });
        }
    }

    CounterpointResult {
        valid: violations.is_empty(),
        violations,
    }
}
