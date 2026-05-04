//! Orbit progression example: orbit sequence → voice-led chord progression.
//!
//! Given an input file with one OTH orbit per line, generates a chord
//! progression by greedily picking minimum-L1 voice-led representatives
//! through the sequence, anchored at a user-specified root pitch.
//!
//! Run with:
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file>
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file> --root-pitch C3
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file> --root-pitch C3 --inversion 2
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file> --export-midi out.mid

use std::env;
use std::fs;
use std::path::PathBuf;

use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

use music_comp_mt::note::parse_midi_pitch;
use music_comp_mt::quintal::{
    classify_orbit, inversion_cycle, pc_to_note_name, quintal_root, t1, BaseSpace, Orbit,
    PcChord, VoicedChord,
};
use music_comp_mt::voice_leading::min_voiced_chord_l1;

fn midi_to_name(midi: u8) -> String {
    let pc = midi % 12;
    let octave = (midi / 12) as i8 - 1;
    format!("{}{}", pc_to_note_name(pc), octave)
}

fn parse_input_file(path: &str) -> Result<Vec<(Orbit, usize)>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path, e))?;
    let mut orbits = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let orbit: Orbit = trimmed
            .parse()
            .map_err(|e| format!("line {}: {}", line_num + 1, e))?;
        orbits.push((orbit, line_num + 1));
    }
    if orbits.is_empty() {
        return Err("input file contains no orbit lines".to_string());
    }
    Ok(orbits)
}

fn chords_in_orbit(orbit: Orbit, space: &BaseSpace) -> Vec<PcChord> {
    space
        .chords()
        .iter()
        .filter(|pc| classify_orbit(pc) == Some(orbit))
        .copied()
        .collect()
}

fn place_first_chord(
    orbit: Orbit,
    root_pitch: u8,
    space: &BaseSpace,
) -> Result<VoicedChord, String> {
    let root_pc = root_pitch % 12;
    let root_octave = root_pitch / 12;
    let orbit_chords = chords_in_orbit(orbit, space);

    // Find all PcChords in the orbit whose quintal root starts at root_pc
    // (i.e., quintal_root(pc, n).pitches[0] % 12 == root_pc)
    let mut matching_chords: Vec<&PcChord> = Vec::new();
    for pc_chord in &orbit_chords {
        let root = quintal_root(pc_chord, 4)
            .expect("all BaseSpace chords have legal stackings");
        if root.pitches[0] % 12 == root_pc {
            matching_chords.push(pc_chord);
        }
    }

    if matching_chords.is_empty() {
        // Fall back: find chords that at least contain root_pc somewhere
        let has_pc: Vec<&PcChord> = orbit_chords
            .iter()
            .filter(|pc| pc.pcs.contains(&root_pc))
            .collect();
        if has_pc.is_empty() {
            let valid_pcs: std::collections::BTreeSet<u8> = orbit_chords
                .iter()
                .flat_map(|pc| pc.pcs.iter().copied())
                .collect();
            let names: Vec<&str> = valid_pcs.iter().map(|&pc| pc_to_note_name(pc)).collect();
            return Err(format!(
                "orbit {:?} has no chord containing PC {}; valid PCs: {}",
                orbit,
                pc_to_note_name(root_pc),
                names.join(", ")
            ));
        }
        // Use the lex-smallest chord containing root_pc; root position starts
        // from a different PC, so we build root and find the inversion with
        // root_pc as bottom voice
        let pc_chord = *has_pc.iter().min_by_key(|pc| pc.pcs).unwrap();
        let root = quintal_root(pc_chord, root_octave)
            .expect("all BaseSpace chords have legal stackings");
        let cycle = inversion_cycle(&root);
        // Find the inversion whose bottom voice has root_pc
        let chosen = cycle
            .iter()
            .find(|inv| inv.pitches[0] % 12 == root_pc)
            .ok_or_else(|| format!(
                "orbit {:?} has no inversion with bottom-voice {}",
                orbit, pc_to_note_name(root_pc)
            ))?;
        let delta = root_pitch as i32 - chosen.pitches[0] as i32;
        let shifted = chosen.pitches.map(|p| (p as i32 + delta) as u8);
        if shifted.iter().any(|&p| p > 127) {
            return Err(format!(
                "root pitch {} places chord out of MIDI range",
                midi_to_name(root_pitch)
            ));
        }
        return Ok(VoicedChord { pitches: shifted });
    }

    // Pick the lexicographically smallest PcChord whose quintal root starts at root_pc
    matching_chords.sort_by_key(|pc| pc.pcs);
    let pc_chord = matching_chords[0];

    // Build the quintal root at the requested octave (always root position)
    let root = quintal_root(pc_chord, root_octave)
        .expect("all BaseSpace chords have legal stackings");

    // Verify all pitches are in MIDI range
    if root.pitches.iter().any(|&p| p > 127) {
        return Err(format!(
            "root pitch {} places chord out of MIDI range",
            midi_to_name(root_pitch),
        ));
    }
    Ok(root)
}

