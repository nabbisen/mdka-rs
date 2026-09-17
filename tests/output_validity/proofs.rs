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

#[test]
fn link_content_does_not_expect_emphasis_negated_by_its_own_style() {
    // RFC 028 Amendment 1 in the harness's HTML model: a <b> whose own style
    // says font-weight:normal is not strong, so a link holding it expects none.
    assert_property(
        "[link-content]",
        r#"<a href="/l"><b style="font-weight:normal">x</b></a>"#,
        "[**x**](/l)\n",
        "[x](/l)\n",
    );
    // A plain <b> still expects strong.
    assert_property(
        "[link-content]",
        r#"<a href="/l"><b>x</b></a>"#,
        "[x](/l)\n",
        "[**x**](/l)\n",
    );
    // And <i> by font-style.
    assert_property(
        "[link-content]",
        r#"<a href="/l"><i style="font-style: normal">x</i></a>"#,
        "[*x*](/l)\n",
        "[x](/l)\n",
    );
}

#[test]
fn harness_style_rule_edge_cases() {
    use crate::harness::emphasis_negated_by_own_style as negated;
    assert!(negated("b", Some("font-weight:normal")));
    assert!(negated("strong", Some("FONT-WEIGHT : Normal")));
    assert!(negated("b", Some("font-weight: normal !important")));
    assert!(negated("b", Some("font-weight:500")));
    assert!(!negated("b", Some("font-weight:600")));
    assert!(negated("b", Some("font-weight:400.5")));
    assert!(!negated("b", Some("font-weight:bold")));
    assert!(!negated("b", Some("font-weight:lighter")));
    assert!(!negated("b", Some("font-weight:bolder")));
    assert!(negated("b", Some("font-weight:700; font-weight:400")));
    assert!(!negated("b", Some("font-weight:400; font-weight:700")));
    assert!(negated(
        "b",
        Some("color:red; font-weight:normal; font-style:italic")
    ));
    assert!(!negated("b", Some("font-style:normal")));
    assert!(negated("i", Some("font-style:normal")));
    assert!(!negated("em", Some("font-style:italic")));
    assert!(!negated("em", Some("font-weight:normal")));
    assert!(!negated("span", Some("font-weight:normal")));
    assert!(!negated("b", None));
}

#[test]
fn link_wrapping_blocks_is_expected_as_one_link_per_run() {
    // RFC 028 criterion 7 in the harness model: an <a> around blocks may be
    // written as consecutive links whose contents, joined, equal its content.
    assert_property(
        "[link-content]",
        r#"<a href="/o"><p>x</p><p>y</p></a>"#,
        "[x](/o)\n\ny\n",
        "[x](/o)\n\n[y](/o)\n",
    );
    assert_property(
        "[link-content]",
        r#"<a href="/o"><ul><li>x <b>b</b></li><li>y</li></ul></a>"#,
        "- [x](/o) **b**\n- [y](/o)\n",
        "- [x **b**](/o)\n- [y](/o)\n",
    );
}

#[test]
fn inline_link_split_in_two_is_still_a_violation() {
    // Only a link that wraps blocks may be split: an inline link written as
    // two adjacent links is a defect.
    assert_property(
        "[link-content]",
        r#"<p><a href="/o">Read more</a></p>"#,
        "[Read](/o) [more](/o)\n",
        "[Read more](/o)\n",
    );
}

#[test]
fn blocks_inside_pre_are_expected_as_text_with_one_line_break() {
    // RFC 024 rule 7: inside a <pre>, a block element is text only and a block
    // boundary is one line break. The code-block property still fires on a
    // blank line written for it, and the blocks property on a container
    // written around the code.
    assert_property(
        "[code-block]",
        "<pre><p>one</p><p>two</p></pre>",
        "```\none\n\ntwo\n```\n",
        "```\none\ntwo\n```\n",
    );
    assert_property(
        "[blocks]",
        "<pre><blockquote><p>q</p></blockquote></pre>",
        "> ```\n> q\n> ```\n",
        "```\nq\n```\n",
    );
}

#[test]
fn br_inside_pre_is_expected_as_one_line_break() {
    // RFC 024 rule 8: inside a <pre>, <br> is one line break of the code text.
    // The code-block property fires on a Markdown hard break written for it.
    assert_property(
        "[code-block]",
        "<pre>line1<br>line2</pre>",
        "```\nline1  \nline2\n```\n",
        "```\nline1\nline2\n```\n",
    );
}

#[test]
fn pre_inside_a_link_is_not_link_content() {
    // A code block holds text only (RFC 024 criterion 3), so a <pre> inside an
    // <a> is not linked; a link holding only a <pre> is an empty link.
    let only_pre = r#"<a href="/o"><pre>x</pre></a>"#;
    for reading in crate::harness::READINGS {
        assert!(properties(only_pre, "```\nx\n```\n", &balanced(), reading).is_empty());
    }
    assert_property(
        "[link-content]",
        r#"<a href="/o"><p>t</p><pre>x</pre></a>"#,
        "[t x](/o)\n\n```\nx\n```\n",
        "[t](/o)\n\n```\nx\n```\n",
    );
}

// ── 028b: guards on the link-around-blocks model ───────────────────────────

