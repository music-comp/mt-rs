# Phase 3: Quintal Fiber Bundle — Extended Space E & Fiber Bundle

## Context

Implementing Phase 3 of the Quintal Fiber Bundle project from design spec `docs/design/02-under-review/0005-quintal-fiber-bundle-implementation-project-plan.md`, based on the paper `workbench/quintal-harmony-as-a-fiber-bundle.md` (Parts VI and VII).

This phase implements Tymoczko's interscalar transposition (the t1/t-1 operators), the fiber bundle structure E over B, the Universal L1 Law, and the quartal/quintal duality theorem. This is the mathematical heart of the paper.

**Depends on:** Phase 1 (types, enumeration, orbits). Phase 2 is independent of Phase 3 — they can be done in either order.

## File Structure

### New source files (`mt/src/quintal/`)

| File | Purpose | ~Lines |
|------|---------|--------|
| `fiber.rs` | Chord-scale, t1/t-1 operators, inversion cycle, projection | 300 |
| `verification.rs` | Universal L1 Law verification, fiber class computation | 200 |
| `duality.rs` | Quartal/quintal duality, orbit self-duality | 150 |

### New test files (`mt/tests/quintal/`)

| File | ~Lines |
|------|--------|
| `test_fiber.rs` | 300 |
| `test_verification.rs` | 250 |
| `test_duality.rs` | 180 |

### Modified files

- `mt/src/quintal/mod.rs` — add `mod fiber; mod verification; mod duality;` and re-exports
- `mt/tests/quintal/mod.rs` — add `mod test_fiber; mod test_verification; mod test_duality;`

## Milestone 3.1: Chord-Scale & Tymoczko Inversion Operators

### `fiber.rs`

