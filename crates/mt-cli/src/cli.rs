use clap::{Parser, Subcommand, ValueEnum};
use music_comp_mt::chord::Chord;
use music_comp_mt::note::Notes;
use music_comp_mt::quintal::{
    all_modes, geodesic_distribution, modes_by_opening_interval, modes_in_cluster, orbit_modes,
    step_vocabulary_cluster, verify_fiber_mode_connection, verify_multiset_uniqueness, BaseSpace,
    GeodesicDistribution, Orbit, PcChord, StepVocabularyCluster,
};
use music_comp_mt::scale::{Direction, Scale};
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
    /// OTH (Open Tone Harmony) mode and scale analysis
    Oth {
        #[command(subcommand)]
        action: OthAction,
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

#[derive(Subcommand)]
pub enum OthAction {
    /// List all OTH modes across 14 orbits
    Modes {
        /// Filter by orbit (e.g., Q777)
        #[arg(long)]
        orbit: Option<String>,
        /// Filter by opening interval
        #[arg(long)]
        opening: Option<u8>,
    },
    /// Summary of all 14 orbits with step-size multisets
    Orbits,
    /// Parent scale analysis for orbits
    ParentScales {
        /// Filter by orbit (e.g., Q777)
        #[arg(long)]
        orbit: Option<String>,
    },
    /// Run all verification checks
    Verify,
    /// Full JSON export of all mode data
    Export,
    /// §6 geodesic-distribution profile from a source chord
    GeodesicDistribution {
        /// Source chord as note names, e.g. "C,G,D,A". Conflicts with `--from-pcs`.
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

/// Output format for the geodesic-distribution subcommand.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum GeodesicFormat {
    /// Markdown table — paste-into-paper consumption.
    Md,
    /// Pretty-printed JSON — machine-readable / piping.
    Json,
}

/// CLI error type.
#[derive(Debug)]
pub enum CliError {
    Scale(String),
    Chord(String),
    MissingArgs(String),
    Oth(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Scale(msg) => write!(f, "{}", msg),
            CliError::Chord(msg) => write!(f, "{}", msg),
            CliError::MissingArgs(msg) => write!(f, "{}", msg),
            CliError::Oth(msg) => write!(f, "{}", msg),
        }
    }
}

/// Run the CLI with the parsed arguments. Returns Ok(output) or Err(error).
pub fn run(cli: Cli) -> Result<String, CliError> {
    match cli.command {
        Commands::Scale {
            action,
            args,
            descending,
        } => run_scale(action, args, descending),
        Commands::Chord { action, args } => run_chord(action, args),
        Commands::Oth { action } => run_oth(action),
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

// ─── OTH subcommands ────────────────────────────────────────────────────

/// Map a PC to its default sharp-spelling note name.
fn pc_to_note_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => unreachable!(),
    }
}

/// Parse an orbit string like "Q777" into an Orbit.
fn parse_orbit(s: &str) -> Result<Orbit, CliError> {
    for orbit in Orbit::all() {
        if format!("{}", orbit).contains(s) || format!("{:?}", orbit).contains(s) {
            return Ok(*orbit);
        }
    }
    Err(CliError::Oth(format!("unknown orbit: {}", s)))
}

/// Parse a single note name into its pitch class. Accepts ASCII (`b`, `#`)
/// and Unicode (`♭`, `♯`) accidentals; rejects double accidentals and
/// anything outside `[A-G][b#♭♯]?`.
fn note_name_to_pc(s: &str) -> Option<u8> {
    let trimmed = s.trim();
    let mut chars = trimmed.chars();
    let letter = chars.next()?;
    let acc = chars.next();
    if chars.next().is_some() {
        // More than two characters → reject (no double accidentals).
        return None;
    }
    let natural: u8 = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let offset: i8 = match acc {
        None => 0,
        Some('#' | '♯') => 1,
        Some('b' | '♭') => -1,
        _ => return None,
    };
    Some(((natural as i8 + offset).rem_euclid(12)) as u8)
}

fn run_oth(action: OthAction) -> Result<String, CliError> {
    match action {
        OthAction::Modes { orbit, opening } => run_oth_modes(orbit, opening),
        OthAction::Orbits => run_oth_orbits(),
        OthAction::ParentScales { orbit } => run_oth_parent_scales(orbit),
        OthAction::Verify => run_oth_verify(),
        OthAction::Export => run_oth_export(),
        OthAction::GeodesicDistribution {
            from,
            from_pcs,
            format,
        } => run_oth_geodesic_distribution(from, from_pcs, format),
    }
}

fn run_oth_modes(
    orbit_filter: Option<String>,
    opening_filter: Option<u8>,
) -> Result<String, CliError> {
    let mut output = String::new();

    if let Some(opening) = opening_filter {
        let modes = modes_by_opening_interval(opening);
        output.push_str(&format!("OTH modes with opening interval {}:\n\n", opening));
        for mode in &modes {
            let notes: Vec<&str> = mode
                .pcs_from_c()
                .iter()
                .map(|&pc| pc_to_note_name(pc))
                .collect();
            output.push_str(&format!(
                "  {} M{}  {:?}  {}\n",
                mode.orbit(),
                mode.rotation() + 1,
                mode.steps(),
                notes.join(" ")
            ));
        }
        return Ok(output);
    }

    if let Some(orbit_str) = orbit_filter {
        let orbit = parse_orbit(&orbit_str)?;
        let om = orbit_modes(&orbit);
        output.push_str(&format!(
            "{} — {} distinct modes, cluster: {}\n",
            orbit,
            om.distinct_count(),
            om.step_cluster()
        ));
        if let Some(forte) = om.forte_number() {
            output.push_str(&format!("  Forte: {}\n", forte));
        }
        output.push_str(&format!(
            "  Step multiset: {:?}\n\n",
            om.step_size_multiset()
        ));
        for mode in om.modes() {
            let notes: Vec<&str> = mode
                .pcs_from_c()
                .iter()
                .map(|&pc| pc_to_note_name(pc))
                .collect();
            output.push_str(&format!(
                "  M{}  {:?}  {}  opening: {}\n",
                mode.rotation() + 1,
                mode.steps(),
                notes.join(" "),
                mode.opening_interval()
            ));
        }
        return Ok(output);
    }

    // Default: show all modes grouped by cluster
    let all = all_modes();
    let total: usize = all.iter().map(|om| om.modes().len()).sum();
    output.push_str(&format!(
        "OTH Modes: {} distinct modes across 14 orbits\n\n",
        total
    ));

    // Group by cluster
    for cluster in &[
        StepVocabularyCluster::NoSemitoneNoTritone,
        StepVocabularyCluster::ContainsSemitone,
        StepVocabularyCluster::EvenStepsOnly,
        StepVocabularyCluster::ContainsTritoneStep,
    ] {
        let in_cluster = modes_in_cluster(*cluster);
        if in_cluster.is_empty() {
            continue;
        }
        output.push_str(&format!(
            "Cluster: {} ({} orbits, provisional grouping)\n",
            cluster,
            in_cluster.len()
        ));
        for om in &in_cluster {
            let forte = om.forte_number().unwrap_or_default();
            output.push_str(&format!(
                "  {} ({}) — multiset {:?}: {} modes\n",
                om.orbit(),
                forte,
                om.step_size_multiset(),
                om.distinct_count()
            ));
            for mode in om.modes() {
                let notes: Vec<&str> = mode
                    .pcs_from_c()
                    .iter()
                    .map(|&pc| pc_to_note_name(pc))
                    .collect();
                output.push_str(&format!(
                    "    M{}  {:?}  {}  opening: {}\n",
                    mode.rotation() + 1,
                    mode.steps(),
                    notes.join(" "),
                    mode.opening_interval()
                ));
            }
        }
        output.push('\n');
    }

    Ok(output)
}

fn run_oth_orbits() -> Result<String, CliError> {
    let mut output = String::from("OTH Orbits: 14 orbits in the base space B\n\n");
    for orbit in Orbit::all() {
        let om = orbit_modes(orbit);
        let forte = om.forte_number().unwrap_or_default();
        let cluster = step_vocabulary_cluster(orbit);
        output.push_str(&format!(
            "  {} ({}) — {} modes, multiset {:?}, cluster: {}\n",
            orbit,
            forte,
            om.distinct_count(),
            om.step_size_multiset(),
            cluster
        ));
    }
    Ok(output)
}

fn run_oth_parent_scales(orbit_filter: Option<String>) -> Result<String, CliError> {
    use music_comp_mt::quintal::{all_parent_scales, parent_scales};

    if let Some(orbit_str) = orbit_filter {
        let orbit = parse_orbit(&orbit_str)?;
        let om = orbit_modes(&orbit);
        let repr_pcs = om.modes()[0].pcs_from_c();
        let scales = parent_scales(&repr_pcs);
        let mut output = format!("Parent scales for {} (PCs {:?}):\n\n", orbit, repr_pcs);
        for ps in &scales {
            output.push_str(&format!(
                "  {} root={} — coverage {}/{} ({:.0}%), PCs {:?}\n",
                ps.scale_type(),
                pc_to_note_name(ps.root()),
                ps.coverage_ratio().0,
                ps.coverage_ratio().1,
                ps.coverage() * 100.0,
                ps.pcs()
            ));
        }
        return Ok(output);
    }

    let all = all_parent_scales();
    let mut output = String::from("Parent scale analysis for all 14 orbits:\n\n");
    for (orbit, scales) in &all {
        output.push_str(&format!("  {} — {} parent scales\n", orbit, scales.len()));
        for ps in scales.iter().take(3) {
            output.push_str(&format!(
                "    {} root={} — {}/{}\n",
                ps.scale_type(),
                pc_to_note_name(ps.root()),
                ps.coverage_ratio().0,
                ps.coverage_ratio().1,
            ));
        }
        if scales.len() > 3 {
            output.push_str(&format!("    ... and {} more\n", scales.len() - 3));
        }
    }
    Ok(output)
}

fn run_oth_export() -> Result<String, CliError> {
    use music_comp_mt::quintal::parent_scales;
    use serde::Serialize;

    #[derive(Serialize)]
    struct ExportRoot {
        meta: ExportMeta,
        clusters: Vec<ExportCluster>,
    }

    #[derive(Serialize)]
    struct ExportMeta {
        total_modes: u32,
        total_orbits: u32,
        generated_by: String,
        version: String,
    }

    #[derive(Serialize)]
    struct ExportCluster {
        id: String,
        label: String,
        description: String,
        provisional_note: String,
        orbits: Vec<ExportOrbit>,
    }

    #[derive(Serialize)]
    struct ExportOrbit {
        quintal_label: String,
        forte: String,
        prime_form: [u8; 4],
        step_size_multiset: [u8; 4],
        step_cluster: String,
        parent_scales: Vec<ExportParentScale>,
        modes: Vec<ExportMode>,
    }

    #[derive(Serialize)]
    struct ExportParentScale {
        scale_type: String,
        root: u8,
        pcs: Vec<u8>,
        coverage_ratio: (u8, u8),
    }

    #[derive(Serialize)]
    struct ExportMode {
        rotation: u8,
        steps: [u8; 4],
        pcs_from_c: [u8; 4],
        spelled_from_c: Vec<String>,
        opening_interval: u8,
    }

    let cluster_info: Vec<(StepVocabularyCluster, &str, &str)> = vec![
        (
            StepVocabularyCluster::NoSemitoneNoTritone,
            "no_semitone_no_tritone",
            "Steps from {2,3,4,5} — no semitone or tritone step",
        ),
        (
            StepVocabularyCluster::ContainsSemitone,
            "contains_semitone",
            "Step vocabulary includes semitone (1) but not tritone step (6)",
        ),
        (
            StepVocabularyCluster::EvenStepsOnly,
            "even_steps_only",
            "Step vocabulary ⊆ {2,4} — all steps are even",
        ),
        (
            StepVocabularyCluster::ContainsTritoneStep,
            "contains_tritone_step",
            "Step vocabulary includes tritone step (6)",
        ),
    ];

    let all = all_modes();
    let total_modes: u32 = all.iter().map(|om| om.distinct_count() as u32).sum();

    let mut clusters = Vec::new();
    for (cluster, id, desc) in &cluster_info {
        let in_cluster: Vec<_> = all
            .iter()
            .filter(|om| om.step_cluster() == *cluster)
            .collect();
        if in_cluster.is_empty() {
            continue;
        }

        let mut orbits = Vec::new();
        for om in &in_cluster {
            let repr_pcs = om.modes()[0].pcs_from_c();
            let scales = parent_scales(&repr_pcs);

            let export_scales: Vec<ExportParentScale> = scales
                .iter()
                .map(|ps| ExportParentScale {
                    scale_type: format!("{}", ps.scale_type()),
                    root: ps.root(),
                    pcs: ps.pcs().to_vec(),
                    coverage_ratio: ps.coverage_ratio(),
                })
                .collect();

            let export_modes: Vec<ExportMode> = om
                .modes()
                .iter()
                .map(|m| ExportMode {
                    rotation: m.rotation(),
                    steps: m.steps(),
                    pcs_from_c: m.pcs_from_c(),
                    spelled_from_c: m
                        .pcs_from_c()
                        .iter()
                        .map(|&pc| pc_to_note_name(pc).to_string())
                        .collect(),
                    opening_interval: m.opening_interval(),
                })
                .collect();

            orbits.push(ExportOrbit {
                quintal_label: format!("{:?}", om.orbit()),
                forte: om.forte_number().unwrap_or_default(),
                prime_form: repr_pcs,
                step_size_multiset: om.step_size_multiset(),
                step_cluster: id.to_string(),
                parent_scales: export_scales,
                modes: export_modes,
            });
        }

        clusters.push(ExportCluster {
            id: id.to_string(),
            label: format!("{}", cluster),
            description: desc.to_string(),
            provisional_note:
                "Grouping by step vocabulary only — not a proven theoretical category.".to_string(),
            orbits,
        });
    }

    let root = ExportRoot {
        meta: ExportMeta {
            total_modes,
            total_orbits: 14,
            generated_by: "mt-cli oth export".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        clusters,
    };

    serde_json::to_string_pretty(&root).map_err(|e| CliError::Oth(format!("JSON error: {}", e)))
}

fn run_oth_verify() -> Result<String, CliError> {
    let mut output = String::new();
    match verify_multiset_uniqueness() {
        Ok(()) => output.push_str("  multiset uniqueness: PASS\n"),
        Err(e) => output.push_str(&format!(
            "  multiset uniqueness: EXPECTED COLLISION — {}\n",
            e
        )),
    }
    match verify_fiber_mode_connection() {
        Ok(()) => output.push_str("  fiber-mode connection: PASS\n"),
        Err(e) => output.push_str(&format!("  fiber-mode connection: FAIL — {}\n", e)),
    }
    Ok(output)
}

// ─── geodesic-distribution ──────────────────────────────────────────────────

/// Default source chord: C–G–D–A (pcs `[0, 2, 7, 9]`), per the spec.
const DEFAULT_SOURCE_PCS: [u8; 4] = [0, 2, 7, 9];

/// Comma-split a string and parse each token through `parse_token`. Returns
/// an error if the count is not 4 or any token fails.
fn parse_four<F>(input: &str, parse_token: F, label: &str) -> Result<[u8; 4], CliError>
where
    F: Fn(&str) -> Option<u8>,
{
    let tokens: Vec<&str> = input.split(',').map(str::trim).collect();
    if tokens.len() != 4 {
        return Err(CliError::Oth(format!(
            "expected 4 comma-separated {label}s, got {}: {:?}",
            tokens.len(),
            tokens
        )));
    }
    let mut out = [0u8; 4];
    for (i, tok) in tokens.iter().enumerate() {
        out[i] =
            parse_token(tok).ok_or_else(|| CliError::Oth(format!("invalid {label} {:?}", tok)))?;
    }
    Ok(out)
}

/// Resolve `--from` / `--from-pcs` / default into a `PcChord`.
fn resolve_source_chord(
    from: Option<String>,
    from_pcs: Option<String>,
) -> Result<PcChord, CliError> {
    let pcs: [u8; 4] = match (from, from_pcs) {
        (Some(notes), None) => parse_four(&notes, note_name_to_pc, "note name")?,
        (None, Some(pcs)) => parse_four(
            &pcs,
            |t| t.parse::<u8>().ok().filter(|n| *n <= 11),
            "pitch class",
        )?,
        (None, None) => DEFAULT_SOURCE_PCS,
        (Some(_), Some(_)) => {
            // clap's `conflicts_with` rejects this at parse time; this branch
            // exists only to keep the match exhaustive.
            return Err(CliError::Oth(
                "--from and --from-pcs are mutually exclusive".into(),
            ));
        }
    };
    PcChord::from_unsorted(&pcs)
        .map_err(|e| CliError::Oth(format!("invalid source chord {:?}: {e:?}", pcs)))
}

/// Render a `PcChord` as en-dash-separated note names ordered by pcs ascending,
/// e.g. `"C–D–F#–G#"`.
fn render_chord_dashed(chord: &PcChord) -> String {
    chord
        .pcs
        .iter()
        .map(|&pc| pc_to_note_name(pc))
        .collect::<Vec<_>>()
        .join("–")
}

/// Markdown formatter — extends the §6 paper table with one extra column for
/// the max-σ chord identity at each distance.
fn format_distribution_md(dist: &GeodesicDistribution) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Geodesic distribution from {} ({:?})\n",
        render_chord_dashed(&dist.source),
        dist.source_orbit
    ));
    out.push_str(&format!(
        "eccentricity = {}, reachable_chords = {}\n\n",
        dist.eccentricity, dist.reachable_chords
    ));
    out.push_str("| Distance | Chords at d | Avg geodesics | Max geodesics | Max-σ chord(s) |\n");
    out.push_str("| -------- | :---------: | :-----------: | :-----------: | -------------- |\n");
    for bucket in &dist.buckets {
        let max_chords = bucket
            .max_chords
            .iter()
            .map(|(c, o)| format!("{} ({:?})", render_chord_dashed(c), o))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| {} | {} | {:.1} | {} | {} |\n",
            bucket.distance,
            bucket.chords_at_d,
            bucket.avg_geodesics,
            bucket.max_geodesics,
            max_chords,
        ));
    }
    out
}

