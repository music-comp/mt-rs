# Orbit Progression Example — Implementation Plan

## Context

A new runnable example that takes a sequence of OTH orbits from a file, generates a voice-led chord progression by picking minimum-L1 representatives through the orbit sequence, and optionally exports to MIDI. Complements `harmonize_melody.rs` (melody→chords) with the dual approach (orbits→chords).

---

## Commit Structure

Two commits:
1. `feat(quintal,note): Orbit::from_str and parse_midi_pitch utilities`
2. `feat(example): orbit_progression — orbit-sequence to voiced-chord MIDI export`

---

## Commit 1: Library Additions

### 1.1 — `Orbit::from_str` in `crates/mt/src/quintal/orbit.rs`

`Orbit` has no `FromStr` currently. Add:

```rust
impl std::str::FromStr for Orbit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "Q777" => Ok(Orbit::Q777),
            "Q767" => Ok(Orbit::Q767),
            "Q787" => Ok(Orbit::Q787),
            "Q676" => Ok(Orbit::Q676),
            "Q686" => Ok(Orbit::Q686),
            "Q878" => Ok(Orbit::Q878),
            "Q868" => Ok(Orbit::Q868),
            "Q776" => Ok(Orbit::Q776),
            "Q877" => Ok(Orbit::Q877),
            "Q867" => Ok(Orbit::Q867),
            "Q876" => Ok(Orbit::Q876),
            "Q788" => Ok(Orbit::Q788),
            "Q786" => Ok(Orbit::Q786),
            "Q688" => Ok(Orbit::Q688),
            other => Err(format!("unknown orbit: {:?}", other)),
        }
    }
}
```

Unit tests: all 14 variants parse (case-insensitive), invalid strings error.

### 1.2 — `parse_midi_pitch` in `crates/mt/src/note/mod.rs`

No MIDI-pitch-from-string parser exists. `Pitch::try_parse` handles note+accidentals but not octave. Add a public function:

