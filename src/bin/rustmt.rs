use clap::{Parser, Subcommand};
use rust_music_theory::chord::Chord;
use rust_music_theory::note::Notes;
use rust_music_theory::scale::{Direction, Scale};
use std::process;

const AVAILABLE_SCALES: [&str; 14] = [
    "Major|Ionian",
    "Minor|Aeolian",
    "Dorian",
    "Phrygian",
    "Lydian",
    "Mixolydian",
    "Locrian",
    "Harmonic Minor",
    "Melodic Minor",
    "Pentatonic Major",
    "Pentatonic Minor",
    "Blues",
    "Chromatic",
    "Whole Tone",
];

const AVAILABLE_CHORDS: [&str; 22] = [
    "Major Triad",
    "Minor Triad",
    "Suspended2 Triad",
    "Suspended4 Triad",
    "Augmented Triad",
    "Diminished Triad",
    "Major Seventh",
    "Minor Seventh",
    "Augmented Seventh",
    "Augmented Major Seventh",
    "Diminished Seventh",
    "Half Diminished Seventh",
    "Minor Major Seventh",
    "Dominant Seventh",
    "Dominant Ninth",
    "Major Ninth",
    "Dominant Eleventh",
    "Major Eleventh",
    "Minor Eleventh",
    "Dominant Thirteenth",
    "Major Thirteenth",
    "Minor Thirteenth",
];

#[derive(Parser)]
#[command(
    name = "rustmt",
    version = env!("CARGO_PKG_VERSION"),
    about = "A music theory guide"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Provides information for the specified scale
    Scale {
        #[command(subcommand)]
        action: Option<ScaleAction>,

        /// Scale args, e.g. "C melodic minor", "D# dorian"
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// List scale in descending order
        #[arg(short, long)]
        descending: bool,
    },
    /// Provides information for the specified chord
    Chord {
        #[command(subcommand)]
        action: Option<ChordAction>,

        /// Chord args, e.g. "C minor", "Ab augmented major seventh"
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ScaleAction {
    /// Prints out the available scales
    List,
}

#[derive(Subcommand)]
enum ChordAction {
    /// Prints out the available chords
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scale { action, args, descending } => {
            if let Some(ScaleAction::List) = action {
                println!("Available Scales:");
                for scale in &AVAILABLE_SCALES {
                    println!(" - {}", scale);
                }
                return;
            }

            if args.is_empty() {
                eprintln!("error: no scale arguments provided");
                eprintln!("usage: rustmt scale <note> <mode>");
                eprintln!("example: rustmt scale C Ionian");
                process::exit(1);
            }

            let scale_args = args.join(" ");
            let direction = if descending {
                Direction::Descending
            } else {
                Direction::Ascending
            };

            match Scale::from_regex_in_direction(&scale_args, direction) {
                Ok(scale) => scale.print_notes(),
                Err(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Chord { action, args } => {
            if let Some(ChordAction::List) = action {
                println!("Available chords:");
                for chord in &AVAILABLE_CHORDS {
                    println!(" - {}", chord);
                }
                return;
            }

            if args.is_empty() {
                eprintln!("error: no chord arguments provided");
                eprintln!("usage: rustmt chord <note> <quality> [number]");
                eprintln!("example: rustmt chord C Major");
                process::exit(1);
            }

            let chord_args = args.join(" ");
            match Chord::from_regex(&chord_args) {
                Ok(chord) => chord.print_notes(),
                Err(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
