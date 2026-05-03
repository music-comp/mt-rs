//! Melody harmonization example.
//!
//! Run with: `cargo run -p music-comp-mt --example harmonize_melody`

use music_comp_mt::harmonize::{
    harmonize_melody, DualityScope, HarmonizeOptions, Harmonization, MelodyInput,
};
use music_comp_mt::quintal::pc_to_note_name;

fn midi_to_name(midi: u8) -> String {
    let pc = midi % 12;
    let octave = (midi / 12) as i8 - 1;
    format!("{}{}", pc_to_note_name(pc), octave)
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
            println!("  Position {}: {}", pos + 1, names.join("\u{2013}"));
        }
        if !h.per_step_movements.is_empty() {
            println!("  Step costs: {:?}", h.per_step_movements);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example A: C4-D4-E4-D4-G4-A4-C5 melody, default options, K=5
    println!("Melody: C4\u{2013}D4\u{2013}E4\u{2013}D4\u{2013}G4\u{2013}A4\u{2013}C5 (MIDI 60, 62, 64, 62, 67, 69, 72)\n");
    let results = harmonize_melody(
        MelodyInput::Pitches(vec![60, 62, 64, 62, 67, 69, 72]),
        HarmonizeOptions {
            k: 5,
            ..Default::default()
        },
    )?;
    print_harmonization(&results, "Both dualities (quintal + quartal), K=5");

    // Example B: Same melody, quartal voicings only, K=3
    let results = harmonize_melody(
        MelodyInput::Pitches(vec![60, 62, 64, 62, 67, 69, 72]),
        HarmonizeOptions {
            k: 3,
            duality: DualityScope::QuartalOnly,
            ..Default::default()
        },
    )?;
    print_harmonization(&results, "Quartal only, K=3");

    Ok(())
}
