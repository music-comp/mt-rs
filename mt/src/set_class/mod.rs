mod forte;

use std::collections::BTreeSet;

/// An unordered set of pitch classes (integers mod 12).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PitchClassSet {
    pcs: BTreeSet<u8>,
}

impl PitchClassSet {
    /// Create from a slice of pitch values (reduced mod 12).
    pub fn new(pitches: &[u8]) -> Self {
        PitchClassSet {
            pcs: pitches.iter().map(|&p| p % 12).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.pcs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pcs.is_empty()
    }

    pub fn contains(&self, pc: u8) -> bool {
        self.pcs.contains(&(pc % 12))
    }

    /// T_n: transpose all pitch classes by n.
    pub fn transpose(&self, n: u8) -> Self {
        PitchClassSet {
            pcs: self.pcs.iter().map(|&pc| (pc + n) % 12).collect(),
        }
    }

    /// I_n: invert then transpose. Each pc becomes (n - pc) mod 12.
    pub fn invert(&self, n: u8) -> Self {
        PitchClassSet {
            pcs: self.pcs.iter().map(|&pc| (n + 12 - pc) % 12).collect(),
        }
    }

    /// Most compact ascending arrangement of the pitch classes.
    pub fn normal_form(&self) -> Vec<u8> {
        if self.pcs.is_empty() {
            return vec![];
        }
        let sorted: Vec<u8> = self.pcs.iter().copied().collect();
        let n = sorted.len();

        let mut best = sorted.clone();
        let mut best_span = (sorted[n - 1] + 12 - sorted[0]) % 12;

        for rotation in 1..n {
            let rotated: Vec<u8> = (0..n).map(|i| sorted[(rotation + i) % n]).collect();
            let span = (rotated[n - 1] + 12 - rotated[0]) % 12;
            if span < best_span || (span == best_span && rotated < best) {
                best = rotated;
                best_span = span;
            }
        }
        best
    }

    /// Lexicographically smallest normal form across all transpositions and inversions.
    pub fn prime_form(&self) -> Vec<u8> {
        if self.pcs.is_empty() {
            return vec![];
        }
        let mut best: Option<Vec<u8>> = None;

        for n in 0..12u8 {
            let nf = self.transpose(n).normal_form();
            let zeroed = zero_based(&nf);
            if best.is_none() || zeroed < *best.as_ref().unwrap() {
                best = Some(zeroed);
            }
        }
        for n in 0..12u8 {
            let nf = self.invert(n).normal_form();
            let zeroed = zero_based(&nf);
            if zeroed < *best.as_ref().unwrap() {
                best = Some(zeroed);
            }
        }

        best.unwrap_or_default()
    }

    /// Count of each interval class (ic1 through ic6).
    pub fn interval_vector(&self) -> [u8; 6] {
        let pcs: Vec<u8> = self.pcs.iter().copied().collect();
        let mut iv = [0u8; 6];
        for i in 0..pcs.len() {
            for j in (i + 1)..pcs.len() {
                let diff = (pcs[j] + 12 - pcs[i]) % 12;
                let ic = if diff > 6 { 12 - diff } else { diff };
                if (1..=6).contains(&ic) {
                    iv[(ic - 1) as usize] += 1;
                }
            }
        }
        iv
    }

    /// Look up the Forte number for this set class (e.g., "3-11").
    pub fn forte_number(&self) -> Option<String> {
        let pf = self.prime_form();
        forte::lookup(&pf)
    }
}

fn zero_based(nf: &[u8]) -> Vec<u8> {
    if nf.is_empty() {
        return vec![];
    }
    let base = nf[0];
    nf.iter().map(|&pc| (pc + 12 - base) % 12).collect()
}
