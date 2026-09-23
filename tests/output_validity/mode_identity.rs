//! 2.4.2 — the two properties that nothing asserted, run over a corpus.
//!
//! `2.4.1` fixed one axis of mode identity (the `id` anchor of an unwrapped
//! wrapper) with 22 fixtures, and its release notes claimed the four modes
//! were identical. They were not: an unwrapped wrapper inside a table cell
//! wrote a real blank line into the row. Every fixture put a single child in
//! the cell; the defect needs two siblings. Those fixtures were written to
//! prove an `id` property and they prove it -- they are blind to everything
//! else about the same cells, and mode identity is not an `id` property.
//!
//! So these assert the *properties*, over inputs, rather than one more
//! fixture:
//!
//! - **P1** -- an option documented as inert is inert. Flipping any one of
//!   the six no-effect fields alone, everything else at the mode's defaults,
//!   is byte-identical, in every one of the five modes.
//! - **P2** -- `Balanced`, `Strict`, `Semantic` and `Preserve` agree,
//!   byte for byte. `Minimal` is documented as genuinely distinct.
//!
//! The corpus is every `.html` file in the directories below, so adding an
//! input adds a case, not code. `mode_corpus/` is the hand-written set this
//! slice adds (wrappers in cells, list items, quotes, `<pre>`, headerless /
//! aligned / spanned / fallback tables); the others already existed and
//! `benches/benchdata` is the six real-shaped documents the benchmarks use.

use std::path::{Path, PathBuf};

use mdka::options::{ConversionMode, ConversionOptions};

use crate::harness::{MODES, mdka_convert};

const DIRS: &[&str] = &[
    "tests/output_validity/mode_corpus",
    "tests/output_validity/runner_fixtures",
    "tests/output_validity/corpus_proof",
    "benches/benchdata",
];

fn corpus() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    for dir in DIRS {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|e| panic!("{dir}: {e}"))
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "html"))
            .collect();
        files.sort();
        for p in files {
            // The larger benchmark files (deep_nest alone is quadratic in depth) cost half a minute in a debug
            // build. None of the excluded files (deep_nest, flat, large, medium, scale_50k, scale_500k,
            // scale_5m) contains a `<td>` or `<th>`, so they cannot express a cell-context defect; everything
            // smaller is here.
            if std::fs::metadata(&p).unwrap().len() > 50_000 {
                continue;
            }
            let html = std::fs::read_to_string(&p).unwrap();
            let name = format!("{dir}/{}", p.file_name().unwrap().to_string_lossy());
            out.push((name, html));
        }
    }
    assert!(
        out.len() >= 50,
        "corpus shrank to {} files; a property over an empty set proves nothing",
        out.len()
    );
    out
}

/// The six fields documented as having no effect, each with the function
/// that flips exactly that one.
///
/// Five of them are `#[deprecated]` -- deliberately touched here: the
/// property is about exactly the fields that claim to do nothing. The allow
/// is on this one function, not the module.
#[allow(clippy::type_complexity, deprecated)]
fn inert_fields() -> Vec<(&'static str, fn(&mut ConversionOptions))> {
    vec![
        ("preserve_classes", |o| {
            o.preserve_classes = !o.preserve_classes
        }),
        ("preserve_data_attrs", |o| {
            o.preserve_data_attrs = !o.preserve_data_attrs
        }),
        ("preserve_aria_attrs", |o| {
            o.preserve_aria_attrs = !o.preserve_aria_attrs
        }),
        ("preserve_unknown_attrs", |o| {
            o.preserve_unknown_attrs = !o.preserve_unknown_attrs
        }),
        ("drop_presentation_attrs", |o| {
            o.drop_presentation_attrs = !o.drop_presentation_attrs
        }),
        ("unwrap_unknown_wrappers", |o| {
            o.unwrap_unknown_wrappers = !o.unwrap_unknown_wrappers
        }),
    ]
}

fn show(s: &str) -> String {
    s.replace('\n', "⏎")
}

#[test]
fn p1_an_option_documented_as_inert_is_inert() {
    let mut violations = Vec::new();
    let corpus = corpus();
    for (name, html) in &corpus {
        for mode in MODES {
            let base_opts = ConversionOptions::for_mode(mode);
            let base = mdka_convert(html, &base_opts);
            for (field, flip) in inert_fields() {
                let mut opts = base_opts.clone();
                flip(&mut opts);
                let flipped = mdka_convert(html, &opts);
                if flipped != base {
                    violations.push(format!(
                        "{name} [{mode}] flipping {field}:\n    default: {}\n    flipped: {}",
                        show(&base),
                        show(&flipped)
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{} of {} (file, mode, field) checks broke inertness:\n{}",
        violations.len(),
        corpus.len() * MODES.len() * inert_fields().len(),
        violations.join("\n")
    );
}

#[test]
fn p2_balanced_strict_semantic_and_preserve_agree() {
    let mut violations = Vec::new();
    let corpus = corpus();
    for (name, html) in &corpus {
        let run = |m| mdka_convert(html, &ConversionOptions::for_mode(m));
        let balanced = run(ConversionMode::Balanced);
        for other in [
            ConversionMode::Strict,
            ConversionMode::Semantic,
            ConversionMode::Preserve,
        ] {
            let got = run(other);
            if got != balanced {
                violations.push(format!(
                    "{name}: Balanced vs {other}:\n    balanced: {}\n    {other}: {}",
                    show(&balanced),
                    show(&got)
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{} disagreement(s) over {} files:\n{}",
        violations.len(),
        corpus.len(),
        violations.join("\n")
    );
}
