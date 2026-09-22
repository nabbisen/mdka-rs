//! RFC 028 — inline elements around block content, and emphasis negated by
//! its own style. The RFC 025 harness checks these shapes by parsing the
//! output; the tests here pin the bytes the consolidated acceptance criteria
//! name.

mod common;
use common::{conv, conv_with};
use mdka::options::{ConversionMode, ConversionOptions};

const MODES: [ConversionMode; 5] = [
    ConversionMode::Balanced,
    ConversionMode::Strict,
    ConversionMode::Minimal,
    ConversionMode::Semantic,
    ConversionMode::Preserve,
];

// ── criterion 1: emphasis around blocks writes no delimiters ───────────────

#[test]
fn emphasis_around_paragraphs_writes_no_delimiters() {
    for tag in ["strong", "b", "em", "i"] {
        assert_eq!(
            conv(&format!("<{tag}><p>x</p><p>y</p></{tag}>")),
            "x\n\ny\n",
            "<{tag}>"
        );
    }
}

#[test]
fn any_block_counts_not_only_paragraphs() {
    assert_eq!(conv("<b><div>x</div></b>"), "x\n");
    assert_eq!(conv("<b><ul><li>a</li></ul></b>"), "- a\n");
    assert_eq!(conv("<b><h2>head</h2></b>"), "## head\n");
    assert_eq!(conv("<b><blockquote><p>q</p></blockquote></b>"), "> q\n");
    assert_eq!(conv("<b><pre>x</pre></b>"), "```\nx\n```\n");
}

#[test]
fn a_mode_unwrapping_a_div_is_still_a_block() {
    // Previously named "...is not a block", asserting Minimal/Semantic kept
    // "**x**\n" (delimiters) because unwrapping <div> was believed to remove
    // its block-ness along with the tag. RFC 036 §5.2 / slice 036d corrected
    // that: unwrapping drops the element, not the paragraph break it stood
    // for, so a div still negates surrounding emphasis in every mode, exactly
    // like any_block_counts_not_only_paragraphs's own (never-unwrapped) case.
    let html = "<b><div>x</div></b>";
    for mode in MODES {
        assert_eq!(
            conv_with(html, &ConversionOptions::for_mode(mode)),
            "x\n",
            "{mode}"
        );
    }
}

// ── criteria 2, 3: Google Docs ─────────────────────────────────────────────

