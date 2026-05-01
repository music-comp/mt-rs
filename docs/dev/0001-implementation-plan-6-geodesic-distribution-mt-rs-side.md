# Implementation Plan: §6 Geodesic Distribution — `mt-rs` side

This plan describes the work needed in **`mt-rs`** to support the geodesic-distribution
verification tool specified in `0006-tooling-spec-6-geodesic-distribution-verification.md`. The MCP
wrapper in `ai-music-theory` is **out of scope** for this document; it should be
written after the mt-rs work lands and exposes the new public API.

Companion file:

- Spec: `./docs/design/05-active/0006-tooling-spec-6-geodesic-distribution-verification.md`

---

## 1. What already exists (do not rebuild)

The library already has every primitive we need; the only missing piece is a
single-pass aggregate over all targets from one source.

- `BaseSpace::new()` — 228-vertex base graph (`crates/mt/src/quintal/base_space.rs`).
- `all_distances_from(space, source) -> HashMap<PcChord, u8>` — BFS distance map
  (`crates/mt/src/quintal/distance.rs`).
- `count_geodesics(space, a, b) -> usize` — σ(b | a) via internal BFS
  (`crates/mt/src/quintal/geodesics.rs:149`).
- `bfs_with_parents(space, source) -> (Vec<Option<u8>>, Vec<Vec<usize>>, Vec<usize>)`
  — **the single-pass BFS that already computes (dist, parents, σ) together**
  (`crates/mt/src/quintal/geodesics.rs:18`). Currently `fn` (private). This is
  the function we will lift into a public, structured API.
- `Orbit` / `classify_orbit(&[PcChord]) -> Orbit` — orbit labelling
  (`crates/mt/src/quintal/orbit.rs`).
- `saddle_chords(space)` — for the Saddle cross-check
  (`crates/mt/src/quintal/centrality.rs`). The function was previously named
  `saddle_chords`; that name still exists as a `#[deprecated]` alias for
  backward compatibility, but new code MUST use `saddle_chords`. Do not
  introduce any new "Crossroads" usage in prose, doc comments, or test names.

The **only** performance issue today is that `count_geodesics` reruns the full
BFS on every call. We must not loop it 227 times for the distribution; we will
expose the σ vector from one BFS pass.

## 2. New public API surface

All additions live in `crates/mt/src/quintal/geodesics.rs` (or a new sibling
module — see §3 for the recommended file layout). The library deals only in
`PcChord` + `Orbit`; **note-name rendering stays in the CLI / MCP layer**.

```rust
/// One row of the per-target geodesic profile.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicProfileEntry {
    pub chord: PcChord,
    pub orbit: Orbit,
    pub distance: u8,
    pub geodesic_count: u64,   // σ(target | source)
}

/// One row of the aggregate distribution table (one per distance bucket).
///
/// `max_chords` is a `Vec<(PcChord, Orbit)>` because ties are possible — the
/// pair keeps each chord paired with its orbit label, avoiding the "two
/// parallel `Vec`s that must stay the same length" smell.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicBucket {
    pub distance: u8,
    pub chords_at_d: usize,
    pub avg_geodesics: f64,
    pub max_geodesics: u64,
    /// Every chord that ties for max_geodesics at this distance, paired with
    /// its orbit. Sorted by `pcs` ascending for determinism.
    pub max_chords: Vec<(PcChord, Orbit)>,
}

/// Full result: aggregate buckets + per-chord detail + sanity totals.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeodesicDistribution {
    pub source: PcChord,
    pub source_orbit: Orbit,
    pub eccentricity: u8,
    pub reachable_chords: usize,                // expected: 227
    /// Buckets ordered by `distance` ascending. The graph is connected and
    /// every distance 1..=eccentricity is populated for every source we
    /// care about, but the API does not guarantee dense indexing — read
    /// `bucket.distance` rather than indexing by `d - 1`.
    pub buckets: Vec<GeodesicBucket>,
    /// Sorted by (distance asc, geodesic_count desc, pcs asc).
    pub per_chord: Vec<GeodesicProfileEntry>,
}

/// The (distances, geodesic counts) pair returned by a single BFS pass.
///
/// Both maps are keyed by reachable `PcChord`s and have the same length —
/// the source itself is included with `distance = 0` and `geodesic_count = 1`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistAndGeodesicCounts {
    pub distances: HashMap<PcChord, u8>,
    pub geodesic_counts: HashMap<PcChord, u64>,
}

/// Single-pass BFS exposing both distance and σ counts for every reachable chord.
/// Returns `None` if `source` is not a member of `space`.
#[must_use]
pub fn distances_and_geodesic_counts(
    space: &BaseSpace,
    source: &PcChord,
) -> Option<DistAndGeodesicCounts>;

/// High-level: compute the §6 distribution from `source`.
/// Returns `None` if `source` is not a member of `space`.
#[must_use]
pub fn geodesic_distribution(
    space: &BaseSpace,
    source: &PcChord,
) -> Option<GeodesicDistribution>;
```

