use crate::note::Pitch;
use std::collections::HashSet;

/// Find pitch classes present in both inputs.
/// Returns pitches using the spelling from the first argument,
/// in the order they appear in the first argument.
pub fn common_tones(a: &[Pitch], b: &[Pitch]) -> Vec<Pitch> {
    let b_pcs: HashSet<u8> = b.iter().map(|p| p.as_u8()).collect();
    a.iter()
        .filter(|p| b_pcs.contains(&p.as_u8()))
        .copied()
        .collect()
}
