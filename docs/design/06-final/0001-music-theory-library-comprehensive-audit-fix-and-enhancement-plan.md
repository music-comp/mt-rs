---
number: 1
title: "Music Theory Library: Comprehensive Audit, Fix, and Enhancement Plan"
author: "library calculation"
component: All
tags: [change-me]
created: 2026-03-20
updated: 2026-03-20
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Music Theory Library: Comprehensive Audit, Fix, and Enhancement Plan

## Context

The mt-rs library is a forked, refactored Rust music theory library that will power a music theory MCP server (`music-theory-skill`). The MCP server already serves 4,315 concept cards from 29 authoritative textbooks. The library needs to provide **correct computational tools** that complement the server's knowledge base — enabling Claude to give verified, calculated music theory answers instead of hallucinating.

This plan has three parts:

1. **Part I**: Audit the library against the MCP server for correctness, write a report, fix all bugs
2. **Part II**: Consumer audit — what calculations are needed to prevent hallucinations, what MCP tools to add
3. **Part III**: Vision — additional capabilities and roadmap

---

## Part I: Code Correctness Audit + Fixes

### Phase 1: Write the Audit Report

**Output**: `./workbench/music-theory-audit.md`

Query the MCP server systematically to verify every hardcoded music theory fact in the library. The report will document each finding with MCP evidence.

#### Bugs Confirmed (to be documented with MCP evidence)

**CRITICAL — Wrong musical output:**

