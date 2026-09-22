//! The third direction: block construct inside a container -- list items and
//! blockquotes (RFC 035; audit A-06, A-07, A-08). Expectations are RFC 035 §4's,
//! written from the HTML's meaning and §3.1's loose/tight rule, not from output.
//! The last two `cells!` groups are RFC 036 §5.1's slice: leading whitespace
//! inside a list item, and the controls it must leave untouched.

use crate::harness::tree;

// ── blocks inside list items ───────────────────────────────────────────────

cells! {
    p_in_li: "<ul><li><p>para</p></li></ul>"
        => tree(r#"ul(li("para"))"#);
    p_in_each_li: "<ul><li><p>a</p></li><li><p>b</p></li></ul>"
        => tree(r#"ul(li("a"), li("b"))"#);
    two_p_in_li: "<ul><li><p>a</p><p>b</p></li></ul>"
        => tree(r#"ul(li(para("a"), para("b")))"#);
    pre_in_li: "<ul><li><pre><code>x</code></pre></li></ul>"
        => tree(r#"ul(li(codeblock("x")))"#);
    text_then_pre_in_li: "<ul><li>see<pre><code>x</code></pre></li></ul>"
        => tree(r#"ul(li(para("see"), codeblock("x")))"#);
    blockquote_in_li: "<ul><li><blockquote><p>q</p></blockquote></li></ul>"
        => tree(r#"ul(li(quote(para("q"))))"#);
    link_around_p_in_li: r#"<ul><li><a href="/x"><p>x</p></a></li></ul>"#
        => tree(r#"ul(li(link[/x]("x")))"#);
}

// ── nested lists ───────────────────────────────────────────────────────────

cells! {
    ol_in_ol: "<ol><li>one<ol><li>inner</li></ol></li></ol>"
        => tree(r#"ol[1](li("one", ol[1](li("inner"))))"#);
    ol_in_ol_two_digit: r#"<ol start="10"><li>ten<ol><li>inner</li></ol></li></ol>"#
        => tree(r#"ol[10](li("ten", ol[1](li("inner"))))"#);
    ul_in_ol: "<ol><li>one<ul><li>inner</li></ul></li></ol>"
        => tree(r#"ol[1](li("one", ul(li("inner"))))"#);
    tight_list_control: "<ul><li>a</li><li>b</li></ul>"
        => tree(r#"ul(li("a"), li("b"))"#);
}

// ── blocks inside blockquotes ──────────────────────────────────────────────

cells! {
    two_p_in_blockquote: "<blockquote><p>one</p><p>two</p></blockquote>"
        => tree(r#"quote(para("one"), para("two"))"#);
    multiline_pre_in_blockquote: "<blockquote><pre><code>l1\nl2</code></pre></blockquote>"
        => tree(r#"quote(codeblock("l1\nl2"))"#);
    ul_in_blockquote: "<blockquote><ul><li>a</li><li>b</li></ul></blockquote>"
        => tree(r#"quote(ul(li("a"), li("b")))"#);
    two_p_in_blockquote_in_li: "<ul><li><blockquote><p>a</p><p>b</p></blockquote></li></ul>"
        => tree(r#"ul(li(quote(para("a"), para("b"))))"#);
    two_p_in_nested_blockquote: "<blockquote><blockquote><p>a</p><p>b</p></blockquote></blockquote>"
        => tree(r#"quote(quote(para("a"), para("b")))"#);
    multiline_pre_in_li_in_blockquote: "<blockquote><ul><li><pre><code>x\ny</code></pre></li></ul></blockquote>"
        => tree(r#"quote(ul(li(codeblock("x\ny"))))"#);
    indented_code_in_pre_in_blockquote: "<blockquote><pre>    <code>x</code></pre></blockquote><p>after</p>"
        => tree(r#"quote(codeblock("    x")), para("after")"#);
}

// ── RFC 036 §5.1: leading whitespace in a list item ────────────────────────
//
// A list item's marker leaves the sink's line-start bookkeeping consumed, so
// leading whitespace in the item's own text was not collapsed the way a
// paragraph's is: the wider unstripped column made CommonMark end the item
// one character early, splitting the list, flattening a nested one, or
// ejecting a code block from its item. Expectations are the HTML's meaning;
// each was a distinct validity failure at `bec40bf` (`2.3.0`), verified by
// parsing, not by a string check on the marker alone.

cells! {
    li_ws_two_p: "<ul><li> <p>a</p><p>b</p></li><li>c</li></ul>"
        => tree(r#"ul(li(para("a"), para("b")), li(para("c")))"#);
    li_ws_nested_ul: "<ul><li> a<ul><li>b</li></ul></li></ul>"
        => tree(r#"ul(li("a", ul(li("b"))))"#);
    li_ws_pre: "<ul><li> <p>a</p><pre><code>x</code></pre></li></ul>"
        => tree(r#"ul(li(para("a"), codeblock("x")))"#);
    ol_ws_two_p: "<ol><li> <p>a</p><p>b</p></li><li>c</li></ol>"
        => tree(r#"ol[1](li(para("a"), para("b")), li(para("c")))"#);
    // The realistic trigger: pretty-printed HTML from an editor or CMS.
    li_newline_pretty: "<ul><li>\n  <p>a</p><p>b</p>\n</li><li>c</li></ul>"
        => tree(r#"ul(li(para("a"), para("b")), li(para("c")))"#);
    // Structurally this already survived at `bec40bf` -- a quote inside a
    // whitespace-led item still parsed as a quote -- but it carried the same
    // stray leading space, which the fix removes along with the rest.
    li_ws_quote: "<ul><li> <blockquote><p>q</p></blockquote></li><li>c</li></ul>"
        => tree(r#"ul(li(quote(para("q"))), li("c"))"#);
    // Found while implementing the fix: an image (or any content that does
    // not itself pass through the renderer's text path) sitting between the
    // item's own leading whitespace and the text that follows must not
    // leave the space AFTER it mistaken for the item's leading whitespace
    // too -- that space separates the image from "text" and must survive.
    li_image_then_text: r#"<ul><li><img src="i.png" alt="i"> text</li></ul>"#
        => tree(r#"ul(li(image[i.png]("i"), " text"))"#);
}

// ── controls: must stay byte-identical to `2.3.0` (RFC 036 §4 criterion 2) ─

cells! {
    li_nows_two_p: "<ul><li><p>a</p><p>b</p></li><li>c</li></ul>"
        => tree(r#"ul(li(para("a"), para("b")), li(para("c")))"#);
    li_nows_nested: "<ul><li>a<ul><li>b</li></ul></li></ul>"
        => tree(r#"ul(li("a", ul(li("b"))))"#);
    // Trailing whitespace was already stripped before RFC 036; the fix only
    // touches the leading side.
    li_ws_trailing: "<ul><li>a </li><li>b </li></ul>"
        => tree(r#"ul(li("a"), li("b"))"#);
}
