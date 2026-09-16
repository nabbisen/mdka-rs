//! §7.2 — run every `.html` file in a directory through the intent-free
//! properties, in every mode and under every reading, with no per-file
//! expectations. Adding real-world input (slice `025b`) adds files, not code.
//!
//! Directories:
//! - `runner_fixtures/`: hand-written inputs that exercise the runner.
//! - `corpus_proof/`: the runner's red-path proof.
//! - `corpus/`: real captures only, for `025b`; empty until they exist and not
//!   run yet (see its README).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use mdka::options::ConversionOptions;

use crate::harness::{MODES, READINGS, mdka_convert, properties};

pub fn harness_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/output_validity")
        .join(name)
}

/// The files checked and every violation, as `file (mode, reading): violation`.
/// A directory with no `.html` files is an error: a runner that checked
/// nothing must not report success.
pub fn run_dir(dir: &Path) -> Result<(Vec<String>, Vec<String>), String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "html"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "{}: no .html files -- nothing was checked",
            dir.display()
        ));
    }
    let mut names = Vec::new();
    let mut violations = Vec::new();
    for path in &files {
        let name = path
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .to_string();
        let html = std::fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
        for mode in MODES {
            let opts = ConversionOptions::for_mode(mode);
            let md = match catch_unwind(AssertUnwindSafe(|| mdka_convert(&html, &opts))) {
                Ok(md) => md,
                Err(_) => {
                    violations.push(format!("{name} ({mode}): [error] conversion panicked"));
                    continue;
                }
            };
            for reading in READINGS {
                match catch_unwind(AssertUnwindSafe(|| properties(&html, &md, &opts, reading))) {
                    Ok(found) => violations.extend(
                        found
                            .into_iter()
                            .map(|v| format!("{name} ({mode}, {reading}): {v}")),
                    ),
                    Err(_) => violations.push(format!(
                        "{name} ({mode}, {reading}): [error] harness panicked"
                    )),
                }
            }
        }
        names.push(name);
    }
    Ok((names, violations))
}

#[test]
fn runner_fixtures_hold_every_property() {
    let (files, violations) =
        run_dir(&harness_dir("runner_fixtures")).expect("runner fixtures directory");
    println!(
        "runner_fixtures: {} file(s): {}",
        files.len(),
        files.join(", ")
    );
    assert!(
        violations.is_empty(),
        "{} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
