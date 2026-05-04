# Geodesic Progression Example — Implementation Plan

## Context

A third example that finds the shortest chord-graph path (geodesic) between two user-specified endpoint chords and renders it as a voice-led progression with optional MIDI export. No new library additions needed — `geodesics(&space, &start, &end) -> Vec<Vec<PcChord>>` already returns full paths.

---

## Commit Structure

Single commit:
`feat(example): geodesic — chord-graph shortest-path progression with MIDI export`

---

## Library Prerequisites (all confirmed present)

- `geodesics(&BaseSpace, &PcChord, &PcChord) -> Vec<Vec<PcChord>>` — returns all shortest paths
- `distance(&BaseSpace, &PcChord, &PcChord) -> Option<u8>` — geodesic distance
- `classify_orbit(&PcChord) -> Option<Orbit>` — orbit classification
- `Orbit::from_str` — orbit name parsing (from orbit_progression bundle)
- `parse_midi_pitch` — pitch string parsing (from orbit_progression bundle)
- `quintal_root`, `inversion_cycle`, `min_voiced_chord_l1` — voicing/voice-leading

No library additions required.

---

## MIDI Infrastructure (replicated from orbit_progression.rs)

- Library: `midly` (crate, already in dev-dependencies)
- Format: Format 1 (Parallel), multi-track
- Timing: 480 ticks per quarter note
- Tracks: conductor (time sig 4/4 + tempo) + one chord track (program change piano, notes)
- Note duration: `--duration` flag (whole=4 beats, half=2, quarter=1) × ticks_per_quarter
- Velocity: 80 for note-on, 0 for note-off
- Delta encoding: all voices note-on at delta=0, first voice note-off at duration delta, remaining at 0

---

## Example File: `crates/mt/examples/geodesic.rs`

### CLI Flags

- `--start-orbit <orbit>` (required)
- `--start-pitch <pitch>` (required): bottom voice of starting chord
- `--end-orbit <orbit>` (required)
- `--end-pitch <pitch>` (required): bottom voice of ending chord
- `--allow-inversions` (flag, default off)
- `--export-midi <path>` (optional)
- `--duration <whole|half|quarter>` (default: whole)
- `--bpm <integer>` (default: 120)
- `--list-all` (flag, default off): print all geodesics sorted by total L1

### Algorithm

**1. Resolve endpoints:**
- Parse orbit + pitch for both start and end.
- Find PcChord in the orbit matching the bottom-voice PC.
  - Mode 1 (root only): check `quintal_root(pc, octave).pitches[0] % 12 == target_pc`
  - Mode 2 (`--allow-inversions`): check all 4 inversions of each PcChord in the orbit
- Build VoicedChord with bottom voice at exact requested MIDI pitch.
- Error with valid PCs listed if no match found.

**2. Find geodesics:**
- Extract start/end PcChords from the resolved VoicedChords.
- Call `geodesics(&space, &start_pc, &end_pc)` → all shortest paths.
- Report distance and count.

**3. Render each geodesic as voicings:**
- Position 0: starting VoicedChord (already resolved).
- Position N (last): ending VoicedChord (already resolved).
- Intermediate positions: for each PcChord in the path, pick voicing minimizing L1 from previous.
  - Mode 1: only root-position voicings at multiple octaves.
  - Mode 2: all 4 inversions at multiple octaves.
- Compute total L1 for each rendered geodesic.

**4. Select best / list all:**
- Default: pick geodesic with minimum total L1. Ties: first found.
- `--list-all`: sort all by total L1, print all.

**5. Output + MIDI export.**

### Output Format

```
Start: C3–G3–D4–A4 (Q777, Summit)
End:   C3–F#3–D4–G#4 (Q686, Saddle)
Chord-graph distance: 2 edges
Geodesics found: 2

Best geodesic (total voice movement: 2 semitones):
  Position 1: C3–G3–D4–A4 (Q777, Summit)
  Position 2: C3–F#3–D4–A4 (Q786, Slope)  (L1: 1)
  Position 3: C3–F#3–D4–G#4 (Q686, Saddle)  (L1: 1)
```

---

## Implementation Order

1. Write `resolve_endpoint` (find PcChord + build VoicedChord)
2. Write `render_geodesic` (PcChord path → voiced progression)
3. Wire `main()`: parse args → resolve → geodesics → render → pick best → print
4. Add MIDI export (copy from orbit_progression.rs)
5. Add `--list-all` mode
6. Verify: `cargo run --example geodesic -- --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3`

---

## Acceptance Criteria

```bash
cargo build --all-features --examples
cargo test --all-features
cargo run -p music-comp-mt --example geodesic -- \
    --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3
cargo run -p music-comp-mt --example geodesic -- \
    --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3 \
    --export-midi /tmp/geodesic.mid
cargo run -p music-comp-mt --example geodesic -- \
    --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3 --list-all
cargo run -p music-comp-mt --example geodesic -- \
    --start-orbit Q777 --start-pitch C3 --end-orbit Q777 --end-pitch C3
cargo clippy --all-features -- -D warnings
```

- C-Summit to C-Saddle: 2 edges, 2 semitones total, intermediate Q786 or Q776
- Same-PcChord case (Q777 C3 to Q777 C3): 1 chord, 0 cost
- `--allow-inversions` never produces worse results than root-only

---

## Observations

1. `geodesics()` already returns full paths — no library addition needed.
2. The same `resolve_endpoint` logic from orbit_progression.rs can be adapted (find PcChord in orbit with matching bottom-voice PC, octave-align).
3. The intermediate-voicing algorithm is identical to orbit_progression's `pick_next_chord` — reuse the same pattern (bracket octave, try candidates, min L1).
4. For `--list-all`, sorting by total L1 is deterministic since each geodesic produces a fixed total.
5. The same-PcChord case (distance 0) is handled by `geodesics()` returning `vec![vec![chord]]`.