fn run_oth_geodesic_distribution(
    from: Option<String>,
    from_pcs: Option<String>,
    format: GeodesicFormat,
) -> Result<String, CliError> {
    let source = resolve_source_chord(from, from_pcs)?;
    let space = BaseSpace::new();
    let dist = geodesic_distribution(&space, &source)
        .ok_or_else(|| CliError::Oth("source chord is not in the base space".into()))?;
    match format {
        GeodesicFormat::Md => Ok(format_distribution_md(&dist)),
        GeodesicFormat::Json => serde_json::to_string_pretty(&dist)
            .map(|mut s| {
                s.push('\n');
                s
            })
            .map_err(|e| CliError::Oth(format!("json serialization failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── pc_to_note_name ────────────────────────────────────────────────

    #[test]
    fn test_pc_to_note_name_naturals() {
        assert_eq!(pc_to_note_name(0), "C");
        assert_eq!(pc_to_note_name(2), "D");
        assert_eq!(pc_to_note_name(4), "E");
        assert_eq!(pc_to_note_name(5), "F");
        assert_eq!(pc_to_note_name(7), "G");
        assert_eq!(pc_to_note_name(9), "A");
        assert_eq!(pc_to_note_name(11), "B");
    }

    #[test]
    fn test_pc_to_note_name_sharps() {
        assert_eq!(pc_to_note_name(1), "C#");
        assert_eq!(pc_to_note_name(3), "D#");
        assert_eq!(pc_to_note_name(6), "F#");
        assert_eq!(pc_to_note_name(8), "G#");
        assert_eq!(pc_to_note_name(10), "A#");
    }

    #[test]
    fn test_pc_to_note_name_wraps_mod_12() {
        assert_eq!(pc_to_note_name(12), "C");
        assert_eq!(pc_to_note_name(14), "D");
    }

    // ─── parse_orbit ────────────────────────────────────────────────────

    #[test]
    fn test_parse_orbit_q777() {
        let orbit = parse_orbit("Q777").unwrap();
        assert_eq!(orbit, Orbit::Q777);
    }

    #[test]
    fn test_parse_orbit_q686() {
        let orbit = parse_orbit("Q686").unwrap();
        assert_eq!(orbit, Orbit::Q686);
    }

    #[test]
    fn test_parse_orbit_unknown() {
        assert!(parse_orbit("Q999").is_err());
    }

    // ─── run_oth_verify ─────────────────────────────────────────────────

    #[test]
    fn test_run_oth_verify_contains_expected_output() {
        let output = run_oth_verify().unwrap();
        assert!(output.contains("fiber-mode connection: PASS"));
        assert!(output.contains("multiset uniqueness: EXPECTED COLLISION"));
    }

    // ─── run_oth_orbits ─────────────────────────────────────────────────

    #[test]
    fn test_run_oth_orbits_lists_14() {
        let output = run_oth_orbits().unwrap();
        assert!(output.contains("14 orbits"));
        // Should mention at least the Summit orbit
        assert!(output.contains("Q(7,7,7)"));
    }

    // ─── run_oth_modes ──────────────────────────────────────────────────

    #[test]
    fn test_run_oth_modes_all() {
        let output = run_oth_modes(None, None).unwrap();
        assert!(output.contains("52 distinct modes"));
        assert!(output.contains("No Semitone, No Tritone"));
    }

    #[test]
    fn test_run_oth_modes_by_orbit() {
        let output = run_oth_modes(Some("Q777".into()), None).unwrap();
        assert!(output.contains("4 distinct modes"));
        assert!(output.contains("M1"));
        assert!(output.contains("M4"));
    }

    #[test]
    fn test_run_oth_modes_by_opening_interval() {
        let output = run_oth_modes(None, Some(1)).unwrap();
        assert!(output.contains("opening interval 1"));
        // All listed modes should have opening: 1
        for line in output.lines() {
            if line.contains("opening:") {
                assert!(line.contains("opening: 1"), "unexpected line: {}", line);
            }
        }
    }

    #[test]
    fn test_run_oth_modes_unknown_orbit() {
        let result = run_oth_modes(Some("Q999".into()), None);
        assert!(result.is_err());
    }

    // ─── run_oth_parent_scales ──────────────────────────────────────────

    #[test]
    fn test_run_oth_parent_scales_all() {
        let output = run_oth_parent_scales(None).unwrap();
        assert!(output.contains("14 orbits"));
    }

    #[test]
    fn test_run_oth_parent_scales_by_orbit() {
        let output = run_oth_parent_scales(Some("Q777".into())).unwrap();
        assert!(output.contains("Parent scales for"));
        assert!(output.contains("PentatonicMajor"));
    }

    // ─── run_oth_export ─────────────────────────────────────────────────

    #[test]
    fn test_run_oth_export_produces_valid_json() {
        let output = run_oth_export().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["meta"]["total_orbits"], 14);
        assert_eq!(parsed["meta"]["total_modes"], 52);
        // Count orbits across all clusters
        let orbits: usize = parsed["clusters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["orbits"].as_array().unwrap().len())
            .sum();
        assert_eq!(orbits, 14);
    }

    #[test]
    fn test_run_oth_export_modes_sum_to_12() {
        let output = run_oth_export().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        for cluster in parsed["clusters"].as_array().unwrap() {
            for orbit in cluster["orbits"].as_array().unwrap() {
                for mode in orbit["modes"].as_array().unwrap() {
                    let sum: u64 = mode["steps"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| s.as_u64().unwrap())
                        .sum();
                    assert_eq!(sum, 12, "steps {:?} don't sum to 12", mode["steps"]);
                }
            }
        }
    }

    // ─── run (dispatch) ─────────────────────────────────────────────────

    #[test]
    fn test_run_oth_dispatch_verify() {
        let cli = Cli::parse_from(["mt", "oth", "verify"]);
        let output = run(cli).unwrap();
        assert!(output.contains("fiber-mode connection: PASS"));
    }

    #[test]
    fn test_run_oth_dispatch_orbits() {
        let cli = Cli::parse_from(["mt", "oth", "orbits"]);
        let output = run(cli).unwrap();
        assert!(output.contains("14 orbits"));
    }

    // ─── CliError display ───────────────────────────────────────────────

    #[test]
    fn test_cli_error_display() {
        let err = CliError::Oth("test error".into());
        assert_eq!(format!("{}", err), "test error");
    }
}
