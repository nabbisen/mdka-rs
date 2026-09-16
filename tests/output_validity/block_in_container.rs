//! The third direction: block construct inside a container -- list items and
//! blockquotes (RFC 035; audit A-06, A-07, A-08). Expectations are RFC 035 §4's,
//! written from the HTML's meaning and §3.1's loose/tight rule, not from output.

use crate::harness::tree;

// ── blocks inside list items ───────────────────────────────────────────────

cells! {
    p_in_li: "<ul><li><p>para</p></li></ul>"
        => tree(r#"ul(li("para"))"#),
        defect(Rfc035, "empty bullet: the paragraph leaves the list (A-06)");
    p_in_each_li: "<ul><li><p>a</p></li><li><p>b</p></li></ul>"
        => tree(r#"ul(li("a"), li("b"))"#),
        defect(Rfc035, "every item an empty bullet; its paragraph lands outside the list (A-06)");
    two_p_in_li: "<ul><li><p>a</p><p>b</p></li></ul>"
        => tree(r#"ul(li(para("a"), para("b")))"#),
        defect(Rfc035, "empty bullet: both paragraphs leave the list (A-06)");
    pre_in_li: "<ul><li><pre><code>x</code></pre></li></ul>"
        => tree(r#"ul(li(codeblock("x")))"#),
        defect(Rfc035, "empty bullet: the code block leaves the list (A-06)");
    text_then_pre_in_li: "<ul><li>see<pre><code>x</code></pre></li></ul>"
        => tree(r#"ul(li(para("see"), codeblock("x")))"#),
        defect(Rfc035, "the code block after the item text leaves the list (A-06)");
    blockquote_in_li: "<ul><li><blockquote><p>q</p></blockquote></li></ul>"
        => tree(r#"ul(li(quote(para("q"))))"#),
        defect(Rfc035, "empty bullet: the quote leaves the list (A-06)");
    link_around_p_in_li: r#"<ul><li><a href="/x"><p>x</p></a></li></ul>"#
        => tree(r#"ul(li(link[/x]("x")))"#),
        defect(Rfc035, "empty bullet: the linked paragraph leaves the list (A-06)");
}

// ── nested lists ───────────────────────────────────────────────────────────

cells! {
    ol_in_ol: "<ol><li>one<ol><li>inner</li></ol></li></ol>"
        => tree(r#"ol[1](li("one", ol[1](li("inner"))))"#),
        defect(Rfc035, "nested list indented 2 spaces, below `1. `'s content column: a lazy continuation, not a sublist (A-07)");
    ol_in_ol_two_digit: r#"<ol start="10"><li>ten<ol><li>inner</li></ol></li></ol>"#
        => tree(r#"ol[10](li("ten", ol[1](li("inner"))))"#),
        defect(Rfc035, "nested list indented 2 spaces, below `10. `'s content column: not a sublist (A-07)");
    ul_in_ol: "<ol><li>one<ul><li>inner</li></ul></li></ol>"
        => tree(r#"ol[1](li("one", ul(li("inner"))))"#),
        defect(Rfc035, "nested unordered list indented 2 spaces under `1. `: not a sublist (A-07)");
    tight_list_control: "<ul><li>a</li><li>b</li></ul>"
        => tree(r#"ul(li("a"), li("b"))"#);
}

// ── blocks inside blockquotes ──────────────────────────────────────────────

cells! {
    two_p_in_blockquote: "<blockquote><p>one</p><p>two</p></blockquote>"
        => tree(r#"quote(para("one"), para("two"))"#),
        defect(Rfc035, "the blank separator line has no `>`: two quotes instead of one (A-08)");
    multiline_pre_in_blockquote: "<blockquote><pre><code>l1\nl2</code></pre></blockquote>"
        => tree(r#"quote(codeblock("l1\nl2"))"#),
        defect(Rfc035, "code content lines have no `>`: the code escapes the quote after the fence (A-08)");
    ul_in_blockquote: "<blockquote><ul><li>a</li><li>b</li></ul></blockquote>"
        => tree(r#"quote(ul(li("a"), li("b")))"#);
    two_p_in_blockquote_in_li: "<ul><li><blockquote><p>a</p><p>b</p></blockquote></li></ul>"
        => tree(r#"ul(li(quote(para("a"), para("b"))))"#),
        defect(Rfc035, "empty bullet, and the quote splits in two at the unprefixed blank line (A-06, A-08)");
    two_p_in_nested_blockquote: "<blockquote><blockquote><p>a</p><p>b</p></blockquote></blockquote>"
        => tree(r#"quote(quote(para("a"), para("b")))"#),
        defect(Rfc035, "the blank separator line has no `> >`: the nested quotes split in two (A-08)");
    multiline_pre_in_li_in_blockquote: "<blockquote><ul><li><pre><code>x\ny</code></pre></li></ul></blockquote>"
        => tree(r#"quote(ul(li(codeblock("x\ny"))))"#),
        defect(Rfc035, "empty bullet, and code content lines lose both the list indent and `>` (A-06, A-08)");
    indented_code_in_pre_in_blockquote: "<blockquote><pre>    <code>x</code></pre></blockquote><p>after</p>"
        => tree(r#"quote(codeblock("    x")), para("after")"#),
        defect(Rfc035, "the indented code line has no `>`: it becomes indented code outside the quote, and the orphaned fence swallows `after` (A-08)");
}
