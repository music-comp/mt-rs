//! Quartal-perspective functional grammar — symmetric counterpart to
//! [`crate::quintal::functional`].
//!
//! The seven [`FunctionalRegion`] variants and two [`Pathway`] variants are
//! perspective-invariant — they describe the *topographic role* of an
//! orbit (Summit, Saddle, Slope, …), not its interval structure. Both
//! types are re-exported as-is from quintal. This module adds quartal-side
//! ergonomics: [`super::orbit::QuartalOrbit::functional_region`] as an
//! extension method, and two free functions returning quartal-labelled
//! orbit lists.
//!
//! Region-level queries (e.g. "which quartal orbits live on the Slope?")
//! and pathway-level queries (e.g. "what is the quartal stop list for the
//! Cadence pathway?") are the primary use cases the walkthrough documents
//! will exercise.
