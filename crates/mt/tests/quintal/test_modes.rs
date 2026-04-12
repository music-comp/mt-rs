extern crate music_comp_mt as theory;

use theory::quintal::{Orbit, OthMode};

// ─── OthMode construction and accessors ─────────────────────────────────

#[test]
fn test_oth_mode_new_and_accessors() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    assert_eq!(mode.orbit(), Orbit::Q777);
    assert_eq!(mode.rotation(), 0);
    assert_eq!(mode.steps(), [2, 5, 2, 3]);
    assert_eq!(mode.pcs_from_c(), [0, 2, 7, 9]);
    assert_eq!(
        mode.opening_interval(),
        2,
        "opening_interval should be steps[0]"
    );
}

#[test]
fn test_oth_mode_opening_interval_derived_from_steps() {
    let mode = OthMode::new(Orbit::Q686, 1, [4, 2, 4, 2], [0, 4, 6, 10]);
    assert_eq!(mode.opening_interval(), 4);
}

#[test]
fn test_oth_mode_is_copy() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let copy = mode;
    assert_eq!(mode, copy);
}

#[test]
fn test_oth_mode_display() {
    let mode = OthMode::new(Orbit::Q777, 0, [2, 5, 2, 3], [0, 2, 7, 9]);
    let s = format!("{}", mode);
    assert!(s.contains("M1"), "Display should include M1 for rotation 0");
}
