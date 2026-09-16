use super::*;

fn normalise(s: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    let mut at_line = true;
    write_normalised(s, &mut out, &mut last_space, false, &mut at_line);
    out
}

#[test]
fn test_escape_asterisk() {
    assert_eq!(normalise("*bold*"), "\\*bold\\*");
}

#[test]
fn test_escape_backtick() {
    assert_eq!(normalise("`code`"), "\\`code\\`");
}

#[test]
fn test_no_escape_in_middle() {
    // '#' は行頭以外ではエスケープしない
    let mut out = String::new();
    let mut last_space = false;
    let mut at_line = false; // 行頭でない
    write_normalised("foo#bar", &mut out, &mut last_space, false, &mut at_line);
    assert_eq!(out, "foo#bar");
}

#[test]
fn test_whitespace_collapse() {
    let mut out = String::new();
    let mut last_space = false;
    let mut at_line = false;
    write_normalised(
        "hello   world\t\nfoo",
        &mut out,
        &mut last_space,
        false,
        &mut at_line,
    );
    assert_eq!(out, "hello world foo");
}

#[test]
fn test_block_start_trims_leading_space_but_keeps_internal() {
    let mut out = String::new();
    let mut last_space = false;
    let mut at_line = true;
    // ブロック先頭でも内部スペースは保持される
    write_normalised("Hello world", &mut out, &mut last_space, true, &mut at_line);
    assert_eq!(out, "Hello world");
}

#[test]
fn test_extract_code_lang() {
    assert_eq!(extract_code_lang(Some("language-rust")), Some("rust"));
    assert_eq!(
        extract_code_lang(Some("highlight language-python extra")),
        Some("python")
    );
    assert_eq!(extract_code_lang(Some("no-lang")), None);
    assert_eq!(extract_code_lang(None), None);
}

// ─── タグ分類ヘルパーのテスト ──────────────────────────────────────────────

#[test]
fn test_is_skip_tag() {
    assert!(is_skip_tag("script"));
    assert!(is_skip_tag("style"));
    assert!(is_skip_tag("head"));
    assert!(is_skip_tag("svg"));
    assert!(!is_skip_tag("div"));
    assert!(!is_skip_tag("p"));
}

#[test]
fn test_is_shell_tag() {
    assert!(is_shell_tag("nav"));
    assert!(is_shell_tag("footer"));
    assert!(!is_shell_tag("main"));
}

#[test]
fn test_is_structural_tag() {
    assert!(is_structural_tag("h1"));
    assert!(is_structural_tag("p"));
    assert!(is_structural_tag("a"));
    assert!(!is_structural_tag("div"));
    assert!(!is_structural_tag("span"));
}

// ─── RFC 028 ───────────────────────────────────────────────────────────────

#[test]
fn block_kind_classifies_every_block_the_renderer_emits() {
    for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        assert!(matches!(block_kind(tag), Some(Block::Heading(_))), "{tag}");
    }
    for tag in [
        "p",
        "div",
        "article",
        "section",
        "main",
        "header",
        "footer",
        "nav",
        "aside",
        "figure",
        "figcaption",
    ] {
        assert_eq!(block_kind(tag), Some(Block::Paragraph), "{tag}");
    }
    assert_eq!(block_kind("ul"), Some(Block::UnorderedList));
    assert_eq!(block_kind("ol"), Some(Block::OrderedList));
    assert_eq!(block_kind("li"), Some(Block::ListItem));
    assert_eq!(block_kind("blockquote"), Some(Block::Quote));
    assert_eq!(block_kind("pre"), Some(Block::Pre));
    assert_eq!(block_kind("hr"), Some(Block::Rule));
    for tag in ["span", "strong", "a", "code", "br", "img", "table", "td"] {
        assert_eq!(block_kind(tag), None, "{tag}");
    }
}

#[test]
fn style_negates_bold_with_normal_or_weight_up_to_500() {
    assert!(emphasis_negated_by_style("b", Some("font-weight:normal")));
    assert!(emphasis_negated_by_style(
        "strong",
        Some("font-weight:normal")
    ));
    assert!(emphasis_negated_by_style("b", Some("font-weight:400")));
    assert!(emphasis_negated_by_style("b", Some("font-weight:500")));
    assert!(!emphasis_negated_by_style("b", Some("font-weight:600")));
    assert!(!emphasis_negated_by_style("b", Some("font-weight:700")));
    assert!(!emphasis_negated_by_style("b", Some("font-weight:bold")));
}

#[test]
fn style_relative_weights_do_not_negate() {
    assert!(!emphasis_negated_by_style("b", Some("font-weight:lighter")));
    assert!(!emphasis_negated_by_style("b", Some("font-weight:bolder")));
}

#[test]
fn style_parsing_edge_cases() {
    // !important stripped, with or without a space
    assert!(emphasis_negated_by_style(
        "b",
        Some("font-weight: normal !important")
    ));
    assert!(emphasis_negated_by_style(
        "b",
        Some("font-weight:normal!important")
    ));
    // case-folded names and values
    assert!(emphasis_negated_by_style("b", Some("FONT-WEIGHT: NORMAL")));
    // whitespace around names, values and separators
    assert!(emphasis_negated_by_style(
        "b",
        Some("  font-weight  :  normal  ;  ")
    ));
    // the last declaration of the property wins
    assert!(emphasis_negated_by_style(
        "b",
        Some("font-weight:700; font-weight:400")
    ));
    assert!(!emphasis_negated_by_style(
        "b",
        Some("font-weight:400; font-weight:700")
    ));
    // other properties present and ignored
    assert!(emphasis_negated_by_style(
        "b",
        Some("color:red; font-weight:normal; text-decoration:underline")
    ));
    // a declaration without a colon is skipped
    assert!(emphasis_negated_by_style(
        "b",
        Some("bogus; font-weight:normal")
    ));
}

#[test]
fn style_reads_only_the_property_for_the_element() {
    assert!(emphasis_negated_by_style("i", Some("font-style:normal")));
    assert!(emphasis_negated_by_style("em", Some("font-style: Normal")));
    assert!(!emphasis_negated_by_style("em", Some("font-style:italic")));
    assert!(!emphasis_negated_by_style("i", Some("font-weight:normal")));
    assert!(!emphasis_negated_by_style("b", Some("font-style:normal")));
    assert!(!emphasis_negated_by_style(
        "span",
        Some("font-weight:normal")
    ));
    assert!(!emphasis_negated_by_style("b", None));
    assert!(!emphasis_negated_by_style("b", Some("")));
}
