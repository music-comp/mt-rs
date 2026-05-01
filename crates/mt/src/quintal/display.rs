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
/// Mirrors the CLI's existing helper of the same name; kept private here
/// so the library renderers don't depend on the CLI crate.
fn pc_to_note_name(pc: u8) -> &'static str {
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
/// `[0, 2, 6, 8]` (Q686 — the Saddle) this yields `"C–F#–D–G#"`.
///
/// **Algorithm.** Scan all 24 permutations of the chord's four pcs and
/// keep every walk whose three forward intervals all lie in `{6, 7, 8}`
/// (the same legality predicate `BaseSpace` uses). Multiple legal walks
/// can coexist for the same chord — every chord with a non-trivial
/// stabilizer (e.g. Q686, Q676) admits two; chords in the larger orbits
/// admit one. Pick the walk with the **smallest starting pc** as a
/// deterministic tiebreak; if the starting pc still ties, pick the walk
/// with the lexicographically smallest `[p0, p1, p2, p3]` array.
///
/// For the pitch-class-set identity (ascending dashed note names), use
/// [`render_pcset_dashed`] instead.
///
/// # Panics
///
/// Panics only if `chord` admits no `[6, 8]`-legal walk, which is
/// impossible for any chord in the quintal base space. Calls outside of
/// `B` are out of scope for this helper.
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

    // Tiebreak: smallest starting pc, then lex-smallest full walk.
    legal_walks.sort();
    let walk = legal_walks
        .first()
        .expect("chord must admit at least one [6,8]-legal walk");

    walk.iter()
        .map(|&p| pc_to_note_name(p))
        .collect::<Vec<_>>()
        .join("–")
}