**Public types:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChordScale {
    pub pcs: [u8; 4],     // pitch classes sorted ascending within one octave
    pub steps: [u8; 4],   // step sizes between consecutive chord-scale degrees (cyclic)
}
```

**Public functions:**

```rust
pub fn chord_scale(chord: &VoicedChord) -> ChordScale
pub fn t1(chord: &VoicedChord) -> VoicedChord
pub fn t_minus1(chord: &VoicedChord) -> VoicedChord
pub fn inversion_cycle(chord: &VoicedChord) -> [VoicedChord; 4]
pub fn project(chord: &VoicedChord) -> PcChord
pub fn l1_distance(a: &VoicedChord, b: &VoicedChord) -> u32
```

**`chord_scale(chord)`:**
1. Extract pitch classes: `pcs[i] = pitches[i] % 12` for each voice
2. Sort the 4 pitch classes ascending within [0,11]: `cs = sorted unique pcs`
3. Compute cyclic step sizes: `steps[i] = (cs[(i+1) % 4] - cs[i] + 12) % 12`
4. Steps always sum to 12 (one octave)

**`t1(chord)` — one step up the inversion cycle:**

Critical algorithm (from paper section 20):
1. Compute chord scale `cs` and step sequence
2. For each voice `p[i]` with pitch class `pc_i = p[i] % 12`:
   a. Find index `j` in chord scale where `cs.pcs[j] == pc_i`
   b. Step size = `cs.steps[j]` (interval from this degree to next)
   c. New pitch = `p[i] + step_size`
3. Sort the 4 new pitches into ascending order
4. Construct `VoicedChord` from sorted result

**`t_minus1(chord)` — one step down:**

Reverse of t1. For each voice:
1. Find index `j` in chord scale where `cs.pcs[j] == pc_i`
2. Previous degree index = `(j + 3) % 4` (i.e., `j - 1 mod 4`)
3. Step size down = `cs.steps[(j + 3) % 4]`
4. New pitch = `p[i] - step_size`
5. Sort ascending, construct VoicedChord

**`inversion_cycle(chord)` — all 4 inversions:**
- `[chord, t1(chord), t1(t1(chord)), t1(t1(t1(chord)))]`
- The 4th application of t1 yields the original chord transposed up 12 (T12), NOT included

**`project(chord)` — pi: E -> B:**
- Same as `VoicedChord::to_pc_chord()`. Included here for API clarity — the fiber bundle projection.

**`l1_distance(a, b)` — L1 (Manhattan) distance in pitch space:**
- `|a[0]-b[0]| + |a[1]-b[1]| + |a[2]-b[2]| + |a[3]-b[3]|`
- Uses absolute differences of MIDI pitches (not mod 12)
- This is NOT the graph distance in B — it measures total voice displacement

### Verification values (Milestone 3.1)

**C3-G3-D4-A4 = (48, 55, 62, 69) inversion cycle (from paper section 20):**

| Inversion | Voices | Intervals | Span |
|-----------|--------|-----------|------|
| Root | (48, 55, 62, 69) | (7, 7, 7) | 21 |
| 1st | (50, 57, 67, 72) | (7, 10, 5) | 22 |
| 2nd | (55, 60, 69, 74) | (5, 9, 5) | 19 |
| 3rd | (57, 62, 72, 79) | (5, 10, 7) | 22 |

- Chord scale of {C,D,G,A} = {0,2,7,9}: steps **[2, 5, 2, 3]**
- `t1^4` = T12 for all tested chords: `t1(t1(t1(t1(chord))))` has all pitches +12
- `t_minus1(t1(chord))` has same pitch-class set as `chord`
- `project` maps all 4 inversions to the **same PcChord**
- Only root position has interval structure in [6,8] for this chord

**Crossroads chord C-F#-D-Ab = voiced as (48, 54, 62, 68) inversion cycle:**

| Inversion | Intervals | In [6,8]? |
|-----------|-----------|-----------|
| Root | (6, 8, 6) | yes |
| 1st | (6, 10, 6) | no |
| 2nd | (6, 8, 6) | yes |
| 3rd | (6, 10, 6) | no |

## Milestone 3.2: L1 Distance, Fiber Classification & Universal L1 Law

### `verification.rs`

**Public functions:**

```rust
pub fn inversions_in_base(chord: &VoicedChord) -> Vec<usize>
pub fn inversion_l1_distances(chord: &VoicedChord) -> [u32; 4]
pub fn fiber_class(chord: &PcChord) -> FiberClass
pub fn verify_universal_l1_law(space: &BaseSpace) -> Result<(), Vec<PcChord>>
pub fn verify_fiber_classes(space: &BaseSpace) -> BTreeMap<Orbit, FiberClass>
```

**`inversions_in_base(chord)`:**
- Compute inversion cycle
- For each inversion (index 0-3), check if its `interval_structure().is_legal()`
- Return indices where it's legal

**`inversion_l1_distances(chord)`:**
- Compute inversion cycle: `[inv0, inv1, inv2, inv3]`
- Compute `t1(inv3)` to get root' (= root + T12)
- Return `[l1(inv0,inv1), l1(inv1,inv2), l1(inv2,inv3), l1(inv3,root')]`

**`fiber_class(chord)`:**
- Needs a representative VoicedChord for the PcChord. Construct one by placing at a default register (e.g., starting at MIDI 48).
- Count how many inversions are in [6,8]
- 1 → ClassA, 2 → ClassB

**`verify_universal_l1_law(space)`:**
- For every chord in B (all 228):
  - Construct a VoicedChord in some default register
  - Compute `inversion_l1_distances`
  - Verify pattern is `[12, 12, 12, 36]`
- Return `Ok(())` if all pass, `Err(counterexamples)` otherwise

**`verify_fiber_classes(space)`:**
- For each orbit, pick one representative chord
- Compute its fiber class
- Return mapping Orbit -> FiberClass

### Verification values (Milestone 3.2)

- L1 distance is symmetric and satisfies triangle inequality
- **Universal L1 Law: [12, 12, 12, 36] for ALL 228 chords** (no exceptions)
- Total cycle cost = 12+12+12+36 = **72** for every chord
- L1 between root and 1st inversion of C-G-D-A: **12**
- Fiber class of [P5,P5,P5] orbit (Q777): **ClassA**
- Fiber class of [d5,A5,d5] orbit (Q686): **ClassB**
- Fiber class of [d5,P5,d5] orbit (Q676): **ClassB**
- Fiber class of [d5,A5,A5] orbit (Q688): **ClassB**
- Crossroads chord has exactly **2** inversions in [6,8] (indices 0 and 2)
- Class A orbits: 10 (Q777, Q767, Q787, Q878, Q868, Q776, Q877, Q867, Q876, Q786) — wait, need to actually count: there are 11 Class A + 3 Class B = 14 total. Let me recount from the paper:
  - **Class B (3 orbits):** Q676 [d5,P5,d5], Q686 [d5,A5,d5], Q688 [d5,A5,A5]
  - **Class A (11 orbits):** all remaining

## Milestone 3.3: Quartal/Quintal Duality

### `duality.rs`

**Public functions:**

```rust
pub fn quartal_reading(chord: &VoicedChord) -> IntervalStructure
pub fn quintal_reading(chord: &VoicedChord) -> IntervalStructure
pub fn reverse_interval_structure(is: &IntervalStructure) -> IntervalStructure
pub fn t1_reversal_equivalence(chord: &VoicedChord) -> bool
pub fn orbit_self_duality(orbit: &Orbit, space: &BaseSpace) -> bool
pub fn verify_all_orbits_self_dual(space: &BaseSpace) -> bool
```

**`quartal_reading(chord)`:** Read intervals top-to-bottom (fourths perspective).
- For ascending pitches `[p0, p1, p2, p3]`, quartal reading is `(p3-p2, p2-p1, p1-p0)` — reversed order.
- Actually: the quartal reading is the intervals when read from highest to lowest voice. Equivalently, it's the reversal of the quintal (bottom-to-top) intervals.

**`quintal_reading(chord)`:** Read intervals bottom-to-top (fifths perspective).
- Same as `VoicedChord::interval_structure()`: `(p1-p0, p2-p1, p3-p2)`.

**`reverse_interval_structure(is)`:** `(is.2, is.1, is.0)` — swap first and third.

**`t1_reversal_equivalence(chord)`:**
- Compute t1 cycle: `[inv0, inv1, inv2, inv3]`
- Compute t_minus1 cycle: `[inv0, t_minus1(inv0), t_minus1(t_minus1(inv0)), ...]`
- Verify t_minus1 cycle visits same 4 pitch-class sets as t1 cycle, in reverse order
- Specifically: t_minus1 cycle should be `[inv0, inv3, inv2, inv1]` (as pitch-class sets, modulo octave)

**`orbit_self_duality(orbit, space)`:**
- Pick a representative chord from the orbit
- Compute its interval structure (i1, i2, i3)
- Compute the reversed interval structure (i3, i2, i1)
- Find a chord with the reversed interval structure
- Check that both chords are in the same T/I orbit (i.e., inversion maps one to the other)

**`verify_all_orbits_self_dual(space)`:**
- Check `orbit_self_duality` for all 14 orbits
- Must return true — all 14 are self-dual

### Verification values (Milestone 3.3)

- C-G-D-A quintal reading: **(7, 7, 7)**, quartal reading from top: **(7, 7, 7)** (palindromic, same)
- Interval reversal of (7,7,7) → (7,7,7) — palindromic, same orbit
- Interval reversal of (7,7,6) → (6,7,7) — different T-orbit, **same T/I orbit**
- t_minus1 cycle visits **same 4 chords** as t1 cycle in reverse order
- **All 14 orbits** are self-dual
- The 20 T-orbits collapse to 14 T/I orbits because 6 asymmetric pairs are merged by inversion
- L1 distance pattern **[12, 12, 12, 36]** is identical for both t1 and t_minus1 traversal directions

## Key Verification Values (Phase 3 Summary)

- C-G-D-A inversion cycle matches paper exactly (all 4 voicings and intervals)
- `t1^4 = T12` universally
- Universal L1 Law: **[12, 12, 12, 36]** for all 228 chords, no exceptions
- Total cycle cost: **72** semitones, always
- Fiber classes: **3 ClassB orbits** (Q676, Q686, Q688), **11 ClassA orbits**
- Crossroads Q686 orbit: **2 inversions in [6,8]**, the only degree-8 orbit with this property
- All **14 orbits are self-dual** under interval reversal
- Quartal/quintal duality = **orientation reversal on the Z4 fiber**

## Build Order

1. **Milestone 3.1** first — t1/t_minus1 operators and chord scale are foundation
2. **Milestone 3.2** second — L1 distances and fiber classification require inversion cycles from 3.1
3. **Milestone 3.3** third — duality verification requires both 3.1 (operators) and 3.2 (fiber classes)

Note: Milestone 3.2 also depends on Phase 1's `BaseSpace` and `Orbit` for the verification sweep.

## Verification

```bash
cargo test --test tests quintal::test_fiber
cargo test --test tests quintal::test_verification
cargo test --test tests quintal::test_duality
cargo clippy
cargo fmt --check
```

## Notes

- No new dependencies needed
- The t1 algorithm is the trickiest part — get the chord-scale step lookup right and everything else follows
- `VoicedChord` construction for fiber class computation: use a canonical register (e.g., starting at MIDI 48) by placing the first pc at 48 and adding intervals
- The `l1_distance` function uses signed/absolute arithmetic on MIDI pitches, not modular arithmetic
- Consider adding `Display` impls for `ChordScale` and other new types for debugging convenience
- The paper's section 20 example (C3-G3-D4-A4) is the primary ground truth — every implementation should reproduce it exactly before moving on
