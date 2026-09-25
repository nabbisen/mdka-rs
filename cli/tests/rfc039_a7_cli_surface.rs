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

/// The CLI has one warning shape, `mdka: warning: `, for every deprecation:
/// the three no-effect flags and the alias modes alike. Each is one line, and
/// the shape is asserted so a later warning cannot quietly introduce a second.
#[test]
fn every_cli_deprecation_warning_has_the_same_shape() {
    for args in [
        &["--preserve-classes"][..],
        &["--preserve-data"],
        &["--preserve-aria"],
        &["--mode", "strict"],
    ] {
        let out = run_mdka(args, "<p>Hi</p>");
        assert!(out.status.success(), "{args:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.starts_with("mdka: warning: ") && stderr.matches('\n').count() == 1,
            "{args:?}: expected one line starting `mdka: warning: `, got: {stderr:?}"
        );
    }
}

/// The flag warning keeps its wording; only the prefix moved (2.8.0).
#[test]
fn preserve_classes_warning_wording_is_unchanged_after_the_prefix() {
    let out = run_mdka(&["--preserve-classes"], "<p>Hi</p>");
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "mdka: warning: `--preserve-classes` has no effect and is deprecated (see \
         https://nabbisen.github.io/mdka-rs/api/options.html). Markdown has no attribute \
         syntax, so this option was never expressible in the output.\n"
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

// ── RFC 041 §9: the alias modes are deprecated (2.8.0), removed in 3.0 ───────

const ALIAS_HTML: &str = r#"<h1 id="t">T</h1><p class="c">a <strong>b</strong></p>"#;

/// `--mode strict|semantic|preserve` keeps working and keeps its output; each
/// prints exactly one line on stderr, and stdout is byte-identical to
/// `--mode balanced`'s, so a pipe is never corrupted.
#[test]
fn alias_modes_warn_on_stderr_and_keep_their_output() {
    let balanced = run_mdka(&["--mode", "balanced"], ALIAS_HTML);
    assert!(balanced.status.success());
    assert!(
        balanced.stderr.is_empty(),
        "balanced must be silent: {}",
        String::from_utf8_lossy(&balanced.stderr)
    );

    for mode in ["strict", "semantic", "preserve"] {
        let out = run_mdka(&["--mode", mode], ALIAS_HTML);
        assert!(out.status.success(), "--mode {mode} must still work");
        assert_eq!(
            out.stdout, balanced.stdout,
            "--mode {mode}: stdout must be identical to balanced's"
        );
        assert_eq!(
            String::from_utf8(out.stderr).unwrap(),
            format!(
                "mdka: warning: --mode {mode} is an alias of balanced and produces identical \
                 output; it is removed in 3.0\n"
            ),
            "--mode {mode}: stderr must be exactly the one deprecation line"
        );
    }
}

/// Case-insensitive parsing is unchanged, and the warning names the canonical
/// mode rather than echoing the caller's spelling.
#[test]
fn alias_mode_warning_is_the_same_for_any_spelling() {
    let out = run_mdka(&["-m", "STRICT"], "<p>Hi</p>");
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "Hi\n");
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .starts_with("mdka: warning: --mode strict is an alias of balanced"),
    );
}

/// Warned only when the caller named a deprecated mode: no `--mode`, the two
/// real modes, and an explicit `--mode balanced` all print nothing.
#[test]
fn no_warning_unless_a_deprecated_mode_was_named() {
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

#[test]
fn help_marks_the_alias_modes_deprecated() {
    let out = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg("--help")
        .output()
        .expect("failed to run mdka --help");
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    for name in ["strict", "semantic", "preserve"] {
        assert!(
            help.contains(name),
            "help must still name `{name}`:\n{help}"
        );
    }
    assert!(
        help.contains("[deprecated] strict | semantic | preserve"),
        "the Options entry must mark the aliases deprecated:\n{help}"
    );
    assert!(
        help.contains("[deprecated, removed in 3.0] Aliases of balanced"),
        "the Modes entry must mark the aliases deprecated and say when they go:\n{help}"
    );
}
