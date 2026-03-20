use crate::chord::errors::ChordError;
use crate::chord::number::Number::Triad;
use crate::chord::{Number, Quality};
use crate::interval::Interval;
use crate::note::{Note, NoteError, Notes, Pitch, NoteLetter};

/// A chord.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chord {
    /// The root note of the chord.
    pub root: Pitch,
    /// The octave of the root note of the chord.
    pub octave: u8,
    /// The intervals within the chord.
    pub intervals: Vec<Interval>,
    /// The quality of the chord: major, minor, diminished, etc.
    pub quality: Quality,
    /// The superscript number of the chord: 3, 7, maj7, etc.
    pub number: Number,
    /// The inversion of the chord: 0=root position, 1=first inversion, etc.
    pub inversion: u8,
}

impl Chord {
    /// Create a new chord.
    pub fn new(root: Pitch, quality: Quality, number: Number) -> Self {
        Self::with_inversion(root, quality, number, 0)
    }

    /// Create a new chord with a given inversion.
    pub fn with_inversion(
        root: Pitch,
        quality: Quality,
        number: Number,
        inversion: u8,
    ) -> Self {
        let intervals = Self::chord_intervals(quality, number);
        let inversion = inversion % (intervals.len() + 1) as u8;
        Chord {
            root,
            octave: 4,
            intervals,
            quality,
            number,
            inversion,
        }
    }

    pub fn from_string(string: &str) -> Result<Self, ChordError> {
        let notes: Vec<Pitch> = string
            .replace(',', "")
            .split_whitespace()
            .map(|x| Pitch::try_parse(x).ok_or_else(|| ChordError::InvalidNote(x.to_string())))
            .collect::<Result<Vec<_>, _>>()?;

        if notes.len() < 2 {
            return Err(ChordError::InvalidNote("need at least 2 notes".to_string()));
        }

        let intervals: Vec<u8> = notes.iter()
            .map(|x| x.as_u8() % 12)
            .zip(notes[1..].iter().map(|x| x.as_u8()))
            .map(|(x, y)| if x < y { y - x } else { y + 12 - x })
            .collect();

        Chord::from_interval(notes[0], &intervals)
    }

    pub fn from_interval(root: Pitch, interval: &[u8]) -> Result<Self, ChordError> {
        use Number::*;
        use Quality::*;
        let (quality, number) = match *interval {
            [4, 3] => (Major, Triad),
            [3, 4] => (Minor, Triad),
            [2, 5] => (Suspended2, Triad),
            [5, 2] => (Suspended4, Triad),
            [4, 4] => (Augmented, Triad),
            [3, 3] => (Diminished, Triad),
            [4, 3, 4] => (Major, Seventh),
            [3, 4, 3] => (Minor, Seventh),
            [4, 4, 2] => (Augmented, Seventh),
            [4, 4, 3] => (Augmented, MajorSeventh),
            [3, 3, 3] => (Diminished, Seventh),
            [3, 3, 4] => (HalfDiminished, Seventh),
            [3, 4, 4] => (Minor, MajorSeventh),
            [4, 3, 3] => (Dominant, Seventh),
            [4, 3, 3, 4] => (Dominant, Ninth),
            [4, 3, 4, 3] => (Major, Ninth),
            [4, 3, 3, 4, 3] => (Dominant, Eleventh),
            [4, 3, 4, 3, 3] => (Major, Eleventh),
            [3, 4, 3, 4, 3] => (Minor, Eleventh),
            [4, 3, 3, 4, 3, 4] => (Dominant, Thirteenth),
            [4, 3, 4, 3, 3, 4] => (Major, Thirteenth),
            [3, 4, 3, 4, 3, 4] => (Minor, Thirteenth),
            _ => return Err(ChordError::UnknownIntervalPattern(interval.to_vec()))
        };
        Ok(Self::new(root, quality, number))
    }

