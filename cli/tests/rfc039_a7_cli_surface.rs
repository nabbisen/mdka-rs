//! Integration tests: RFC 039 §3 A7 — CLI surface corrections.
//!
//! Covers the slice's own criterion 5: `--no-preserve-ids` turns anchors off
//! in every mode, `--help` documents `--preserve-ids`/`--no-preserve-ids`
//! correctly, a deprecated flag's notice goes to stderr with stdout left
//! untouched, and single-file conversion's progress line goes to stderr, so it
//! cannot mix into a redirected stdin conversion. (This header used to claim
//! `mdka page.html > out.md` leaves `out.md` holding the conversion; a file
//! argument writes a sibling `.md` and leaves stdout empty, so that redirect
//! gives an empty `out.md` -- corrected in 2.4.1, and pinned below.)

use std::io::Write;
use std::process::{Command, Stdio};

fn run_mdka(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn mdka");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().expect("failed to wait on mdka")
}

const HTML_WITH_ID: &str = r#"<h2 id="x">T</h2>"#;

#[test]
fn no_preserve_ids_turns_anchors_off_in_every_mode() {
    for mode in ["balanced", "strict", "minimal", "semantic", "preserve"] {
        let out = run_mdka(&["--mode", mode, "--no-preserve-ids"], HTML_WITH_ID);
        assert!(out.status.success(), "mdka exited non-zero for mode {mode}");
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(
            !stdout.contains("<a id="),
            "mode {mode}: --no-preserve-ids left an anchor in: {stdout}"
        );
    }
}

#[test]
fn preserve_ids_still_emits_anchors_by_default_outside_minimal() {
    let out = run_mdka(&["--mode", "balanced"], HTML_WITH_ID);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains(r#"<a id="x"></a>"#),
        "balanced mode should still emit an anchor by default: {stdout}"
    );
}

#[test]
fn help_documents_preserve_ids_and_no_preserve_ids() {
    let out = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg("--help")
        .output()
        .expect("failed to run mdka --help");
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(
        help.contains("--no-preserve-ids"),
        "help text missing --no-preserve-ids:\n{help}"
    );
    assert!(
        help.contains("anchors") || help.contains("anchor"),
        "help text for --preserve-ids should describe anchor emission:\n{help}"
    );
}

#[test]
fn deprecated_flag_notice_goes_to_stderr_stdout_stays_clean() {
    let out = run_mdka(&["--preserve-classes"], "<p class=\"x\">Hi</p>");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(
        stdout, "Hi\n",
        "stdout must hold only the conversion: {stdout}"
    );
    assert!(
        stderr.contains("--preserve-classes") && stderr.contains("deprecated"),
        "stderr must carry the deprecation notice: {stderr}"
    );
}

#[test]
fn single_file_progress_goes_to_stderr_not_stdout() {
    let dir = std::env::temp_dir().join("mdka_cli_test_a7_progress");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("page.html");
    std::fs::write(&src, "<h1>Hello</h1>").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg(&src)
        .output()
        .expect("failed to run mdka");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stdout.is_empty(),
        "single-file conversion must write nothing to stdout: {stdout}"
    );
    assert!(
        stderr.contains(&src.display().to_string()) && stderr.contains("->"),
        "progress line must appear on stderr: {stderr}"
    );

    let dest = dir.join("page.md");
    assert!(dest.exists(), "output file was not created");
    let content = std::fs::read_to_string(&dest).unwrap();
    assert_eq!(content.trim(), "# Hello");

    std::fs::remove_dir_all(&dir).unwrap();
}

/// 2.4.1: `--help` and reality agree. A file argument's conversion is the
/// sibling file, so redirecting stdout captures nothing; `--help` must say so
/// rather than promise the opposite.
#[test]
fn file_argument_redirect_is_empty_and_help_says_so() {
    let dir = std::env::temp_dir().join("mdka_cli_test_241_redirect");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("page.html");
    std::fs::write(&src, "<h1>Hello</h1>").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg(&src)
        .output()
        .expect("failed to run mdka");
    assert!(
        out.stdout.is_empty(),
        "stdout must be empty for a file argument"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("page.md")).unwrap().trim(),
        "# Hello"
    );

    let help = run_mdka(&["--help"], "");
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(
        help.contains("gives an empty out.md"),
        "--help must say the file-argument redirect is empty: {help}"
    );
    assert!(
        !help.contains("leaves out.md holding only the"),
        "--help still carries the old, false claim: {help}"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}
