use std::process::Command;

fn mt() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mt"))
}

#[test]
fn test_scale_c_major() {
    let output = mt().args(["scale", "C", "major"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("C"));
    assert!(stdout.contains("E"));
    assert!(stdout.contains("G"));
}

#[test]
fn test_chord_c_major() {
    let output = mt().args(["chord", "C", "Major"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("C"));
    assert!(stdout.contains("E"));
    assert!(stdout.contains("G"));
}

#[test]
fn test_scale_list() {
    let output = mt().args(["scale", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Ionian") || stdout.contains("Major"));
}

#[test]
fn test_chord_list() {
    let output = mt().args(["chord", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Major") || stdout.contains("Triad"));
}

#[test]
fn test_scale_descending() {
    let output = mt()
        .args(["scale", "--descending", "C", "major"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_no_args_shows_help() {
    let output = mt().output().unwrap();
    assert!(!output.status.success()); // clap exits non-zero with no subcommand
}
