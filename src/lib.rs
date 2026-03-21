//! `Rust Music Theory` is a library that provides programmatic implementation of the basis of music theory.
//!
//! ## About
//!
//! `Rust Music Theory` is used to procedurally utilize music theory notions like Note, Chord, Scale,
//! Interval, Key and more. All these theoretical concepts of sound and music are implemented as
//! separate modules in the library. They sometimes can be used individually, and sometimes need
//! to be used together to correctly reflect the music theory to the code.
//!
//! ## Quick Example
//!
//! ```no_run
//! extern crate mt_rs as rustmt;
//! use rustmt::note::{Note, Notes, Pitch, PitchSymbol::*};
//! use rustmt::scale::{Direction, Scale, ScaleType, Mode};
//! use rustmt::chord::{Chord, Number as ChordNumber, Quality as ChordQuality};
//!
//! // to create a Note, specify a pitch class and an octave;
//! let note = Note::new(Pitch::from(As), 4);
//!
//! // Scale Example
//! let scale = Scale::new(
//!     ScaleType::Diatonic,
//!     Pitch::from(C),
//!     4,
//!     Some(Mode::Ionian),
//!     Direction::Ascending,
//! ).unwrap();
//!
//! let scale_notes = scale.notes();
//!
//! // Chord Example
//! let chord = Chord::new(Pitch::from(C), ChordQuality::Major, ChordNumber::Triad);
//!
//! let chord_notes = chord.notes();
//! ```

pub mod analysis;
pub mod chord;
pub mod cli;
pub mod counterpoint;
pub mod figured_bass;
pub mod harmony;
pub mod interval;
pub mod neo_riemannian;
pub mod note;
pub mod scale;
pub mod set_class;
pub mod voice_leading;
