use crate::chord::Chord;
use crate::note::Notes;
use crate::scale::{Direction, Scale};
use clap::{Parser, Subcommand};
use std::fmt;

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
    name = "mt",
    version = env!("CARGO_PKG_VERSION"),
    about = "A music theory command-line tool"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum ScaleAction {
    /// Prints out the available scales
    List,
}

#[derive(Subcommand)]
pub enum ChordAction {
    /// Prints out the available chords
    List,
}

/// CLI error type.
#[derive(Debug)]
pub enum CliError {
    Scale(String),
    Chord(String),
    MissingArgs(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Scale(msg) => write!(f, "{}", msg),
            CliError::Chord(msg) => write!(f, "{}", msg),
            CliError::MissingArgs(msg) => write!(f, "{}", msg),
        }
    }
}

/// Run the CLI with the parsed arguments. Returns Ok(output) or Err(error).
pub fn run(cli: Cli) -> Result<String, CliError> {
    match cli.command {
        Commands::Scale { action, args, descending } => run_scale(action, args, descending),
        Commands::Chord { action, args } => run_chord(action, args),
    }
}

fn run_scale(
    action: Option<ScaleAction>,
    args: Vec<String>,
    descending: bool,
) -> Result<String, CliError> {
    if let Some(ScaleAction::List) = action {
        let mut output = String::from("Available Scales:\n");
        for scale in &AVAILABLE_SCALES {
            output.push_str(&format!(" - {}\n", scale));
        }
        return Ok(output);
    }

    if args.is_empty() {
        return Err(CliError::MissingArgs(
            "no scale arguments provided\nusage: mt scale <note> <mode>\nexample: mt scale C Ionian".into(),
        ));
    }

    let scale_args = args.join(" ");
    let direction = if descending {
        Direction::Descending
    } else {
        Direction::Ascending
    };

    match Scale::from_regex_in_direction(&scale_args, direction) {
        Ok(scale) => Ok(scale.format_notes()),
        Err(e) => Err(CliError::Scale(format!("{}", e))),
    }
}

fn run_chord(action: Option<ChordAction>, args: Vec<String>) -> Result<String, CliError> {
    if let Some(ChordAction::List) = action {
        let mut output = String::from("Available chords:\n");
        for chord in &AVAILABLE_CHORDS {
            output.push_str(&format!(" - {}\n", chord));
        }
        return Ok(output);
    }

    if args.is_empty() {
        return Err(CliError::MissingArgs(
            "no chord arguments provided\nusage: mt chord <note> <quality> [number]\nexample: mt chord C Major".into(),
        ));
    }

    let chord_args = args.join(" ");
    match Chord::from_regex(&chord_args) {
        Ok(chord) => Ok(chord.format_notes()),
        Err(e) => Err(CliError::Chord(format!("{}", e))),
    }
}
