//! §4.2 — block construct × inline element. Required by RFC 025's
//! 2026-09-16 amendment.
//!
//! Rows: `<p>` (two), `<ul>`/`<li>`, `<blockquote>`, `<pre>`, heading.
//! Columns: `<strong>`, `<em>`, `<a>`, `<code>`.
//!
//! `<strong>`/`<em>` cells assert RFC 028's accepted behaviour A: when the
//! children are blocks, the emphasis delimiters are dropped and the blocks
//! kept. `<a>` and `<code>` around blocks joined RFC 028 by its Amendment 2
//! (owner, 2026-09-16), which decided their structure: a link around blocks
//! links each block's content and leaves a code block unlinked; code around
//! blocks writes no backticks and keeps the blocks. Those cells assert it
//! (slice 028b; structures written by the architect).

use crate::harness::tree;

// ── in <strong> ────────────────────────────────────────────────────────────

cells! {
    p_in_strong: "<strong><p>x</p><p>y</p></strong>"
        => tree(r#"para("x"), para("y")"#);
    ul_in_strong: "<strong><ul><li>x</li><li>y</li></ul></strong>"
        => tree(r#"ul(li("x"), li("y"))"#);
    blockquote_in_strong: "<strong><blockquote><p>x</p></blockquote></strong>"
        => tree(r#"quote(para("x"))"#);
    pre_in_strong: "<strong><pre>x</pre></strong>"
        => tree(r#"codeblock("x")"#);
    heading_in_strong: "<strong><h2>x</h2></strong>"
        => tree(r#"h2("x")"#);
}

// ── in <em> ────────────────────────────────────────────────────────────────

cells! {
    p_in_em: "<em><p>x</p><p>y</p></em>"
        => tree(r#"para("x"), para("y")"#);
    ul_in_em: "<em><ul><li>x</li><li>y</li></ul></em>"
        => tree(r#"ul(li("x"), li("y"))"#);
    blockquote_in_em: "<em><blockquote><p>x</p></blockquote></em>"
        => tree(r#"quote(para("x"))"#);
    pre_in_em: "<em><pre>x</pre></em>"
        => tree(r#"codeblock("x")"#);
    heading_in_em: "<em><h2>x</h2></em>"
        => tree(r#"h2("x")"#);
}

// ── in <a> ─────────────────────────────────────────────────────────────────

cells! {
    p_in_a: r#"<a href="/out"><p>x</p><p>y</p></a>"#
        => tree(r#"para(link[/out]("x")), para(link[/out]("y"))"#);
    ul_in_a: r#"<a href="/out"><ul><li>x</li><li>y</li></ul></a>"#
        => tree(r#"ul(li(link[/out]("x")), li(link[/out]("y")))"#);
    blockquote_in_a: r#"<a href="/out"><blockquote><p>x</p></blockquote></a>"#
        => tree(r#"quote(para(link[/out]("x")))"#);
    pre_in_a: r#"<a href="/out"><pre>x</pre></a>"#
        => tree(r#"codeblock("x")"#);
    heading_in_a: r#"<a href="/out"><h2>x</h2></a>"#
        => tree(r#"h2(link[/out]("x"))"#);
}

// ── in <code> ──────────────────────────────────────────────────────────────

cells! {
    p_in_code: "<code><p>x</p><p>y</p></code>"
        => tree(r#"para("x"), para("y")"#);
    ul_in_code: "<code><ul><li>x</li><li>y</li></ul></code>"
        => tree(r#"ul(li("x"), li("y"))"#);
    blockquote_in_code: "<code><blockquote><p>x</p></blockquote></code>"
        => tree(r#"quote(para("x"))"#);
    pre_in_code: "<code><pre>x</pre></code>"
        => tree(r#"codeblock("x")"#);
    heading_in_code: "<code><h2>x</h2></code>"
        => tree(r#"h2("x")"#);
}
