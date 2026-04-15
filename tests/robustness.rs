//! Integration tests: robustness and edge cases
//! Covers: deep nesting, malformed HTML, empty/whitespace input, HTML entities

mod common;
use common::conv;

#[test]
fn deep_nest_no_stack_overflow() {
    let open: String = "<div>".repeat(5_000);
    let close: String = "</div>".repeat(5_000);
    let html = format!("{}<p>deep</p>{}", open, close);
    let md = conv(&html);
    assert!(
        md.contains("deep"),
        "content missing: {}",
        &md[..md.len().min(50)]
    );
}

#[test]
fn missing_closing_tags() {
    let md = conv("<p>Unclosed<ul><li>Item");
    assert!(md.contains("Unclosed"), "got: {md}");
    assert!(md.contains("Item"), "got: {md}");
}

#[test]
fn empty_input() {
    assert!(conv("").trim().is_empty());
}

#[test]
fn only_whitespace_text() {
    assert!(conv("   \n\t  ").trim().is_empty());
}

#[test]
fn text_with_html_entities_passthrough() {
    let md = conv("<p>&amp; &lt; &gt;</p>");
    assert!(md.contains("&"), "ampersand missing: {md}");
}
