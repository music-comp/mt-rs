//! Geodesic progression example: shortest chord-graph path between two endpoints.
//!
//! Given two endpoint chords (orbit + pitch), finds the shortest chord-graph
//! geodesic and renders it as a voice-led progression with optional MIDI export.
//!
//! Run with:
//!   cargo run -p music-comp-mt --example geodesic -- \
//!       --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3
//!   cargo run -p music-comp-mt --example geodesic -- \
//!       --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3 \
//!       --allow-inversions --export-midi out.mid
//!   cargo run -p music-comp-mt --example geodesic -- \
//!       --start-orbit Q777 --start-pitch C3 --end-orbit Q686 --end-pitch C3 --list-all

use std::env;
use std::fs;
use std::path::PathBuf;

use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

use music_comp_mt::note::parse_midi_pitch;
use music_comp_mt::quintal::{
    classify_orbit, distance, geodesics, inversion_cycle, pc_to_note_name, quintal_root,
    BaseSpace, Orbit, PcChord, VoicedChord,
};
use music_comp_mt::voice_leading::min_voiced_chord_l1;

fn midi_to_name(midi: u8) -> String {
    let pc = midi % 12;
    let octave = (midi / 12) as i8 - 1;
    format!("{}{}", pc_to_note_name(pc), octave)
}

fn chords_in_orbit(orbit: Orbit, space: &BaseSpace) -> Vec<PcChord> {
    space
        .chords()
        .iter()
        .filter(|pc| classify_orbit(pc) == Some(orbit))
        .copied()
        .collect()
}

fn resolve_endpoint(
    orbit: Orbit,
    pitch: u8,
    allow_inversions: bool,
    space: &BaseSpace,
) -> Result<(PcChord, VoicedChord), String> {
    let target_pc = pitch % 12;
    let target_octave = pitch / 12;
    let orbit_chords = chords_in_orbit(orbit, space);

    if allow_inversions {
        // Any inversion can match — find a PcChord containing target_pc
        for pc_chord in &orbit_chords {
            if !pc_chord.pcs.contains(&target_pc) {
                continue;
            }
            let root = quintal_root(pc_chord, target_octave)
                .expect("all BaseSpace chords have legal stackings");
            let cycle = inversion_cycle(&root);
            for inv in &cycle {
                if inv.pitches[0] % 12 == target_pc {
                    let delta = pitch as i32 - inv.pitches[0] as i32;
                    let shifted = inv.pitches.map(|p| (p as i32 + delta) as u8);
                    if shifted.iter().all(|&p| p <= 127) {
                        return Ok((*pc_chord, VoicedChord { pitches: shifted }));
                    }
                }
            }
        }
        let valid_pcs: std::collections::BTreeSet<u8> = orbit_chords
            .iter()
            .flat_map(|pc| pc.pcs.iter().copied())
            .collect();
        let names: Vec<&str> = valid_pcs.iter().map(|&pc| pc_to_note_name(pc)).collect();
        Err(format!(
            "orbit {:?} has no chord with bottom-voice {} (with inversions); valid PCs: {}",
            orbit,
            pc_to_note_name(target_pc),
            names.join(", ")
        ))
    } else {
        // Root position only — quintal_root bottom voice must match
        for pc_chord in &orbit_chords {
            let root = quintal_root(pc_chord, target_octave)
                .expect("all BaseSpace chords have legal stackings");
            if root.pitches[0] % 12 == target_pc {
                let delta = pitch as i32 - root.pitches[0] as i32;
                let shifted = root.pitches.map(|p| (p as i32 + delta) as u8);
                if shifted.iter().all(|&p| p <= 127) {
                    return Ok((*pc_chord, VoicedChord { pitches: shifted }));
                }
            }
        }
        // Error with valid root-position bottom PCs
        let valid_pcs: std::collections::BTreeSet<u8> = orbit_chords
            .iter()
            .filter_map(|pc| {
                let root = quintal_root(pc, 4).ok()?;
                Some(root.pitches[0] % 12)
            })
            .collect();
        let names: Vec<&str> = valid_pcs.iter().map(|&pc| pc_to_note_name(pc)).collect();
        Err(format!(
            "orbit {:?} in root position has no chord with bottom-voice {}; valid bottom PCs: {}",
            orbit,
            pc_to_note_name(target_pc),
            names.join(", ")
        ))
    }
}

fn render_geodesic(
    path: &[PcChord],
    start_vc: &VoicedChord,
    end_vc: &VoicedChord,
    allow_inversions: bool,
) -> (Vec<VoicedChord>, u32) {
    if path.len() <= 1 {
        return (vec![*start_vc], 0);
    }
    if path.len() == 2 {
        let cost = min_voiced_chord_l1(start_vc, end_vc);
        return (vec![*start_vc, *end_vc], cost);
    }

    let mut voicings: Vec<VoicedChord> = Vec::with_capacity(path.len());
    voicings.push(*start_vc);

    // Render intermediate positions (1..len-1)
    for i in 1..path.len() - 1 {
        let prev = voicings.last().unwrap();
        let pc_chord = &path[i];
        let low_octave = (prev.pitches[0] / 12).saturating_sub(1);
        let high_octave = (prev.pitches[3] / 12) + 1;

        let mut best: Option<(u32, VoicedChord)> = None;

        for base_octave in low_octave..=high_octave {
            let root = match quintal_root(pc_chord, base_octave) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let candidates: Vec<VoicedChord> = if allow_inversions {
                inversion_cycle(&root).to_vec()
            } else {
                vec![root]
            };

            for candidate in candidates {
                if candidate.pitches.iter().any(|&p| p > 127) {
                    continue;
                }
                let cost = min_voiced_chord_l1(prev, &candidate);
                let dominated = match &best {
                    Some((best_cost, best_vc)) => {
                        cost < *best_cost
                            || (cost == *best_cost && candidate.pitches[0] < best_vc.pitches[0])
                    }
                    None => true,
                };
                if dominated {
                    best = Some((cost, candidate));
                }
            }
        }

        voicings.push(best.expect("at least one valid voicing exists").1);
    }

    // Final position: use the resolved end voicing
    voicings.push(*end_vc);

    // Compute total L1
    let total: u32 = voicings
        .windows(2)
        .map(|w| min_voiced_chord_l1(&w[0], &w[1]))
        .sum();

    (voicings, total)
}

