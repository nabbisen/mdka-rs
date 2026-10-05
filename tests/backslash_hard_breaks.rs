//! RFC 052 — `backslash_hard_breaks`: a hard break as `\` + newline.
//!
//! A break is only a break where it survives as one. The two forms parse to
//! the same events in a paragraph, blockquote and list item; in a heading
//! there is no hard break at all, so the backslash would stay visible in the
//! heading text, and the heading path keeps its spaces. Parsed events are
//! compared, not bytes: the two forms are different bytes by design.

use mdka::options::ConversionOptions;
use pulldown_cmark::{Event, Options, Parser};

fn convert(html: &str, backslash: bool) -> String {
    mdka::html_to_markdown_with(
        html,
        &ConversionOptions::default().backslash_hard_breaks(backslash),
    )
}

/// The parsed event stream, with adjacent text merged so that how the parser
/// splits a run of text does not count as a difference.
fn events(md: &str) -> Vec<String> {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut out = Vec::new();
    let mut text: Option<String> = None;
    for e in Parser::new_ext(md, opts) {
        if let Event::Text(t) = &e {
            text.get_or_insert_with(String::new).push_str(t);
            continue;
        }
        if let Some(t) = text.take() {
            out.push(format!("Text({t:?})"));
        }
        out.push(format!("{e:?}"));
    }
    if let Some(t) = text.take() {
        out.push(format!("Text({t:?})"));
    }
    out
}

/// Asserts the backslash form parses to the same events as the spaces form.
fn assert_same_parse(html: &str) {
    let spaces = convert(html, false);
    let backslash = convert(html, true);
    assert_eq!(
        events(&backslash),
        events(&spaces),
        "{html}\n  spaces    {spaces:?}\n  backslash {backslash:?}"
    );
}

#[test]
fn option_off_writes_the_two_spaces_it_always_has() {
    assert_eq!(convert("<p>one<br>two</p>", false), "one  \ntwo\n");
    assert_eq!(
        convert("<blockquote><p>one<br>two</p></blockquote>", false),
        "> one  \n> two\n"
    );
}

#[test]
fn option_on_writes_a_backslash_in_a_paragraph() {
    assert_eq!(convert("<p>one<br>two</p>", true), "one\\\ntwo\n");
    assert_eq!(
        convert("<p>one<br>two<br>three</p>", true),
        "one\\\ntwo\\\nthree\n"
    );
}

#[test]
fn paragraph_blockquote_and_list_item_parse_the_same_either_way() {
    assert_same_parse("<p>one<br>two</p>");
    assert_same_parse("<blockquote><p>one<br>two</p></blockquote>");
    assert_same_parse("<ul><li>one<br>two</li></ul>");
    assert_same_parse("<ol><li>one<br>two<br>three</li></ol>");
}

/// Criterion 3, the one way this change goes wrong: a backslash in a heading
/// stays visible in the heading text. The heading path keeps two spaces,
/// option on and off, so a heading's output is identical either way.
#[test]
fn the_heading_case_is_unchanged_with_the_option_on_and_off() {
    for html in [
        "<h2>one<br>two</h2>",
        "<h1>one<br>two</h1>",
        "<h2><b>one</b><br>two</h2>",
        "<blockquote><h2>one<br>two</h2></blockquote>",
    ] {
        assert_eq!(convert(html, true), convert(html, false), "{html}");
    }
    assert_eq!(convert("<h2>one<br>two</h2>", true), "## one  \ntwo\n");
}

/// A break at the end of its block is not a break; the spaces form drops it
/// as trailing whitespace, and the backslash form must not leave a `\` behind.
#[test]
fn a_break_that_ends_its_block_leaves_no_backslash() {
    assert_eq!(convert("<p>one<br></p><p>two</p>", true), "one\n\ntwo\n");
    assert_eq!(
        convert("<ul><li>one<br></li><li>two</li></ul>", true),
        "- one\n- two\n"
    );
    assert_same_parse("<p>one<br></p><p>two</p>");
}

/// Two breaks in a row read as a blank line, which the spaces form already does.
#[test]
fn consecutive_breaks_parse_as_a_blank_line() {
    assert_same_parse("<p>one<br><br>two</p>");
}

/// A run of breaks has no content between its breaks, so none of them can be a
/// backslash: the output is the spaces form exactly, in a paragraph and in a
/// tight list item, where a stray `\` line would otherwise be a literal.
#[test]
fn a_run_of_breaks_is_written_exactly_as_the_spaces_form() {
    for html in [
        "<p>one<br><br><br>two</p>",
        "<ul><li>one<br><br>two</li></ul>",
    ] {
        assert_eq!(convert(html, true), convert(html, false), "{html}");
    }
}

/// A break that starts its block, or follows a block boundary, has no content
/// before it on its line; it keeps the spaces form and parses as before.
#[test]
fn a_break_at_a_block_start_or_after_a_boundary_parses_the_same() {
    for html in [
        "<p><br>two</p>",
        "<p>one<br></p><p><br>two</p>",
        "<blockquote><p>one<br></p><p>two</p></blockquote>",
    ] {
        assert_same_parse(html);
    }
}

/// Criterion 4: table cells, `<pre>` and code spans are untouched.
#[test]
fn table_cell_pre_and_code_span_are_unchanged_on_and_off() {
    for html in [
        "<table><tr><td>a<br>b</td></tr></table>",
        "<pre>a<br>b</pre>",
        "<p><code>a<br>b</code></p>",
    ] {
        assert_eq!(convert(html, true), convert(html, false), "{html}");
    }
}

/// A backslash in the text before the break is escaped, and the break is
/// still a break.
#[test]
fn text_ending_in_a_backslash_before_a_break() {
    assert_same_parse("<p>a\\<br>b</p>");
}

/// Criterion 5, the Rust surface: the option can change output.
#[test]
fn the_option_can_affect_output() {
    assert_ne!(
        convert("<p>one<br>two</p>", false),
        convert("<p>one<br>two</p>", true)
    );
}

#[test]
fn a_break_beside_emphasis_and_code_parses_the_same_either_way() {
    for html in [
        "<p><b>x</b><br>y</p>",
        "<p><code>a</code><br>b</p>",
        "<p>a*<br>b</p>",
        "<p><em>a<br>b</em></p>",
        "<p><strong>x<br>y</strong></p>",
    ] {
        assert_same_parse(html);
    }
}
