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

#[test]
fn test_fmt_link() {
    assert_eq!(
        fmt_link("Click", "https://example.com", None),
        "[Click](https://example.com)"
    );
    assert_eq!(
        fmt_link("Click", "https://example.com", Some("Tip")),
        "[Click](https://example.com \"Tip\")"
    );
}

#[test]
fn test_fmt_image() {
    assert_eq!(
        fmt_image("alt text", "img.png", None),
        "![alt text](img.png)"
    );
    assert_eq!(
        fmt_image("alt", "img.png", Some("caption")),
        "![alt](img.png \"caption\")"
    );
}