fn print_geodesic(voicings: &[VoicedChord], _total_cost: u32, label: &str) {
    println!("{}", label);
    for (i, chord) in voicings.iter().enumerate() {
        let names: Vec<String> = chord.pitches.iter().map(|&p| midi_to_name(p)).collect();
        let pc_chord = chord.to_pc_chord().expect("valid chord");
        let orbit = classify_orbit(&pc_chord)
            .map(|o| format!("{}", o))
            .unwrap_or_else(|| "?".to_string());
        let region = classify_orbit(&pc_chord)
            .map(|o| format!("{}", o.functional_region()))
            .unwrap_or_else(|| "?".to_string());
        let step_cost = if i > 0 {
            let c = min_voiced_chord_l1(&voicings[i - 1], chord);
            format!("  (L1: {})", c)
        } else {
            String::new()
        };
        println!(
            "  Position {}: {} ({}, {}){}",
            i + 1,
            names.join("\u{2013}"),
            orbit,
            region,
            step_cost
        );
    }
    println!();
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
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });
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

    let get_flag = |name: &str| -> Option<&str> {
        args.iter()
            .position(|a| a == name)
            .map(|i| args[i + 1].as_str())
    };

    let start_orbit_str = get_flag("--start-orbit").ok_or("--start-orbit is required")?;
    let start_pitch_str = get_flag("--start-pitch").ok_or("--start-pitch is required")?;
    let end_orbit_str = get_flag("--end-orbit").ok_or("--end-orbit is required")?;
    let end_pitch_str = get_flag("--end-pitch").ok_or("--end-pitch is required")?;

    let allow_inversions = args.iter().any(|a| a == "--allow-inversions");
    let list_all = args.iter().any(|a| a == "--list-all");

    let midi_path = get_flag("--export-midi").map(PathBuf::from);

    let duration_str = get_flag("--duration").unwrap_or("whole");
    let bpm: u32 = get_flag("--bpm")
        .map(|s| s.parse().expect("--bpm must be an integer"))
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

    let start_orbit: Orbit = start_orbit_str
        .parse()
        .map_err(|e| format!("invalid --start-orbit: {}", e))?;
    let end_orbit: Orbit = end_orbit_str
        .parse()
        .map_err(|e| format!("invalid --end-orbit: {}", e))?;
    let start_pitch = parse_midi_pitch(start_pitch_str)
        .map_err(|_| format!("invalid --start-pitch: {:?}", start_pitch_str))?;
    let end_pitch = parse_midi_pitch(end_pitch_str)
        .map_err(|_| format!("invalid --end-pitch: {:?}", end_pitch_str))?;

    let space = BaseSpace::new();

    // Resolve endpoints
    let (start_pc, start_vc) = resolve_endpoint(start_orbit, start_pitch, allow_inversions, &space)?;
    let (end_pc, end_vc) = resolve_endpoint(end_orbit, end_pitch, allow_inversions, &space)?;

    // Find geodesics
    let dist = distance(&space, &start_pc, &end_pc);
    let paths = geodesics(&space, &start_pc, &end_pc);

    let start_names: Vec<String> = start_vc.pitches.iter().map(|&p| midi_to_name(p)).collect();
    let end_names: Vec<String> = end_vc.pitches.iter().map(|&p| midi_to_name(p)).collect();

    println!("Start: {} ({}, PCs {:?})", start_names.join("\u{2013}"), start_orbit, start_pc.pcs);
    println!("End:   {} ({}, PCs {:?})", end_names.join("\u{2013}"), end_orbit, end_pc.pcs);
    println!(
        "Chord-graph distance: {} edge{}",
        dist.unwrap_or(0),
        if dist == Some(1) { "" } else { "s" }
    );
    println!("Geodesics found: {}\n", paths.len());

    if paths.is_empty() {
        println!("No geodesic found (chords may not be in the base space).");
        return Ok(());
    }

    // Render all geodesics
    let mut rendered: Vec<(Vec<VoicedChord>, u32)> = paths
        .iter()
        .map(|path| render_geodesic(path, &start_vc, &end_vc, allow_inversions))
        .collect();

    // Sort by total cost
    rendered.sort_by_key(|(_, cost)| *cost);

    if list_all {
        for (i, (voicings, total)) in rendered.iter().enumerate() {
            print_geodesic(
                voicings,
                *total,
                &format!("Geodesic {} (total voice movement: {} semitones):", i + 1, total),
            );
        }
    } else {
        let (best_voicings, best_cost) = &rendered[0];
        print_geodesic(
            best_voicings,
            *best_cost,
            &format!("Best geodesic (total voice movement: {} semitones):", best_cost),
        );
    }

    // MIDI export (best geodesic only)
    if let Some(path) = midi_path {
        let (best_voicings, _) = &rendered[0];
        export_midi(&path, best_voicings, bpm, duration_beats)?;
    }

    Ok(())
}