    /// Identify possible chords from a set of pitches.
    /// Tries each pitch as the potential root and determines inversion
    /// from which chord tone is in the bass (first input pitch).
    pub fn identify(pitches: &[Pitch]) -> Vec<Chord> {
        if pitches.len() < 2 {
            return vec![];
        }

        let pcs: Vec<u8> = pitches.iter().map(|p| p.as_u8()).collect();
        let n = pcs.len();
        let mut results = Vec::new();

        for root_idx in 0..n {
            let root = pitches[root_idx];
            let root_pc = pcs[root_idx];

            // Compute intervals from this root to all other pitches
            let mut above_root: Vec<u8> = pcs.iter()
                .enumerate()
                .filter(|(i, _)| *i != root_idx)
                .map(|(_, &pc)| (pc + 12 - root_pc) % 12)
                .collect();
            above_root.sort();

            // Convert to adjacent step intervals (what from_interval expects)
            let mut steps = Vec::new();
            let mut prev = 0u8;
            for &interval in &above_root {
                steps.push(interval - prev);
                prev = interval;
            }

            if let Ok(chord) = Self::from_interval(root, &steps) {
                // Determine inversion: which chord tone is the bass (first input pitch)?
                let bass_pc = pcs[0];
                let root_chord = Self::new(root, chord.quality, chord.number);
                let chord_pcs: Vec<u8> = root_chord.notes().iter()
                    .map(|n| n.pitch.as_u8())
                    .collect();
                let inversion = chord_pcs.iter()
                    .position(|&pc| pc == bass_pc)
                    .unwrap_or(0);

                let mut chord = chord;
                chord.inversion = inversion as u8;
                results.push(chord);
            }
        }

        results
    }

    pub fn chord_intervals(quality: Quality, number: Number) -> Vec<Interval> {
        use Number::*;
        use Quality::*;
        match (&quality, &number) {
            // Triads
            (Major, Triad) => Interval::from_semitones(&[4, 3]),
            (Minor, Triad) => Interval::from_semitones(&[3, 4]),
            (Suspended2, Triad) => Interval::from_semitones(&[2, 5]),
            (Suspended4, Triad) => Interval::from_semitones(&[5, 2]),
            (Augmented, Triad) => Interval::from_semitones(&[4, 4]),
            (Diminished, Triad) => Interval::from_semitones(&[3, 3]),
            (HalfDiminished, Triad) => Interval::from_semitones(&[3, 3]),
            (Dominant, Triad) => Interval::from_semitones(&[4, 3]),
            // Sevenths
            (Major, Seventh) => Interval::from_semitones(&[4, 3, 4]),
            (Major, MajorSeventh) => Interval::from_semitones(&[4, 3, 4]),
            (Minor, Seventh) => Interval::from_semitones(&[3, 4, 3]),
            (Augmented, Seventh) => Interval::from_semitones(&[4, 4, 2]),
            (Augmented, MajorSeventh) => Interval::from_semitones(&[4, 4, 3]),
            (Diminished, Seventh) => Interval::from_semitones(&[3, 3, 3]),
            (HalfDiminished, Seventh) => Interval::from_semitones(&[3, 3, 4]),
            (Minor, MajorSeventh) => Interval::from_semitones(&[3, 4, 4]),
            (Dominant, Seventh) => Interval::from_semitones(&[4, 3, 3]),
            (Dominant, MajorSeventh) => Interval::from_semitones(&[4, 3, 4]),
            (Diminished, MajorSeventh) => Interval::from_semitones(&[3, 3, 5]),
            (HalfDiminished, MajorSeventh) => Interval::from_semitones(&[3, 3, 5]),
            // Ninths
            (Dominant, Ninth) => Interval::from_semitones(&[4, 3, 3, 4]),
            (Major, Ninth) => Interval::from_semitones(&[4, 3, 4, 3]),
            (Minor, Ninth) => Interval::from_semitones(&[3, 4, 3, 4]),
            (Augmented, Ninth) => Interval::from_semitones(&[4, 4, 2, 4]),
            (Diminished, Ninth) => Interval::from_semitones(&[3, 3, 3, 5]),
            (HalfDiminished, Ninth) => Interval::from_semitones(&[3, 3, 4, 4]),
            // Elevenths
            (Dominant, Eleventh) => Interval::from_semitones(&[4, 3, 3, 4, 3]),
            (Major, Eleventh) => Interval::from_semitones(&[4, 3, 4, 3, 3]),
            (Minor, Eleventh) => Interval::from_semitones(&[3, 4, 3, 4, 3]),
            (Augmented, Eleventh) => Interval::from_semitones(&[4, 4, 2, 4, 3]),
            (Diminished, Eleventh) => Interval::from_semitones(&[3, 3, 3, 5, 3]),
            (HalfDiminished, Eleventh) => Interval::from_semitones(&[3, 3, 4, 4, 3]),
            // Thirteenths
            (Dominant, Thirteenth) => Interval::from_semitones(&[4, 3, 3, 4, 3, 4]),
            (Major, Thirteenth) => Interval::from_semitones(&[4, 3, 4, 3, 3, 4]),
            (Minor, Thirteenth) => Interval::from_semitones(&[3, 4, 3, 4, 3, 4]),
            (Augmented, Thirteenth) => Interval::from_semitones(&[4, 4, 2, 4, 3, 4]),
            (Diminished, Thirteenth) => Interval::from_semitones(&[3, 3, 3, 5, 3, 4]),
            (HalfDiminished, Thirteenth) => Interval::from_semitones(&[3, 3, 4, 4, 3, 4]),
            // Suspended extensions default to their base triad
            (Suspended2, _) => Interval::from_semitones(&[2, 5]),
            (Suspended4, _) => Interval::from_semitones(&[5, 2]),
        }
        .unwrap()
    }

