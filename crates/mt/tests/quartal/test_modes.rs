extern crate music_comp_mt as theory;

use theory::quartal::{quartal_orbit_modes, QuartalOrbit};
use theory::quintal::{orbit_modes, Orbit};

// ─── quartal_orbit_modes ────────────────────────────────────────────────

#[test]
fn test_quartal_modes_match_quintal_for_summit() {
    let quintal = orbit_modes(&Orbit::Q777);
    let quartal = quartal_orbit_modes(&QuartalOrbit::Q555);
    assert_eq!(quintal.modes(), quartal.modes());
}

#[test]
fn test_quartal_modes_match_quintal_for_all_orbits() {
    for q_orbit in QuartalOrbit::all() {
        let quintal = orbit_modes(&q_orbit.to_quintal());
        let quartal = quartal_orbit_modes(q_orbit);
        assert_eq!(
            quintal.modes(),
            quartal.modes(),
            "quartal {} should produce same modes as quintal {}",
            q_orbit,
            q_orbit.to_quintal()
        );
    }
}

#[test]
fn test_quartal_modes_distinct_count_matches() {
    for q_orbit in QuartalOrbit::all() {
        let quintal = orbit_modes(&q_orbit.to_quintal());
        let quartal = quartal_orbit_modes(q_orbit);
        assert_eq!(quintal.distinct_count(), quartal.distinct_count());
    }
}
