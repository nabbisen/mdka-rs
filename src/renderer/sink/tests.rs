use super::*;

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "container prefix stack unbalanced at document end")]
fn unbalanced_container_stack_fires_at_finish() {
    // A deliberately unbalanced push: a quote entered and never left.
    let mut sink = Sink::new(16);
    sink.enter_blockquote();
    sink.text("x");
    let _ = sink.finish();
}

#[test]
fn balanced_container_stack_finishes() {
    let mut sink = Sink::new(16);
    sink.enter_blockquote();
    sink.text("x");
    sink.leave_blockquote();
    assert_eq!(sink.finish(), "> x\n");
}

// ─── RFC 010: escaping by context ───────────────────────────────────────────

fn prose(text: &str) -> String {
    let mut sink = Sink::new(16);
    sink.text(text);
    sink.finish()
}

#[test]
fn whitespace_collapses_and_a_line_start_drops_it() {
    assert_eq!(prose("  hello   world\t\nfoo"), "hello world foo\n");
}

#[test]
fn delimiters_are_escaped_where_they_could_open_or_close() {
    assert_eq!(prose("*bold*"), "\\*bold\\*\n");
    assert_eq!(prose("a * b"), "a * b\n");
    assert_eq!(prose("snake_case_here"), "snake_case_here\n");
    assert_eq!(prose("_x_"), "\\_x\\_\n");
    assert_eq!(prose("`code`"), "\\`code\\`\n");
}

#[test]
fn escapes_that_wait_are_settled_by_the_next_write() {
    assert_eq!(prose("Wow! Really!"), "Wow! Really!\n");
    let mut sink = Sink::new(16);
    sink.text("Wow!");
    sink.markup("[x](/y)");
    assert_eq!(sink.finish(), "Wow\\![x](/y)\n");
    // Split across text nodes, the digits and the delimiter still meet.
    let mut sink = Sink::new(16);
    sink.text("1986");
    sink.text(". A");
    assert_eq!(sink.finish(), "1986\\. A\n");
}

#[test]
fn a_block_starts_after_a_container_prefix() {
    let mut sink = Sink::new(16);
    sink.enter_blockquote();
    sink.text("1986. A");
    sink.leave_blockquote();
    assert_eq!(sink.finish(), "> 1986\\. A\n");
    let mut sink = Sink::new(16);
    sink.item_marker("- ", true);
    sink.text("# x");
    sink.leave_item();
    assert_eq!(sink.finish(), "- \\# x\n");
}

#[test]
fn a_code_span_capture_is_not_escaped() {
    let mut sink = Sink::new(16);
    sink.begin_capture(Capture::CodeSpan);
    sink.text("snake_case *a*");
    let (_, content, _) = sink.end_capture().expect("capture");
    assert_eq!(content, "snake_case *a*");
}

#[test]
fn a_link_capture_escapes_a_closing_bracket_and_settles_against_it() {
    let mut sink = Sink::new(16);
    sink.begin_capture(Capture::Link {
        href: "/x".into(),
        title: None,
    });
    sink.text("a] \\");
    let (_, content, _) = sink.end_capture().expect("capture");
    assert_eq!(content, "a\\] \\\\");
}

#[test]
fn a_fence_is_longer_than_any_backtick_run_at_a_line_start() {
    let mut sink = Sink::new(16);
    sink.open_fence("");
    sink.code_block_content("a\n````\nb\n");
    sink.close_fence();
    assert_eq!(sink.finish(), "`````\na\n````\nb\n`````\n");
}

#[test]
fn touching_emphasis_runs_are_separated() {
    let mut sink = Sink::new(16);
    let (_, mark) = sink.emphasis_open("*", false);
    sink.text("a");
    sink.emphasis_close_or_remove(mark);
    let (_, mark) = sink.emphasis_open("**", false);
    sink.text("b");
    sink.emphasis_close_or_remove(mark);
    assert_eq!(sink.finish(), "_a_**b**\n");
    // A chain: the second span cannot take `_` next to the first, so the
    // third does.
    let mut sink = Sink::new(16);
    let mut written = Vec::new();
    for text in ["a", "b", "c"] {
        let (delimiter, mark) = sink.emphasis_open("**", false);
        sink.text(text);
        sink.emphasis_close_or_remove(mark);
        written.push(delimiter);
    }
    assert_eq!(written, ["**", "**", "__"]);
    assert_eq!(sink.finish(), "__a__**b**__c__\n");
}

#[test]
fn image_alt_destination_and_title_are_escaped() {
    assert_eq!(
        Sink::image_syntax("a]b", "a b.png", Some("say \"hi\"")),
        "![a\\]b](<a b.png> 'say \"hi\"')"
    );
}
