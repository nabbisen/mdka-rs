//! §5.1 and §9.5 — the helper and every property, each shown failing.

use mdka::options::{ConversionMode, ConversionOptions};

use crate::corpus::{corpus_dir, run_dir};
use crate::harness::{Owner, known_defect_with, mdka_convert, properties, tree};

const STRONG_IN_A: &str = r#"<a href="/out"><strong>b</strong></a>"#;
const STRONG_IN_A_TREE: &str = r#"para(link[/out](strong("b")))"#;

fn correct_strong_in_a(_: &str, _: &ConversionOptions) -> String {
    "[**b**](/out)\n".to_string()
}

fn correct_in_minimal_only(html: &str, opts: &ConversionOptions) -> String {
    if opts.mode == ConversionMode::Minimal {
        correct_strong_in_a(html, opts)
    } else {
        mdka_convert(html, opts)
    }
}

fn panicking(_: &str, _: &ConversionOptions) -> String {
    panic!("deliberate panic in a stub converter")
}

// ── §5.1 the strict expected-failure helper ────────────────────────────────

#[test]
fn marked_defect_passes_while_the_defect_is_present() {
    let r = known_defect_with(
        mdka_convert,
        Owner::Rfc024,
        "proof",
        STRONG_IN_A,
        tree(STRONG_IN_A_TREE),
    );
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn marked_defect_fails_once_the_output_is_correct() {
    let r = known_defect_with(
        correct_strong_in_a,
        Owner::Rfc024,
        "proof",
        STRONG_IN_A,
        tree(STRONG_IN_A_TREE),
    );
    let e = r.expect_err("a fixed defect must fail");
    assert!(
        e.contains("known defect now passes -- remove the known_defect marker (owner: RFC 024)"),
        "{e}"
    );
}

#[test]
fn marked_defect_fails_when_fixed_in_one_mode_only() {
    let r = known_defect_with(
        correct_in_minimal_only,
        Owner::Rfc024,
        "proof",
        STRONG_IN_A,
        tree(STRONG_IN_A_TREE),
    );
    let e = r.expect_err("a defect fixed in one mode must fail");
    assert!(e.contains("passing modes: minimal"), "{e}");
}

#[test]
fn marked_defect_fails_when_the_conversion_panics() {
    let r = known_defect_with(
        panicking,
        Owner::Rfc024,
        "proof",
        STRONG_IN_A,
        tree(STRONG_IN_A_TREE),
    );
    let e = r.expect_err("a panic is never \"still defective\"");
    assert!(e.contains("could not be evaluated"), "{e}");
    assert!(e.contains("deliberate panic in a stub converter"), "{e}");
}

// ── each property: red on a deliberate break, green on the correct form ────

fn balanced() -> ConversionOptions {
    ConversionOptions::for_mode(ConversionMode::Balanced)
}

fn assert_property(name: &str, html: &str, broken_md: &str, correct_md: &str) {
    let broken = properties(html, broken_md, &balanced());
    assert!(
        broken.iter().any(|v| v.starts_with(name)),
        "{name} did not fire on {broken_md:?}: {broken:?}"
    );
    let correct = properties(html, correct_md, &balanced());
    assert!(
        correct.is_empty(),
        "{correct_md:?} should hold every property: {correct:?}"
    );
}

#[test]
fn property_stray_delimiter_fires() {
    assert_property("[stray-delimiter]", "<p>a</p>", "**a\n", "a\n");
}

#[test]
fn property_text_fires() {
    assert_property(
        "[text]",
        "<p>Read <b>more</b> now</p>",
        "Readmore now\n",
        "Read **more** now\n",
    );
}

#[test]
fn property_link_lost_fires() {
    assert_property(
        "[link-lost]",
        r#"<p><a href="/a b">x</a></p>"#,
        "[x](/a b)\n",
        "[x](</a b>)\n",
    );
}

#[test]
fn property_image_lost_fires() {
    assert_property(
        "[image-lost]",
        r#"<img src="i.png" alt="a">"#,
        "a\n",
        "![a](i.png)\n",
    );
}

#[test]
fn property_code_block_fires() {
    assert_property(
        "[code-block]",
        "<pre><code>a\n```\nb</code></pre>",
        "```\na\n```\nb\n```\n",
        "````\na\n```\nb\n````\n",
    );
}

#[test]
fn property_blocks_fires() {
    assert_property("[blocks]", "<h2>x</h2>", "x\n", "## x\n");
}

#[test]
fn property_unterminated_fires() {
    assert_property(
        "[unterminated]",
        "<pre><code>x</code></pre>",
        "```\nx\n",
        "```\nx\n```\n",
    );
}

// ── which modes drop content (§6.3: "check which; do not assume") ───────────

#[test]
fn only_minimal_drops_shell_content() {
    let html = r#"<nav><a href="/n">home</a></nav><p>content</p>"#;
    for mode in crate::harness::MODES {
        let md = mdka_convert(html, &ConversionOptions::for_mode(mode));
        assert_eq!(
            md.contains("home"),
            mode != ConversionMode::Minimal,
            "{mode}: {md:?}"
        );
    }
    // The properties follow the documented option rather than exempting a mode.
    let minimal = ConversionOptions::for_mode(ConversionMode::Minimal);
    assert!(properties(html, "content\n", &minimal).is_empty());
    let found = properties(html, "content\n", &balanced());
    assert!(found.iter().any(|v| v.starts_with("[text]")), "{found:?}");
    assert!(
        found.iter().any(|v| v.starts_with("[link-lost]")),
        "{found:?}"
    );
}

// ── §7.2 the directory runner ──────────────────────────────────────────────

#[test]
fn runner_reports_the_violating_file_and_only_it() {
    let (files, violations) = run_dir(&corpus_dir("corpus_proof")).expect("proof directory");
    assert_eq!(files, ["valid.html", "violating.html"]);
    assert!(!violations.is_empty());
    assert!(
        violations.iter().all(|v| v.starts_with("violating.html (")),
        "{violations:#?}"
    );
    for mode in ["balanced", "strict", "minimal", "semantic", "preserve"] {
        let prefix = format!("violating.html ({mode}): [unterminated]");
        assert!(
            violations.iter().any(|v| v.starts_with(&prefix)),
            "{prefix} missing: {violations:#?}"
        );
    }
    println!("{}", violations.join("\n"));
}

#[test]
fn runner_refuses_an_empty_directory() {
    let empty = std::env::temp_dir().join(format!("mdka-harness-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).expect("temp dir");
    let r = run_dir(&empty);
    std::fs::remove_dir(&empty).ok();
    assert!(
        r.expect_err("empty directory")
            .contains("nothing was checked")
    );
}
