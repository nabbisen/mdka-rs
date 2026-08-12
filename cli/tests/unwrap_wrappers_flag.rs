//! Integration test: RFC 006 Slice C — `--unwrap-wrappers` CLI flag.
//!
//! Uses a bare-sibling-text fixture, not a block-element fixture: RFC 005
//! Slice A found that block-element fixtures (e.g. `<p>` on either side of
//! the wrapper) cannot discriminate `unwrap_unknown_wrappers` at all, since
//! the neighbouring blocks' own spacing already dominates the output either
//! way. See tests/characterisation_structural.rs in the workspace root for
//! the full explanation.

use std::io::Write;
use std::process::{Command, Stdio};

const HTML: &str = r#"Before<div class="wrap"><span>inner</span></div>After"#;

fn run_mdka(args: &[&str], input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn mdka");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("failed to wait on mdka");
    assert!(output.status.success(), "mdka exited non-zero");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn unwrap_wrappers_flag_changes_output() {
    let without = run_mdka(&[], HTML);
    let with = run_mdka(&["--unwrap-wrappers"], HTML);
    assert_ne!(
        without, with,
        "--unwrap-wrappers did not change output on a fixture designed to discriminate it"
    );
    assert_eq!(without, "Before\n\ninner\n\nAfter\n");
    assert_eq!(with, "BeforeinnerAfter\n");
}
