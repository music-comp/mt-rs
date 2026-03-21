extern crate rust_music_theory as theory;
use theory::set_class::PitchClassSet;

#[cfg(test)]
mod set_class_tests {
    use super::*;

    #[test]
    fn test_create_from_slice() {
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        assert_eq!(pcs.len(), 3);
        assert!(pcs.contains(0));
        assert!(pcs.contains(4));
        assert!(pcs.contains(7));
    }

    #[test]
    fn test_wraps_mod_12() {
        let pcs = PitchClassSet::new(&[13, 16, 19]);
        assert_eq!(pcs.len(), 3);
        assert!(pcs.contains(1));
        assert!(pcs.contains(4));
        assert!(pcs.contains(7));
    }

    #[test]
    fn test_deduplicates() {
        let pcs = PitchClassSet::new(&[0, 0, 4, 7, 7]);
        assert_eq!(pcs.len(), 3);
    }

    #[test]
    fn test_transpose() {
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        let t2 = pcs.transpose(2);
        assert!(t2.contains(2));
        assert!(t2.contains(6));
        assert!(t2.contains(9));
    }

    #[test]
    fn test_transpose_wraps() {
        let pcs = PitchClassSet::new(&[10, 11]);
        let t3 = pcs.transpose(3);
        assert!(t3.contains(1));
        assert!(t3.contains(2));
    }

    #[test]
    fn test_invert_i0() {
        // I_0 of {0,4,7} = {0, 12-4, 12-7} = {0, 8, 5}
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        let i0 = pcs.invert(0);
        assert!(i0.contains(0));
        assert!(i0.contains(5));
        assert!(i0.contains(8));
    }

    #[test]
    fn test_normal_form_already_compact() {
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        assert_eq!(pcs.normal_form(), vec![0, 4, 7]);
    }

    #[test]
    fn test_normal_form_reorders() {
        let pcs = PitchClassSet::new(&[7, 0, 4]);
        assert_eq!(pcs.normal_form(), vec![0, 4, 7]);
    }

    #[test]
    fn test_normal_form_chromatic_cluster() {
        let pcs = PitchClassSet::new(&[11, 0, 1]);
        assert_eq!(pcs.normal_form(), vec![11, 0, 1]);
    }

    #[test]
    fn test_prime_form_major_triad() {
        // Major triad {0,4,7}: prime form = [0,3,7]
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        assert_eq!(pcs.prime_form(), vec![0, 3, 7]);
    }

    #[test]
    fn test_prime_form_minor_triad() {
        let pcs = PitchClassSet::new(&[0, 3, 7]);
        assert_eq!(pcs.prime_form(), vec![0, 3, 7]);
    }

    #[test]
    fn test_prime_form_transposition_invariant() {
        // D major {2,6,9} should have same prime form as C major {0,4,7}
        let c = PitchClassSet::new(&[0, 4, 7]);
        let d = PitchClassSet::new(&[2, 6, 9]);
        assert_eq!(c.prime_form(), d.prime_form());
    }

    #[test]
    fn test_interval_vector_major_triad() {
        // {0,4,7}: ic3=1 (0-4→4→ic4... wait)
        // 0-4=4→ic4, 0-7=7→ic5, 4-7=3→ic3
        // iv = [0, 0, 1, 1, 1, 0]
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        assert_eq!(pcs.interval_vector(), [0, 0, 1, 1, 1, 0]);
    }

    #[test]
    fn test_interval_vector_chromatic_trichord() {
        // {0,1,2}: 0-1=1→ic1, 0-2=2→ic2, 1-2=1→ic1
        // iv = [2, 1, 0, 0, 0, 0]
        let pcs = PitchClassSet::new(&[0, 1, 2]);
        assert_eq!(pcs.interval_vector(), [2, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn test_interval_vector_tritone_dyad() {
        // {0,6}: ic6=1
        let pcs = PitchClassSet::new(&[0, 6]);
        assert_eq!(pcs.interval_vector(), [0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_forte_number_major_triad() {
        let pcs = PitchClassSet::new(&[0, 4, 7]);
        assert_eq!(pcs.forte_number(), Some("3-11".to_string()));
    }

    #[test]
    fn test_forte_number_diminished_triad() {
        let pcs = PitchClassSet::new(&[0, 3, 6]);
        assert_eq!(pcs.forte_number(), Some("3-10".to_string()));
    }

    #[test]
    fn test_forte_number_augmented_triad() {
        let pcs = PitchClassSet::new(&[0, 4, 8]);
        assert_eq!(pcs.forte_number(), Some("3-12".to_string()));
    }

    #[test]
    fn test_forte_number_chromatic_trichord() {
        let pcs = PitchClassSet::new(&[0, 1, 2]);
        assert_eq!(pcs.forte_number(), Some("3-1".to_string()));
    }

    #[test]
    fn test_forte_number_dim7_tetrachord() {
        // {0,3,6,9} = 4-28
        let pcs = PitchClassSet::new(&[0, 3, 6, 9]);
        assert_eq!(pcs.forte_number(), Some("4-28".to_string()));
    }

    #[test]
    fn test_forte_number_transposed_set() {
        // D major {2,6,9} should get same Forte number as C major
        let pcs = PitchClassSet::new(&[2, 6, 9]);
        assert_eq!(pcs.forte_number(), Some("3-11".to_string()));
    }

    #[test]
    fn test_empty_set() {
        let pcs = PitchClassSet::new(&[]);
        assert!(pcs.is_empty());
        assert_eq!(pcs.normal_form(), vec![]);
        assert_eq!(pcs.prime_form(), vec![]);
        assert_eq!(pcs.forte_number(), None);
    }
}
