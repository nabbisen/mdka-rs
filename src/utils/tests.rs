use super::*;

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
        // RFC 008: a table row or cell, on its own, is paragraph-like -- the
        // floor that keeps cells from welding together wherever a table
        // isn't rendered specially.
        "tr",
        "td",
        "th",
        "caption",
    ] {
        assert_eq!(block_kind(tag), Some(Block::Paragraph), "{tag}");
    }
    assert_eq!(block_kind("ul"), Some(Block::UnorderedList));
    assert_eq!(block_kind("ol"), Some(Block::OrderedList));
    assert_eq!(block_kind("li"), Some(Block::ListItem));
    assert_eq!(block_kind("blockquote"), Some(Block::Quote));
    assert_eq!(block_kind("pre"), Some(Block::Pre));
    assert_eq!(block_kind("hr"), Some(Block::Rule));
    for tag in ["span", "strong", "a", "code", "br", "img", "table"] {
        assert_eq!(block_kind(tag), None, "{tag}");
    }
}

#[test]
fn font_weight_bold_or_not_bold_by_keyword_or_threshold() {
    assert_eq!(style_font_weight(Some("font-weight:normal")), Some(false));
    assert_eq!(style_font_weight(Some("font-weight:400")), Some(false));
    assert_eq!(style_font_weight(Some("font-weight:500")), Some(false));
    assert_eq!(style_font_weight(Some("font-weight:600")), Some(true));
    assert_eq!(style_font_weight(Some("font-weight:700")), Some(true));
    assert_eq!(style_font_weight(Some("font-weight:bold")), Some(true));
}

#[test]
fn font_weight_relative_keywords_and_unparseable_middle_say_nothing() {
    // `bolder`/`lighter` are relative to the parent's own computed weight,
    // which mdka has no parent weight to resolve against -- unresolved, not
    // guessed at, the same as before this RFC (RFC 028 Amendment 1) and
    // the same as an unparseable value (RFC 049).
    assert_eq!(style_font_weight(Some("font-weight:lighter")), None);
    assert_eq!(style_font_weight(Some("font-weight:bolder")), None);
    assert_eq!(style_font_weight(Some("font-weight:550")), None);
    assert_eq!(style_font_weight(Some("font-weight:not-a-number")), None);
}

#[test]
fn font_weight_parsing_edge_cases() {
    // !important stripped, with or without a space
    assert_eq!(
        style_font_weight(Some("font-weight: normal !important")),
        Some(false)
    );
    assert_eq!(
        style_font_weight(Some("font-weight:normal!important")),
        Some(false)
    );
    // case-folded names and values
    assert_eq!(style_font_weight(Some("FONT-WEIGHT: NORMAL")), Some(false));
    // whitespace around names, values and separators
    assert_eq!(
        style_font_weight(Some("  font-weight  :  normal  ;  ")),
        Some(false)
    );
    // the last declaration of the property wins
    assert_eq!(
        style_font_weight(Some("font-weight:700; font-weight:400")),
        Some(false)
    );
    assert_eq!(
        style_font_weight(Some("font-weight:400; font-weight:700")),
        Some(true)
    );
    // other properties present and ignored
    assert_eq!(
        style_font_weight(Some(
            "color:red; font-weight:normal; text-decoration:underline"
        )),
        Some(false)
    );
    // a declaration without a colon is skipped
    assert_eq!(
        style_font_weight(Some("bogus; font-weight:normal")),
        Some(false)
    );
    // absent style, or no font-weight declaration at all
    assert_eq!(style_font_weight(None), None);
    assert_eq!(style_font_weight(Some("")), None);
    assert_eq!(style_font_weight(Some("color:red")), None);
}

#[test]
fn font_style_italic_or_not_italic() {
    assert_eq!(style_font_style(Some("font-style:normal")), Some(false));
    assert_eq!(style_font_style(Some("font-style: Normal")), Some(false));
    assert_eq!(style_font_style(Some("font-style:italic")), Some(true));
    assert_eq!(style_font_style(Some("font-style:oblique")), Some(true));
    assert_eq!(style_font_style(Some("font-weight:normal")), None);
    assert_eq!(style_font_style(None), None);
}