1. **Minor chord enharmonic spelling** (`src/chord/chord.rs:221`)
   - Root cause: `Chord::notes()` calls `KeySignature::new(self.root)` with NO mode context
   - G minor outputs [G, A#, D] instead of [G, Bb, D]
   - Bb minor outputs [Bb, C#, F] instead of [Bb, Db, F]
   - Eb minor outputs [Eb, F#, Bb] instead of [Eb, Gb, Bb]
   - Ab minor outputs [Ab, B, Eb] instead of [Ab, Cb, Eb]
   - Tests in `tests/chord/test_enharmonics.rs:64-88` encode the WRONG values
   - **Fix**: Pass chord quality context to KeySignature; minor chords use the minor key's relative major

2. **Harmonic minor scale spelling** (KeySignature falls through to `_ =>` for HarmonicMinor)
   - C harmonic minor outputs D#, G# instead of Eb, Ab
   - **Fix**: Add HarmonicMinor/MelodicMinor mode handling in `get_preferred_spelling()`

3. **Melodic minor scale spelling** — same root cause as #2

4. **Blues scale spelling** — C blues outputs D#, F#, A# instead of Eb, Gb, Bb
   - **Fix**: Blues mode should use flat-biased spelling

5. **Dominant 11th chord pattern** (`src/chord/chord.rs:134`)
   - Library: `[4,3,3,4,4]` = C-E-G-Bb-D-**F#** (augmented 11th, 18 semitones)
   - Correct: `[4,3,3,4,3]` = C-E-G-Bb-D-**F** (perfect 11th, 17 semitones)
   - MCP evidence: "C11 = C-E-G-Bb-D-F" (Hutchinson, Ch. 31, p. 447)
   - Also inconsistent: the Dominant 13th `[4,3,3,4,3,4]` correctly has perfect 11th

**MODERATE — Missing/redundant representations:**

1. **Missing `PitchSymbol::Fb`** (`src/note/pitch_symbol.rs`)
   - Only 20 variants; Fb is needed for Db minor (Db-Fb-Ab), Gb major degree 4, Cb major
   - **Fix**: Add `Fb` variant with `Pitch::new(NoteLetter::F, -1)`

2. **Redundant/meaningless chord types** (`src/chord/chord.rs:111-112,115,123-125`)
   - `(HalfDiminished, Triad)` = `[3,3]` = same as `(Diminished, Triad)` — half-dim is a 7th chord concept
   - `(Dominant, Triad)` = `[4,3]` = same as `(Major, Triad)` — dominant quality only manifests at 7th
   - `(Major, Seventh)` = `(Major, MajorSeventh)` = `(Dominant, MajorSeventh)` = `[4,3,4]` — triple duplicate
   - `(Diminished, MajorSeventh)` = `(HalfDiminished, MajorSeventh)` = `[3,3,5]` — duplicate
   - **Fix**: Remove meaningless types, document the canonical mappings

3. **Suspended chord extensions** (`src/chord/chord.rs:148-149`)
   - Sus2/Sus4 always return triad regardless of Number parameter
   - C7sus4 (C-F-G-Bb) is a real, common chord type
   - **Fix**: Add suspended seventh patterns

**LOW — Design/readability:**

1. **Aeolian/Locrian rotation direction** (`src/scale/scale.rs:61-62`)
   - `rotate_right(2)` and `rotate_right(1)` are correct but break the left-rotation pattern
   - **Fix**: Change to `rotate_left(5)` and `rotate_left(6)` for consistency

2. **Descending melodic minor** — uses ascending pattern; traditional practice uses natural minor descending
    - **Fix**: Detect MelodicMinor + Descending and use Aeolian intervals

### Phase 2: Implement Fixes

**Order of implementation** (each fix builds on the previous):

#### Step 1: Add `PitchSymbol::Fb` (prerequisite for correct flat-key spelling)

- **Files**: `src/note/pitch_symbol.rs` — add variant, Display, From<PitchSymbol> for Pitch
- **Tests**: Add Fb to existing pitch symbol tests

#### Step 2: Fix KeySignature for minor/modal chord context

- **Files**: `src/note/key_signature.rs`
  - Add a `quality: Option<Quality>` parameter or separate method for chord-quality-aware spelling
  - For minor chords: compute relative major from the minor key (root - 3 semitones = relative major)
  - For diminished chords: use the relative major of the associated minor key
  - Add HarmonicMinor and MelodicMinor mode handling (use Aeolian-equivalent relative major)
  - Add Blues mode handling (always prefer flats for blue notes)
- **Files**: `src/chord/chord.rs:221` — pass quality context to KeySignature

#### Step 3: Fix Chord::notes() to use quality-aware spelling

- **Files**: `src/chord/chord.rs:210-245`
  - Change `KeySignature::new(self.root)` to quality-aware variant
  - Minor chord roots → use minor key signature (relative major)
  - Diminished chord roots → use diminished context
- **Tests**: Update `tests/chord/test_enharmonics.rs` lines 64-88 with CORRECT values:
  - G minor = [G, Bb, D], Bb minor = [Bb, Db, F], Eb minor = [Eb, Gb, Bb], etc.

#### Step 4: Fix scale spelling for HarmonicMinor, MelodicMinor, Blues

- **Files**: `src/note/key_signature.rs` — add mode cases for HarmonicMinor, MelodicMinor, Blues
- **Tests**: Update scale tests that assert wrong spellings

#### Step 5: Fix Dominant 11th interval pattern

- **Files**: `src/chord/chord.rs:134` — change `[4,3,3,4,4]` to `[4,3,3,4,3]`
- **Files**: `src/chord/chord.rs:89` — update `from_interval` match for `[4,3,3,4,3]`
- **Tests**: Add test verifying C dominant 11th = C-E-G-Bb-D-F

#### Step 6: Clean up redundant chord types

- **Files**: `src/chord/chord.rs:100-152` — remove/consolidate duplicate patterns
- Document which Quality+Number combinations are canonical

#### Step 7: Add suspended seventh chords

- **Files**: `src/chord/chord.rs:148-149` — add `(Suspended4, Seventh)` = `[5,2,3]` (C-F-G-Bb)

#### Step 8: Fix rotation direction consistency

- **Files**: `src/scale/scale.rs:61-62` — change to `rotate_left(5)` and `rotate_left(6)`

#### Step 9: Fix descending melodic minor

- **Files**: `src/scale/scale.rs` — when MelodicMinor + Descending, use natural minor intervals

### Phase 3: Verification

- `cargo test` — all 114+ tests pass (some updated with correct values, new tests added)
- `cargo clippy` — clean (4 expected module_inception warnings)
- Run MCP queries to confirm library output matches authoritative sources
- All-keys sweep: test every chord quality and scale type across all 15 standard keys

---

## Part II: Consumer Audit — Calculations & MCP Tools

### What Calculations Are Needed (to prevent hallucinations)

| # | Calculation | Current Status | Priority |
|---|------------|---------------|----------|
| 1 | Notes in a named chord | EXISTS: `Chord::new().notes()` | -- |
| 2 | Notes in a named scale | EXISTS: `Scale::new().notes()` | -- |
| 3 | Interval from semitone count | EXISTS: `Interval::from_semitone()` | -- |
| 4 | Transpose note by interval | EXISTS: `Interval::second_note_from()` | -- |
| 5 | **Interval between two notes** | MISSING | Tier 1 |
| 6 | **Diatonic chords of a key** | MISSING | Tier 1 |
| 7 | **Roman numeral analysis** | MISSING | Tier 1 |
| 8 | **Scale identification from notes** | MISSING | Tier 1 |
| 9 | **Chord identification (improved)** | PARTIAL (no inversion detection) | Tier 1 |
| 10 | **Enharmonic equivalence API** | IMPLICIT (as_u8 comparison) | Tier 1 |
| 11 | **Common tones between chords** | MISSING | Tier 2 |
| 12 | **Voice-leading distance** | MISSING | Tier 2 |
| 13 | **Chord-scale compatibility** | MISSING | Tier 2 |
| 14 | **Pivot chord finding** | MISSING | Tier 2 |
| 15 | **Neo-Riemannian transforms** | MISSING | Tier 3 |
| 16 | **Pitch-class set operations** | MISSING | Tier 3 |

### MCP Tools to Add (calling the library as dependency)

#### Tier 1 — Essential for basic correctness

**`get_scale_notes`** — Generate notes of a named scale

- Params: `{ tonic: string, mode: string, direction?: "ascending"|"descending" }`
- Returns: `{ notes: string[], intervals: string[], scale_type: string }`
- Library: `Scale::from_regex() -> .notes()`

**`get_chord_notes`** — Generate notes of a named chord

- Params: `{ root: string, quality: string, number?: string, inversion?: number }`
- Returns: `{ notes: string[], quality: string, intervals: string[] }`
- Library: `Chord::from_regex() -> .notes()`

**`get_interval`** — Calculate interval between two notes (**NEW library function**)

- Params: `{ from: string, to: string }`
- Returns: `{ semitones: number, quality: string, number: string }`
- Library: NEW `Interval::between(Pitch, Pitch) -> Interval`

**`transpose`** — Transpose note(s) by interval (**NEW library function**)

- Params: `{ notes: string[], interval: string, direction: "up"|"down" }`
- Returns: `{ original: string[], transposed: string[] }`
- Library: NEW `Pitch::transpose(Interval, Direction) -> Pitch`

**`get_diatonic_chords`** — Chords built on each scale degree (**NEW module**)

- Params: `{ tonic: string, mode: string, chord_type?: "triad"|"seventh" }`
- Returns: `{ chords: [{ degree: number, roman: string, root: string, quality: string, notes: string[] }] }`
- Library: NEW `harmony::diatonic_chords(&Scale, Number) -> Vec<DiatonicChord>`

**`identify_chord`** — Identify chord from notes (improved)

- Params: `{ notes: string[] }`
- Returns: `{ matches: [{ root: string, quality: string, inversion: number }] }`
- Library: Enhanced `Chord::identify(&[Pitch]) -> Vec<ChordMatch>`

**`identify_scale`** — Find scales containing given notes (**NEW library function**)

- Params: `{ notes: string[] }`
- Returns: `{ matches: [{ tonic: string, mode: string }] }`
- Library: NEW `Scale::identify(&[Pitch]) -> Vec<(Pitch, Mode)>`

**`check_enharmonic`** — Check enharmonic equivalence

- Params: `{ note_a: string, note_b: string }`
- Returns: `{ equivalent: bool, spellings: string[] }`
- Library: NEW `Pitch::is_enharmonic_to(&Pitch) -> bool`

**`analyze_roman_numerals`** — Label progression with Roman numerals (**NEW module**)

- Params: `{ key_tonic: string, key_mode: string, chords: string[] }`
- Returns: `{ analysis: [{ chord: string, roman: string, function: string }] }`
- Library: NEW `analysis::roman_numeral(&Scale, &Chord) -> RomanNumeral`

#### Tier 2 — Important for comprehensive theory

**`get_common_tones`** — Notes shared between two chords/scales

- Library: NEW `harmony::common_tones(&[Pitch], &[Pitch]) -> Vec<Pitch>`

**`get_voice_leading`** — Minimal voice-leading between two chords

- Library: NEW `voice_leading::minimal_movement(&[Note], &[Note]) -> VoiceLeading`

**`get_chord_scale_compatibility`** — Scales compatible with a chord

- Library: Reuses `Scale::identify` logic

**`find_pivot_chords`** — Common chords between two keys

- Library: NEW `harmony::pivot_chords(&Scale, &Scale) -> Vec<PivotChord>`

#### Tier 3 — Advanced/specialized

**`neo_riemannian_transform`** — P, R, L operations on triads

- Library: NEW `neo_riemannian` module

**`set_class_operations`** — Prime form, Forte number, T_n, I_n, interval vector

- Library: NEW `set_class` module

### Infrastructure: Serde Serialization

Add `serde` feature flag; derive `Serialize`/`Deserialize` on all public types. This is the bridge between the library and the MCP server. Must be done before any MCP tool work.

---

## Part III: Vision — What's Missing

### New Library Modules (ordered by tier)

```
src/
  lib.rs              (existing — add pub mod harmony, analysis, voice_leading, ...)
  note/               (existing — extend Pitch with transpose, is_enharmonic_to, all_spellings)
  interval/           (existing — add Interval::between, compound interval support)
  chord/              (existing — add Chord::identify with multi-match + inversion)
  scale/              (existing — add Scale::identify)
  harmony/            (NEW — Tier 1)
    mod.rs
    diatonic.rs       (diatonic chord generation, Roman numerals depend on this)
    common_tones.rs   (Tier 2: pitch-class set intersection)
    pivot.rs          (Tier 2: pivot chord finding)
    compatibility.rs  (Tier 2: chord-scale compatibility)
  analysis/           (NEW — Tier 1)
    mod.rs
    roman.rs          (Roman numeral labeling)
    progression.rs    (Tier 2: common progression detection)
    secondary.rs      (Tier 2: secondary dominant detection)
  voice_leading/      (NEW — Tier 2)
    mod.rs
  neo_riemannian/     (NEW — Tier 3)
    mod.rs
  set_class/          (NEW — Tier 3)
    mod.rs
    forte.rs          (Forte number lookup table — 224 set classes)
  counterpoint/       (NEW — Tier 3)
    mod.rs
  figured_bass/       (NEW — Tier 3)
    mod.rs
```

### Key Architectural Decision: Letter-Aware Intervals

The current `Interval` is semitone-based only — it cannot distinguish augmented 4th (F→B, 6 semitones) from diminished 5th (F→Cb, 6 semitones). The new `Interval::between(Pitch, Pitch)` must count both:

- **Letter distance** (for interval number: F→B = 4th, F→C = 5th)
- **Semitone distance** (for quality: 6 semitones on a 4th = augmented, 6 semitones on a 5th = diminished)

This is the single most important architectural improvement for correct music theory.

### What Makes a Professor Trust Claude's Answers

1. **Correct enharmonic spelling** — never say C# when Db is contextually correct
2. **Proper Roman numeral notation** — uppercase/lowercase, degree symbols, figured bass extensions
3. **Exhaustive identification** — list ALL matching scales/chords, not just one
4. **Computed, not memorized** — every answer backed by library calculation, cross-referenced with MCP concept cards

### Bridge Strategy: Library ↔ MCP Knowledge Base

Each computational MCP tool response should include `related_concepts` referencing concept IDs that the LLM can fetch for pedagogical context. Pattern: (1) compute answer via library, (2) fetch concept card for explanation, (3) present both. This ensures Claude never makes up note names while also providing textbook-quality explanations.

---

## Implementation Sequence

### Immediate (this session): Part I

1. Write audit report to `./workbench/music-theory-audit.md`
2. Fix all confirmed bugs (Steps 1-9 above)
3. Verify with `cargo test` + `cargo clippy`

### Next session: Part II Tier 1

1. Add serde feature flag
2. Implement `Interval::between()` (letter-aware)
3. Implement `Pitch::transpose()`, `Pitch::is_enharmonic_to()`
4. Implement `Scale::identify()`, enhanced `Chord::identify()`
5. New `harmony` module: `diatonic_chords()`
6. New `analysis` module: `roman_numeral()`

### Future sessions: Tiers 2-3

- Voice-leading, common tones, pivot chords, chord-scale compatibility
- Neo-Riemannian, set-class theory, counterpoint checking

---

## Critical Files

| File | Role | Changes |
|------|------|---------|
| `src/note/pitch_symbol.rs` | PitchSymbol enum | Add Fb variant |
| `src/note/key_signature.rs` | Enharmonic spelling engine | Add quality-aware + modal spelling |
| `src/chord/chord.rs` | Chord construction + notes | Fix spelling, fix dom 11th, clean up redundants |
| `src/scale/scale.rs` | Scale construction + notes | Fix descending melodic minor, rotation consistency |
| `src/interval/interval.rs` | Interval system | (Tier 1: add `between()`, compound intervals) |
| `src/note/pitch.rs` | Pitch type | (Tier 1: add `transpose()`, `is_enharmonic_to()`) |
| `tests/chord/test_enharmonics.rs` | Chord spelling tests | Fix wrong expected values |
| `tests/scale/test_scale.rs` | Scale tests | Fix wrong expected values |
| `./workbench/music-theory-audit.md` | Audit report | NEW — write with MCP evidence |

## Verification

1. `cargo test` — all tests pass (updated + new)
2. `cargo clippy` — clean (4 expected module_inception warnings)
3. MCP spot-checks: query server for chord/scale definitions, compare to library output
4. All-keys sweep: verify enharmonic spelling across all 15 standard keys for minor triads, 7th chords, harmonic minor, melodic minor, blues scales
