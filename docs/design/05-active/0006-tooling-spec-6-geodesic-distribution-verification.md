---
number: 6
title: "Tooling Spec: §6 Geodesic Distribution Verification"
author: "Ozan Kasikci"
component: All
tags: [change-me]
created: 2026-05-01
updated: 2026-05-01
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# Tooling Spec: §6 Geodesic Distribution Verification

## Background

The §6 "Geodesics" section of the *Quintal Harmony as a Fiber Bundle* paper currently contains a distribution table summarizing geodesic counts from the C-quintal stack C–G–D–A:

| Distance | Chords at d | Avg geodesics | Max geodesics |
| ----- | :---: | :---: | :---: |
| 1 | 8 | 1 | 1 |
| 2 | 18 | 1.9 | 2 |
| 3 | 36 | 3.5 | 6 |
| 4 | 45 | 7.8 | 24 |
| 5 | 66 | 17.8 | 40 |
| 6 | 44 | 67.9 | 176 |
| 7 | 10 | 226.8 | 298 |

The accompanying prose previously claimed that the chord at distance 7 with 298 geodesics is **A♭–E♭–B♭–F**, identified as "the chord diametrically opposite on the circle of fifths — the antipode of the quintal stack." The OTH MCP server (`get_oth_distance` and `get_oth_chord_info`) reports:

- **A♭–E♭–B♭–F** is at distance **6** from C–G–D–A (not 7), with **82** geodesics (not 298).
- **A♭–E♭–B♭–F** is the T₈ image of C–G–D–A (a transposition by minor 6th), **not** the tritone-antipode (which would be F♯–C♯–G♯–D♯).
- F♯–C♯–G♯–D♯ (the actual T₆ tritone-antipode) is at distance **4** from C–G–D–A, with **12** geodesics — strikingly *close*, not distant.

The prose has been corrected in the current draft to remove the verifiably-wrong claims, and a footnote in §6 marks the **table itself** as pending re-verification. This spec describes the tooling needed to perform that verification.

## What the tool needs to produce

For a fixed source chord (default: C–G–D–A, pcs `[0,2,7,9]`, orbit Q777), generate the **complete geodesic-distribution profile**: for every other chord in B, the distance and the number of distinct geodesics from the source.

Output should support:

1. **The aggregate distribution table** (the §6 table as it currently appears):
   - For each distance `d` in `1..=eccentricity(source)`:
     - `chords_at_d`: count of chords at distance exactly `d`
     - `avg_geodesics`: mean geodesic count to the chords at distance `d`
     - `max_geodesics`: maximum geodesic count to the chords at distance `d`
2. **The maximum-geodesic chord at each distance**: the actual chord(s) achieving `max_geodesics` at each distance, with their pitch-class set, note names, and orbit label. (Multiple chords may tie; report all.)
3. **Sanity totals**: total chords reachable (should equal `|B| - 1 = 227`); eccentricity of the source.

Format: JSON, with both the aggregate table and the per-chord detail. Optional Markdown formatter for paste-into-paper consumption.

## CLI interface

```
mt-oth geodesic-distribution --from "C,G,D,A" [--from-pcs "0,2,7,9"] [--format json|md]
```

Or as an MCP tool:

```
get_oth_geodesic_distribution(source: ChordRef) -> GeodesicDistribution
```

Where `ChordRef` accepts either `notes` or `pcs`, matching the existing OTH MCP convention.

## Implementation notes

The OTH library (`crates/mt/src/quintal/`) already exposes everything needed:

- `BaseSpace::new()` — constructs the 228-vertex base graph
- `all_distances_from(space, source)` — BFS-derived distance map (returns `HashMap<PcChord, u8>`)
- `count_geodesics(space, source, target)` — exact geodesic count between two chords (uses Brandes-style sigma counts, already cached if computed via `betweenness_centrality`)
- `saddle_chords(space)` — for cross-checks (the highest-betweenness orbit, formerly named "Crossroads"; `crossroads_chords` remains as a deprecated alias for backward compatibility)

**Critical for performance**: don't call `count_geodesics` 227 times in isolation; it duplicates BFS work. Either:

- Use a single combined BFS pass that records both distance and predecessor counts (Brandes' algorithm already does this internally — expose the σ counts), or
- Compute σ counts once via `betweenness_centrality`'s internal pass and re-use them.

A naive implementation that runs `count_geodesics` per target may still complete in <1 second since |B| is small, but the right thing to do is one BFS pass.

## Verification cross-checks

When the tool is built and run, please run it twice as a sanity check:

1. **From C–G–D–A** — compare against the §6 table in the paper. The "chords at d" column should match (8, 18, 36, 45, 66, 44, 10) summing to 227. The avg and max columns will tell us whether 226.8 / 298 / 10 are correct or whether the numbers have shifted as the library evolved.

2. **From a Saddle chord** (e.g., C–F♯–D–A♭, pcs `[0,2,6,8]`) — produces a different but equally informative profile. We expect the Saddle to have a *different* distance distribution than the Summit because of its higher betweenness centrality and smaller orbit.

3. **For each pure quintal stack** (12 of them) — verify the distribution is invariant under T (it should be, since transposition is an isometry). If any of the 12 produces a different profile, something is wrong.

## What to bring back

When the tool is ready, bring back:

- The Rust source (one or two files, plus a test or two) + the MCP wrapper if added
- The output for C–G–D–A in both JSON and Markdown formats
- A note on whether the current §6 table is correct, off, or needs replacement

If the current table is wrong, paste the corrected table; we'll drop it into §6 and update the prose accordingly. If it's correct, we'll just remove the "pending re-verification" footnote in §6 and add the correct identification of the maximum-geodesic chord at d = 7.

## Out of scope

- Geodesic distributions in **E** (the extended fiber-bundle space) — that's a separate question and would need a different algorithm because E's connectivity isn't fully defined yet.
- All-pairs geodesic distribution — only the from-one-source profile is needed for §6. (All-pairs would be useful for diameter / eccentricity verification but is a separate tool.)
- Visualizations — leave to a future tool.
