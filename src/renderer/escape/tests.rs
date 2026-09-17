use super::*;

fn escapes(wait: Wait, next: Option<char>) -> bool {
    wait.escapes(next)
}

#[test]
fn flank_escapes_unless_whitespace_on_both_sides() {
    let ws = Class::Whitespace;
    assert!(!escapes(Wait::Flank { prev: ws }, Some(' ')));
    assert!(!escapes(Wait::Flank { prev: ws }, None));
    assert!(escapes(Wait::Flank { prev: ws }, Some('a')));
    assert!(escapes(Wait::Flank { prev: Class::Word }, Some(' ')));
    assert!(escapes(Wait::Flank { prev: Class::Word }, Some('a')));
    assert!(escapes(
        Wait::Flank {
            prev: Class::Punctuation
        },
        Some('.')
    ));
}

#[test]
fn underscore_is_unescaped_intraword_and_between_spaces() {
    let u = |prev, next| escapes(Wait::Underscore { prev }, next);
    assert!(!u(Class::Word, Some('b')));
    assert!(!u(Class::Word, Some('é')));
    assert!(!u(Class::Whitespace, Some(' ')));
    assert!(u(Class::Whitespace, Some('a')));
    assert!(u(Class::Word, Some(' ')));
    assert!(u(Class::Word, None));
    assert!(u(Class::Punctuation, Some('a')));
    assert!(u(Class::Word, Some('.')));
}

#[test]
fn line_start_waits_on_the_next_character() {
    assert!(escapes(Wait::ListDelimiter, Some(' ')));
    assert!(escapes(Wait::ListDelimiter, None));
    assert!(!escapes(Wait::ListDelimiter, Some('5')));
    assert!(escapes(Wait::Bullet { run: true }, Some('-')));
    assert!(!escapes(Wait::Bullet { run: false }, Some('+')));
    assert!(!escapes(Wait::Bullet { run: true }, Some('5')));
    assert!(escapes(Wait::Hash, Some('#')));
    assert!(!escapes(Wait::Hash, Some('h')));
    assert!(escapes(Wait::Bang, Some('[')));
    assert!(!escapes(Wait::Bang, Some(' ')));
    assert!(escapes(Wait::Lt, Some('d')));
    assert!(escapes(Wait::Lt, Some('/')));
    assert!(!escapes(Wait::Lt, Some(' ')));
    assert!(escapes(Wait::Amp, Some('#')));
    assert!(!escapes(Wait::Amp, Some(' ')));
    assert!(escapes(Wait::Backslash, Some('*')));
    assert!(escapes(Wait::Backslash, Some('\n')));
    assert!(!escapes(Wait::Backslash, Some('d')));
    assert!(!escapes(Wait::Backslash, None));
}

fn decide_all(s: &str, mut line: Line) -> Vec<Decision> {
    let mut prev = None;
    s.chars()
        .map(|c| {
            let d = decide(c, prev, &mut line, false);
            prev = Some(c);
            d
        })
        .collect()
}

#[test]
fn digits_then_delimiter_wait_only_at_a_line_start() {
    let d = decide_all("1986.", Line::START);
    assert_eq!(d[4], Decision::Wait(Wait::ListDelimiter));
    assert!(d[..4].iter().all(|d| *d == Decision::Plain));
    assert_eq!(decide_all("1986.", Line::INLINE)[4], Decision::Plain);
    // Ten digits cannot start an ordered list.
    assert_eq!(decide_all("1234567890.", Line::START)[10], Decision::Plain);
}

#[test]
fn continuation_lines_escape_setext_and_table_delimiters() {
    let mut line = Line::START;
    line.pipe_here = true;
    line.newline(true);
    assert_eq!(decide_all("=", line)[0], Decision::Escape);
    assert_eq!(decide_all("|", line)[0], Decision::Escape);
    assert_eq!(decide_all(":", line)[0], Decision::Escape);
    let mut first = Line::START;
    first.newline(false);
    assert_eq!(decide_all("=", first)[0], Decision::Plain);
    assert_eq!(decide_all("|", first)[0], Decision::Plain);
}

#[test]
fn code_span_delimiter_is_longer_than_any_run() {
    assert_eq!(code_span("a"), "`a`");
    assert_eq!(code_span("snake_case"), "`snake_case`");
    assert_eq!(code_span("a`b"), "``a`b``");
    assert_eq!(code_span("a``b"), "```a``b```");
    assert_eq!(code_span("`a"), "`` `a ``");
    assert_eq!(code_span("a`"), "`` a` ``");
}

#[test]
fn fence_scan_counts_runs_at_line_starts_only() {
    let mut scan = FenceScan::NEW;
    scan.feed("a```b\n");
    assert_eq!(scan.fence_len(), 3);
    scan.feed("   ````\n");
    assert_eq!(scan.fence_len(), 5);
    let mut indented = FenceScan::NEW;
    indented.feed("    `````\n");
    assert_eq!(indented.fence_len(), 3);
    let mut split = FenceScan::NEW;
    split.feed("``");
    split.feed("`");
    assert_eq!(split.fence_len(), 4);
}

#[test]
fn destinations() {
    assert_eq!(destination("/x"), "/x");
    assert_eq!(destination("/a b.html"), "</a b.html>");
    assert_eq!(destination("/a)b"), "</a)b>");
    assert_eq!(destination("/a(b)c"), "/a(b)c");
    assert_eq!(destination("<x"), "<\\<x>");
    assert_eq!(destination("/a\\*b"), "/a\\\\*b");
    assert_eq!(destination("/a\\b"), "/a\\b");
    assert_eq!(destination(""), "");
    assert_eq!(destination("/a\nb"), "/a&#10;b");
    assert_eq!(destination("/?a&copy=1"), "/?a\\&copy=1");
    assert_eq!(destination("/a & b"), "</a & b>");
}

#[test]
fn titles_pick_a_delimiter_that_needs_no_escape() {
    assert_eq!(title("plain"), "\"plain\"");
    assert_eq!(title("say \"hi\""), "'say \"hi\"'");
    assert_eq!(title("a \"b\" 'c'"), "(a \"b\" 'c')");
    assert_eq!(title("a \"b\" 'c' (d)"), "\"a \\\"b\\\" 'c' (d)\"");
    assert_eq!(title("C:\\"), "\"C:\\\\\"");
    assert_eq!(title("a\n\nb"), "\"a&#10;&#10;b\"");
    assert_eq!(title("&amp;"), "\"\\&amp;\"");
}

#[test]
fn may_escape_is_exactly_the_characters_decide_can_escape_past_a_line_start() {
    let listed = ['`', '[', ']', '*', '~', '_', '!', '<', '&', '\\', '#'];
    for code in 0u32..0x250 {
        let Some(c) = char::from_u32(code) else {
            continue;
        };
        assert_eq!(may_escape(c), listed.contains(&c), "{c:?}");
        if !listed.contains(&c) {
            let mut line = Line::INLINE;
            line.heading = true;
            assert_eq!(
                decide(c, Some(' '), &mut line, true),
                Decision::Plain,
                "{c:?}"
            );
        }
    }
}
