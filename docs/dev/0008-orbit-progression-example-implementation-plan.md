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
# Plateau is Q877 (degree-6 Plateau, A5-flavored hub) — directly adjacent
# to Summit at the chord-graph level. Q787 (Upper Plateau, ridge,
# degree 8) is NOT directly adjacent to Summit (see oth_atomic_ridge_walks.md);
# using Q877 here reflects the research-doc convention in
# oth_modulation_via_saddle_pivot.md.
Q777
Q877
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

1. Parse starting pitch to MIDI `p_start`. Compute `target_pc = p_start % 12`.
2. Find all PcChords in `BaseSpace::chords()` whose `classify_orbit` matches the first orbit in the input file.
3. For each candidate `PcChord`, build a canonical voicing at any reasonable base octave (e.g., `base_octave = 4`) via `quintal_root(pc_chord, base_octave)?`, then take the inversion cycle (4 voicings).
4. Across all `(PcChord, inversion)` pairs, find one whose bottom-voice PC equals `target_pc`. (For any chord in B, all 4 of its PCs appear as bottom voice across the 4 inversions, so a chord-with-target-PC will produce a matching inversion if its PC set contains `target_pc`.)
5. **Octave-align the matching voicing**: the matching inversion's bottom voice has PC `target_pc` but its MIDI value may be at any octave. Shift all four pitches up or down by an integer number of octaves so the bottom voice equals `p_start` exactly. Specifically, with `delta = p_start - matching_inversion.pitches[0]`, the shifted voicing is `matching_inversion.pitches.map(|p| p + delta)`. Verify all shifted pitches are in 0..=127; if not, the chord doesn't fit at this register and the algorithm should try a different (PcChord, inversion) pair, or error if no fit exists.
6. If multiple `(PcChord, inversion)` pairs produce valid octave-aligned voicings (which happens when multiple PcChords in the orbit contain `target_pc`), pick deterministically: smallest PcChord by lexicographic order of sorted PCs, then smallest inversion index.
7. If no `(PcChord, inversion)` pair across the orbit has a matching bottom-voice PC, error with a clear message listing the valid bottom-voice PCs for the requested orbit. Concretely: enumerate all PCs that appear in any chord of the orbit, render each as a note name (sharps), and include in the error message: `"orbit Q686 has no chord with bottom voice C; valid bottom-voice PCs for Q686 are: C, C#, D, D#"`.

**Algorithm (subsequent chords):**

1. Take the chord at position `i-1`.
2. Compute the previous chord's "register bracket": `low_octave = prev.pitches[0] / 12 - 1`, `high_octave = prev.pitches[3] / 12 + 1`. This gives a small range of base octaves to consider for each candidate PcChord.
3. For each PcChord in the orbit at position `i`:
   - For each `base_octave` in `low_octave..=high_octave`:
     - Build the canonical voicing via `quintal_root(pc_chord, base_octave)?`. Skip if construction fails (would only happen at extreme octaves where pitches exceed u8 range).
     - Take the inversion cycle (4 voicings).
     - For each of the 4 inversions, verify all pitches are in 0..=127 (skip if not).
     - Compute `min_voiced_chord_l1(prev_chord, candidate)`.
     - Track the running minimum.
4. Pick the candidate with minimum L1.
5. **Tie-breaking** (deterministic): among candidates tied for minimum L1, pick the one with the lowest bottom-voice MIDI. Among those still tied, pick the one whose underlying PcChord has lexicographically smallest sorted-PCs tuple.
6. If no valid candidate exists (e.g., all candidates exceed MIDI range), error with a clear message naming the position and the orbit.

**Output format (stdout):**
```
Position 1: C3–G3–D4–A4 (Q777, Summit)
Position 2: C3–F#3–D4–A4 (Q786, Slope)
...
Total voice-leading cost: N semitones
```

**MIDI export:** Replicate the harmonize_melody example's MIDI infrastructure exactly. Before writing this section of the example, **read `crates/mt/examples/harmonize_melody.rs` carefully and document its MIDI output structure in this section of the dev plan**. Specifically, the dev plan must record:

- Which MIDI library is used (e.g., `midly`, `ghakuf`, hand-rolled, etc.).
- File format (Format 0 single-track vs Format 1 multi-track).
- Time division (ticks per quarter note).
- Track structure (number of tracks, what each track contains).
- Per-track meta-events (track name, instrument/program change, time signature, tempo).
- Note-on/note-off conventions (velocity, duration encoding).

