# Orbit Progression Example — CC Planning Prompt

This is a focused feature: a new runnable example in mt-rs that takes a sequence of orbits (one per line in a file) plus a starting pitch, generates a chord progression by voice-leading through the orbits, and exports the result as a MIDI file. Builds on the existing `crates/mt/examples/harmonize_melody.rs` example's MIDI-export infrastructure.

This is **not** a Phase 2 sub-bundle. It's a self-contained example file that exercises the public OTH API in a new way.

---

## Prompt to send to CC

Copy from the next horizontal rule down to the closing horizontal rule, paste into a message to CC.

---

You are entering Plan Mode for a new runnable example in `crates/mt/examples/`. This is a self-contained example that approaches OTH chord-progression generation from the *orbit-sequence* side rather than the *melody* side, complementing the existing `harmonize_melody.rs` example.

## Context

The `crates/mt/examples/harmonize_melody.rs` example takes a melody, runs the harmonize-melody algorithm, and exports the resulting voice-led progression to a MIDI file. The orbit-sequence problem is the dual: given a hand-specified sequence of OTH orbits (e.g., `Q777, Q786, Q686, Q776, Q676, Q776, Q767, Q867, Q777`), pick specific chord representatives that voice-lead smoothly through the sequence, anchor the first chord at a user-specified starting pitch, and export to MIDI.

This is useful for OTH research and composition: a researcher writes down a functional progression at the orbit level (e.g., "Summit → Slope → Saddle → ...") and the example materializes it into actual notes that can be played and heard.

## Setup

1. Load the project's Rust skill: Read `assets/ai/rust/SKILL.md` and any sub-guides in `assets/ai/rust/guides/` that the skill points to as relevant for example-program code, file I/O, error handling with custom error types, and command-line argument parsing. Per the project CLAUDE.md, this is the authoritative Rust guidance and overrides any default training-time behavior. Also load Anthropic's `anthropic-skills:rust-guidelines` skill.

2. Read the implementation workflow supplement (the test-as-you-go discipline applies — implement, run, observe output, then move on; for an example program with audio output, "observe" means actually examining the resulting MIDI file structure and ideally listening to it):
   - `workbench/0007-phase-1-implementation-workflow-supplement.md`

3. Read the existing harmonize_melody example to understand the MIDI-export infrastructure already in place:
   - `crates/mt/examples/harmonize_melody.rs`

   Pay particular attention to: how it parses the `--export-midi` flag, which MIDI library it uses, what chord-duration / time-signature convention it follows, and how it converts `VoicedChord` data into MIDI events. The new example should reuse this infrastructure as faithfully as possible — same MIDI library, same conventions, same code shape where reasonable.

4. Read the relevant library API surfaces to understand what's available:
   - `crates/mt/src/quintal/orbit.rs` — the `Orbit` enum; check whether a string-to-Orbit parser already exists. If not, you'll need to add one (or write a small local helper in the example).
   - `crates/mt/src/quintal/base_space.rs` — `BaseSpace`, `chords()`, `classify_orbit`.
   - `crates/mt/src/quintal/constructors.rs` — `quintal_root`.
   - `crates/mt/src/quintal/fiber.rs` — `inversion_cycle` (for getting all 4 inversions of a voicing).
   - `crates/mt/src/quintal/display.rs` — `pc_to_note_name` (for output formatting).
   - `crates/mt/src/voice_leading/min_l1.rs` — `min_voiced_chord_l1` (for picking the closest chord in the next orbit).
   - `crates/mt/src/note/` (or wherever) — see if there's an existing pitch-string parser ("C1", "F#3", etc.). If not, you'll need to write one.

## Task

Enter Plan Mode. Produce a dev plan at `docs/dev/0007-orbit-progression-example-implementation-plan.md` covering this feature. The dev plan should:

