//! Orbit progression example: orbit sequence → voice-led chord progression.
//!
//! Given an input file with one OTH orbit per line, generates a chord
//! progression by greedily picking minimum-L1 voice-led representatives
//! through the sequence, anchored at a user-specified starting pitch.
//!
//! Run with:
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file>
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file> --starting-pitch C3
//!   cargo run -p music-comp-mt --example orbit_progression -- --input <file> --export-midi out.mid

use std::env;
use std::fs;
use std::path::PathBuf;

use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

use music_comp_mt::note::parse_midi_pitch;
use music_comp_mt::quintal::{
    classify_orbit, inversion_cycle, pc_to_note_name, quintal_root, BaseSpace, Orbit, PcChord,
    VoicedChord,
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
    p_start: u8,
    space: &BaseSpace,
) -> Result<VoicedChord, String> {
    let target_pc = p_start % 12;
    let orbit_chords = chords_in_orbit(orbit, space);

    let mut candidates: Vec<(VoicedChord, PcChord, usize)> = Vec::new();

    for pc_chord in &orbit_chords {
        let root = quintal_root(pc_chord, 4)
            .expect("all BaseSpace chords have legal stackings");
        let cycle = inversion_cycle(&root);
        for (inv_idx, inv) in cycle.iter().enumerate() {
            if inv.pitches[0] % 12 == target_pc {
                candidates.push((*inv, *pc_chord, inv_idx));
            }
        }
    }

    if candidates.is_empty() {
        let valid_pcs: std::collections::BTreeSet<u8> = orbit_chords
            .iter()
            .flat_map(|pc| {
                let root = quintal_root(pc, 4).unwrap();
                let cycle = inversion_cycle(&root);
                cycle.iter().map(|inv| inv.pitches[0] % 12).collect::<Vec<_>>()
            })
            .collect();
        let names: Vec<&str> = valid_pcs.iter().map(|&pc| pc_to_note_name(pc)).collect();
        return Err(format!(
            "orbit {:?} has no chord with bottom-voice {}; valid bottom-voice PCs: {}",
            orbit,
            pc_to_note_name(target_pc),
            names.join(", ")
        ));
    }

    // Sort candidates deterministically: prefer root position (inv 0), then smallest
    // PcChord (lex), then smallest inv index
    candidates.sort_by(|a, b| a.2.cmp(&b.2).then(a.1.pcs.cmp(&b.1.pcs)));
    let (chosen, _, _) = candidates[0];

    // Octave-align so bottom voice = p_start exactly
    let delta = p_start as i32 - chosen.pitches[0] as i32;
    let shifted = chosen.pitches.map(|p| (p as i32 + delta) as u8);
    if shifted.iter().any(|&p| p > 127) {
        return Err(format!(
            "starting pitch {} places chord out of MIDI range",
            midi_to_name(p_start)
        ));
    }
    Ok(VoicedChord { pitches: shifted })
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
            for inv in inversion_cycle(&root) {
                if inv.pitches.iter().any(|&p| p > 127) {
                    continue;
                }
                let cost = min_voiced_chord_l1(prev, &inv);
                let dominated = match &best {
                    Some((best_cost, best_vc, best_pcs)) => {
                        cost < *best_cost
                            || (cost == *best_cost && inv.pitches[0] < best_vc.pitches[0])
                            || (cost == *best_cost
                                && inv.pitches[0] == best_vc.pitches[0]
                                && pc_chord.pcs < *best_pcs)
                    }
                    None => true,
                };
                if dominated {
                    best = Some((cost, inv, pc_chord.pcs));
                }
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

    let starting_pitch_str = args
        .iter()
        .position(|a| a == "--starting-pitch")
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

    let duration_beats: u32 = match duration_str {
        "whole" => 4,
        "half" => 2,
        "quarter" => 1,
        other => {
            eprintln!("unknown --duration: {:?} (expected: whole, half, quarter)", other);
            std::process::exit(1);
        }
    };

    let p_start = parse_midi_pitch(starting_pitch_str)
        .map_err(|_| format!("invalid --starting-pitch: {:?}", starting_pitch_str))?;

    let orbit_sequence = parse_input_file(&input_path)?;
    let space = BaseSpace::new();

    // Build the progression
    let mut progression: Vec<VoicedChord> = Vec::with_capacity(orbit_sequence.len());

    let first = place_first_chord(orbit_sequence[0].0, p_start, &space)?;
    progression.push(first);

    for i in 1..orbit_sequence.len() {
        let next = pick_next_chord(&progression[i - 1], orbit_sequence[i].0, &space)?;
        progression.push(next);
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
