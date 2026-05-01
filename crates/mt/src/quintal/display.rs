//! Rendering helpers for `PcChord` — note-name strings used by the CLI,
//! the §6 paper formatter, and the upcoming `ai-music-theory` MCP wrapper.
//!
//! Two renderers, deliberately distinct:
//!
//! - [`render_chord_dashed`] — root chord form, ordered by the chord's
//!   `[6, 8]`-legal stack walk. For C-G-D-A this yields `"C–G–D–A"`. Use
//!   when you want the chord-as-quintal-stack identity.
//! - [`render_pcset_dashed`] — pitch-class-set form, ordered by ascending
//!   pc. For the same chord this yields `"C–D–G–A"`. Use when you want
//!   pcset identity (set-class comparison, sorted display).
//!
//! Spelling is sharps-only (matching the existing `pc_to_note_name`
//! convention used throughout the OTH CLI). Enharmonic spelling is a
//! separate concern and out of scope for these helpers.
//!
//! Both renderers accept any `PcChord` whose four pcs admit a `[6, 8]`
//! interval ordering — i.e. every chord in the base space. They will not
//! panic on any other in-space chord.

use super::PcChord;

/// All 24 permutations of `[0, 1, 2, 3]` — used to scan for a `[6, 8]`-legal
/// walk through a chord's four pcs.
const PERMS_4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// Map a pitch class to its sharps-only note name (`0 → "C"`, `1 → "C#"`,
/// …, `11 → "B"`).
///
/// Wraps mod 12, so values ≥ 12 are normalised before lookup. Used by both
/// renderers in this module and by any caller that wants a single
/// note-name string for a single pc.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::pc_to_note_name;
///
/// assert_eq!(pc_to_note_name(0), "C");
/// assert_eq!(pc_to_note_name(11), "B");
/// assert_eq!(pc_to_note_name(13), "C#");  // wraps mod 12
/// ```
#[must_use]
pub fn pc_to_note_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        // Unreachable: `pc % 12` is always in 0..=11. Branch retained for
        // match exhaustiveness; exempt from the coverage gate.
        _ => unreachable!(),
    }
}

/// Render the chord as ascending dashed note names (e.g., `"C–D–G–A"` for
/// pcs `[0, 2, 7, 9]`). Use this when you want the **pitch-class-set**
/// identity — sorted display, set-class comparison, lex-stable output.
///
/// For the chord-as-quintal-stack identity (root form ordered by the
/// chord's `[6, 8]` stacking), use [`render_chord_dashed`] instead.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{render_pcset_dashed, PcChord};
///
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(render_pcset_dashed(&cgda), "C–D–G–A");
/// ```
#[must_use]
pub fn render_pcset_dashed(chord: &PcChord) -> String {
    chord
        .pcs
        .iter()
        .map(|&pc| pc_to_note_name(pc))
        .collect::<Vec<_>>()
        .join("–")
}

/// Render the chord in **root chord form** — the four pcs ordered by the
/// chord's canonical `[6, 8]` stack walk, dashed by en-dash.
///
/// For pcs `[0, 2, 7, 9]` (Q777, four perfect fifths) this yields
/// `"C–G–D–A"`, matching the §6 paper's stack convention. For pcs
/// `[0, 2, 6, 8]` (Q686 — the Saddle) this yields `"C–F#–D–G#"`. For an
/// inverted-recipe chord like pcs `[1, 4, 5, 11]` (Q867, one of the d=7
/// max-σ chords from C-G-D-A) this yields `"E–B–F–C#"` — the renderer
/// uses the chord's own `[6, 8]`-legal walk, not the orbit's canonical
/// representative, so inversions are handled correctly.
///
/// **Algorithm.** Scan all 24 permutations of the chord's four pcs and
/// keep every walk whose three forward intervals all lie in `{6, 7, 8}`
/// (the same legality predicate `BaseSpace` uses; equivalent to walking
/// the chord's own `interval_structure()`). Multiple legal walks can
/// coexist for the same chord — every chord with a non-trivial
/// stabilizer (e.g. Q686, Q676) admits two; chords in the larger orbits
/// admit one. Pick the walk with the **smallest starting pc** as a
/// deterministic tiebreak; if the starting pc still ties, pick the walk
/// with the lexicographically smallest `[p0, p1, p2, p3]` array.
///
/// **Fallback for non-`[6, 8]`-legal chords.** A `PcChord` may be
/// constructed from any four distinct pcs in `0..=11` — including those
/// with no `[6, 8]`-legal stacking (e.g., `PcChord::new([0, 1, 2, 3])`).
/// For such chords this function falls back to ascending-pc rendering,
/// equivalent to [`render_pcset_dashed`]. The function is therefore total
/// over all `PcChord` values and never panics; pass only chords from
/// [`BaseSpace`] if you need root-form output guaranteed.
///
/// For the pitch-class-set identity (ascending dashed note names), use
/// [`render_pcset_dashed`] instead.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quintal::{render_chord_dashed, PcChord};
///
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(render_chord_dashed(&cgda), "C–G–D–A");
///
/// let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
/// assert_eq!(render_chord_dashed(&saddle), "C–F#–D–G#");
/// ```
///
/// [`BaseSpace`]: super::BaseSpace
#[must_use]
pub fn render_chord_dashed(chord: &PcChord) -> String {
    let pcs = chord.pcs;
    let mut legal_walks: Vec<[u8; 4]> = PERMS_4
        .iter()
        .map(|perm| [pcs[perm[0]], pcs[perm[1]], pcs[perm[2]], pcs[perm[3]]])
        .filter(|walk| {
            let i1 = (walk[1] as i16 - walk[0] as i16).rem_euclid(12);
            let i2 = (walk[2] as i16 - walk[1] as i16).rem_euclid(12);
            let i3 = (walk[3] as i16 - walk[2] as i16).rem_euclid(12);
            (6..=8).contains(&i1) && (6..=8).contains(&i2) && (6..=8).contains(&i3)
        })
        .collect();

    // Fallback for non-[6,8]-legal chords (PcChord allows them; BaseSpace
    // does not). Documented above.
    if legal_walks.is_empty() {
        return render_pcset_dashed(chord);
    }

    // Tiebreak: smallest starting pc, then lex-smallest full walk.
    legal_walks.sort();
    let walk = &legal_walks[0];
    walk.iter()
        .map(|&p| pc_to_note_name(p))
        .collect::<Vec<_>>()
        .join("–")
}
