//! Integration tests: inline elements
//! Covers: headings, paragraphs, emphasis, inline code, links, images, br, hr

mod common;
use common::conv;

// ─── Headings ─────────────────────────────────────────────────────────────

#[test]
fn heading_h1() {
    assert!(conv("<h1>Hello</h1>").contains("# Hello"));
}

#[test]
fn heading_h2() {
    assert!(conv("<h2>Hello</h2>").contains("## Hello"));
}

#[test]
fn heading_h6() {
    assert!(conv("<h6>Hello</h6>").contains("###### Hello"));
}

#[test]
fn heading_levels_all() {
    for i in 1..=6 {
        let md = conv(&format!("<h{i}>T</h{i}>"));
        assert!(md.contains(&format!("{} T", "#".repeat(i))), "h{i}: {md}");
    }
}

// ─── Paragraphs ───────────────────────────────────────────────────────────

#[test]
fn paragraph_simple() {
    assert!(conv("<p>Hello world</p>").contains("Hello world"));
}

#[test]
fn paragraph_whitespace_collapse() {
    let md = conv("<p>Hello   world</p>");
    assert!(md.contains("Hello world"), "got: {md}");
}

#[test]
fn two_paragraphs_have_blank_line() {
    let md = conv("<p>First</p><p>Second</p>");
    assert!(md.contains("First"), "first missing");
    assert!(md.contains("Second"), "second missing");
    assert!(md.contains("\n\n"), "blank line missing: {md:?}");
}

// ─── Emphasis ─────────────────────────────────────────────────────────────

#[test]
fn strong() {
    let md = conv("<p><strong>bold</strong></p>");
    assert!(md.contains("**bold**"), "got: {md}");
}

#[test]
fn em() {
    let md = conv("<p><em>italic</em></p>");
    assert!(md.contains("*italic*"), "got: {md}");
}

#[test]
fn bold_and_em_combined() {
    let md = conv("<p><strong><em>both</em></strong></p>");
    assert!(md.contains("both"), "got: {md}");
}

// ─── Inline code ──────────────────────────────────────────────────────────

#[test]
fn inline_code() {
    let md = conv("<p><code>snippet</code></p>");
    assert!(md.contains("`snippet`"), "got: {md}");
}

// ─── Links ────────────────────────────────────────────────────────────────

#[test]
fn link_basic() {
    let md = conv(r#"<a href="https://example.com">Click</a>"#);
    assert!(md.contains("[Click](https://example.com)"), "got: {md}");
}

#[test]
fn link_with_title() {
    let md = conv(r#"<a href="https://example.com" title="Tip">Click</a>"#);
    assert!(
        md.contains(r#"[Click](https://example.com "Tip")"#),
        "got: {md}"
    );
}

#[test]
fn link_empty_href() {
    let md = conv(r#"<a href="">Click</a>"#);
    assert!(md.contains("Click"), "got: {md}");
}

#[test]
fn link_text_with_inline_formatting() {
    let md = conv(r#"<a href="https://example.com"><strong>bold link</strong></a>"#);
    assert!(md.contains("bold link"), "got: {md}");
    assert!(md.contains("https://example.com"), "got: {md}");
}

// ─── Images ───────────────────────────────────────────────────────────────

#[test]
fn image_basic() {
    let md = conv(r#"<img src="img.png" alt="Alt text">"#);
    assert!(md.contains("![Alt text](img.png)"), "got: {md}");
}

#[test]
fn image_with_title() {
    let md = conv(r#"<img src="img.png" alt="Alt" title="Cap">"#);
    assert!(md.contains(r#"![Alt](img.png "Cap")"#), "got: {md}");
}

#[test]
fn img_missing_alt() {
    let md = conv(r#"<img src="img.png">"#);
    assert!(md.contains("img.png"), "got: {md}");
}

// ─── Line break / horizontal rule ─────────────────────────────────────────

#[test]
fn line_break() {
    let md = conv("<p>line1<br>line2</p>");
    assert!(md.contains("  \n"), "hard line break missing: {md:?}");
}

#[test]
fn horizontal_rule() {
    let md = conv("<hr>");
    assert!(md.contains("---"), "got: {md}");
}

// ─── Whitespace ───────────────────────────────────────────────────────────

#[test]
fn whitespace_between_inline_elements() {
    let md = conv("<p><strong>a</strong> <em>b</em></p>");
    assert!(md.contains("a"), "a missing: {md}");
    assert!(md.contains("b"), "b missing: {md}");
}

#[test]
fn consecutive_inline_elements() {
    let md = conv("<p><strong>bold</strong><em>italic</em></p>");
    assert!(md.contains("bold"), "got: {md}");
    assert!(md.contains("italic"), "got: {md}");
}