- Describe the new file `crates/mt/examples/orbit_progression.rs` with its full structure: argument parsing, file reading, orbit-sequence-to-voicings algorithm, MIDI export.
- Describe any small library additions needed (e.g., a `FromStr` impl on `Orbit` if it doesn't exist; a pitch-string parser if not already present). Library additions should be minimal — only what the example genuinely needs and what's broadly useful.
- Specify the input file format with concrete examples: one orbit per line, `#` comments allowed, blank lines ignored. Show what a valid input file looks like (3-5 example files in different shapes — short, long, with comments, with various orbits).
- Sequence the implementation steps with explicit verification points (`cargo check`, `cargo run --example orbit_progression -- ...` with test inputs).
- Apply the test-as-you-go workflow: each function gets unit tests where applicable, plus end-to-end runs with example inputs.
- List the acceptance criteria mapped to specific commands.
- Surface any uncertainties or design questions.

## Specific items in scope

### Library additions (only what's needed)

- **`Orbit::from_str` (or equivalent string parser).** Check whether `Orbit` already implements `FromStr` (or `TryFrom<&str>`). If yes, use it. If no, add one in `crates/mt/src/quintal/orbit.rs`. The parser should accept "Q777", "Q686", etc. case-insensitively. Add tests covering all 14 orbit variants and a few invalid inputs.

- **MIDI pitch string parser.** A function like `parse_pitch(s: &str) -> Result<u8, ...>` accepting "C1", "C#1", "Db1", "F4", "B♭3", etc. (with both ASCII `#`/`b` and Unicode `♯`/`♭`). The MIDI convention is C-1 = 0, C0 = 12, C4 = 60 (Middle C), C9 = 120. If a parser like this already exists in the crate (check `note/`, `interval/`), use it. If not, write a small one — either as a private helper in the example file, or as a public utility if the parser is broadly useful (probably the latter; "parse a pitch string into a MIDI number" is a generic need).

### Example file: `crates/mt/examples/orbit_progression.rs`

**CLI flags.**

- `--input <path>` (required, positional or named): path to the input file containing one orbit per line.
- `--starting-pitch <pitch>` (default: `"C4"`): the pitch for the bottom voice of the first chord. Uses the same pitch-string format as the parser above.
- `--export-midi <path>` (optional): path to write the MIDI file. If omitted, the example just prints the progression to stdout (matching the harmonize_melody example's no-flag default).
- `--duration <unit>` (default: `whole`): chord duration. Accepted values: `whole`, `half`, `quarter`. In 4/4 time, this controls how long each chord sustains. Default `whole` means each chord is one measure.
- `--bpm <integer>` (default: `120`): tempo for the MIDI file.

**Input file format.** Plain text, one orbit per line. Lines starting with `#` are comments and are ignored. Blank lines are ignored. Whitespace around orbit names is trimmed. Example:

```
# A d5-wing tour that returns to the starting Summit
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

(This particular example reproduces the closed-cycle d5-wing tour from `oth_d5_wing_tour.md`.)

**Algorithm: orbit sequence → voiced-chord progression.**

For position 0 (the first chord):

1. Parse the user's `--starting-pitch` to a MIDI value `p_start`. Let `target_pc = p_start % 12` and `target_octave = p_start / 12 - 1` (using the C-1 = 0 convention).
2. Find every `PcChord` in `BaseSpace::chords()` whose `classify_orbit` matches the first orbit in the input file.
3. For each candidate `PcChord`, build its canonical voicing via `quintal_root(pc_chord, target_octave)`. Take the inversion cycle (4 voicings).
4. Pick the inversion whose bottom voice (`pitches[0]`) has PC equal to `target_pc`. If multiple inversions across multiple PcChords match, pick the one whose bottom-voice MIDI is exactly `p_start` (or as close as possible by octave shift).
5. If no inversion of any chord in the first orbit has a bottom-voice PC matching `target_pc`, error out with a clear message ("orbit Q686 has no chord with bottom-voice C; valid bottom-voice pitches for Q686 are: C, C#, D, D#"). Include the list of valid bottom-voice PCs in the error message.

For subsequent positions `i ≥ 1`:

1. Take the chord at position `i-1`.
2. For each `PcChord` in the orbit at position `i`, generate all 4 inversions of the canonical voicing (via `inversion_cycle(quintal_root(pc_chord, base_octave))` for several `base_octave` values that bracket the previous chord's register — say, `prev_chord.pitches[0] / 12 - 1` through `prev_chord.pitches[3] / 12 + 1`).
3. For each candidate voicing, compute `min_voiced_chord_l1(prev_chord, candidate)`.
4. Pick the candidate with minimum L1. Ties broken deterministically (e.g., lowest bottom-voice MIDI).

**Output (no `--export-midi` flag).**

Print each chord one per line, with note names and the orbit it represents. Match the prose-output format of the harmonize_melody example as closely as possible. Include a final summary line with the total voice-leading cost (sum of L1 distances between consecutive chords).

Example:

```
Position 1: C4 – G4 – D5 – A5 (Q777, C-Summit)
Position 2: C4 – F#4 – D5 – A5 (Q786, Slope)
Position 3: C4 – F#4 – D5 – Ab5 (Q686, Saddle)
...
Total voice-leading cost: 8 semitones
```

**Output (with `--export-midi`).**

Generate a Standard MIDI File at the given path, in 4/4 time, with each chord as a vertical sonority playing for the duration specified by `--duration`. Use the same MIDI library and code shape as the existing harmonize_melody example. Tempo from `--bpm`. Include track metadata (instrument, tempo, time signature).

After writing the file, also print the progression to stdout (same format as the no-flag case) so the user can verify what was written.

### Example input files

Include 3-5 example input files in `crates/mt/examples/data/orbit_progressions/` (or wherever fits the project convention):

- `cadence.txt`: a Cadence (Saddle → Slope → Summit, 3 lines).
- `departure.txt`: a Departure (Summit → Plateau → Slope → Saddle, 4 lines).
- `d5_tour_descending.txt`: the descending d5-wing arch from `oth_d5_wing_tour.md` (9 lines, ends at neighbor Summit).
- `d5_tour_closed.txt`: the closed-cycle d5-wing tour (9 lines, returns to start).
- `axis_pivot.txt`: a Saddle-pivot modulation (5 lines, from `oth_modulation_via_saddle_pivot.md`).

Each file should have a header comment naming the progression and citing the OTH research doc that motivates it.

### Tests

In-module `#[cfg(test)] mod tests` for the example's helper functions:

- Pitch parser: round-trip `parse → format → parse`; specific cases ("C-1" → 0, "C0" → 12, "C4" → 60, "F#3" → 54, "Bb3" → 58, "B♭3" → 58, "C9" → 120); invalid inputs return errors.
- Orbit parser: all 14 orbit names parse correctly (case-insensitively); invalid inputs return errors.
- File parser: comments and blank lines are skipped; orbit lines are parsed; invalid orbits raise errors with line numbers.
- First-chord placement: given orbit Q777 and starting pitch C4, the first chord's bottom voice is exactly C4 (MIDI 60).
- Voice-leading: given two consecutive orbits, the chosen second chord has the minimum L1 distance to the first.

End-to-end smoke tests: run each example input file with `--export-midi` and verify a non-empty MIDI file is produced. Inspect the stdout output to confirm the chords are physically reasonable (no out-of-range pitches, ascending voicings, sensible note names).

### Acceptance criteria

```bash
cargo build --all-features --examples                                 # clean
cargo test --all-features                                              # all pass
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt    # prints 3 chords
cargo run -p music-comp-mt --example orbit_progression -- \
    --input crates/mt/examples/data/orbit_progressions/cadence.txt \
    --export-midi /tmp/cadence.mid                                     # writes file
cargo clippy --all-features -- -D warnings                             # clean
```

For each example input, the printed progression must:
- Have one chord per orbit line in the input.
- Have all four voices in ascending order in each chord.
- Have all MIDI pitches in 0..=127.
- Have the first chord's bottom voice equal to the `--starting-pitch` (or as close as achievable).
- Have monotonic-or-small voice movements between consecutive chords (sanity check: typical L1 between consecutive chords should be small, not 20+ semitones).

The `cadence.txt` and `d5_tour_*.txt` examples should produce the same chord sequences as the worked examples in `oth_modulation_via_saddle_pivot.md` and `oth_d5_wing_tour.md` respectively, when the starting pitch matches the worked example. This is a structural consistency check.

## Hard rules

- No `if let Ok(...)` swallowing of errors.
- No `Option`-returning skip paths to dodge invariant violations.
- The first-chord-placement step's "no chord matches starting pitch" error must produce a *useful* error message listing valid bottom-voice PCs for the requested orbit, not a generic failure.
- The voice-leading step picks the *globally* minimum-L1 candidate across all inversions and reasonable octave placements. Don't shortcut to "use canonical voicing always" — that produces jagged output and defeats the purpose of the example.
- Reuse the harmonize_melody example's MIDI infrastructure (same library, same conventions). Don't introduce a different MIDI crate or a different time-signature convention without flagging it explicitly.
- Default chord duration is whole notes in 4/4 (one chord per measure). Other durations available via `--duration` flag.
- If `Orbit::from_str` (or equivalent) doesn't exist, add it to the library — that's a broadly useful addition. If `parse_pitch` doesn't exist somewhere reusable, surface it as a public utility (probably in `note/` or a new helper module). Write tests for both.

## Halt condition

Do NOT exit Plan Mode. Do NOT begin implementation. Stop after writing the dev plan and report back. We will review the dev plan together before you start coding.