#[test]
fn google_docs_multi_paragraph_wrapper() {
    assert_eq!(
        conv(r#"<b style="font-weight:normal"><p>para one</p><p>para two</p></b>"#),
        "para one\n\npara two\n"
    );
}

#[test]
fn google_docs_single_paragraph_wrapper_has_no_emphasis_in_any_mode() {
    let html = r#"<b style="font-weight:normal;" id="docs-internal-guid-x"><span style="font-weight:400">Hello world</span></b>"#;
    for mode in MODES {
        let md = conv_with(html, &ConversionOptions::for_mode(mode));
        assert!(!md.contains('*'), "{mode}: {md:?}");
        assert!(md.ends_with("Hello world\n"), "{mode}: {md:?}");
    }
}

// ── criteria 4, 5: style negation ──────────────────────────────────────────

#[test]
fn own_style_negates_bold() {
    for style in [
        "font-weight:normal",
        "font-weight:400",
        "font-weight:500",
        "font-weight: normal !important",
        "FONT-WEIGHT: NORMAL",
        "font-weight:700; font-weight:400",
        "color:red; font-weight : normal ; text-decoration:underline",
    ] {
        assert_eq!(
            conv(&format!(r#"<p><b style="{style}">x</b></p>"#)),
            "x\n",
            "{style}"
        );
    }
    assert_eq!(
        conv(r#"<p><strong style="font-weight:normal">x</strong></p>"#),
        "x\n"
    );
}

#[test]
fn style_that_does_not_negate_keeps_bold() {
    for style in [
        "font-weight:600",
        "font-weight:700",
        "font-weight:bold",
        "font-weight:lighter",
        "font-weight:bolder",
        "font-weight:400; font-weight:700",
        "font-style:normal",
    ] {
        assert_eq!(
            conv(&format!(r#"<p><b style="{style}">x</b></p>"#)),
            "**x**\n",
            "{style}"
        );
    }
}

#[test]
fn own_style_negates_italic() {
    assert_eq!(conv(r#"<p><i style="font-style:normal">x</i></p>"#), "x\n");
    assert_eq!(
        conv(r#"<p><em style="font-style: Normal">x</em></p>"#),
        "x\n"
    );
    assert_eq!(
        conv(r#"<p><em style="font-style:italic">x</em></p>"#),
        "*x*\n"
    );
    assert_eq!(
        conv(r#"<p><i style="font-weight:normal">x</i></p>"#),
        "*x*\n"
    );
}

#[test]
fn style_never_adds_emphasis_and_is_not_inherited() {
    assert_eq!(
        conv(r#"<p><span style="font-weight:700">x</span></p>"#),
        "x\n"
    );
    assert_eq!(
        conv(r#"<p><span style="font-weight:normal"><b>x</b></span></p>"#),
        "**x**\n"
    );
    assert_eq!(
        conv(r#"<p><b style="font-weight:normal">a <b>b</b> c</b></p>"#),
        "a **b** c\n"
    );
}

// ── criterion 6: code around blocks ────────────────────────────────────────

#[test]
fn code_around_blocks_writes_no_backticks_and_keeps_the_blocks() {
    assert_eq!(conv("<code><p>x</p><p>y</p></code>"), "x\n\ny\n");
    assert_eq!(conv("<code><ul><li>a</li></ul></code>"), "- a\n");
    assert_eq!(conv("<code><pre>x</pre></code>"), "```\nx\n```\n");
}

// ── criterion 7: a link around blocks links each block's content ───────────

#[test]
fn link_around_blocks_links_each_block() {
    assert_eq!(
        conv(r#"<a href="/x"><h2>Title</h2></a>"#),
        "## [Title](/x)\n"
    );
    assert_eq!(
        conv(r#"<a href="/out"><p>x</p><p>y</p></a>"#),
        "[x](/out)\n\n[y](/out)\n"
    );
    assert_eq!(
        conv(r#"<a href="/x"><ul><li>one</li><li>two</li></ul></a>"#),
        "- [one](/x)\n- [two](/x)\n"
    );
    assert_eq!(
        conv(r#"<a href="/x"><blockquote><p>q</p></blockquote></a>"#),
        "> [q](/x)\n"
    );
    assert_eq!(
        conv(r#"<a href="/x" title="T"><h2>Title</h2></a>"#),
        "## [Title](/x \"T\")\n"
    );
}

#[test]
fn link_around_pre_does_not_link_the_code_block() {
    // A code block holds text only (RFC 024 criterion 3).
    assert_eq!(
        conv(r#"<a href="/x"><pre>code</pre></a>"#),
        "```\ncode\n```\n"
    );
}

#[test]
fn block_with_no_text_inside_a_link_gets_no_link() {
    assert_eq!(conv(r#"<a href="/x"><p></p><p>y</p></a>"#), "[y](/x)\n");
}

#[test]
fn inline_content_inside_a_linked_block_stays_inside_its_link() {
    assert_eq!(
        conv(r#"<a href="/x"><p>see <strong>bold</strong> and <img src="i.png" alt="i"></p></a>"#),
        "[see **bold** and ![i](i.png)](/x)\n"
    );
}

#[test]
fn inner_link_inside_a_linked_block_contributes_text_only() {
    // Never [[t](/in)](/out). html5ever closes the outer <a> when the inner one
    // opens, so the paragraph holds the inner link, and the outer link is left
    // with no text and emits nothing.
    assert_eq!(
        conv(r#"<a href="/out"><p><a href="/in">t</a></p></a>"#),
        "[t](/in)\n"
    );
}

// ── criterion 8: nested inline-around-block ────────────────────────────────

#[test]
fn nested_inline_elements_around_blocks() {
    assert_eq!(conv("<b><em><p>x</p></em></b>"), "x\n");
    assert_eq!(conv("<em><strong><p>x</p></strong></em>"), "x\n");
    assert_eq!(
        conv(r#"<a href="/x"><strong><p>x</p></strong></a>"#),
        "[x](/x)\n"
    );
    assert_eq!(
        conv(r#"<strong><a href="/x"><p>x</p></a></strong>"#),
        "[x](/x)\n"
    );
}

// ── criterion 9: mixed inline and block children ───────────────────────────

#[test]
fn mixed_inline_and_block_children() {
    // Emphasis: no delimiters anywhere -- the inline runs are plain text.
    assert_eq!(conv("<b>text<p>para</p></b>"), "text\n\npara\n");
    assert_eq!(
        conv("<em>lead<p>para</p>tail</em>"),
        "lead\n\npara\n\ntail\n"
    );
    // Code: no backticks anywhere.
    assert_eq!(conv("<code>lead<p>para</p></code>"), "lead\n\npara\n");
    // Link: every run of inline content is linked, blocks and runs alike.
    assert_eq!(
        conv(r#"<a href="/x">lead <p>para</p> tail</a>"#),
        "[lead](/x)\n\n[para](/x)\n\n[tail](/x)\n"
    );
}

// ── criterion 10: purely inline content does not move ──────────────────────

#[test]
fn emphasis_and_links_around_inline_content_are_unchanged() {
    assert_eq!(conv("<p>a <strong>b</strong> c</p>"), "a **b** c\n");
    assert_eq!(conv("<p>a <em>b</em> c</p>"), "a *b* c\n");
    // RFC 037 §1.1 B: `**` then `*` concatenate into `***`, which reparses
    // the same way regardless of which order wrote it -- and before RFC
    // 037, this order wrote it wrong (`em(strong(x))`, source order
    // inverted). Fixed by swapping the inner `<em>`'s delimiter to `_`,
    // which only this direction needs -- `<em><strong>` already reparses
    // correctly as plain `*`/`**` and is unchanged (see
    // `tests/output_validity/emphasis_fidelity.rs`).
    assert_eq!(conv("<p><strong><em>x</em></strong></p>"), "**_x_**\n");
    assert_eq!(
        conv(r#"<p><a href="/x"><strong>t</strong></a></p>"#),
        "[**t**](/x)\n"
    );
    assert_eq!(conv("<p>use <code>x</code></p>"), "use `x`\n");
}

// ── criterion 11: deep nesting stays linear and stack-safe ─────────────────

#[test]
fn deeply_nested_emphasis_around_a_large_payload() {
    let depth = 5_000;
    let paragraphs = 2_000;
    let mut html = String::new();
    for _ in 0..depth {
        html.push_str("<b>");
    }
    for i in 0..paragraphs {
        html.push_str(&format!("<p>p{i}</p>"));
    }
    for _ in 0..depth {
        html.push_str("</b>");
    }
    let md = conv(&html);
    assert!(!md.contains('*'));
    assert!(md.starts_with("p0\n\np1\n"));
    assert_eq!(md.matches("\n\n").count(), paragraphs - 1);
}
