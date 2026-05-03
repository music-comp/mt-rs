//! Melody harmonization example with optional MIDI export and functional analysis.
//!
//! Run with:
//!   cargo run -p music-comp-mt --example harmonize_melody
//!   cargo run -p music-comp-mt --example harmonize_melody -- --sort=functional
//!   cargo run -p music-comp-mt --example harmonize_melody -- --export-midi output.mid
//!   cargo run -p music-comp-mt --example harmonize_melody -- --sort=functional --export-midi output.mid

use std::env;
use std::fs;
use std::path::PathBuf;

use midly::num::{u15, u24, u28, u4, u7};
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
};

use music_comp_mt::harmonize::{
    classify_trajectory, harmonize_melody, match_functional_pathways, DualityScope, Harmonization,
    HarmonizeOptions, MelodyInput,
};
use music_comp_mt::quartal::{quartal_inversion_cycle, quartal_root};
use music_comp_mt::quintal::FunctionalRegion;
use music_comp_mt::quintal::{
    classify_orbit, inversion_cycle, pc_to_note_name, quintal_root, VoicedChord,
};

fn midi_to_name(midi: u8) -> String {
    let pc = midi % 12;
    let octave = (midi / 12) as i8 - 1;
    format!("{}{}", pc_to_note_name(pc), octave)
}

fn orbit_and_inversion(chord: &VoicedChord) -> (String, String) {
    let pc_chord = chord.to_pc_chord().expect("valid voiced chord");
    let orbit_label = match classify_orbit(&pc_chord) {
        Some(orbit) => format!("{}", orbit),
        None => "unknown".to_string(),
    };

    let chord_pcs: [u8; 4] = chord.pitches.map(|p| p % 12);

    // Check quintal cycle
    let q_root = quintal_root(&pc_chord, 4).expect("legal chord");
    let q_cycle = inversion_cycle(&q_root);
    if let Some(idx) = q_cycle
        .iter()
        .position(|vc| vc.pitches.map(|p| p % 12) == chord_pcs)
    {
        return (orbit_label, format!("quintal inv {}", idx));
    }

    // Check quartal cycle
    let qv_root = quartal_root(&pc_chord, 4).expect("legal chord");
    let qv_cycle = quartal_inversion_cycle(&qv_root);
    if let Some(idx) = qv_cycle
        .iter()
        .position(|vc| vc.as_voiced().pitches.map(|p| p % 12) == chord_pcs)
    {
        return (orbit_label, format!("quartal inv {}", idx));
    }

    (orbit_label, "unknown inv".to_string())
}

fn print_harmonization(results: &[Harmonization], label: &str) {
    println!("=== {} ===\n", label);
    for (i, h) in results.iter().enumerate() {
        println!(
            "Progression {} (total movement: {} semitones)",
            i + 1,
            h.total_movement
        );
        for (pos, chord) in h.chords.iter().enumerate() {
            let names: Vec<String> = chord.pitches.iter().map(|&p| midi_to_name(p)).collect();
            let (orbit, inv) = orbit_and_inversion(chord);
            println!(
                "  Position {}: {} (Orbit: {}, {})",
                pos + 1,
                names.join("\u{2013}"),
                orbit,
                inv
            );
        }
        if !h.per_step_movements.is_empty() {
            println!("  Step costs: {:?}", h.per_step_movements);
        }
        println!();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Movement,
    Functional,
}

fn parse_sort_mode(args: &[String]) -> Result<SortMode, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix("--sort=") {
            return match value {
                "movement" => Ok(SortMode::Movement),
                "functional" => Ok(SortMode::Functional),
                other => Err(format!(
                    "unknown --sort value: {:?} (expected one of: movement, functional)",
                    other
                )),
            };
        }
    }
    Ok(SortMode::Movement)
}

fn chord_region(chord: &VoicedChord) -> Option<FunctionalRegion> {
    chord
        .to_pc_chord()
        .ok()
        .and_then(|pc| classify_orbit(&pc))
        .map(|orb| orb.functional_region())
}

