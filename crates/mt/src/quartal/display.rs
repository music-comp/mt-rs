//! Quartal-perspective rendering helpers — symmetric counterpart to
//! [`crate::quintal::display`].
//!
//! Two perspectives, two stack walks. [`render_quartal_chord_dashed`] walks
//! a [`PcChord`]'s pcs by [4,5,6]-legal forward intervals; the quintal
//! counterpart [`crate::quintal::render_chord_dashed`] uses [6,7,8]. Both
//! render the same chord, but in different stacking orders.
//!
//! Set-class identity (ascending-pc dashed form) is perspective-invariant;
//! [`render_pcset_dashed`] is re-exported as-is, alongside the spelling
//! helper [`pc_to_note_name`].

use crate::quintal::PcChord;

use super::types::QuartalIntervalStructure;

// Re-exports of perspective-invariant utilities.

/// Spell a pitch class as a sharps-only note name.
///
/// Re-exported from [`crate::quintal::pc_to_note_name`].
pub use crate::quintal::pc_to_note_name;

/// Render a chord as ascending dashed note names.
///
/// Re-exported from [`crate::quintal::render_pcset_dashed`].
/// The pitch-class-set form is perspective-invariant — both quartal and
/// quintal callers see the same string for the same chord.
pub use crate::quintal::render_pcset_dashed;

/// All 24 permutations of `[0, 1, 2, 3]` — used to scan for a `[4, 6]`-legal
/// walk through a chord's four pcs.
///
/// **Justified duplication (D-quartal-T1-001):** a verbatim copy of the
/// `PERMS_4` constant in [`crate::quintal::display`] (which is private).
/// Extracting a shared helper would touch quintal and is out of T1 scope.
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

/// Render a [`PcChord`] in **quartal root form** — the four pcs ordered by
/// the chord's canonical `[4, 6]` stack walk, dashed by en-dash.
///
/// For pcs `[0, 2, 7, 9]` (Q777 / Q555) this yields `"A–D–G–C"` — the same
/// chord that [`crate::quintal::render_chord_dashed`] renders as
/// `"C–G–D–A"`, read here as a stack of fourths bottom-up. For pcs
/// `[0, 2, 6, 8]` (Q686 / Q646 Saddle) this yields `"D–G#–C–F#"`.
///
/// # Algorithm
///
/// Identical in shape to [`crate::quintal::render_chord_dashed`] but with
/// the legal-interval predicate `(4..=6)` instead of `(6..=8)`. Scans all
/// 24 permutations of the chord's four pcs, retains those whose three
/// forward intervals are all in `{4, 5, 6}`, and returns the walk with
/// the smallest starting pc (lex tiebreak on the full `[p0, p1, p2, p3]`
/// array). Orbits with non-trivial stabilizers (e.g. Q646, Q656) admit
/// two legal walks; the smallest-start tiebreak makes the choice
/// deterministic.
///
/// # Fallback
///
/// For chords with no `[4, 6]`-legal walk (possible only for `PcChord`
/// values constructed outside [`crate::quintal::BaseSpace`]), falls back
/// to ascending-pc rendering — equivalent to [`render_pcset_dashed`].
/// Total over all `PcChord` values; never panics.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::render_quartal_chord_dashed;
/// use music_comp_mt::quintal::PcChord;
///
/// let cgda = PcChord::new([0, 2, 7, 9]).unwrap();
/// assert_eq!(render_quartal_chord_dashed(&cgda), "A–D–G–C");
///
/// let saddle = PcChord::new([0, 2, 6, 8]).unwrap();
/// assert_eq!(render_quartal_chord_dashed(&saddle), "D–G#–C–F#");
/// ```
#[must_use]
pub fn render_quartal_chord_dashed(chord: &PcChord) -> String {
    let pcs = chord.pcs;
    let mut legal_walks: Vec<[u8; 4]> = PERMS_4
        .iter()
        .map(|perm| [pcs[perm[0]], pcs[perm[1]], pcs[perm[2]], pcs[perm[3]]])
        .filter(|walk| {
            let i1 = (walk[1] as i16 - walk[0] as i16).rem_euclid(12);
            let i2 = (walk[2] as i16 - walk[1] as i16).rem_euclid(12);
            let i3 = (walk[3] as i16 - walk[2] as i16).rem_euclid(12);
            (4..=6).contains(&i1) && (4..=6).contains(&i2) && (4..=6).contains(&i3)
        })
        .collect();

    // Fallback for non-[4,6]-legal chords (PcChord allows them; BaseSpace
    // does not). Documented above; mirrors the quintal-side fallback.
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

/// Render a [`QuartalIntervalStructure`] as dashed semitone counts.
///
/// `(5, 5, 5) → "5–5–5"`, `(6, 4, 6) → "6–4–6"`, `(5, 5, 4) → "5–5–4"`.
/// Used in narrative documents and table outputs where the quartal IS
/// should appear as a single readable cell.
///
/// # Examples
///
/// ```
/// use music_comp_mt::quartal::{render_quartal_is, QuartalIntervalStructure};
///
/// assert_eq!(
///     render_quartal_is(&QuartalIntervalStructure(5, 5, 5)),
///     "5–5–5"
/// );
/// assert_eq!(
///     render_quartal_is(&QuartalIntervalStructure(5, 5, 4)),
///     "5–5–4"
/// );
/// ```
#[must_use]
pub fn render_quartal_is(is: &QuartalIntervalStructure) -> String {
    format!("{}–{}–{}", is.0, is.1, is.2)
}
