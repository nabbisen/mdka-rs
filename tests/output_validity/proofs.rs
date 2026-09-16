//! §5.1 and §9.5 — the helper and every property, each shown failing.
//!
//! Every red path here uses a stub converter with frozen broken output, never
//! a live mdka defect: a proof built on a real defect breaks the moment that
//! defect is fixed (RFC 024 fixed the three these proofs first used).

use mdka::options::{ConversionMode, ConversionOptions};

use crate::corpus::{harness_dir, run_dir, run_dir_with};
use crate::harness::{
    ModeResult, Owner, Reading, evaluate, known_defect_with, mdka_convert, properties, structure,
    tree,
};

const STRONG_IN_A: &str = r#"<a href="/out"><strong>b</strong></a>"#;
const STRONG_IN_A_TREE: &str = r#"para(link[/out](strong("b")))"#;

fn correct_strong_in_a(_: &str, _: &ConversionOptions) -> String {
    "[**b**](/out)\n".to_string()
}

/// mdka 2.2.3's output for `STRONG_IN_A`, frozen.
fn broken_strong_in_a(_: &str, _: &ConversionOptions) -> String {
    "****[b](/out)\n".to_string()
}

fn correct_in_minimal_only(html: &str, opts: &ConversionOptions) -> String {
    if opts.mode == ConversionMode::Minimal {
        correct_strong_in_a(html, opts)
    } else {
        broken_strong_in_a(html, opts)
    }
}

fn panicking(_: &str, _: &ConversionOptions) -> String {
    panic!("deliberate panic in a stub converter")
}

// ── §5.1 the strict expected-failure helper ────────────────────────────────

#[test]
fn marked_defect_passes_while_the_defect_is_present() {
    let r = known_defect_with(
        broken_strong_in_a,
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
    for reading in crate::harness::READINGS {
        assert_property_in(reading, name, html, broken_md, correct_md);
    }
}

fn assert_property_in(reading: Reading, name: &str, html: &str, broken_md: &str, correct_md: &str) {
    let broken = properties(html, broken_md, &balanced(), reading);
    assert!(
        broken.iter().any(|v| v.starts_with(name)),
        "{name} did not fire on {broken_md:?}: {broken:?}"
    );
    let correct = properties(html, correct_md, &balanced(), reading);
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

#[test]
fn property_link_content_fires() {
    assert_property(
        "[link-content]",
        r#"<a href="/out"><img src="i.png" alt="pic"></a>"#,
        "![pic](i.png)[](/out)\n",
        "[![pic](i.png)](/out)\n",
    );
    assert_property(
        "[link-content]",
        r#"<a href="/out">Read <strong>more</strong></a>"#,
        "[Read more](/out)\n",
        "[Read **more**](/out)\n",
    );
}

#[test]
fn empty_link_is_exempt_from_link_properties() {
    // RFC 025 review Q1: a link with no text and no image emits nothing.
    let html = r#"<a href="/x"></a><p>t</p>"#;
    for reading in crate::harness::READINGS {
        assert!(properties(html, "t\n", &balanced(), reading).is_empty());
    }
}

// ── both readings ──────────────────────────────────────────────────────────

fn strikethrough_unescaped(_: &str, _: &ConversionOptions) -> String {
    "~~x~~\n".to_string()
}

#[test]
fn gfm_reading_fails_where_commonmark_passes_and_says_which() {
    let html = "<p>~~x~~</p>";
    let e = evaluate(
        strikethrough_unescaped,
        html,
        tree(r#"para("~~x~~")"#),
        ConversionMode::Balanced,
    );
    let ModeResult::Mismatch(problems) = e.result else {
        panic!("expected a mismatch, got {:?}", e.result);
    };
    assert!(
        problems.iter().all(|p| p.starts_with("(gfm) ")),
        "{problems:#?}"
    );
    assert!(
        problems.iter().any(|p| p.starts_with("(gfm) [structure]")),
        "{problems:#?}"
    );
    assert!(
        problems.iter().any(|p| p.starts_with("(gfm) [text]")),
        "{problems:#?}"
    );
}

#[test]
fn id_anchor_is_skipped_only_when_preserve_ids_is_on() {
    let md = "**<a id=\"docs-internal-guid-x\"></a>Hello world**\n";
    for reading in crate::harness::READINGS {
        assert_eq!(
            structure(md, reading, true),
            r#"para(strong("Hello world"))"#
        );
        assert_eq!(
            structure(md, reading, false),
            r#"para(strong(html("<a id=\"docs-internal-guid-x\">"), html("</a>"), "Hello world"))"#
        );
    }
    // Any other inline HTML is kept even with the option on.
    assert_eq!(
        structure("a <b>x</b>\n", Reading::CommonMark, true),
        r#"para("a ", html("<b>"), "x", html("</b>"))"#
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
    assert!(properties(html, "content\n", &minimal, Reading::CommonMark).is_empty());
    let found = properties(html, "content\n", &balanced(), Reading::CommonMark);
    assert!(found.iter().any(|v| v.starts_with("[text]")), "{found:?}");
    assert!(
        found.iter().any(|v| v.starts_with("[link-lost]")),
        "{found:?}"
    );
}

// ── §7.2 the directory runner ──────────────────────────────────────────────

/// mdka 2.2.3's output for the two violating proof files, frozen; the valid
/// file goes through mdka.
fn frozen_2_2_3_for_proof_files(html: &str, opts: &ConversionOptions) -> String {
    if html.contains("<pre>plain</pre>") {
        "plain\n```\n\nAfter\n".to_string()
    } else if html.contains(r#"href="/logo""#) {
        "![Logo](logo.png)[](/logo)\n\n****[Readmore](/more)\n".to_string()
    } else {
        mdka_convert(html, opts)
    }
}

#[test]
fn runner_reports_the_violating_files_and_only_them() {
    let (files, violations) =
        run_dir_with(&harness_dir("corpus_proof"), frozen_2_2_3_for_proof_files)
            .expect("proof directory");
    assert_eq!(
        files,
        ["linked_inline.html", "valid.html", "violating.html"]
    );
    assert!(
        !violations.iter().any(|v| v.starts_with("valid.html (")),
        "{violations:#?}"
    );
    for mode in ["balanced", "strict", "minimal", "semantic", "preserve"] {
        for reading in ["commonmark", "gfm"] {
            let has = |prefix: String| violations.iter().any(|v| v.starts_with(&prefix));
            let unterminated = format!("violating.html ({mode}, {reading}): [unterminated]");
            assert!(
                has(unterminated.clone()),
                "{unterminated} missing: {violations:#?}"
            );
            for dest in ["/logo", "/more"] {
                let content = format!(
                    "linked_inline.html ({mode}, {reading}): [link-content] link \"{dest}\""
                );
                assert!(has(content.clone()), "{content} missing: {violations:#?}");
            }
        }
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