fn two_blocks_collapsed_into_one_link(_: &str, _: &ConversionOptions) -> String {
    "[x y](/o)\n".to_string()
}

#[test]
fn collapsed_link_around_blocks_is_caught_by_the_tree_not_the_properties() {
    // `joined_run` accepts one link for several blocks, so the link properties
    // pass `[x y](/o)` -- the collapse RFC 024's review rejected. The cell's
    // tree assertion (p_in_a's, written by the architect) is what catches it.
    let html = r#"<a href="/o"><p>x</p><p>y</p></a>"#;
    let expect = tree(r#"para(link[/o]("x")), para(link[/o]("y"))"#);
    for reading in crate::harness::READINGS {
        let found = properties(html, "[x y](/o)\n", &balanced(), reading);
        assert!(
            found.is_empty(),
            "{reading}: properties should pass: {found:?}"
        );
    }
    for mode in crate::harness::MODES {
        let e = evaluate(two_blocks_collapsed_into_one_link, html, expect, mode);
        let ModeResult::Mismatch(problems) = e.result else {
            panic!("{mode}: expected a mismatch, got {:?}", e.result);
        };
        assert!(
            problems.iter().all(|p| p.contains("[structure]")),
            "{mode}: only the tree should fire: {problems:#?}"
        );
        for reading in ["commonmark", "gfm"] {
            let prefix = format!("({reading}) [structure]");
            assert!(
                problems.iter().any(|p| p.starts_with(&prefix)),
                "{mode}: {prefix} missing"
            );
        }
    }
}

/// The number of top-level blocks in `md` (CommonMark).
fn top_level_blocks(md: &str) -> usize {
    let tree = structure(md, Reading::CommonMark, false);
    let mut depth = 0usize;
    let mut blocks = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in tree.chars() {
        if in_string {
            match (escaped, ch) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => {
                if depth == 0 {
                    blocks += 1;
                }
                depth += 1;
            }
            ')' => depth -= 1,
            _ => {}
        }
    }
    blocks
}

/// Tags the harness's `starts_markdown_block` names, each with a probe of the
/// form `a<TAG>b</TAG>` at body level. A list item needs its list; `hr` is
/// empty, so it separates `a` from `b` on its own.
const HARNESS_BLOCK_PROBES: &[(&str, &str)] = &[
    ("h1", "a<h1>b</h1>"),
    ("h2", "a<h2>b</h2>"),
    ("h3", "a<h3>b</h3>"),
    ("h4", "a<h4>b</h4>"),
    ("h5", "a<h5>b</h5>"),
    ("h6", "a<h6>b</h6>"),
    ("p", "a<p>b</p>"),
    ("ul", "a<ul><li>b</li></ul>"),
    ("ol", "a<ol><li>b</li></ol>"),
    ("li", "a<ul><li>b</li></ul>"),
    ("blockquote", "a<blockquote>b</blockquote>"),
    ("pre", "a<pre>b</pre>"),
    ("hr", "a<hr>b"),
    ("header", "a<header>b</header>"),
    ("footer", "a<footer>b</footer>"),
    ("nav", "a<nav>b</nav>"),
    ("aside", "a<aside>b</aside>"),
    ("figure", "a<figure>b</figure>"),
    ("figcaption", "a<figcaption>b</figcaption>"),
    ("div", "a<div>b</div>"),
    ("article", "a<article>b</article>"),
    ("section", "a<section>b</section>"),
    ("main", "a<main>b</main>"),
];

const INLINE_PROBE_TAGS: &[&str] = &[
    "span", "b", "i", "em", "strong", "code", "a", "abbr", "small", "sub", "sup", "mark",
];

#[test]
fn harness_block_list_agrees_with_what_mdka_renders() {
    // Behavioural agreement: mdka is only converted, never imported. For each
    // tag and each mode where the tag is not dropped, the harness says "starts
    // a block" exactly when mdka's output has more than one top-level block.
    // The five modes cover both settings of unwrap_unknown_wrappers
    // (Balanced/Strict/Preserve off; Minimal/Semantic on).
    let mut checked = Vec::new();
    for mode in crate::harness::MODES {
        let opts = ConversionOptions::for_mode(mode);
        for (tag, html) in HARNESS_BLOCK_PROBES {
            if opts.drop_interactive_shell && matches!(*tag, "nav" | "header" | "footer" | "aside")
            {
                continue;
            }
            let md = mdka_convert(html, &opts);
            let harness = crate::harness::starts_markdown_block(tag, &opts);
            let mdka = top_level_blocks(&md) > 1;
            assert_eq!(
                harness, mdka,
                "{mode} <{tag}>: harness {harness}, mdka {mdka}, output {md:?}"
            );
            checked.push(format!("{mode}:{tag}={harness}"));
        }
        for tag in INLINE_PROBE_TAGS {
            let html = format!("<p>a<{tag}>b</{tag}></p>");
            let md = mdka_convert(&html, &opts);
            assert!(!crate::harness::starts_markdown_block(tag, &opts), "{tag}");
            assert_eq!(top_level_blocks(&md), 1, "{mode} <{tag}>: {md:?}");
            checked.push(format!("{mode}:{tag}=inline"));
        }
    }
    println!("checked: {}", checked.join(" "));
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
