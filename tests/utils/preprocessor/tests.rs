use super::*;
use crate::options::ConversionOptions;

fn run(html: &str, opts: &ConversionOptions) -> String {
    let doc = Html::parse_document(html);
    utils::preprocessor::preprocess(&doc, opts)
}

#[test]
fn test_script_always_dropped() {
    let out = run(
        "<p>Text</p><script>alert(1)</script>",
        &ConversionOptions::for_mode(ConversionMode::Strict),
    );
    assert!(!out.contains("script"), "script leaked: {out}");
    assert!(out.contains("Text"));
}

#[test]
fn test_balanced_drops_class_style() {
    let out = run(
        r#"<p class="foo" style="color:red">Hi</p>"#,
        &ConversionOptions::for_mode(ConversionMode::Balanced),
    );
    assert!(!out.contains("class="), "class leaked");
    assert!(!out.contains("style="), "style leaked");
    assert!(out.contains("Hi"));
}

#[test]
fn test_strict_keeps_class_data() {
    let out = run(
        r#"<p class="foo" data-x="1">Hi</p>"#,
        &ConversionOptions::for_mode(ConversionMode::Strict),
    );
    assert!(out.contains("class="), "class missing in strict");
    assert!(out.contains("data-x="), "data-x missing in strict");
}

#[test]
fn test_minimal_drops_shell() {
    let out = run(
        "<nav><a href='/'>Home</a></nav><main><p>Content</p></main>",
        &ConversionOptions::for_mode(ConversionMode::Minimal),
    );
    assert!(!out.contains("<nav>"), "nav leaked in minimal");
    assert!(out.contains("Content"));
}

#[test]
fn test_semantic_keeps_aria() {
    let out = run(
        r#"<button aria-label="close" class="btn">X</button>"#,
        &ConversionOptions::for_mode(ConversionMode::Semantic),
    );
    assert!(out.contains("aria-label="), "aria missing in semantic");
    assert!(!out.contains("class="), "class leaked in semantic");
}

#[test]
fn test_preserve_keeps_comments() {
    let out = run(
        "<!-- note --><p>Text</p>",
        &ConversionOptions::for_mode(ConversionMode::Preserve),
    );
    assert!(out.contains("<!-- note -->"), "comment missing in preserve");
}

#[test]
fn test_balanced_drops_comment() {
    let out = run(
        "<!-- note --><p>Text</p>",
        &ConversionOptions::for_mode(ConversionMode::Balanced),
    );
    assert!(!out.contains("<!--"), "comment leaked in balanced");
}

#[test]
fn test_href_always_preserved() {
    for mode in [
        ConversionMode::Balanced,
        ConversionMode::Minimal,
        ConversionMode::Strict,
        ConversionMode::Semantic,
        ConversionMode::Preserve,
    ] {
        let out = run(
            r#"<a href="https://example.com">Link</a>"#,
            &ConversionOptions::for_mode(mode),
        );
        assert!(out.contains("href="), "href missing in {mode}");
    }
}

#[test]
fn test_code_lang_class_always_kept() {
    let out = run(
        r#"<pre><code class="language-rust">fn main(){}</code></pre>"#,
        &ConversionOptions::for_mode(ConversionMode::Balanced),
    );
    assert!(
        out.contains("language-rust"),
        "lang class missing in balanced"
    );
}

#[test]
fn test_deep_nest_no_stack_overflow() {
    let open = "<div>".repeat(10_000);
    let close = "</div>".repeat(10_000);
    let html = format!("{}<p>deep</p>{}", open, close);
    let doc = Html::parse_document(&html);
    let out = utils::preprocessor::preprocess(&doc, &ConversionOptions::default());
    assert!(out.contains("deep"), "deep content missing");
}
