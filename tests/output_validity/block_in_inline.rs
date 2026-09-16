//! §4.2 — block construct × inline element. Required by RFC 025's
//! 2026-09-16 amendment.
//!
//! Rows: `<p>` (two), `<ul>`/`<li>`, `<blockquote>`, `<pre>`, heading.
//! Columns: `<strong>`, `<em>`, `<a>`, `<code>`.
//!
//! `<strong>`/`<em>` cells assert RFC 028's accepted behaviour A: when the
//! children are blocks, the emphasis delimiters are dropped and the blocks
//! kept. `<a>` and `<code>` around blocks are outside RFC 028's stated scope
//! (`strong`/`b`, `em`/`i`); their intended structure is a question (Q4, Q5)
//! and their owner is recorded as UNOWNED (Q6).

use crate::harness::{tree, undecided};

// ── in <strong> ────────────────────────────────────────────────────────────

cells! {
    p_in_strong: "<strong><p>x</p><p>y</p></strong>"
        => tree(r#"para("x"), para("y")"#),
        defect(Rfc028, "`**` emitted around the paragraphs as stray `**` lines");
    ul_in_strong: "<strong><ul><li>x</li><li>y</li></ul></strong>"
        => tree(r#"ul(li("x"), li("y"))"#),
        defect(Rfc028, "`**` emitted around the list as stray `**` lines");
    blockquote_in_strong: "<strong><blockquote><p>x</p></blockquote></strong>"
        => tree(r#"quote(para("x"))"#),
        defect(Rfc028, "`**` emitted around the quote as stray `**` lines");
    pre_in_strong: "<strong><pre>x</pre></strong>"
        => tree(r#"codeblock("x")"#),
        defect(Rfc028, "`**` around the block, plus bare <pre> (RFC 024): the trailing `**` lands in an unterminated code block");
    heading_in_strong: "<strong><h2>x</h2></strong>"
        => tree(r#"h2("x")"#),
        defect(Rfc028, "`**` emitted around the heading as stray `**` lines");
}

// ── in <em> ────────────────────────────────────────────────────────────────

cells! {
    p_in_em: "<em><p>x</p><p>y</p></em>"
        => tree(r#"para("x"), para("y")"#),
        defect(Rfc028, "a lone `*` line around the paragraphs parses as an empty list item, before and after");
    ul_in_em: "<em><ul><li>x</li><li>y</li></ul></em>"
        => tree(r#"ul(li("x"), li("y"))"#),
        defect(Rfc028, "lone `*` lines parse as two extra empty lists around the real one");
    blockquote_in_em: "<em><blockquote><p>x</p></blockquote></em>"
        => tree(r#"quote(para("x"))"#),
        defect(Rfc028, "lone `*` lines parse as empty lists around the quote");
    pre_in_em: "<em><pre>x</pre></em>"
        => tree(r#"codeblock("x")"#),
        defect(Rfc028, "lone `*` line becomes an empty list, plus bare <pre> (RFC 024): unterminated code block");
    heading_in_em: "<em><h2>x</h2></em>"
        => tree(r#"h2("x")"#),
        defect(Rfc028, "lone `*` lines parse as empty lists around the heading");
}

// ── in <a> ─────────────────────────────────────────────────────────────────

cells! {
    p_in_a: r#"<a href="/out"><p>x</p><p>y</p></a>"#
        => undecided("Q4: block content inside <a> (valid HTML5): where does the link go?"),
        defect(Unowned, "the paragraphs collapse into one link text `xy`: the paragraph break and the space are lost");
    ul_in_a: r#"<a href="/out"><ul><li>x</li><li>y</li></ul></a>"#
        => undecided("Q4"),
        defect(Unowned, "the list items are emptied and their text moves into one link `xy` after the list");
    blockquote_in_a: r#"<a href="/out"><blockquote><p>x</p></blockquote></a>"#
        => undecided("Q4"),
        defect(Unowned, "the quote disappears; only a link with its text remains");
    pre_in_a: r#"<a href="/out"><pre>x</pre></a>"#
        => undecided("Q4"),
        defect(Unowned, "bare <pre> inside <a>: no opening fence, and the link lands in an unterminated code block");
    heading_in_a: r#"<a href="/out"><h2>x</h2></a>"#
        => undecided("Q4"),
        defect(Unowned, "the heading is emptied; its text becomes a link paragraph after it");
}

// ── in <code> ──────────────────────────────────────────────────────────────

cells! {
    p_in_code: "<code><p>x</p><p>y</p></code>"
        => undecided("Q5: block content inside inline <code>"),
        defect(Unowned, "lone backtick lines around the paragraphs: stray backticks, no code span");
    ul_in_code: "<code><ul><li>x</li><li>y</li></ul></code>"
        => undecided("Q5"),
        defect(Unowned, "lone backtick lines around the list: stray backticks");
    blockquote_in_code: "<code><blockquote><p>x</p></blockquote></code>"
        => undecided("Q5"),
        defect(Unowned, "lone backtick lines around the quote: stray backticks");
    pre_in_code: "<code><pre>x</pre></code>"
        => undecided("Q5"),
        defect(Unowned, "stray backtick, plus bare <pre>: unterminated code block");
    heading_in_code: "<code><h2>x</h2></code>"
        => undecided("Q5"),
        defect(Unowned, "lone backtick lines around the heading: stray backticks");
}
