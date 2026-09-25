//! Integration tests: RFC 039 §3 A7 — CLI surface corrections.
//!
//! Covers the slice's own criterion 5: `--no-preserve-ids` turns anchors off
//! in every mode, `--help` documents `--preserve-ids`/`--no-preserve-ids`
//! correctly, an error or notice goes to stderr with stdout left untouched
//! (the deprecation notices this file used to assert are gone with the things
//! they warned about in 3.0; what replaced them is the last section, the
//! removed-name errors), and single-file conversion's progress line goes to stderr, so it
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
    for mode in ["balanced", "minimal"] {
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

// ── 3.0: removed modes and flags say *removed*, and what to do ──────────────

/// The CLI's response to a mode name that existed until 2.9.0. `FromStr` says
/// *removed* for it, and the CLI must not append the "Valid:" list that only
/// belongs on an *unknown* name (RFC 048 §6). Exit 1, on stderr, nothing on
/// stdout, exactly one line.
#[test]
fn a_removed_mode_is_an_obsolete_config_not_a_wrong_one() {
    for mode in ["strict", "semantic", "preserve"] {
        let out = run_mdka(&["--mode", mode], "<p>Hi</p>");
        assert_eq!(out.status.code(), Some(1), "--mode {mode}");
        assert!(
            out.stdout.is_empty(),
            "--mode {mode}: nothing may reach stdout"
        );
        assert_eq!(
            String::from_utf8(out.stderr).unwrap(),
            format!(
                "error: conversion mode '{mode}' was removed in 3.0; it was an alias of \
                 'balanced'. Use 'balanced'.\n"
            ),
            "--mode {mode}"
        );
    }
}

#[test]
fn a_removed_mode_is_recognised_in_any_case() {
    let out = run_mdka(&["-m", "STRICT"], "<p>Hi</p>");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .starts_with("error: conversion mode 'strict' was removed in 3.0"),
    );
}

/// A name that never existed keeps the *unknown* message, with the list of the
/// modes that do exist. Two different errors, both asserted.
#[test]
fn an_unknown_mode_still_says_unknown_and_lists_the_valid_ones() {
    let out = run_mdka(&["--mode", "bogus"], "<p>Hi</p>");
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "error: unknown conversion mode: bogus. Valid: balanced|minimal\n"
    );
}

/// The four flags removed in 3.0 fail cleanly and say what to do. Not the
/// generic "unknown option" plus the whole usage text: these were documented
/// as deprecated for several releases running, and a bare "unknown option"
/// would not say what to do. One line, on stderr, exit 1, stdout empty -- so a
/// script that still passes one fails loudly.
#[test]
fn a_removed_flag_fails_cleanly_and_says_what_to_do() {
    for flag in [
        "--preserve-classes",
        "--preserve-data",
        "--preserve-aria",
        "--unwrap-wrappers",
    ] {
        let out = run_mdka(&[flag], "<p>Hi</p>");
        assert_eq!(out.status.code(), Some(1), "{flag}");
        assert!(out.stdout.is_empty(), "{flag}: nothing may reach stdout");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.starts_with(&format!("error: `{flag}` was removed in 3.0; ")),
            "{flag}: {stderr}"
        );
        assert!(
            stderr.contains("Remove it from the command line: the output is the same without it."),
            "{flag}: must say what to do: {stderr}"
        );
        assert_eq!(
            stderr.matches('\n').count(),
            1,
            "{flag}: one line, not the usage text: {stderr}"
        );
        assert!(!stderr.contains("Usage:"), "{flag}: {stderr}");
    }
}

/// The reason given is the true one for each: an attribute flag never changed
/// the output because Markdown has no attribute syntax; the wrapper flag could
/// not change it either, for a different reason.
#[test]
fn a_removed_flag_gives_its_own_true_reason() {
    let attr = run_mdka(&["--preserve-classes"], "<p>Hi</p>");
    assert!(
        String::from_utf8(attr.stderr)
            .unwrap()
            .contains("Markdown has no attribute syntax")
    );
    let wrap = run_mdka(&["--unwrap-wrappers"], "<p>Hi</p>");
    let wrap = String::from_utf8(wrap.stderr).unwrap();
    assert!(wrap.contains("it could not change the output"), "{wrap}");
    assert!(!wrap.contains("attribute syntax"), "{wrap}");
}

/// A removed flag placed after `--` is a path, as it always was.
#[test]
fn a_removed_flag_after_double_dash_is_still_a_file_name() {
    let out = run_mdka(&["--", "--unwrap-wrappers"], "");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(!stderr.contains("was removed in 3.0"), "{stderr}");
}

/// No `--mode`, `balanced` and `minimal` print nothing on stderr: there is no
/// deprecation left to warn about.
#[test]
fn the_two_modes_are_silent() {
    for args in [&[][..], &["--mode", "balanced"], &["-m", "minimal"]] {
        let out = run_mdka(args, "<p>Hi</p>");
        assert!(out.status.success(), "{args:?}");
        assert!(
            out.stderr.is_empty(),
            "{args:?} must be silent, got: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// `--help` names two modes and none of the removed names or flags.
#[test]
fn help_lists_two_modes_and_none_of_the_removed_names() {
    let out = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg("--help")
        .output()
        .expect("failed to run mdka --help");
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(help.contains("balanced(default) | minimal"), "{help}");
    for gone in [
        "strict",
        "semantic",
        "--preserve-classes",
        "--preserve-data",
        "--preserve-aria",
        "--unwrap-wrappers",
        "deprecated",
    ] {
        assert!(
            !help.contains(gone),
            "help still mentions `{gone}`:\n{help}"
        );
    }
    // `--preserve-ids` stays, and must not be caught by the checks above.
    assert!(help.contains("--preserve-ids") && help.contains("--no-preserve-ids"));
}