Notes on the API choices:

- `geodesic_count` is `u64`. The current spec table tops out at 298, but the
  combinatorics scale fast and we want headroom; `usize` would tie us to 32-bit
  on some targets and we may want serialised values to be portable.
- `max_chords` is a `Vec<(PcChord, Orbit)>` so the orbit label travels with
  the chord identity. Two parallel `Vec`s where length-must-equal-length is
  an invariant the type system can enforce — we just use a tuple.
- `per_chord` is materialised. With |B|=228 it costs nothing and the MCP
  wrapper will want it for the "max-geodesic chord at distance d" report.
- `Orbit` is included on every entry so the MCP wrapper can label "Q777" /
  "Saddle (Q686)" without re-running classification.
- `DistAndGeodesicCounts` is a named struct rather than a `(HashMap, HashMap)`
  tuple because two anonymous HashMaps with the same key type are easy to
  swap by accident; the field names make the result self-documenting (TD-04).
- `#[non_exhaustive]` on every public struct so future fields (e.g. a
  `radius` or per-bucket diameter) can be added without a major version bump
  (TD-07).
- **`Orbit`'s actual signature** is `pub fn classify_orbit(chord: &PcChord)
  -> Option<Orbit>` (per `crates/mt/src/quintal/orbit.rs:169`). The companion
  spec doc lists `classify_orbit(&[PcChord]) -> Orbit`, which is wrong on
  both arity and return shape — note for the spec author. Every chord that
  appears in `BaseSpace` is legal by construction, so an `expect("BaseSpace
  members are always classifiable")` is justified at call sites that
  iterate over space members (EH-02 case 1: invariant from constructor).

## 3. File layout — recommended

The geodesics module is already a reasonable home for this work; adding four
public types and two functions to `geodesics.rs` is fine and keeps the API
discoverable. **Do not** create a new top-level module under `quintal/` for
this; the feature is conceptually a query over the existing graph.

Concretely:

- `crates/mt/src/quintal/geodesics.rs` — add the four structs + the two new
  `pub fn`s. Wrap (don't refactor) `bfs_with_parents` per §4 — it stays
  private and index-keyed; the new public path takes care of conversion.
- `crates/mt/src/quintal/mod.rs` — re-export the new symbols alongside the
  existing `pub use geodesics::{count_geodesics, geodesics, passing_chords};`.
- `crates/mt/tests/quintal/test_geodesics.rs` — extend with the cross-checks
  in §6.
- `crates/mt-cli/src/cli.rs` — add a new `OthAction::GeodesicDistribution { … }`
  variant + handler.

## 4. Implementation steps

### 4.1 Wrap `bfs_with_parents` in a public, chord-keyed helper

The internals are correct; the existing code at
`crates/mt/src/quintal/geodesics.rs:18-54` produces `(dist, parents, sigma)` in
the right shape. Two options:

(a) **Keep `bfs_with_parents` as-is** (private) and write a new public
`distances_and_geodesic_counts` that calls it and converts the index-keyed
`Vec`s into chord-keyed `HashMap`s wrapped in a named struct. This is the
lowest-risk path. **Recommended.**

(b) Make `bfs_with_parents` itself `pub(crate)` and rename. Slightly cleaner but
exposes the index-based representation, which leaks `BaseSpace`'s internal
layout. Don't do this unless we discover a third caller.

Pseudocode for (a):

```rust
pub fn distances_and_geodesic_counts(
    space: &BaseSpace,
    source: &PcChord,
) -> Option<DistAndGeodesicCounts> {
    let si = space.chord_index(source)?;
    let (dist, _parents, sigma) = bfs_with_parents(space, si);
    let mut distances = HashMap::with_capacity(space.len());
    let mut geodesic_counts = HashMap::with_capacity(space.len());
    for (i, chord) in space.chords().iter().enumerate() {
        if let Some(d) = dist[i] {
            distances.insert(*chord, d);
            geodesic_counts.insert(*chord, sigma[i] as u64);
        }
    }
    Some(DistAndGeodesicCounts { distances, geodesic_counts })
}
```

Note: `sigma[i]` is `usize`. On 32-bit targets that's `u32`; on 64-bit targets
it's already `u64`. The `as u64` cast widens safely (no truncation possible).

Verification step: confirm that `bfs_with_parents`'s `sigma[source] = 1` is
preserved in the map. The spec uses σ(source | source) = 1 as a base case in
several places; we should not strip it.

### 4.2 Build `geodesic_distribution`

Driven entirely off the result of §4.1. Algorithm:

1. Call `distances_and_geodesic_counts(space, source)?` → unpack
   `DistAndGeodesicCounts { distances, geodesic_counts }`. (`?` short-circuits
   to the outer `Option` return.)
2. Bucket entries by distance into a `BTreeMap<u8, Vec<PcChord>>`, **excluding
   the source itself** (distance 0). `PcChord` is `Copy`, so storing values
   beats threading lifetimes from `distances` for the bucket map.
3. For each bucket, compute:
   - `chords_at_d = bucket.len()`
   - `sum_geodesics = bucket.iter().map(|c| geodesic_counts[c]).sum::<u64>()`
   - `avg_geodesics = sum_geodesics as f64 / chords_at_d as f64`
   - `max_geodesics = bucket.iter().map(|c| geodesic_counts[c]).max()
       .expect("bucket is non-empty by construction")` — `expect` (not
     `unwrap`) per EH-07/AP-09; the message documents the invariant.
   - `max_chords`: every `(chord, orbit)` in the bucket whose σ equals
     `max_geodesics`, with `chord` sorted by `pcs` ascending for determinism.
     Each `orbit` is `classify_orbit(&chord).expect("BaseSpace chords are
     always classifiable")`. **Note the corrected signature** — `classify_orbit`
     takes `&PcChord` and returns `Option<Orbit>` (see §2 note).
4. `eccentricity` = highest distance in the bucket map.
5. `reachable_chords` = sum of `chords_at_d` across buckets (= |B|−1 = 227 on a
   connected graph).
6. Materialise `per_chord` from `distances` + `geodesic_counts`, sorted by
   `(distance ascending, geodesic_count descending, pcs ascending)`. Use
   `slice::sort_by_key` with a tuple `(d, std::cmp::Reverse(σ), pcs)` to get
   the descending second component cleanly.

The whole thing is O(|B| log |B|) once the BFS is done. On |B|=228 it is
sub-millisecond.

Edge case: if `source ∉ space`, return `None`. Don't panic, don't return an
empty `GeodesicDistribution`.

### 4.3 Re-exports

In `crates/mt/src/quintal/mod.rs`, change:

```rust
pub use geodesics::{count_geodesics, geodesics, passing_chords};
```

to:

```rust
pub use geodesics::{
    count_geodesics, distances_and_geodesic_counts, geodesic_distribution,
    geodesics, passing_chords, DistAndGeodesicCounts, GeodesicBucket,
    GeodesicDistribution, GeodesicProfileEntry,
};
```

### 4.4 CLI subcommand

Add to `crates/mt-cli/src/cli.rs`:

```rust
pub enum OthAction {
    Modes        { /* existing */ },
    Orbits,
    ParentScales { /* existing */ },
    Verify,
    Export,
    /// §6 geodesic distribution from a source chord.
    GeodesicDistribution {
        /// Source chord as note names, e.g. "C,G,D,A".
        #[arg(long, conflicts_with = "from_pcs")]
        from: Option<String>,
        /// Source chord as pitch classes, e.g. "0,2,7,9".
        #[arg(long = "from-pcs")]
        from_pcs: Option<String>,
        /// Output format.
        #[arg(long, default_value = "md")]
        format: GeodesicFormat,
    },
}

#[derive(Copy, Clone, ValueEnum)]
pub enum GeodesicFormat { Md, Json }
```

Default source if neither flag is given: C–G–D–A (pcs `[0,2,7,9]`), as the spec
specifies. Reject if both flags are given (clap's `conflicts_with` already
handles this).

Handler steps:

1. Parse `from` / `from_pcs` into a `[u8; 4]`. The note-name parser can reuse
   `pc_to_note_name`'s inverse — since that helper is sharps-only, accept both
   sharp and flat input by normalising `Db → C#`, `Eb → D#`, etc., before
   pc-mapping. Define a small `note_name_to_pc(&str) -> Option<u8>` next to the
   existing `pc_to_note_name` in `cli.rs`. Accept ASCII (`b`, `#`) and Unicode
   (`♭`, `♯`) accidentals; reject double accidentals and anything not in
   `[A-G][b#♭♯]?`. Comma-split on `from`, trim each token, reject any count ≠ 4.
2. Build a `PcChord` via `PcChord::from_unsorted(&pcs)` (which validates range
   and uniqueness; equivalent to `PcChord::new(arr)` after a 4-element check).
3. Call `geodesic_distribution(&space, &source)`. If `None`, return a
   `CliError::Oth("source chord is not in the base space")`.
4. Render. The Markdown formatter extends the §6 paper table with one extra
   column for the max-σ chord(s) (the spec asks for that as a separate output;
   inlining it keeps the human cross-check workflow to a single command):

   ```
   | Distance | Chords at d | Avg geodesics | Max geodesics | Max-σ chord(s) |
   | -------- | :---------: | :-----------: | :-----------: | -------------- |
   | 1        | 8           | 1.0           | 1             | …              |
   ```

   The "Max-σ chord(s)" cell renders each tying chord as
   `"C–G–D–A (Q777)"` (en-dashes between note names, parenthesised orbit).
   The paper uses en-dashes; please match.

   `avg_geodesics` is rendered to one decimal place. `max_geodesics` is integer.

5. JSON output: `serde_json::to_string_pretty(&distribution)?`. The CLI's
   `Cargo.toml` already pulls `music-comp-mt` with `features = ["serde"]` and
   has `serde_json = "1"` as a direct dependency, so no Cargo changes are
   needed here.

CLI invocation, matching the spec:

```
mt oth geodesic-distribution --from "C,G,D,A" --format md
mt oth geodesic-distribution --from-pcs "0,2,7,9" --format json
```

(Note: spec uses `mt-oth`; the existing CLI uses `mt oth`. Defer to whatever
the existing binary calls itself — `mt oth` per the survey. The spec text can
be reconciled in the §6 footnote when we update the paper.)

## 5. Backward compatibility

- `count_geodesics`, `geodesics`, `passing_chords` keep their current signatures
  and behaviour.
- `bfs_with_parents` stays private.
- New symbols are purely additive.
- No changes to `BaseSpace`, `PcChord`, `Orbit`, or `centrality` modules.

## 6. Tests (add to `crates/mt/tests/quintal/test_geodesics.rs`)

Each numbered test below should be its own `#[test]` function; keep them small
and named after what they prove.

The existing test file uses `extern crate music_comp_mt as theory;` — every
new symbol must be reached through `theory::quintal::...`, not `crate::...`.
Extend the existing `use theory::quintal::{ ... }` import block at the top
of `test_geodesics.rs` with `geodesic_distribution`, `transpose`, and
`saddle_chords`.

### 6.1 Sanity totals

```rust
#[test]
fn geodesic_distribution_sums_to_b_minus_1() {
    let space = BaseSpace::new();
    let source = PcChord::new([0, 2, 7, 9]).unwrap(); // C-G-D-A
    let dist = geodesic_distribution(&space, &source).unwrap();
    let total: usize = dist.buckets.iter().map(|b| b.chords_at_d).sum();
    assert_eq!(total, 227);
    assert_eq!(dist.reachable_chords, 227);
}
```

### 6.2 Source matches the §6 "chords at d" column for the Summit

```rust
#[test]
fn cgda_chords_at_d_matches_section_6_table() {
    let space = BaseSpace::new();
    let source = PcChord::new([0, 2, 7, 9]).unwrap();
    let dist = geodesic_distribution(&space, &source).unwrap();
    let counts: Vec<usize> = dist.buckets.iter().map(|b| b.chords_at_d).collect();
    assert_eq!(counts, vec![8, 18, 36, 45, 66, 44, 10]);
}
```

If this test fails, the spec's "chords_at_d" column is wrong and we need to
update the paper rather than the code. (Independent verification: the §6 table
came from an older version of the library; we expect this row to still be
correct, but the test pins it.)

### 6.3 σ-aggregate row matches `count_geodesics` per-target

A consistency test that re-runs the slow path for a small subset and checks
agreement:

```rust
#[test]
fn distribution_geodesic_counts_match_pairwise() {
    let space = BaseSpace::new();
    let source = PcChord::new([0, 2, 7, 9]).unwrap();
    let dist = geodesic_distribution(&space, &source).unwrap();
    for entry in dist.per_chord.iter().take(20) {
        let pairwise = count_geodesics(&space, &source, &entry.chord) as u64;
        assert_eq!(pairwise, entry.geodesic_count, "chord {:?}", entry.chord);
    }
}
```

(20 entries is plenty; full 227 also runs in a fraction of a second.)

### 6.4 Distribution is invariant under transposition

This is the spec's "12 pure quintal stacks should produce identical profiles"
check. The full version is below; CC may also assert against `T1..T11` if it
wants to cover the entire orbit.

```rust
#[test]
fn distribution_invariant_under_transposition() {
    // `transpose` is re-exported from `theory::quintal::group` via the parent
    // module. Already in the import block at the top of the file.
    let space = BaseSpace::new();
    let source = PcChord::new([0, 2, 7, 9]).unwrap();
    let baseline = geodesic_distribution(&space, &source).unwrap();
    for k in 1u8..12 {
        let shifted = transpose(&source, k);
        let candidate = geodesic_distribution(&space, &shifted).unwrap();
        let baseline_buckets: Vec<_> = baseline.buckets.iter()
            .map(|b| (b.distance, b.chords_at_d, b.max_geodesics))
            .collect();
        let candidate_buckets: Vec<_> = candidate.buckets.iter()
            .map(|b| (b.distance, b.chords_at_d, b.max_geodesics))
            .collect();
        assert_eq!(baseline_buckets, candidate_buckets, "T{} broke isometry", k);
    }
}
```

Note: we compare the *aggregate* fields, not the chord identities, because the
identities themselves are translated by `Tk`. (The whole `per_chord` vector
should agree under the bijection `c ↦ T_{-k}(c)`, but we don't need to assert
that here — the aggregate equality is sufficient evidence.)

### 6.5 Saddle profile differs from Summit profile

Pull the Saddle representative from `saddle_chords` instead of hard-coding
`[0,2,6,8]` — that pin couples the test to a canonicalization choice we don't
own. `saddle_chords` returns the 6 highest-betweenness members (currently
the entire Q686 orbit), and any one of them works for this test by
transposition invariance (proven in §6.4). We also assert the chord we picked
is in the Saddle orbit so a future change to `saddle_chords` returning
something else surfaces here, not as a silent test-strength regression.

```rust
#[test]
fn saddle_distribution_differs_from_summit() {
    let space = BaseSpace::new();
    let summit = PcChord::new([0, 2, 7, 9]).unwrap();          // C-G-D-A
    let saddle = *saddle_chords(&space)
        .first()
        .expect("saddle_chords always returns 6 members");
    assert_eq!(
        classify_orbit(&saddle),
        Some(Orbit::Q686),
        "saddle_chords should pick from the Saddle (Q686) orbit",
    );
    let s = geodesic_distribution(&space, &summit).unwrap();
    let q = geodesic_distribution(&space, &saddle).unwrap();
    let s_counts: Vec<usize> = s.buckets.iter().map(|b| b.chords_at_d).collect();
    let q_counts: Vec<usize> = q.buckets.iter().map(|b| b.chords_at_d).collect();
    assert_ne!(s_counts, q_counts);
}
```

(Add `classify_orbit` and `Orbit` to the import block at the top of the
file alongside `saddle_chords`.)

### 6.6 Source not in space → `None`

```rust
#[test]
fn distribution_returns_none_for_chord_not_in_base_space() {
    let space = BaseSpace::new();
    // PcChord::new validates range and uniqueness, so this always succeeds —
    // but [0,1,2,3] has no [6,8]-legal interval ordering, so it is NOT in
    // BaseSpace. That is the case we want to exercise.
    let bogus = PcChord::new([0, 1, 2, 3]).expect("range/unique checks pass");
    assert!(space.chord_index(&bogus).is_none());
    assert!(geodesic_distribution(&space, &bogus).is_none());
}
```

### 6.7 (CLI smoke test, optional) `crates/mt-cli/tests/`

If `mt-cli` already has integration tests (check `crates/mt-cli/tests/`), add a
test that runs the new subcommand with `--format json`, parses the output as
`GeodesicDistribution`, and asserts `reachable_chords == 227`. If the binary
has no test harness today, skip — CC should not introduce one for this single
case.

## 7. Cross-checks the human will run after CC is done

These are not unit tests; Duncan will run them by hand and compare against the
spec. They live in the run-the-binary workflow, not in `cargo test`.

1. `mt oth geodesic-distribution --from "C,G,D,A" --format md` — paste output
   and compare against the §6 paper table. If "chords_at_d" disagrees, the
   library has changed since the table was generated; the table needs updating
   in the paper. If "avg" or "max" disagrees, the paper text claiming
   "226.8 / 298 / 10" is wrong and needs to be replaced with the produced
   values, with the actual maximum-σ chord at d=7 named.

2. `mt oth geodesic-distribution --from-pcs "0,2,6,8" --format md` — Saddle
   profile. We expect a meaningfully different shape (lower max distance, or
   different distribution of σ counts at the high-distance buckets).

3. The transposition-invariance unit test (§6.4) covers the "12 stacks
   identical" check — no extra binary work needed for that.

## 8. Performance contract

- One BFS per call to `geodesic_distribution`. The internal
  `bfs_with_parents` runs in O(|V|+|E|) ≈ O(228 + ~1200) operations. The whole
  function should complete in microseconds. **Do not** loop `count_geodesics`
  per target.
- Memory: 3 `Vec`s of length |V| during BFS, then two `HashMap`s of length
  |V|, then a single materialised `GeodesicDistribution` of size O(|V|). All
  trivially fits.
- No `Rc`/`Arc`, no async, no rayon. Single-threaded straight-line.

## 9. Style and project conventions

Per `mt-rs/CLAUDE.md` and `assets/ai/rust/SKILL.md`:

- **Error shape**: return `Option`, not `Result`. The only failure mode is
  "source not a member of `space`" — same convention as `eccentricity`,
  `distance`, and `count_geodesics` (the closest analog returns `0` on
  missing; we prefer the more explicit `Option` for the new functions). Per
  EH-06, `Option` is right when the absence has one self-evident reason.
- **Doc discipline (DC-01, DC-04)**: every new `pub` item has a doc comment.
  `geodesic_distribution` carries a runnable doctest matching the style of
  the existing `count_geodesics` doctest (verify `reachable_chords == 227`).
  No `# Errors` section needed since we return `Option`, not `Result`. No
  `# Panics` section because none of the new public functions panic on
  reachable input — the `expect`s in §4.2 fire only on invariants the
  function itself just established.
- **Lints (ID-47, AP-65)**: prefer `#[expect(lint, reason = "...")]` over
  `#[allow]` if any lint suppression turns out to be necessary. The
  implementation as planned shouldn't need any.
- **Float formatting**: one decimal place for `avg_geodesics` in markdown
  output, integer for `max_geodesics`.
- Run `cargo build`, `cargo test`, and `cargo clippy` before reporting back.
  The 4 `module_inception` warnings are expected; everything else should be
  clean. Spot-check that the new types don't trigger
  `clippy::struct_excessive_bools`, `clippy::large_types_passed_by_value`,
  or `clippy::needless_collect`.
- **Test layout**: integration tests go in `crates/mt/tests/quintal/`,
  declared via `crates/mt/tests/quintal/mod.rs`. Add the new tests inline
  in `test_geodesics.rs` rather than creating a new file. Reach symbols via
  `theory::quintal::...` (the existing `extern crate music_comp_mt as
  theory;` aliasing).

## 10. What to bring back

When the work is complete, hand back:

1. A diff of the changed files (or the resulting paths and a summary of the
   public API delta).
2. The output of `cargo test --test tests quintal::test_geodesics` showing
   green for the new tests.
3. The output of `mt oth geodesic-distribution --from "C,G,D,A" --format md`
   pasted verbatim, plus the same with `--format json`.
4. The output of `mt oth geodesic-distribution --from-pcs "0,2,6,8" --format md`
   for the Saddle cross-check.
5. A one-paragraph note: did the §6 "chords_at_d" column reproduce (8, 18, 36,
   45, 66, 44, 10)? If yes, what were the corrected `avg_geodesics` and
   `max_geodesics` columns? What chord(s) achieve the maximum σ at distance 7?

That last note is what lets us close the §6 footnote in the paper.

## 11. Out of scope for this CC task

- The MCP wrapper in `ai-music-theory` (`get_oth_geodesic_distribution`) — this
  will be implemented as a separate task once the mt-rs API is stable.
- All-pairs geodesic distribution (per the spec).
- Geodesic distribution in **E** (the extended fiber-bundle space).
- Visualisations.
- Any change to `BaseSpace`, `PcChord`, or `Orbit`.

---

## Appendix A — File-touch summary

| File | Change |
| ---- | ------ |
| `crates/mt/src/quintal/geodesics.rs` | Add 4 structs (`DistAndGeodesicCounts`, `GeodesicProfileEntry`, `GeodesicBucket`, `GeodesicDistribution`), 2 `pub fn`s (`distances_and_geodesic_counts`, `geodesic_distribution`); keep `bfs_with_parents` private; keep all existing public fns intact. |
| `crates/mt/src/quintal/mod.rs` | Extend `pub use geodesics::{…}` re-export list with the 4 new types and 2 new fns. |
| `crates/mt/tests/quintal/test_geodesics.rs` | Add ~6 tests; extend the existing `use theory::quintal::{…}` import block with `geodesic_distribution`, `transpose`, `saddle_chords`, `classify_orbit`, `Orbit`. |
| `crates/mt-cli/src/cli.rs` | New `OthAction::GeodesicDistribution` variant + `GeodesicFormat` enum + handler + tiny `note_name_to_pc` helper. |

`crates/mt-cli/Cargo.toml` already enables `serde` on `music-comp-mt` and
depends on `serde_json` directly — **no Cargo.toml change required**.

No other files should change. No new modules. No new crates.