    /// Parse a chord using a regex.
    pub fn from_regex(string: &str) -> Result<Self, ChordError> {
        let (pitch, pitch_match) = Pitch::from_regex(string)?;

        let slash_option = string.find('/');
        let bass_note_result = if let Some(slash) = slash_option {
            Pitch::from_regex(string[slash + 1..].trim())
        } else {
            Err(NoteError::InvalidPitch)
        };
        let inversion_num_option = if let Some(slash) = slash_option {
            string[slash + 1..].trim().parse::<u8>().ok()
        } else {
            None
        };

        let (quality, quality_match_option) = Quality::from_regex(
            string[pitch_match.end()..slash_option.unwrap_or(string.len())].trim(),
        )?;

        let number = if let Some(quality_match) = quality_match_option {
            Number::from_regex(&string[quality_match.end()..])
                .unwrap_or((Triad, None))
                .0
        } else {
            Triad
        };

        let chord = Chord::with_inversion(
            pitch,
            quality,
            number,
            inversion_num_option.unwrap_or(0),
        );

        if let Ok((bass_note, _)) = bass_note_result {
            let inversion = chord
                .notes()
                .iter()
                .position(|note| note.pitch == bass_note)
                .unwrap_or(0);

            if inversion != 0 {
                return Ok(Chord::with_inversion(
                    pitch,
                    quality,
                    number,
                    inversion as u8,
                ));
            }
        }

        Ok(chord)
    }
}

impl Notes for Chord {
    fn notes(&self) -> Vec<Note> {
        use crate::note::KeySignature;
        
        let root_note = Note {
            pitch: self.root,
            octave: self.octave,
        };
        let mut notes = Interval::to_notes(root_note, self.intervals.clone());
        
        // Apply proper enharmonic spelling based on chord quality.
        // Minor chords use the minor key signature (relative major context).
        // The root note (index 0) is preserved as-is — it was specified by the user.
        let key_signature = KeySignature::for_chord(self.root, self.quality);
        for note in &mut notes[1..] {
            let preferred_spelling = key_signature.get_preferred_spelling(note.pitch);
            note.pitch = crate::note::Pitch::from(preferred_spelling);
        }
        
        notes.rotate_left(self.inversion as usize);

        // Normalize to the correct octave
        if notes[0].octave > self.octave {
            let diff = notes[0].octave - self.octave;
            notes.iter_mut().for_each(|note| note.octave -= diff);
        }

        // Ensure that octave increments at the right notes
        for i in 1..notes.len() {
            if notes[i].pitch.as_u8() <= notes[i - 1].pitch.as_u8() {
                notes[i].octave = notes[i - 1].octave + 1;
            } else if notes[i].octave < notes[i - 1].octave {
                notes[i].octave = notes[i - 1].octave;
            }
        }
        notes
    }
}

impl Default for Chord {
    fn default() -> Self {
        let quality = Quality::Major;
        let number = Number::Triad;
        Chord {
            root: Pitch { letter: NoteLetter::C, accidental: 0 },
            octave: 4,
            intervals: Self::chord_intervals(quality, number),
            quality,
            number,
            inversion: 0,
        }
    }
}
