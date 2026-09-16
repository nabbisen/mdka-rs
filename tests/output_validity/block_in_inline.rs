//! §4.2 — block construct × inline element. Required by RFC 025's
//! 2026-09-16 amendment.
//!
//! Rows: `<p>` (two), `<ul>`/`<li>`, `<blockquote>`, `<pre>`, heading.
//! Columns: `<strong>`, `<em>`, `<a>`, `<code>`.
//!
//! `<strong>`/`<em>` cells assert RFC 028's accepted behaviour A: when the
//! children are blocks, the emphasis delimiters are dropped and the blocks
//! kept. `<a>` and `<code>` around blocks joined RFC 028 by its Amendment 2
//! (owner, 2026-09-16); their cells still assert only the intent-free
//! properties (Q4, Q5).

use crate::harness::{tree, undecided};

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
        => undecided("Q4: block content inside <a> (valid HTML5): where does the link go?");
    ul_in_a: r#"<a href="/out"><ul><li>x</li><li>y</li></ul></a>"#
        => undecided("Q4");
    blockquote_in_a: r#"<a href="/out"><blockquote><p>x</p></blockquote></a>"#
        => undecided("Q4");
    pre_in_a: r#"<a href="/out"><pre>x</pre></a>"#
        => undecided("Q4");
    heading_in_a: r#"<a href="/out"><h2>x</h2></a>"#
        => undecided("Q4");
}

// ── in <code> ──────────────────────────────────────────────────────────────

cells! {
    p_in_code: "<code><p>x</p><p>y</p></code>"
        => undecided("Q5: block content inside inline <code>");
    ul_in_code: "<code><ul><li>x</li><li>y</li></ul></code>"
        => undecided("Q5");
    blockquote_in_code: "<code><blockquote><p>x</p></blockquote></code>"
        => undecided("Q5");
    pre_in_code: "<code><pre>x</pre></code>"
        => undecided("Q5");
    heading_in_code: "<code><h2>x</h2></code>"
        => undecided("Q5");
}
