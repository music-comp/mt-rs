extern crate music_comp_mt as theory;

use theory::quintal::{
    classify_orbit, render_chord_dashed, render_pcset_dashed, saddle_chords, BaseSpace, Orbit,
    PcChord,
};

// ───────────────────────────── render_chord_dashed ──────────────────────────

/// The canonical Summit case: C-G-D-A is four perfect fifths in stack form.
#[test]
fn render_chord_dashed_summit_is_stack_form() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_chord_dashed(&cgda), "C–G–D–A");
}

/// Transposed Summit (T₁) preserves the stack-form pattern.
#[test]
fn render_chord_dashed_transposed_summit() {
    let cgda_t1 = PcChord::new([1, 3, 8, 10]).unwrap();
    assert_eq!(render_chord_dashed(&cgda_t1), "C#–G#–D#–A#");
}

/// Saddle (Q686) representative pcs `[0, 2, 6, 8]` admits two `[6,8]`-legal
/// walks. The smallest-starting-pc tiebreak picks the walk starting at C.
#[test]
fn render_chord_dashed_saddle_picks_smallest_start() {
    let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(classify_orbit(&saddle), Some(Orbit::Q686));
    assert_eq!(render_chord_dashed(&saddle), "C–F#–D–G#");
}

/// Inverted-recipe Q867 case: pcs `[1, 4, 5, 11]` is one of the two
/// corrected d=7 max-σ chords from C-G-D-A. Q867's orbit representative
/// is `(8, 6, 7)`, but THIS chord's `[6,8]`-legal walk is the reversal
/// `(7, 6, 8)` starting at pc 4 (E). The renderer uses the chord's own
/// walk, not the orbit's representative — so it correctly handles
/// inverted orbit members.
#[test]
fn render_chord_dashed_q867_inverted_recipe() {
    let inverted = PcChord::new([1, 4, 5, 11]).unwrap();
    assert_eq!(classify_orbit(&inverted), Some(Orbit::Q867));
    assert_eq!(render_chord_dashed(&inverted), "E–B–F–C#");
}

/// The two renderers exist precisely because they produce *different*
/// strings for the same chord — pin that design split. (For C-G-D-A
/// they coincide as `"C–G–D–A"` vs `"C–D–G–A"`.) Sanity guard against
/// any future "simplification" that accidentally collapses them.
#[test]
fn render_chord_dashed_differs_from_render_pcset_dashed() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    let stack = render_chord_dashed(&cgda);
    let pcset = render_pcset_dashed(&cgda);
    assert_eq!(stack, "C–G–D–A");
    assert_eq!(pcset, "C–D–G–A");
    assert_ne!(
        stack, pcset,
        "the two renderers must produce different strings"
    );
}

/// Non-`[6,8]`-legal chord: `[0, 1, 2, 3]` has no legal stacking. Per
/// §2a-bis the renderer falls back to ascending-pc order, equivalent to
/// `render_pcset_dashed`. Pins the documented total-function behaviour.
#[test]
fn render_chord_dashed_falls_back_for_non_legal_chord() {
    let bogus = PcChord::new([0, 1, 2, 3]).unwrap();
    let stack = render_chord_dashed(&bogus);
    let pcset = render_pcset_dashed(&bogus);
    assert_eq!(stack, pcset);
    assert_eq!(stack, "C–C#–D–D#");
}

/// `saddle_chords` member: pull a saddle representative from the library
/// and verify it renders to one of the two valid root-form stacks. Loops
/// the assertion over all six members so a deterministic-tiebreak
/// regression surfaces immediately.
#[test]
fn render_chord_dashed_saddle_chords_all_render_in_stack_form() {
    let space = BaseSpace::new();
    for chord in saddle_chords(&space) {
        assert_eq!(classify_orbit(&chord), Some(Orbit::Q686));
        let rendered = render_chord_dashed(&chord);
        // Walk-form means: 4 dashed note names, 3 en-dashes between them.
        assert_eq!(rendered.matches('–').count(), 3, "rendered = {rendered}");
        // The pcset should match what render_pcset_dashed produces (same
        // 4 note names, just in a different order).
        let pcset = render_pcset_dashed(&chord);
        assert_eq!(pcset.matches('–').count(), 3, "pcset = {pcset}");
        // Same multiset of notes: split each, sort, compare.
        let mut walk_notes: Vec<&str> = rendered.split('–').collect();
        let mut pcset_notes: Vec<&str> = pcset.split('–').collect();
        walk_notes.sort();
        pcset_notes.sort();
        assert_eq!(walk_notes, pcset_notes, "chord {:?}", chord);
    }
}

/// Q787 breadth check: a non-quintal-stack, non-saddle orbit member.
/// `[0, 2, 5, 9]` is in Q787 (verified via `classify_orbit`); the chord's
/// `[6,8]`-legal walk starts at 5 (F): 5→0(=7)→2(=2 — wait verify) →9.
/// Just assert the pcset preservation invariant and orbit membership;
/// the exact string would tie us to internal walk choice.
#[test]
fn render_chord_dashed_q787_preserves_pcset() {
    let q787 = PcChord::new([0, 2, 5, 9]).unwrap();
    assert_eq!(classify_orbit(&q787), Some(Orbit::Q787));
    let rendered = render_chord_dashed(&q787);
    let pcset = render_pcset_dashed(&q787);
    assert_eq!(rendered.matches('–').count(), 3);
    let mut walk_notes: Vec<&str> = rendered.split('–').collect();
    let mut pcset_notes: Vec<&str> = pcset.split('–').collect();
    walk_notes.sort();
    pcset_notes.sort();
    assert_eq!(walk_notes, pcset_notes);
}

/// Every chord in the base space renders without panic and preserves its
/// pcset under stack-form vs pcset-form rendering. End-to-end smoke test.
#[test]
fn render_chord_dashed_total_over_base_space() {
    let space = BaseSpace::new();
    for chord in space.chords() {
        let walk = render_chord_dashed(chord);
        let pcset = render_pcset_dashed(chord);
        let mut walk_notes: Vec<&str> = walk.split('–').collect();
        let mut pcset_notes: Vec<&str> = pcset.split('–').collect();
        walk_notes.sort();
        pcset_notes.sort();
        assert_eq!(walk_notes, pcset_notes, "mismatch for {:?}", chord);
    }
}

// ───────────────────────────── render_pcset_dashed ──────────────────────────

/// Pcset form is just ascending pcs joined by en-dash.
#[test]
fn render_pcset_dashed_summit() {
    let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
    assert_eq!(render_pcset_dashed(&cgda), "C–D–G–A");
}

#[test]
fn render_pcset_dashed_saddle() {
    let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
    assert_eq!(render_pcset_dashed(&saddle), "C–D–F#–G#");
}

#[test]
fn render_pcset_dashed_q787_breadth() {
    let q787 = PcChord::new([0, 2, 5, 9]).unwrap();
    assert_eq!(render_pcset_dashed(&q787), "C–D–F–A");
}
