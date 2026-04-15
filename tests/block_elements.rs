//! Integration tests: block elements
//! Covers: lists, blockquotes, code blocks, escape, script/style filtering

mod common;
use common::conv;

// ─── Unordered list ───────────────────────────────────────────────────────

#[test]
fn unordered_list() {
    let md = conv("<ul><li>A</li><li>B</li><li>C</li></ul>");
    assert!(md.contains("- A"), "got: {md}");
    assert!(md.contains("- B"), "got: {md}");
    assert!(md.contains("- C"), "got: {md}");
}

#[test]
fn unordered_list_nested() {
    let md = conv("<ul><li>P<ul><li>C</li></ul></li></ul>");
    assert!(md.contains("- P"), "parent missing: {md}");
    assert!(md.contains("  - C"), "child missing: {md}");
}

// ─── Ordered list ─────────────────────────────────────────────────────────

#[test]
fn ordered_list() {
    let md = conv("<ol><li>One</li><li>Two</li><li>Three</li></ol>");
    assert!(md.contains("1. One"), "got: {md}");
    assert!(md.contains("2. Two"), "got: {md}");
}

#[test]
fn ordered_list_with_start_attr() {
    let md = conv(r#"<ol start="5"><li>Five</li><li>Six</li></ol>"#);
    assert!(md.contains("5. Five"), "got: {md}");
    assert!(md.contains("6. Six"), "got: {md}");
}

#[test]
fn ordered_list_nested_inside_unordered() {
    let md = conv("<ul><li>X<ol><li>A</li><li>B</li></ol></li></ul>");
    assert!(md.contains("- X"), "outer missing: {md}");
    assert!(md.contains("1. A"), "inner missing: {md}");
}

// ─── Blockquote ───────────────────────────────────────────────────────────

#[test]
fn blockquote() {
    let md = conv("<blockquote><p>Quoted</p></blockquote>");
    assert!(md.contains("> "), "prefix missing: {md}");
    assert!(md.contains("Quoted"), "content missing: {md}");
}

#[test]
fn blockquote_nested() {
    let md = conv("<blockquote><blockquote><p>Deep</p></blockquote></blockquote>");
    assert!(md.contains("> > "), "nested prefix missing: {md}");
    assert!(md.contains("Deep"), "content missing: {md}");
}

#[test]
fn blockquote_with_list() {
    let md = conv("<blockquote><ul><li>Item</li></ul></blockquote>");
    assert!(md.contains(">"), "blockquote prefix missing: {md}");
    assert!(md.contains("Item"), "list item missing: {md}");
}

// ─── Code blocks ──────────────────────────────────────────────────────────

#[test]
fn pre_code_no_lang() {
    let md = conv("<pre><code>fn main() {}</code></pre>");
    assert!(md.contains("```\nfn main()"), "got: {md}");
}

#[test]
fn pre_code_with_lang() {
    let md = conv(r#"<pre><code class="language-rust">fn main() {}</code></pre>"#);
    assert!(md.contains("```rust\nfn main()"), "got: {md}");
}

#[test]
fn pre_preserves_whitespace() {
    let md = conv("<pre><code>line1\n  line2\n    line3</code></pre>");
    assert!(md.contains("  line2"), "indented line missing: {md}");
}

#[test]
fn pre_without_code_child() {
    let md = conv("<pre>plain text</pre>");
    assert!(md.contains("plain text"), "got: {md}");
}

// ─── Markdown escaping ────────────────────────────────────────────────────

#[test]
fn escape_asterisk_in_text() {
    let md = conv("<p>2 * 3</p>");
    assert!(md.contains("\\*"), "escape missing: {md}");
}

#[test]
fn escape_hash_at_line_start() {
    let md = conv("<p># not a heading</p>");
    assert!(md.contains("\\#"), "escape missing: {md}");
}

// ─── Script / style filtering ─────────────────────────────────────────────

#[test]
fn script_ignored() {
    let md = conv("<script>alert(1)</script><p>Visible</p>");
    assert!(!md.contains("alert"), "script leaked: {md}");
    assert!(md.contains("Visible"), "content missing: {md}");
}

#[test]
fn style_ignored() {
    let md = conv("<style>body { color: red; }</style><p>Text</p>");
    assert!(!md.contains("color"), "style leaked: {md}");
    assert!(md.contains("Text"), "content missing: {md}");
}

// ─── Output format ────────────────────────────────────────────────────────

#[test]
fn output_ends_with_single_newline() {
    let md = conv("<p>Hello</p>");
    assert!(md.ends_with('\n'), "no trailing newline: {md:?}");
    assert!(!md.ends_with("\n\n"), "double trailing newline: {md:?}");
}

#[test]
fn div_does_not_add_blank_lines_when_empty() {
    let md = conv("<div></div><p>Text</p>");
    assert!(md.trim() == "Text", "got: {md:?}");
}
