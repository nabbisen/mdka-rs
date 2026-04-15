//! Integration tests: cross-library compatibility
//! Verifies that html2md and fast_html2md coexist (Cargo package alias),
//! and documents behaviour differences vs mdka.

mod common;
use common::conv;

use mdka::options::{ConversionMode, ConversionOptions};

// ─── html2md (0.2) + fast_html2md coexistence ────────────────────────────

#[test]
fn html2md_coexists_with_fast_html2md() {
    // Cargo エイリアス設定により両者を同一バイナリで使用できることを確認:
    //   html2md      = "0.2"
    //   fast-html2md = { package = "fast_html2md", version = "0.0.61" }
    let html = "<h1>Hello</h1><p>World</p>";
    let md_h2md = html2md::parse_html(html);
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(md_h2md.contains("Hello"), "html2md: {md_h2md:?}");
    assert!(md_fast.contains("Hello"), "fast_html2md: {md_fast:?}");
}

#[test]
fn all_three_produce_heading() {
    let html = "<h1>Test Heading</h1>";
    let md_mdka = conv(html);
    let md_h2md = html2md::parse_html(html);
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(md_mdka.contains("# Test Heading"), "mdka: {md_mdka:?}");
    assert!(md_h2md.contains("Test Heading"), "html2md: {md_h2md:?}");
    assert!(
        md_fast.contains("Test Heading"),
        "fast_html2md: {md_fast:?}"
    );
}

// ─── mdka vs fast_html2md behaviour differences ───────────────────────────

#[test]
fn both_produce_heading_from_h1() {
    let html = "<h1>Hello World</h1>";
    let md_mdka = conv(html);
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(md_mdka.contains("# Hello World"), "mdka: {md_mdka:?}");
    assert!(md_fast.contains("Hello World"), "fast_html2md: {md_fast:?}");
}

#[test]
fn both_produce_list_items() {
    let html = "<ul><li>A</li><li>B</li></ul>";
    let md_mdka = conv(html);
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(
        md_mdka.contains("A") && md_mdka.contains("B"),
        "mdka: {md_mdka:?}"
    );
    assert!(
        md_fast.contains("A") && md_fast.contains("B"),
        "fast: {md_fast:?}"
    );
}

#[test]
fn both_preserve_link_href() {
    let html = r#"<a href="https://example.com">Click</a>"#;
    let md_mdka = conv(html);
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(md_mdka.contains("https://example.com"), "mdka: {md_mdka:?}");
    assert!(md_fast.contains("https://example.com"), "fast: {md_fast:?}");
}

#[test]
fn mdka_handles_deep_nest_fast_html2md_does_not() {
    // fast_html2md は深いネストでスタックオーバーフローするため除外
    let html = format!(
        "{}<p>deep</p>{}",
        "<div>".repeat(5_000),
        "</div>".repeat(5_000)
    );
    let md = conv(&html);
    assert!(
        md.contains("deep"),
        "mdka deep nest: {}",
        &md[..md.len().min(50)]
    );
}

#[test]
fn mdka_mode_minimal_strips_nav_fast_html2md_keeps_it() {
    let html = "<nav><a href='/'>Home</a></nav><main><p>Content</p></main>";
    let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
    opts.drop_interactive_shell = true;
    let md_minimal = mdka::html_to_markdown_with(html, &opts);
    assert!(
        !md_minimal.to_lowercase().contains("home"),
        "nav leaked: {md_minimal:?}"
    );
    assert!(
        md_minimal.contains("Content"),
        "content missing: {md_minimal:?}"
    );
    // fast_html2md にはモードがない
    let md_fast = fast_html2md::rewrite_html(html, false);
    assert!(
        md_fast.contains("Home") || md_fast.contains("Content"),
        "fast_html2md output: {md_fast:?}"
    );
}
