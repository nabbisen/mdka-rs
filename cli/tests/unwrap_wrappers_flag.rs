//! What `--unwrap-wrappers` and `unwrap_unknown_wrappers` used to guard: a wrapper
//! element keeps its paragraph break.
//!
//! The flag was removed in 3.0 (it could not change the output), so this no
//! longer runs it; `rfc039_a7_cli_surface.rs` asserts the removed-flag error.
//! What is kept is the behaviour the flag existed around, on the fixture that
//! can see it -- bare sibling text, not a block-element fixture, because the
//! neighbouring blocks' own spacing dominates the output otherwise (see
//! `tests/characterisation_structural.rs` in the workspace root for the full
//! explanation). `Minimal` unwraps the wrapper and `Balanced` renders it, and
//! RFC 036 §5.2 / slice `036d` made the two write the same bytes.

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
fn a_wrapper_keeps_its_separation_in_both_modes() {
    for args in [&[][..], &["--mode", "balanced"], &["--mode", "minimal"]] {
        assert_eq!(
            run_mdka(args, HTML),
            "Before\n\ninner\n\nAfter\n",
            "{args:?}"
        );
    }
}
