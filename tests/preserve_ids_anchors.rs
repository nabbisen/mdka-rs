//! Integration tests: RFC 005 Slice B1 — `preserve_ids` anchor emission
//! Covers: all required output and escaping cases from the RFC 005 Slice
//! B1 placement-correction handoff §4, verifying `preserve_ids` emits an
//! escaped `<a id="...">` anchor as the leading *content* of a non-empty-
//! `id` element (after any prefix/marker), and emits nothing for an
//! absent or empty `id`, or when `preserve_ids` is off.
//!
//! The heading/paragraph/escaping cases below previously asserted the
//! anchor emitted *before* the element as its own block (e.g.
//! `"<a id=\"install\"></a>\n\n## Install\n"`), under the original Slices
//! B/C handoff. That placement was wrong in list and blockquote contexts
//! (see the placement-correction handoff), so it was corrected to place
//! the anchor inside the element instead. Updated here, not deleted.

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

fn with_preserve_ids(v: bool) -> ConversionOptions {
    ConversionOptions::for_mode(ConversionMode::Balanced).preserve_ids(v)
}

#[test]
fn heading_with_id_gets_anchor() {
    // Previously asserted "<a id=\"install\"></a>\n\n## Install\n" (anchor
    // before the element, as its own block). Corrected: anchor is now
    // leading content, after the "## " marker.
    assert_eq!(
        conv_with(r#"<h2 id="install">Install</h2>"#, &with_preserve_ids(true)),
        "## <a id=\"install\"></a>Install\n"
    );
}

#[test]
fn paragraph_with_id_gets_anchor() {
    // Previously asserted "<a id=\"intro\"></a>\n\nText\n" (anchor before
    // the element). Corrected: anchor is now leading content of the
    // paragraph, no separating blank line.
    assert_eq!(
        conv_with(r#"<p id="intro">Text</p>"#, &with_preserve_ids(true)),
        "<a id=\"intro\"></a>Text\n"
    );
}

#[test]
fn inline_element_with_id_gets_inline_anchor() {
    let md = conv_with(
        r#"<p>a <span id="s">b</span> c</p>"#,
        &with_preserve_ids(true),
    );
    assert_eq!(md, "a <a id=\"s\"></a>b c\n");
}

#[test]
fn heading_without_id_is_unchanged() {
    assert_eq!(
        conv_with("<h2>No id</h2>", &with_preserve_ids(true)),
        "## No id\n"
    );
}

#[test]
fn empty_id_emits_no_anchor() {
    assert_eq!(
        conv_with(r#"<h2 id="">Empty</h2>"#, &with_preserve_ids(true)),
        "## Empty\n"
    );
}

#[test]
fn preserve_ids_false_emits_no_anchor() {
    assert_eq!(
        conv_with(
            r#"<h2 id="install">Install</h2>"#,
            &with_preserve_ids(false)
        ),
        "## Install\n"
    );
}

#[test]
fn id_escaping_quote_and_attribute_injection() {
    // "&" must be escaped before """, or the escaped output double-escapes.
    // Previously asserted a leading "\n\n" before "Hi" (anchor-before-element
    // placement); corrected to leading content, no separating blank line.
    let html = "<p id=\"x&quot; onload=&quot;alert(1)\">Hi</p>";
    let md = conv_with(html, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"x&quot; onload=&quot;alert(1)\"></a>Hi\n");
}

#[test]
fn id_escaping_ampersand() {
    // Previously asserted "<a id=\"a&amp;b\"></a>\n\nHi\n"; corrected as above.
    let md = conv_with(r#"<p id="a&b">Hi</p>"#, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"a&amp;b\"></a>Hi\n");
}

#[test]
fn id_escaping_plain_id_unaffected() {
    // Previously asserted "<a id=\"plain-id\"></a>\n\nHi\n"; corrected as above.
    let md = conv_with(r#"<p id="plain-id">Hi</p>"#, &with_preserve_ids(true));
    assert_eq!(md, "<a id=\"plain-id\"></a>Hi\n");
}

// ─── Placement correction: §7 required new tests ───────────────────────────

#[test]
fn single_item_list_anchor_sits_after_marker() {
    assert_eq!(
        conv_with(r#"<ul><li id="b">two</li></ul>"#, &with_preserve_ids(true)),
        "- <a id=\"b\"></a>two\n"
    );
}

#[test]
fn mid_list_anchor_sits_in_its_own_item() {
    // The defect this corrects: emitting the anchor before the element made
    // it a CommonMark lazy continuation of the *preceding* item's paragraph,
    // so an id on the second <li> rendered inside the first item.
    assert_eq!(
        conv_with(
            r#"<ul><li>one</li><li id="b">two</li><li>three</li></ul>"#,
            &with_preserve_ids(true)
        ),
        "- one\n- <a id=\"b\"></a>two\n- three\n"
    );
}

#[test]
fn ordered_list_item_anchor_sits_after_marker() {
    assert_eq!(
        conv_with(r#"<ol><li id="x">a</li></ol>"#, &with_preserve_ids(true)),
        "1. <a id=\"x\"></a>a\n"
    );
}

#[test]
fn blockquote_paragraph_anchor_sits_after_prefix() {
    // The other defect this corrects: push_raw bypassed emit_pending_prefix,
    // so the anchor escaped the blockquote entirely (emitted outside it,
    // unprefixed, as a separate paragraph above).
    assert_eq!(
        conv_with(
            r#"<blockquote><p id="p">Q</p></blockquote>"#,
            &with_preserve_ids(true)
        ),
        "> <a id=\"p\"></a>Q\n"
    );
}

#[test]
fn nested_id_bearing_elements_each_get_their_own_anchor() {
    assert_eq!(
        conv_with(
            r#"<div id="outer"><p id="inner">Text</p></div>"#,
            &with_preserve_ids(true)
        ),
        "<a id=\"outer\"></a>\n\n<a id=\"inner\"></a>Text\n"
    );
}

#[test]
fn id_inside_link_capture_is_guarded() {
    // capture_depth > 0 guard: an id on an element nested inside a link's
    // captured text must not emit into the main output stream, which would
    // corrupt the link's buffered content.
    assert_eq!(
        conv_with(
            r#"<a href="/"><span id="s">Home</span></a>"#,
            &with_preserve_ids(true)
        ),
        "[Home](/)\n"
    );
}

#[test]
fn id_inside_pre_is_guarded() {
    // in_pre guard: an id on an element inside a code block must not emit
    // into the block's literal content.
    assert_eq!(
        conv_with(
            r#"<pre><code id="c">x</code></pre>"#,
            &with_preserve_ids(true)
        ),
        "```\nx\n```\n"
    );
}

// ─── Placement correction round 2: elements that open their own capture ────
//
// `a` and `pre` set capture_depth/in_pre themselves as part of entering the
// element. When emit_id_anchor ran only after the match, this meant an <a>
// or <pre> with its OWN id tripped its own guard and silently lost its
// anchor -- a real regression, caught by testing beyond the required table.
// The fix: for these two tags only, the anchor is emitted before the match
// (the exception to the "leading content" placement rule), so it uses the
// pre-match (inherited-only) guard state. A descendant's id, nested inside
// an already-open capture/pre, is still correctly suppressed -- see
// id_inside_link_capture_is_guarded / id_inside_pre_is_guarded above.

#[test]
fn link_with_own_id_gets_anchor_before_the_link() {
    assert_eq!(
        conv_with(
            r#"<a id="link" href="/">text</a>"#,
            &with_preserve_ids(true)
        ),
        "<a id=\"link\"></a>[text](/)\n"
    );
}

#[test]
fn pre_with_own_id_gets_anchor_before_the_block() {
    assert_eq!(
        conv_with(
            r#"<pre id="x"><code>y</code></pre>"#,
            &with_preserve_ids(true)
        ),
        "<a id=\"x\"></a>\n\n```\ny\n```\n"
    );
}

#[test]
fn link_with_own_id_inline_in_a_paragraph() {
    assert_eq!(
        conv_with(
            r#"<p>see <a id="l" href="/">here</a> now</p>"#,
            &with_preserve_ids(true)
        ),
        "see <a id=\"l\"></a>[here](/) now\n"
    );
}
