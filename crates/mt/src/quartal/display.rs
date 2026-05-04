//! Quartal-perspective rendering helpers — symmetric counterpart to
//! [`crate::quintal::display`].
//!
//! Two perspectives, two stack walks. [`render_quartal_chord_dashed`] walks
//! a [`crate::quintal::PcChord`]'s pcs by [4,5,6]-legal forward intervals;
//! the quintal counterpart [`crate::quintal::render_chord_dashed`] uses
//! [6,7,8]. Both render the same chord, but in different stacking orders.
//!
//! Set-class identity (ascending-pc dashed form) is perspective-invariant;
//! [`render_pcset_dashed`] is re-exported as-is, alongside the spelling
//! helper [`pc_to_note_name`].
