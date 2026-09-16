//! The third direction: block construct inside a container -- list items and
//! blockquotes (RFC 035; audit A-06, A-07, A-08). Expectations are RFC 035 §4's,
//! written from the HTML's meaning and §3.1's loose/tight rule, not from output.

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