fn pick_next_chord(
    prev: &VoicedChord,
    orbit: Orbit,
    space: &BaseSpace,
) -> Result<VoicedChord, String> {
    let orbit_chords = chords_in_orbit(orbit, space);
    let low_octave = (prev.pitches[0] / 12).saturating_sub(1);
    let high_octave = (prev.pitches[3] / 12) + 1;

    let mut best: Option<(u32, VoicedChord, [u8; 4])> = None;

    for pc_chord in &orbit_chords {
        for base_octave in low_octave..=high_octave {
            let root = match quintal_root(pc_chord, base_octave) {
                Ok(r) => r,
                Err(_) => continue,
            };
            // Only consider root position (inversion 0) for voice-leading
            if root.pitches.iter().any(|&p| p > 127) {
                continue;
            }
            let cost = min_voiced_chord_l1(prev, &root);
            let dominated = match &best {
                Some((best_cost, best_vc, best_pcs)) => {
                    cost < *best_cost
                        || (cost == *best_cost && root.pitches[0] < best_vc.pitches[0])
                        || (cost == *best_cost
                            && root.pitches[0] == best_vc.pitches[0]
                            && pc_chord.pcs < *best_pcs)
                }
                None => true,
            };
            if dominated {
                best = Some((cost, root, pc_chord.pcs));
            }
        }
    }

    best.map(|(_, vc, _)| vc)
        .ok_or_else(|| format!("no valid candidate in orbit {:?}", orbit))
}

fn export_midi(
    path: &PathBuf,
    chords: &[VoicedChord],
    bpm: u32,
    duration_beats: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let ticks_per_quarter: u16 = 480;
    let note_duration = ticks_per_quarter as u32 * duration_beats;

    let mut conductor: Track = Vec::new();
    // Time signature: 4/4
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });
    // Tempo
    let microseconds_per_beat = 60_000_000 / bpm;
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(microseconds_per_beat))),
    });
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let mut track: Track = Vec::new();
    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Midi {
            channel: u4::new(0),
            message: MidiMessage::ProgramChange { program: u7::new(0) },
        },
    });

    for chord in chords {
        // Note-on for all 4 voices
        for &pitch in chord.pitches.iter() {
            track.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOn {
                        key: u7::new(pitch),
                        vel: u7::new(80),
                    },
                },
            });
        }
        // Note-off after duration
        for (i, &pitch) in chord.pitches.iter().enumerate() {
            let delta = if i == 0 { note_duration } else { 0 };
            track.push(TrackEvent {
                delta: u28::new(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOff {
                        key: u7::new(pitch),
                        vel: u7::new(0),
                    },
                },
            });
        }
    }

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header {
            format: Format::Parallel,
            timing: Timing::Metrical(u15::new(ticks_per_quarter)),
        },
        tracks: vec![conductor, track],
    };

    let mut buf = Vec::new();
    smf.write(&mut buf)?;
    fs::write(path, &buf)?;
    println!("Exported {} chords to {:?}", chords.len(), path);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let input_path = args
        .iter()
        .position(|a| a == "--input")
        .map(|i| args[i + 1].clone())
        .ok_or("--input <path> is required")?;

    let root_pitch_str = args
        .iter()
        .position(|a| a == "--root-pitch")
        .map(|i| args[i + 1].as_str())
        .unwrap_or("C3");

    let midi_path = args
        .iter()
        .position(|a| a == "--export-midi")
        .map(|i| PathBuf::from(&args[i + 1]));

    let duration_str = args
        .iter()
        .position(|a| a == "--duration")
        .map(|i| args[i + 1].as_str())
        .unwrap_or("whole");

    let bpm: u32 = args
        .iter()
        .position(|a| a == "--bpm")
        .map(|i| args[i + 1].parse().expect("--bpm must be an integer"))
        .unwrap_or(120);

    let inversion: usize = args
        .iter()
        .position(|a| a == "--inversion")
        .map(|i| {
            let v: usize = args[i + 1].parse().expect("--inversion must be 0, 1, 2, or 3");
            if v > 3 {
                eprintln!("--inversion must be 0, 1, 2, or 3 (got {})", v);
                std::process::exit(1);
            }
            v
        })
        .unwrap_or(0);

    let duration_beats: u32 = match duration_str {
        "whole" => 4,
        "half" => 2,
        "quarter" => 1,
        other => {
            eprintln!("unknown --duration: {:?} (expected: whole, half, quarter)", other);
            std::process::exit(1);
        }
    };

    let root_midi = parse_midi_pitch(root_pitch_str)
        .map_err(|_| format!("invalid --root-pitch: {:?}", root_pitch_str))?;

    let orbit_sequence = parse_input_file(&input_path)?;
    let space = BaseSpace::new();

    // Build the progression
    let mut progression: Vec<VoicedChord> = Vec::with_capacity(orbit_sequence.len());

    let first = place_first_chord(orbit_sequence[0].0, root_midi, &space)?;
    progression.push(first);

    for i in 1..orbit_sequence.len() {
        let next = pick_next_chord(&progression[i - 1], orbit_sequence[i].0, &space)?;
        progression.push(next);
    }

    // Apply inversion transform: t1 applied N times to each chord
    if inversion > 0 {
        for chord in progression.iter_mut() {
            for _ in 0..inversion {
                *chord = t1(chord);
            }
        }
    }

    // Print the progression
    let mut total_cost: u32 = 0;
    for (i, chord) in progression.iter().enumerate() {
        let orbit = orbit_sequence[i].0;
        let region = orbit.functional_region();
        let names: Vec<String> = chord.pitches.iter().map(|&p| midi_to_name(p)).collect();
        let step_cost = if i > 0 {
            let c = min_voiced_chord_l1(&progression[i - 1], chord);
            total_cost += c;
            format!("  (L1: {})", c)
        } else {
            String::new()
        };
        println!(
            "Position {}: {} ({}, {}){}",
            i + 1,
            names.join("\u{2013}"),
            orbit,
            region,
            step_cost
        );
    }
    println!("\nTotal voice-leading cost: {} semitones", total_cost);

    // MIDI export
    if let Some(path) = midi_path {
        export_midi(&path, &progression, bpm, duration_beats)?;
    }

    Ok(())
}
