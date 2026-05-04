extern crate music_comp_mt as theory;

use theory::quartal::{
    pc_to_note_name, render_pcset_dashed, render_quartal_chord_dashed, render_quartal_is,
    QuartalIntervalStructure,
};
use theory::quintal::{render_chord_dashed as quintal_render_chord_dashed, PcChord};

// ───────────────────────────── render_quartal_chord_dashed ──────────────────

/// Q555 Summit case: pcs `[0, 2, 7, 9]` admits exactly one [4,5,6]-legal walk
/// — `9 → 2 → 7 → 0` with intervals `(5, 5, 5)`. Renders as `"A–D–G–C"`.
#[test]
fn test_render_quartal_chord_q555_summit() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
}

/// Pin BOTH renderers on the same chord so a regression in either one
/// surfaces immediately and the perspective-difference invariant is
/// preserved.
#[test]
fn test_render_quintal_chord_q555_summit_for_comparison() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(quintal_render_chord_dashed(&cgda), "C–G–D–A");
    assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
    assert_ne!(
        quintal_render_chord_dashed(&cgda),
        render_quartal_chord_dashed(&cgda),
        "the two perspectives must produce different walks"
    );
}

/// Q646 Saddle case: pcs `[0, 2, 6, 8]` admits two [4,5,6]-legal walks
/// (the orbit has a non-trivial T6 stabilizer); the smallest-start tiebreak
/// picks `2 → 8 → 0 → 6` with intervals `(6, 4, 6)`. Renders as `"D–G#–C–F#"`.
#[test]
fn test_render_quartal_chord_q646_saddle() {
    let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&saddle), "D–G#–C–F#");
}

/// ASYMMETRIC Q554 case: pcs `[0, 3, 8, 10]` admits one [4,5,6]-legal walk
/// — `10 → 3 → 8 → 0` with intervals `(5, 5, 4)`. Renders as `"A#–D#–G#–C"`.
/// This is the regression test for any future predicate-direction bug:
/// a buggy implementation that reverses the predicate would produce the
/// reversed-and-complemented walk and fail this assertion.
#[test]
fn test_render_quartal_chord_q554_asymmetric() {
    let q554 = PcChord::new([0, 3, 8, 10]).unwrap();
    assert_eq!(render_quartal_chord_dashed(&q554), "A#–D#–G#–C");
}

/// Non-[4,5,6]-legal chord falls back to ascending-pc rendering, equivalent
/// to `render_pcset_dashed`. Pins the documented total-function behaviour.
#[test]
fn test_render_quartal_chord_falls_back_for_non_legal_chord() {
    let bogus = PcChord::new([0, 1, 2, 3]).unwrap();
    let walk = render_quartal_chord_dashed(&bogus);
    let pcset = render_pcset_dashed(&bogus);
    assert_eq!(walk, pcset);
    assert_eq!(walk, "C–C#–D–D#");
}

// ─────────────────────────── render_pcset_dashed re-export ──────────────────

/// `render_pcset_dashed` is reachable via `crate::quartal::*` and behaves
/// identically to the quintal counterpart (perspective-invariant).
#[test]
fn test_render_quartal_pcset_re_export() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_pcset_dashed(&cgda), "C–D–G–A");
}

// ─────────────────────────── pc_to_note_name re-export ──────────────────────

#[test]
fn test_pc_to_note_name_re_export() {
    assert_eq!(pc_to_note_name(0), "C");
    assert_eq!(pc_to_note_name(8), "G#");
    // mod-12 wrap: pc=13 normalises to pc=1 → "C#"
    assert_eq!(pc_to_note_name(13), "C#");
}

// ─────────────────────────── render_quartal_is ──────────────────────────────

#[test]
fn test_render_quartal_is_summit() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(5, 5, 5)),
        "5–5–5"
    );
}

#[test]
fn test_render_quartal_is_saddle() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(6, 4, 6)),
        "6–4–6"
    );
}

/// ASYMMETRIC IS (5, 5, 4) — pins component-ordering invariance: the
/// renderer must NOT sort or rearrange components.
#[test]
fn test_render_quartal_is_asymmetric() {
    assert_eq!(
        render_quartal_is(&QuartalIntervalStructure(5, 5, 4)),
        "5–5–4"
    );
}
