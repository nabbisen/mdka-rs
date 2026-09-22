//! Integration tests: RFC 039 §3 A2 — `html_to_markdown_many` / `_with`.
//!
//! Criterion 3: `html_to_markdown_many` must return the same values as
//! mapping `html_to_markdown` over the inputs, and its `_with` form must
//! match `html_to_markdown_with`.

use mdka::options::{ConversionMode, ConversionOptions};

#[test]
fn many_matches_mapping_html_to_markdown_over_the_inputs() {
    let inputs = [
        "<h1>A</h1>",
        "<p>B</p>",
        "<em>C</em>",
        "<ul><li>D</li></ul>",
    ];
    let many = mdka::html_to_markdown_many(&inputs);
    let mapped: Vec<String> = inputs.iter().map(|h| mdka::html_to_markdown(h)).collect();
    assert_eq!(many, mapped);
}

#[test]
fn many_with_matches_mapping_html_to_markdown_with_over_the_inputs() {
    let inputs = [
        r#"<nav><a href="/">Home</a></nav><main><p>Content</p></main>"#,
        "<h2>Title</h2>",
    ];
    let opts = {
        let mut o = ConversionOptions::for_mode(ConversionMode::Minimal);
        o.drop_interactive_shell = true;
        o
    };
    let many = mdka::html_to_markdown_many_with(&inputs, &opts);
    let mapped: Vec<String> = inputs
        .iter()
        .map(|h| mdka::html_to_markdown_with(h, &opts))
        .collect();
    assert_eq!(many, mapped);
}

#[test]
fn many_empty_input_returns_empty_vec() {
    let empty: [&str; 0] = [];
    assert!(mdka::html_to_markdown_many(&empty).is_empty());
    assert!(mdka::html_to_markdown_many_with(&empty, &ConversionOptions::default()).is_empty());
}

#[test]
fn many_preserves_input_order() {
    let inputs: Vec<String> = (0..30).map(|i| format!("<h1>Item {i}</h1>")).collect();
    let results = mdka::html_to_markdown_many(&inputs);
    assert_eq!(results.len(), inputs.len());
    for (i, md) in results.iter().enumerate() {
        assert!(md.contains(&format!("Item {i}")), "got: {md}");
    }
}
