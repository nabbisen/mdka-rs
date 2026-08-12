//! Integration tests: RFC 005 Slice B1 — `preserve_ids` anchor emission
//! Covers: all required output and escaping cases from the RFC 005
//! Slices B/C handoff §5, verifying `preserve_ids` emits an escaped
//! `<a id="...">` anchor before a non-empty-`id` element's own output,
//! and emits nothing for an absent or empty `id`, or when `preserve_ids`
//! is off.

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

fn with_preserve_ids(v: bool) -> ConversionOptions {
    ConversionOptions::for_mode(ConversionMode::Balanced).preserve_ids(v)
}

#[test]
fn heading_with_id_gets_anchor() {
    assert_eq!(
        conv_with(r#"<h2 id="install">Install</h2>"#, &with_preserve_ids(true)),
        "<a id=\"install\"></a>\n\n## Install\n"
    );
}

#[test]
fn paragraph_with_id_gets_anchor() {
    assert_eq!(
        conv_with(r#"<p id="intro">Text</p>"#, &with_preserve_ids(true)),
        "<a id=\"intro\"></a>\n\nText\n"
    );
}

#[test]
fn inline_element_with_id_gets_inline_anchor() {
    let md = conv_with(
        r#"<p>a <span id="s">b</span> c</p>"#,
        &with_preserve_ids(true),
    );
    assert_eq!(md, "a <a id=\"s\"></a>b c\n");
}

#[test]
fn heading_without_id_is_unchanged() {
    assert_eq!(
        conv_with("<h2>No id</h2>", &with_preserve_ids(true)),
        "## No id\n"
    );
}

#[test]
fn empty_id_emits_no_anchor() {
    assert_eq!(
        conv_with(r#"<h2 id="">Empty</h2>"#, &with_preserve_ids(true)),
        "## Empty\n"
    );
}

#[test]
fn preserve_ids_false_emits_no_anchor() {
    assert_eq!(
        conv_with(
            r#"<h2 id="install">Install</h2>"#,
            &with_preserve_ids(false)
        ),
        "## Install\n"
    );
}

#[test]
fn id_escaping_quote_and_attribute_injection() {
    // "&" must be escaped before """, or the escaped output double-escapes.
    let html = "<p id=\"x&quot; onload=&quot;alert(1)\">Hi</p>";
    let md = conv_with(html, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"x&quot; onload=&quot;alert(1)\"></a>\n\nHi\n");
}

#[test]
fn id_escaping_ampersand() {
    let md = conv_with(r#"<p id="a&b">Hi</p>"#, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"a&amp;b\"></a>\n\nHi\n");
}

#[test]
fn id_escaping_plain_id_unaffected() {
    let md = conv_with(r#"<p id="plain-id">Hi</p>"#, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"plain-id\"></a>\n\nHi\n");
}
