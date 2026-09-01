//! Integration test: RFC 021 — bulk conversion output-collision safety.
//!
//! Matches the handoff's §2 reproduction exactly, driven through the real
//! binary: two inputs sharing an output stem, converted together, must no
//! longer both report success while one's content is silently discarded.

use std::process::Command;

#[test]
fn colliding_stems_error_and_exit_nonzero() {
    let dir = std::env::temp_dir().join("mdka_cli_test_collision");
    let dir_a = dir.join("a");
    let dir_b = dir.join("b");
    let out = dir.join("out");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();
    std::fs::create_dir_all(&out).unwrap();

    let src_a = dir_a.join("index.html");
    let src_b = dir_b.join("index.html");
    std::fs::write(&src_a, "<h1>FROM A</h1>").unwrap();
    std::fs::write(&src_b, "<h1>FROM B</h1>").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_mdka"))
        .arg("-o")
        .arg(&out)
        .arg(&src_a)
        .arg(&src_b)
        .output()
        .expect("failed to run mdka");

    assert!(
        !output.status.success(),
        "exit code must be non-zero when any input fails"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        stdout.contains(&format!(
            "{} -> {}",
            src_a.display(),
            out.join("index.md").display()
        )),
        "first input must report success, got stdout: {stdout}"
    );
    assert!(
        !stdout.contains(&src_b.display().to_string()),
        "rejected input must not report success, got stdout: {stdout}"
    );
    assert!(
        stderr.contains(&src_b.display().to_string())
            && stderr.contains(&src_a.display().to_string()),
        "error must name both source paths, got stderr: {stderr}"
    );

    let content = std::fs::read_to_string(out.join("index.md")).unwrap();
    assert_eq!(
        content.trim(),
        "# FROM A",
        "surviving file must hold the first input's content, got: {content}"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}