fn print_harmonization_functional(results: &[Harmonization], label: &str) {
    println!("=== {} ===\n", label);
    for (i, h) in results.iter().enumerate() {
        let pathway_matches = match_functional_pathways(h);
        let trajectory = classify_trajectory(h);

        let pathway_summary: String = if pathway_matches.is_empty() {
            "no pathways".to_string()
        } else {
            pathway_matches
                .iter()
                .map(|m| {
                    format!(
                        "{}@{}\u{2013}{}",
                        m.pathway,
                        m.start_position + 1,
                        m.end_position + 1
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "Progression {} (total movement: {} semitones | {}, {})",
            i + 1,
            h.total_movement,
            pathway_summary,
            trajectory
        );
        for (pos, chord) in h.chords.iter().enumerate() {
            let names: Vec<String> = chord.pitches.iter().map(|&p| midi_to_name(p)).collect();
            let (orbit, inv) = orbit_and_inversion(chord);
            let region = chord_region(chord)
                .map(|r| format!("{}", r))
                .unwrap_or_else(|| "?".to_string());
            println!(
                "  Position {}: {} [{}] (Orbit: {}, {})",
                pos + 1,
                names.join("\u{2013}"),
                region,
                orbit,
                inv
            );
        }
        if !h.per_step_movements.is_empty() {
            println!("  Step costs: {:?}", h.per_step_movements);
        }
        println!();
    }
}

fn export_midi(
    path: &PathBuf,
    melody: &[u8],
    results: &[&Harmonization],
) -> Result<(), Box<dyn std::error::Error>> {
    let ticks_per_quarter = 480u16;
    let eighth_note = (ticks_per_quarter / 2) as u32;
    let melody_len = melody.len();

    // Track 0: conductor (time signature + tempo)
    let mut conductor: Track = Vec::new();

    // Time signature: melody_len / 8
    // Format: nn dd cc bb where nn=numerator, dd=log2(denominator), cc=clocks, bb=32nds
    let time_sig_data = [melody_len as u8, 3, 24, 8]; // N/8
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
            time_sig_data[0],
            time_sig_data[1],
            time_sig_data[2],
            time_sig_data[3],
        )),
    });

    // Tempo: 120 BPM = 500000 microseconds per quarter
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(500_000))),
    });

    // End of conductor track
    conductor.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    // Track 1: Melody (treble clef, channel 0)
    let mut treble: Track = Vec::new();

    // Program change: Acoustic Grand Piano (program 0)
    treble.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Midi {
            channel: u4::new(0),
            message: MidiMessage::ProgramChange {
                program: u7::new(0),
            },
        },
    });

    // Repeat melody for each progression (one measure per progression)
    for _ in 0..results.len() {
        for &pitch in melody.iter() {
            // Note on (delta=0: immediately after previous note-off)
            treble.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOn {
                        key: u7::new(pitch),
                        vel: u7::new(80),
                    },
                },
            });

            // Note off after one eighth note
            treble.push(TrackEvent {
                delta: u28::new(eighth_note),
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

    treble.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    // Track 2: Harmony (bass clef, channel 1)
    let mut bass: Track = Vec::new();

    // Program change: Acoustic Grand Piano (program 0) on channel 1
    bass.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Midi {
            channel: u4::new(1),
            message: MidiMessage::ProgramChange {
                program: u7::new(0),
            },
        },
    });

    for result in results {
        for chord in result.chords.iter() {
            // All 4 chord notes on simultaneously (delta=0 for all)
            for &pitch in chord.pitches.iter() {
                bass.push(TrackEvent {
                    delta: u28::new(0),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(1),
                        message: MidiMessage::NoteOn {
                            key: u7::new(pitch),
                            vel: u7::new(70),
                        },
                    },
                });
            }

            // All 4 notes off after one eighth note
            for (voice_idx, &pitch) in chord.pitches.iter().enumerate() {
                let delta = if voice_idx == 0 { eighth_note } else { 0 };

                bass.push(TrackEvent {
                    delta: u28::new(delta),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(1),
                        message: MidiMessage::NoteOff {
                            key: u7::new(pitch),
                            vel: u7::new(0),
                        },
                    },
                });
            }
        }
    }

    bass.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header {
            format: Format::Parallel,
            timing: Timing::Metrical(u15::new(ticks_per_quarter)),
        },
        tracks: vec![conductor, treble, bass],
    };

    let mut buf = Vec::new();
    smf.write(&mut buf)?;
    fs::write(path, &buf)?;

    println!(
        "Exported {} progressions ({} measures of {}/8) to {:?}",
        results.len(),
        results.len(),
        melody_len,
        path
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let midi_path = args
        .iter()
        .position(|a| a == "--export-midi")
        .map(|i| PathBuf::from(&args[i + 1]));

    let sort_mode = match parse_sort_mode(&args) {
        Ok(mode) => mode,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            std::process::exit(1);
        }
    };

    let melody: Vec<u8> = vec![60, 62, 64, 62, 67, 69, 72];

    let print_fn: fn(&[Harmonization], &str) = match sort_mode {
        SortMode::Movement => print_harmonization,
        SortMode::Functional => print_harmonization_functional,
    };

    // Example A: C4-D4-E4-D4-G4-A4-C5 melody, default options, K=5
    println!(
        "Melody: C4\u{2013}D4\u{2013}E4\u{2013}D4\u{2013}G4\u{2013}A4\u{2013}C5 (MIDI 60, 62, 64, 62, 67, 69, 72)\n"
    );
    let mut results = harmonize_melody(
        MelodyInput::Pitches(melody.clone()),
        HarmonizeOptions {
            k: 5,
            ..Default::default()
        },
    )?;
    if sort_mode == SortMode::Functional {
        results.sort_by(|a, b| {
            let a_matches = match_functional_pathways(a).len();
            let b_matches = match_functional_pathways(b).len();
            b_matches
                .cmp(&a_matches)
                .then(a.total_movement.cmp(&b.total_movement))
        });
    }
    print_fn(&results, "Both dualities (quintal + quartal), K=5");

    // Example B: Same melody, quintal voicings only, K=3
    let mut quintal_results = harmonize_melody(
        MelodyInput::Pitches(melody.clone()),
        HarmonizeOptions {
            k: 3,
            duality: DualityScope::QuintalOnly,
            ..Default::default()
        },
    )?;
    if sort_mode == SortMode::Functional {
        quintal_results.sort_by(|a, b| {
            let a_matches = match_functional_pathways(a).len();
            let b_matches = match_functional_pathways(b).len();
            b_matches
                .cmp(&a_matches)
                .then(a.total_movement.cmp(&b.total_movement))
        });
    }
    print_fn(&quintal_results, "Quintal only, K=3");

    // Example C: Same melody, quartal voicings only, K=3
    let mut quartal_results = harmonize_melody(
        MelodyInput::Pitches(melody.clone()),
        HarmonizeOptions {
            k: 3,
            duality: DualityScope::QuartalOnly,
            ..Default::default()
        },
    )?;
    if sort_mode == SortMode::Functional {
        quartal_results.sort_by(|a, b| {
            let a_matches = match_functional_pathways(a).len();
            let b_matches = match_functional_pathways(b).len();
            b_matches
                .cmp(&a_matches)
                .then(a.total_movement.cmp(&b.total_movement))
        });
    }
    print_fn(&quartal_results, "Quartal only, K=3");

    // MIDI export if requested — includes all progressions from all sections
    if let Some(path) = midi_path {
        let mut all_results: Vec<&Harmonization> = Vec::new();
        all_results.extend(results.iter());
        all_results.extend(quintal_results.iter());
        all_results.extend(quartal_results.iter());
        export_midi(&path, &melody, &all_results)?;
    }

    Ok(())
}
