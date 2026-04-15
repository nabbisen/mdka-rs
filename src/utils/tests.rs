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
