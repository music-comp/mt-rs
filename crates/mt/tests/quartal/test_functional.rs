extern crate music_comp_mt as theory;

use theory::quartal::{
    quartal_orbits_in, quartal_pathway_stops, FunctionalRegion, Pathway, QuartalOrbit,
};

// ────────────────────────── QuartalOrbit::functional_region ─────────────────

/// Table-driven: every QuartalOrbit variant maps to the expected
/// FunctionalRegion. Includes asymmetric orbits (Q554, Q655, Q654, Q564,
/// Q645, Q445, Q446, Q565, Q464, Q454) — analog of the asymmetric-chord
/// regression discipline established in T0/T1.
#[test]
fn test_all_14_quartal_orbits_map_to_correct_region() {
    use FunctionalRegion::*;
    let expected: [(QuartalOrbit, FunctionalRegion); 14] = [
        (QuartalOrbit::Q555, Summit),
        // Plateau (palindromic + asymmetric):
        (QuartalOrbit::Q545, Plateau),
        (QuartalOrbit::Q554, Plateau),
        // Slope (all asymmetric):
        (QuartalOrbit::Q655, Slope),
        (QuartalOrbit::Q564, Slope),
        (QuartalOrbit::Q654, Slope),
        (QuartalOrbit::Q645, Slope),
        // Valley (palindromic):
        (QuartalOrbit::Q565, Valley),
        (QuartalOrbit::Q454, Valley),
        (QuartalOrbit::Q464, Valley),
        // Saddle:
        (QuartalOrbit::Q646, Saddle),
        // Precipice (asymmetric):
        (QuartalOrbit::Q445, Precipice),
        (QuartalOrbit::Q446, Precipice),
        // Narrows:
        (QuartalOrbit::Q656, Narrows),
    ];
    assert_eq!(expected.len(), 14);
    for (orbit, region) in expected {
        assert_eq!(
            orbit.functional_region(),
            region,
            "{:?} should map to {:?}",
            orbit,
            region
        );
    }
}

// ────────────────────────── quartal_orbits_in ───────────────────────────────

#[test]
fn test_quartal_orbits_in_summit() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Summit),
        vec![QuartalOrbit::Q555]
    );
}

#[test]
fn test_quartal_orbits_in_plateau() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Plateau),
        vec![QuartalOrbit::Q545, QuartalOrbit::Q554]
    );
}

#[test]
fn test_quartal_orbits_in_slope() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Slope),
        vec![
            QuartalOrbit::Q655,
            QuartalOrbit::Q564,
            QuartalOrbit::Q654,
            QuartalOrbit::Q645,
        ]
    );
}

#[test]
fn test_quartal_orbits_in_valley() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Valley),
        vec![QuartalOrbit::Q565, QuartalOrbit::Q454, QuartalOrbit::Q464]
    );
}

#[test]
fn test_quartal_orbits_in_saddle() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Saddle),
        vec![QuartalOrbit::Q646]
    );
}

#[test]
fn test_quartal_orbits_in_precipice() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Precipice),
        vec![QuartalOrbit::Q445, QuartalOrbit::Q446]
    );
}

#[test]
fn test_quartal_orbits_in_narrows() {
    assert_eq!(
        quartal_orbits_in(FunctionalRegion::Narrows),
        vec![QuartalOrbit::Q656]
    );
}

#[test]
fn test_quartal_orbits_in_partition_is_complete() {
    // Across all 7 regions, every QuartalOrbit variant must appear exactly
    // once; total cardinality is 14 (the size of QuartalOrbit::all()).
    use std::collections::BTreeSet;
    let regions = [
        FunctionalRegion::Summit,
        FunctionalRegion::Plateau,
        FunctionalRegion::Slope,
        FunctionalRegion::Valley,
        FunctionalRegion::Saddle,
        FunctionalRegion::Precipice,
        FunctionalRegion::Narrows,
    ];
    let mut seen: BTreeSet<QuartalOrbit> = BTreeSet::new();
    let mut total = 0usize;
    for region in regions {
        let orbits = quartal_orbits_in(region);
        total += orbits.len();
        for orbit in orbits {
            assert!(
                seen.insert(orbit),
                "{:?} appears in more than one region",
                orbit
            );
        }
    }
    assert_eq!(
        total, 14,
        "total orbit count across all regions should be 14"
    );
    assert_eq!(
        seen.len(),
        14,
        "every QuartalOrbit variant should appear exactly once"
    );
    for variant in QuartalOrbit::all() {
        assert!(
            seen.contains(variant),
            "QuartalOrbit::{:?} missing from partition",
            variant
        );
    }
}

// ────────────────────────── quartal_pathway_stops ───────────────────────────

#[test]
fn test_quartal_pathway_stops_cadence() {
    let stops = quartal_pathway_stops(Pathway::Cadence);
    assert_eq!(stops.len(), 3);
    assert_eq!(stops[0].0, FunctionalRegion::Saddle);
    assert_eq!(stops[0].1, vec![QuartalOrbit::Q646]);
    assert_eq!(stops[1].0, FunctionalRegion::Slope);
    assert_eq!(stops[1].1.len(), 4);
    assert_eq!(stops[2].0, FunctionalRegion::Summit);
    assert_eq!(stops[2].1, vec![QuartalOrbit::Q555]);
}

#[test]
fn test_quartal_pathway_stops_departure() {
    let stops = quartal_pathway_stops(Pathway::Departure);
    assert_eq!(stops.len(), 4);
    assert_eq!(stops[0].0, FunctionalRegion::Summit);
    assert_eq!(stops[0].1, vec![QuartalOrbit::Q555]);
    assert_eq!(stops[1].0, FunctionalRegion::Plateau);
    assert_eq!(stops[1].1, vec![QuartalOrbit::Q545, QuartalOrbit::Q554]);
    assert_eq!(stops[2].0, FunctionalRegion::Slope);
    assert_eq!(stops[2].1.len(), 4);
    assert_eq!(stops[3].0, FunctionalRegion::Saddle);
    assert_eq!(stops[3].1, vec![QuartalOrbit::Q646]);
}

#[test]
fn test_quartal_pathway_stops_match_region_sequence() {
    // For every canonical pathway: stop count matches region_sequence; each
    // stop's region matches; each stop's orbit list equals quartal_orbits_in.
    for &pathway in Pathway::all() {
        let stops = quartal_pathway_stops(pathway);
        let regions = pathway.region_sequence();
        assert_eq!(
            stops.len(),
            regions.len(),
            "stop count for {:?} must match region_sequence length",
            pathway
        );
        for (i, &expected_region) in regions.iter().enumerate() {
            assert_eq!(
                stops[i].0, expected_region,
                "stop {} region for {:?} mismatch",
                i, pathway
            );
            assert_eq!(
                stops[i].1,
                quartal_orbits_in(expected_region),
                "stop {} orbits for {:?} must equal quartal_orbits_in({:?})",
                i,
                pathway,
                expected_region
            );
        }
    }
}

// ────────────────────────── Re-export reachability ──────────────────────────

#[test]
fn test_functional_region_re_export() {
    // Reachable via crate::quartal::*; Display impl renders the variant name.
    let region = FunctionalRegion::Summit;
    assert_eq!(format!("{}", region), "Summit");
}

#[test]
fn test_pathway_re_export() {
    let pathway = Pathway::Cadence;
    assert_eq!(format!("{}", pathway), "Cadence");
}
