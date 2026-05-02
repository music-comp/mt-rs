use crate::quintal::VoicedChord;
use crate::voice_leading::min_voiced_chord_l1;

use super::Harmonization;

#[derive(Clone)]
struct PathState {
    cost: u32,
    prev_candidate: Option<usize>,
    prev_path: Option<usize>,
}

pub(crate) fn top_k_viterbi(layers: &[Vec<VoicedChord>], k: usize) -> Vec<Harmonization> {
    if layers.is_empty() || k == 0 {
        return Vec::new();
    }

    let n = layers.len();
    let mut dp: Vec<Vec<Vec<PathState>>> = Vec::with_capacity(n);

    let first_layer: Vec<Vec<PathState>> = layers[0]
        .iter()
        .map(|_| {
            vec![PathState {
                cost: 0,
                prev_candidate: None,
                prev_path: None,
            }]
        })
        .collect();
    dp.push(first_layer);

    for layer_idx in 1..n {
        let mut layer_dp: Vec<Vec<PathState>> = Vec::with_capacity(layers[layer_idx].len());

        for c_new in layers[layer_idx].iter() {
            let mut incoming: Vec<PathState> = Vec::new();

            for (c_prev_idx, c_prev) in layers[layer_idx - 1].iter().enumerate() {
                let edge_cost = min_voiced_chord_l1(c_prev, c_new);

                for (p_idx, p_state) in dp[layer_idx - 1][c_prev_idx].iter().enumerate() {
                    incoming.push(PathState {
                        cost: p_state.cost + edge_cost,
                        prev_candidate: Some(c_prev_idx),
                        prev_path: Some(p_idx),
                    });
                }
            }

            incoming.sort_by_key(|ps| ps.cost);
            incoming.truncate(k);
            layer_dp.push(incoming);
        }

        dp.push(layer_dp);
    }

    let last = n - 1;
    let mut finals: Vec<(u32, usize, usize)> = Vec::new();
    for (c_idx, paths) in dp[last].iter().enumerate() {
        for (p_idx, ps) in paths.iter().enumerate() {
            finals.push((ps.cost, c_idx, p_idx));
        }
    }
    finals.sort_by_key(|&(cost, _, _)| cost);
    finals.truncate(k);

    finals
        .iter()
        .map(|&(total_cost, mut c_idx, mut p_idx)| {
            let mut chord_indices: Vec<usize> = Vec::with_capacity(n);
            chord_indices.push(c_idx);

            for layer_idx in (1..n).rev() {
                let state = &dp[layer_idx][c_idx][p_idx];
                c_idx = state.prev_candidate.unwrap();
                p_idx = state.prev_path.unwrap();
                chord_indices.push(c_idx);
            }
            chord_indices.reverse();

            let chords: Vec<VoicedChord> = chord_indices
                .iter()
                .enumerate()
                .map(|(layer, &ci)| layers[layer][ci])
                .collect();

            let per_step_movements: Vec<u32> = chords
                .windows(2)
                .map(|w| min_voiced_chord_l1(&w[0], &w[1]))
                .collect();

            Harmonization {
                chords,
                per_step_movements,
                total_movement: total_cost,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_layer_single_candidate() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let layers = vec![vec![c]];
        let results = top_k_viterbi(&layers, 1);
        eprintln!("single_layer_single: {:?}", results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chords, vec![c]);
        assert_eq!(results[0].per_step_movements, Vec::<u32>::new());
        assert_eq!(results[0].total_movement, 0);
    }

    #[test]
    fn test_two_layers_cheapest_path() {
        let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let b = VoicedChord::new([49, 56, 63, 70]).unwrap();
        let c = VoicedChord::new([60, 67, 74, 81]).unwrap();

        // Layer 0: [a, b]. Layer 1: [c].
        // Edge a→c = min_voiced_chord_l1(a, c)
        // Edge b→c = min_voiced_chord_l1(b, c)
        let ac = min_voiced_chord_l1(&a, &c);
        let bc = min_voiced_chord_l1(&b, &c);
        eprintln!("edges: a→c={}, b→c={}", ac, bc);

        let layers = vec![vec![a, b], vec![c]];
        let results = top_k_viterbi(&layers, 2);
        eprintln!("two_layers results: {:?}", results.iter().map(|r| r.total_movement).collect::<Vec<_>>());

        assert_eq!(results.len(), 2);
        assert!(results[0].total_movement <= results[1].total_movement);
        assert_eq!(results[0].total_movement, ac.min(bc));
    }

    #[test]
    fn test_same_candidates_zero_movement() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        // Two layers, same single candidate in each
        let layers = vec![vec![c], vec![c]];
        let results = top_k_viterbi(&layers, 5);
        eprintln!("same_candidate: {:?}", results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_movement, 0);
        assert_eq!(results[0].chords, vec![c, c]);
    }

    #[test]
    fn test_empty_layers_returns_empty() {
        let results = top_k_viterbi(&[], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_k_zero_returns_empty() {
        let c = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let results = top_k_viterbi(&[vec![c]], 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_results_sorted_ascending() {
        let a = VoicedChord::new([48, 55, 62, 69]).unwrap();
        let b = VoicedChord::new([49, 56, 63, 70]).unwrap();
        let c = VoicedChord::new([50, 57, 64, 71]).unwrap();
        let layers = vec![vec![a, b, c], vec![a, b, c], vec![a, b, c]];
        let results = top_k_viterbi(&layers, 10);
        eprintln!(
            "sorted_check: {:?}",
            results.iter().map(|r| r.total_movement).collect::<Vec<_>>()
        );
        for w in results.windows(2) {
            assert!(w[0].total_movement <= w[1].total_movement);
        }
    }
}