```rust
/// Parse a pitch string like "C4", "F#3", "Bb-1" into a MIDI pitch number (0–127).
///
/// Accepts ASCII accidentals (`#`, `b`) and Unicode (`♯`, `♭`).
/// Octave convention: C-1 = 0, C0 = 12, C4 = 60 (Middle C).
///
/// # Errors
///
/// Returns an error if the string is malformed or the resulting pitch
/// is outside MIDI range 0–127.
pub fn parse_midi_pitch(s: &str) -> Result<u8, NoteError> { ... }
```

Implementation: split the string into pitch-part (letters+accidentals) and octave-part (trailing integer, possibly negative). Parse pitch via `Pitch::try_parse`, get semitone via `pitch.as_u8()`, compute MIDI = `(octave + 1) * 12 + semitone`, validate 0..=127.

Unit tests: "C-1"→0, "C0"→12, "C4"→60, "A4"→69, "F#3"→54, "Bb3"→58, "B♭3"→58, "C9"→120, invalid→error.

### 1.3 — Wire exports

- Add `pub use note::parse_midi_pitch;` to `lib.rs` re-exports? No — per locked resolution #1, no crate-root re-exports. Access via `music_comp_mt::note::parse_midi_pitch`.
- `Orbit::from_str` is automatically available since `Orbit` is already re-exported from `quintal::`.

### 1.4 — Verification

```bash
cargo check
cargo test --all-features
cargo clippy --all-features -- -D warnings
```

---

## Commit 2: Example + Data Files

### 2.1 — Example input files in `crates/mt/examples/data/orbit_progressions/`

**`cadence.txt`:**
```
# OTH Cadence: Saddle → Slope → Summit
Q686
Q786
Q777
```

**`departure.txt`:**
```
# OTH Departure: Summit → Plateau → Slope → Saddle
Q777
Q787
Q786
Q686
```

**`d5_tour_descending.txt`:**
```
# d5-wing descending arch tour (n=-1, ends at neighbor Summit)
# From oth_d5_wing_tour.md — visits Saddle, Narrows, Valley d5
Q777
Q786
Q686
Q776
Q676
Q776
Q767
Q867
Q777
```

**`d5_tour_closed.txt`:**
```
# d5-wing closed cycle tour (n=0, returns to starting Summit)
# From oth_d5_wing_tour.md
Q777
Q786
Q686
Q776
Q676
Q776
Q767
Q786
Q777
```

**`saddle_pivot.txt`:**
```
# Saddle-pivot modulation: C-key → F#-key in 5 chords
# From oth_modulation_via_saddle_pivot.md
Q777
Q786
Q686
Q786
Q777
```

### 2.2 — `crates/mt/examples/orbit_progression.rs`

**Structure:**
1. Parse CLI args: `--input <path>`, `--starting-pitch <str>` (default "C3"), `--export-midi <path>`, `--duration <whole|half|quarter>` (default "whole"), `--bpm <int>` (default 120)
2. Read + parse input file (skip comments/blanks, parse orbits with line numbers in errors)
3. Algorithm: orbit sequence → voiced progression (first-chord placement + greedy min-L1)
4. Print progression to stdout
5. Optionally export MIDI

**Algorithm (first chord):**
- Parse starting pitch to MIDI `p_start`
- Find all PcChords in BaseSpace whose orbit matches input[0]
- For each, build `quintal_root(pc, p_start / 12)` and get `inversion_cycle`
- Find inversion with `pitches[0] % 12 == p_start % 12`
- If multiple matches, pick the one closest to `p_start` by octave
- If none match the PC, error with valid bottom-voice PCs listed

**Algorithm (subsequent chords):**
- For each PcChord in the next orbit, build inversions at nearby octaves (bracket previous chord's register: `prev.pitches[0] / 12 - 1` through `prev.pitches[3] / 12 + 1`)
- Compute `min_voiced_chord_l1(prev, candidate)` for all candidates
- Pick minimum; ties broken by lowest bottom-voice MIDI

**Output format (stdout):**
```
Position 1: C3–G3–D4–A4 (Q777, Summit)
Position 2: C3–F#3–D4–A4 (Q786, Slope)
...
Total voice-leading cost: N semitones
```

**MIDI export:** Same as harmonize_melody — Format 1, 3 tracks (conductor, treble register, bass register split at middle C? or single track with all voices). Actually simpler: one track for the 4-voice chord, since it's all one instrument. Use `--duration` for note length, `--bpm` for tempo, 4/4 time sig.

### 2.3 — Verification

```bash
cargo build --all-features --examples
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/d5_tour_descending.txt \
    --starting-pitch C3 --export-midi /tmp/d5_tour.mid
cargo clippy --all-features -- -D warnings
```

Verify d5_tour_descending output matches the worked example from the research doc:
- Position 0: C3–G3–D4–A4 (Q777)
- Position 1: C3–F#3–D4–A4 (Q786)
- etc.

---

## Acceptance Criteria

```bash
cargo build --all-features --examples
cargo test --all-features
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt \
    --export-midi /tmp/cadence.mid
cargo clippy --all-features -- -D warnings
```

For each example input, the printed progression must:
- Have one chord per orbit line
- Have all four voices ascending in each chord
- Have all MIDI pitches in 0..=127
- Have the first chord's bottom voice matching `--starting-pitch`
- Have small L1 movements between consecutive chords

The `d5_tour_descending.txt` output with `--starting-pitch C3` should match the research doc's worked example (voicings from the descending arch tour table).

---

## Observations

1. `Orbit` has no `FromStr` — must add it (library addition).
2. No MIDI-pitch-from-string parser exists — must add `parse_midi_pitch` to `note/`.
3. The research docs give exact voicings for the d5-wing tours starting at C3 (MIDI 48) — these serve as ground-truth for verification.
4. The saddle-pivot example starts at D3 (MIDI 50) per the research doc — the `--starting-pitch D3` flag should reproduce those voicings.
5. For the greedy voice-leading algorithm, using `min_voiced_chord_l1` (the 24-permutation brute-force) ensures optimal per-step assignment. The greedy approach (locally optimal per step) won't necessarily produce the globally optimal progression, but it's the right choice for an example that demonstrates OTH voice-leading concepts.
