//! Nocturne for two pianos (two MIDI channels) - with true polyphony.
//! Run with: cargo run --example nocturne_duo --features midi-playback
//!
//! Setup in Ableton:
//! - Track 1: MIDI From = IAC Driver, Channel = 1, load a bright piano
//! - Track 2: MIDI From = IAC Driver, Channel = 2, load a warm piano

use rust_music_theory::midi::playback::{MidiPorts, MidiPlayer};
use rust_music_theory::midi::{Duration, Velocity, Channel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ports = MidiPorts::list()?;
    if ports.is_empty() {
        println!("No MIDI ports found.");
        return Ok(());
    }

    let mut player = MidiPlayer::connect_index(0)?;
    player.set_tempo(58);

    let ch1 = Channel::new(0).unwrap(); // Melody
    let ch2 = Channel::new(1).unwrap(); // Accompaniment

    let pp = Velocity::new(40).unwrap();
    let p = Velocity::new(55).unwrap();
    let mp = Velocity::new(70).unwrap();
    let mf = Velocity::new(85).unwrap();
    let f = Velocity::new(100).unwrap();

    println!("Nocturne for Two Pianos in E minor");
    println!("Channel 1: Melody");
    println!("Channel 2: Accompaniment\n");

    player.start_clock();

    // Helper: schedule accompaniment arpeggio (4 sixteenths = 1 beat)
    fn arpeggio(player: &mut MidiPlayer, notes: &[u8], vel: Velocity) {
        for &note in notes {
            player.play_note_async(note, Duration::Sixteenth, vel);
        }
    }

    // === Bar 1: Em ===
    let bar_start = player.cursor();

    // Accompaniment (ch2) - arpeggiated E minor
    player.set_channel(ch2);
    arpeggio(&mut player, &[40, 47, 52, 47], pp); // E2, B2, E3, B2
    let ch2_end = player.cursor();

    // Seek back to bar start for melody
    player.seek(bar_start);
    player.set_channel(ch1);
    player.rest_async(Duration::Eighth); // Melody enters after 8th rest
    player.play_note_async(64, Duration::Eighth, mp);  // E4
    player.play_note_async(67, Duration::Eighth, mf);  // G4
    player.play_note_async(71, Duration::Eighth, mp);  // B4
    let ch1_end = player.cursor();

    // Advance to end of bar (whichever is longer)
    player.seek(ch1_end.max(ch2_end));

    // === Bar 2: Em ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[40, 47, 52, 47], pp);
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(72, Duration::Dotted(Box::new(Duration::Quarter)), mf); // C5
    player.play_note_async(71, Duration::Eighth, mp);  // B4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 3: Am ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[45, 52, 57, 52], pp); // A2, E3, A3, E3
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(69, Duration::Quarter, mf);  // A4
    player.play_note_async(67, Duration::Eighth, mp);   // G4
    player.play_note_async(64, Duration::Eighth, p);    // E4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 4: B7 ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[47, 54, 59, 54], p); // B2, F#3, B3, F#3
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(66, Duration::Quarter, mf);  // F#4
    player.play_note_async(64, Duration::Quarter, mp);  // E4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 5: C major - brightness ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[48, 55, 60, 55], mp); // C3, G3, C4, G3
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(67, Duration::Eighth, mf);   // G4
    player.play_note_async(72, Duration::Eighth, f);    // C5
    player.play_note_async(74, Duration::Quarter, f);   // D5
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 6: G major - climax ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[43, 50, 55, 50], mf); // G2, D3, G3, D3
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(76, Duration::Quarter, f);   // E5 - peak!
    player.play_note_async(74, Duration::Eighth, mf);   // D5
    player.play_note_async(72, Duration::Eighth, mp);   // C5
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 7: Am - descending ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[45, 52, 57, 52], p);
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(71, Duration::Eighth, mp);   // B4
    player.play_note_async(69, Duration::Eighth, p);    // A4
    player.play_note_async(67, Duration::Eighth, p);    // G4
    player.play_note_async(66, Duration::Eighth, pp);   // F#4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Bar 8: Em - return home ===
    let bar_start = player.cursor();

    player.set_channel(ch2);
    arpeggio(&mut player, &[40, 47, 52, 47], p);
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.play_note_async(64, Duration::Half, mp);     // E4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Coda: Rising together ===
    let bar_start = player.cursor();

    // Both pianos ascend in parallel
    player.set_channel(ch2);
    for &note in &[40, 47, 52, 59] {
        player.play_note_async(note, Duration::Eighth, pp);
    }
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    for &note in &[64, 71, 76, 83] {
        player.play_note_async(note, Duration::Eighth, pp);
    }
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // === Final chord: E minor - both together ===
    let bar_start = player.cursor();

    // Sustain on both channels
    player.set_channel(ch2);
    player.control_change(64, 127); // Sustain
    player.play_note_async(40, Duration::Whole, pp);  // E2
    player.seek(bar_start);
    player.play_note_async(52, Duration::Whole, pp);  // E3
    let ch2_end = player.cursor();

    player.seek(bar_start);
    player.set_channel(ch1);
    player.control_change(64, 127); // Sustain
    player.play_note_async(64, Duration::Whole, pp);  // E4
    player.seek(bar_start);
    player.play_note_async(67, Duration::Whole, pp);  // G4
    player.seek(bar_start);
    player.play_note_async(71, Duration::Whole, pp);  // B4
    let ch1_end = player.cursor();

    player.seek(ch1_end.max(ch2_end));

    // Wait for everything to finish
    player.wait();
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Release sustain
    player.set_channel(ch1);
    player.control_change(64, 0);
    player.set_channel(ch2);
    player.control_change(64, 0);

    player.stop_clock();

    println!("Fine.");
    Ok(())
}