Once recorded, this example's MIDI output should mirror that structure with only the following differences:

- The *content* of the notes is the orbit-progression's voicings rather than the harmonize_melody's progression.
- Tempo from `--bpm` (default 120).
- Note duration from `--duration`: `whole` (default) = 4 beats, `half` = 2 beats, `quarter` = 1 beat. In ticks-per-quarter-note, multiply by 4, 2, 1 respectively.
- Time signature 4/4 (matching the duration semantics).

Do *not* introduce a different MIDI library, a different file format, or a different track structure than harmonize_melody uses. If the harmonize_melody example uses a track structure that doesn't fit the orbit-progression use case, surface the conflict in the dev plan and propose the minimum change needed; don't deviate silently.

### 2.3 — Tests

Tests live in `#[cfg(test)] mod tests` blocks alongside the helper functions in the example file (and in `quintal/orbit.rs` and `note/mod.rs` for the library additions). Each test prints values via `eprintln!` so `--nocapture` runs let us read the actual data.

**Library-level tests (Commit 1):**

In `crates/mt/src/quintal/orbit.rs`, alongside `Orbit::from_str`:

- All 14 orbit names parse case-insensitively: `"Q777"`, `"q777"`, `"Q777 "` (trailing space), `"  Q686  "` all return the right variant.
- Invalid strings return errors with useful messages: `""`, `"Q999"`, `"foo"`, `"Q77"`, `"Q7777"`.

In `crates/mt/src/note/mod.rs`, alongside `parse_midi_pitch`:

- Boundary cases: `"C-1"` → 0, `"C0"` → 12, `"C4"` → 60, `"A4"` → 69, `"C9"` → 120, `"G9"` → 127.
- Sharps: `"C#3"` → 49, `"F#4"` → 66.
- Flats (ASCII): `"Db3"` → 49, `"Bb3"` → 58, `"Bb-1"` → 10.
- Flats (Unicode `♭`): `"B♭3"` → 58, `"D♭3"` → 49.
- Sharps (Unicode `♯`): `"C♯4"` → 61.
- Out-of-range errors: `"C-2"` (would be -12), `"C10"` (would be 132), `"G#9"` (would be 128).
- Malformed errors: `""`, `"X3"`, `"C"` (no octave), `"C3.5"`, `"3C"`.

**Example-level tests (Commit 2), in `#[cfg(test)] mod tests` of `orbit_progression.rs`:**

1. **File parser**: a multi-line input string with comments (`# foo`), blank lines, leading/trailing whitespace on orbit lines, and one invalid orbit name. Test asserts: comments and blanks skipped, valid orbit lines parsed in order, invalid orbit raises an error with the line number (e.g., "line 7: unknown orbit `Q999`").

2. **First-chord placement** — basic case: `--starting-pitch C4` (MIDI 60) and orbit Q777. Expected first chord: bottom voice exactly at MIDI 60, full voicing whose PCs form a valid Q777 chord. Specifically `[60, 67, 74, 81]` (C4-G4-D5-A5).

