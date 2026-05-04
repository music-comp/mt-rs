//! Quartal-labelled centrality views — symmetric counterpart to
//! [`crate::quintal::centrality`].
//!
//! The mathematical content (Brandes' algorithm, betweenness centrality
//! values) lives in quintal. This module surfaces those values keyed by
//! [`super::orbit::QuartalOrbit`] and [`super::types::QuartalIntervalStructure`]
//! so quartal-minded callers can read centrality in their native vocabulary
//! without quintal-side conversion at every call site.