3. **First-chord placement** — error case: `--starting-pitch D#1` and orbit Q686. There is no Saddle chord with bottom-voice PC 3 (Saddles have bottom-voice PCs only in {0, 1, 2, 3} — actually {0, 1, 2, 3}, so D# = 3 *is* a valid bottom-voice PC for Saddle). Pick a confirmed-invalid case instead: e.g., `--starting-pitch F1` (PC 5) and orbit Q686. F is *not* a valid Saddle bottom-voice PC, so this should error with the listed valid PCs (C, C#, D, D#).

4. **Voice-leading correctness**: given a hand-built `prev_chord = [48, 55, 62, 69]` (C-Summit Q777) and the next orbit Q686, the chosen chord must minimize L1 from `prev_chord`. Compute the L1 to all candidate chords in Q686 (across all 4 inversions × octave bracket); the algorithm's pick must equal the minimum. Expected pick: a Saddle chord adjacent to C-Summit in 2 chord-graph edges, such as the Saddle in the descending arch's worked example.

5. **End-to-end against research-doc ground truth**: run the algorithm with `d5_tour_descending.txt` and `--starting-pitch C3`. The 9 produced voicings must equal exactly:

   ```
   Position 0: [48, 55, 62, 69]   # C3-G3-D4-A4
   Position 1: [48, 54, 62, 69]   # C3-F#3-D4-A4
   Position 2: [48, 54, 62, 68]   # C3-F#3-D4-Ab4
   Position 3: [49, 54, 62, 68]   # C#3-F#3-D4-Ab4
   Position 4: [49, 55, 62, 68]   # C#3-G3-D4-Ab4
   Position 5: [48, 55, 62, 68]   # C3-G3-D4-Ab4
   Position 6: [48, 55, 61, 68]   # C3-G3-C#4-Ab4
   Position 7: [47, 55, 61, 68]   # B2-G3-C#4-Ab4
   Position 8: [47, 54, 61, 68]   # B2-F#3-C#4-Ab4
   ```

   This is byte-identical to the descending arch tour in `oth_d5_wing_tour.md`. If the algorithm produces a different sequence with the same total L1, the test should still fail (we want the *exact* research-doc voicings, not just any minimum-cost progression). To make the test deterministic, pin tie-breaking rules.

6. **End-to-end second case**: run with `saddle_pivot.txt` and `--starting-pitch D3`. The 5 produced voicings must equal the table in `oth_modulation_via_saddle_pivot.md`:

   ```
   Position 0: [50, 57, 67, 72]   # D3-A3-G4-C5
   Position 1: [50, 57, 66, 72]   # D3-A3-F#4-C5
   Position 2: [50, 56, 66, 72]   # D3-G#3-F#4-C5
   Position 3: [51, 56, 66, 72]   # D#3-G#3-F#4-C5
   Position 4: [51, 56, 66, 73]   # D#3-G#3-F#4-C#5
   ```

7. **Closed-cycle case**: run with `d5_tour_closed.txt` and `--starting-pitch C3`. Should produce the closed-cycle table from `oth_d5_wing_tour.md` Tour 2, ending at C3-G3-D4-A4 (same as start).

If any of these end-to-end tests fail, the algorithm or the orbit-data file is wrong. These tests are the most powerful structural correctness checks in the suite — they validate that the orbit-progression example faithfully reproduces published OTH research material.

### 2.4 — Verification

```bash
cargo build --all-features --examples
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/d5_tour_descending.txt \
    --starting-pitch C3 --export-midi /tmp/d5_tour.mid
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/saddle_pivot.txt \
    --starting-pitch D3 --export-midi /tmp/saddle_pivot.mid
cargo clippy --all-features -- -D warnings
```

Visual verification of `d5_tour_descending` stdout output: must match the descending arch tour table from `oth_d5_wing_tour.md` voice-for-voice.

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
3. The research docs give exact voicings for the d5-wing tours starting at C3 (MIDI 48) — these serve as ground-truth for verification, encoded in the end-to-end tests in §2.3.
4. The saddle-pivot example starts at D3 (MIDI 50) per the research doc — the `--starting-pitch D3` flag should reproduce those voicings, also encoded in §2.3 tests.
5. For the greedy voice-leading algorithm, using `min_voiced_chord_l1` (the 24-permutation brute-force) ensures optimal per-step assignment. The greedy approach (locally optimal per step) won't necessarily produce the globally optimal progression, but it's the right choice for an example that demonstrates OTH voice-leading concepts.
6. The `departure.txt` example uses `Q877` (degree-6 Plateau, A5-flavored) for the Plateau, not `Q787` (degree-8 Upper Plateau). Q787 is *not* directly adjacent to Q777 in the chord graph (per `oth_atomic_ridge_walks.md`), so the Departure pathway *Summit → Plateau → Slope → Saddle* read as single-edge moves requires Q877. This matches the research-doc convention in `oth_modulation_via_saddle_pivot.md`'s 7-chord expansion.
7. The default `--starting-pitch` is `C3` rather than `C4` so that the no-flag run with `d5_tour_descending.txt` reproduces the worked example from `oth_d5_wing_tool.md` exactly, giving the example pedagogical value even at default settings.
8. Tie-breaking in voice-leading is deterministic: minimum L1, then lowest bottom-voice MIDI, then lexicographically smallest sorted-PCs tuple of the underlying PcChord. Required for the end-to-end tests in §2.3 to match the research-doc voicings byte-for-byte.
