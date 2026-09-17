//! §4.1 — inline construct × container.
//!
//! Rows: `<img>`, `<strong>`, `<em>`, `<code>`, `<a>`, text with `_ * [ ] ( )`.
//! Columns: `<a>`, `<pre>`, `<code>`, `<li>`, `<blockquote>`, heading.
//!
//! Question ids (Q1…) refer to the review request's question list.

use crate::harness::{tree, undecided};

// ── in <a> ─────────────────────────────────────────────────────────────────

cells! {
    img_in_a: r#"<a href="/out"><img src="i.png" alt="pic"></a>"#
        => tree(r#"para(link[/out](image[i.png]("pic")))"#);
    strong_in_a: r#"<a href="/out"><strong>b</strong></a>"#
        => tree(r#"para(link[/out](strong("b")))"#);
    em_in_a: r#"<a href="/out"><em>e</em></a>"#
        => tree(r#"para(link[/out](em("e")))"#);
    code_in_a: r#"<a href="/out"><code>c</code></a>"#
        => tree(r#"para(link[/out](code("c")))"#);
    a_in_a: r#"<a href="/out"><a href="/in">t</a></a>"#
        => undecided("Q1: html5ever turns nested anchors into an empty /out link and a /in link; should the empty link be emitted?");
    text_in_a: r#"<a href="/out">a_b *c* [d] (e)</a>"#
        => tree(r#"para(link[/out]("a_b *c* [d] (e)"))"#);
}

// ── in <pre> ───────────────────────────────────────────────────────────────

cells! {
    img_in_pre: r#"<pre><img src="i.png" alt="pic"></pre>"#
        => tree(r#"codeblock()"#);
    strong_in_pre: r#"<pre><strong>b</strong></pre>"#
        => tree(r#"codeblock("b")"#);
    em_in_pre: r#"<pre><em>e</em></pre>"#
        => tree(r#"codeblock("e")"#);
    code_in_pre: r#"<pre><code>c</code></pre>"#
        => tree(r#"codeblock("c")"#);
    a_in_pre: r#"<pre><a href="/in">t</a></pre>"#
        => tree(r#"codeblock("t")"#);
    text_in_pre: r#"<pre>a_b *c* [d] (e)</pre>"#
        => tree(r#"codeblock("a_b *c* [d] (e)")"#);
}

// ── in <code> ──────────────────────────────────────────────────────────────

cells! {
    img_in_code: r#"<code><img src="i.png" alt="pic"></code>"#
        => tree(r#""#);
    strong_in_code: r#"<code><strong>b</strong></code>"#
        => tree(r#"para(code("b"))"#);
    em_in_code: r#"<code><em>e</em></code>"#
        => tree(r#"para(code("e"))"#);
    code_in_code: r#"<code><code>c</code></code>"#
        => tree(r#"para(code("c"))"#);
    a_in_code: r#"<code><a href="/in">t</a></code>"#
        => tree(r#"para(code("t"))"#);
    text_in_code: r#"<code>a_b *c* [d] (e)</code>"#
        => tree(r#"para(code("a_b *c* [d] (e)"))"#),
        defect(Rfc010, "escaping applied inside the code span, where CommonMark does not honour it: literal backslashes (audit A-03)");
}

// ── in <li> ────────────────────────────────────────────────────────────────

cells! {
    img_in_li: r#"<ul><li><img src="i.png" alt="pic"></li></ul>"#
        => tree(r#"ul(li(image[i.png]("pic")))"#);
    strong_in_li: r#"<ul><li><strong>b</strong></li></ul>"#
        => tree(r#"ul(li(strong("b")))"#);
    em_in_li: r#"<ul><li><em>e</em></li></ul>"#
        => tree(r#"ul(li(em("e")))"#);
    code_in_li: r#"<ul><li><code>c</code></li></ul>"#
        => tree(r#"ul(li(code("c")))"#);
    a_in_li: r#"<ul><li><a href="/in">t</a></li></ul>"#
        => tree(r#"ul(li(link[/in]("t")))"#);
    text_in_li: r#"<ul><li>a_b *c* [d] (e)</li></ul>"#
        => tree(r#"ul(li("a_b *c* [d] (e)"))"#);
}

// ── in <blockquote> ────────────────────────────────────────────────────────

cells! {
    img_in_blockquote: r#"<blockquote><img src="i.png" alt="pic"></blockquote>"#
        => tree(r#"quote(para(image[i.png]("pic")))"#);
    strong_in_blockquote: r#"<blockquote><strong>b</strong></blockquote>"#
        => tree(r#"quote(para(strong("b")))"#);
    em_in_blockquote: r#"<blockquote><em>e</em></blockquote>"#
        => tree(r#"quote(para(em("e")))"#);
    code_in_blockquote: r#"<blockquote><code>c</code></blockquote>"#
        => tree(r#"quote(para(code("c")))"#);
    a_in_blockquote: r#"<blockquote><a href="/in">t</a></blockquote>"#
        => tree(r#"quote(para(link[/in]("t")))"#);
    text_in_blockquote: r#"<blockquote>a_b *c* [d] (e)</blockquote>"#
        => tree(r#"quote(para("a_b *c* [d] (e)"))"#);
}

// ── in a heading ───────────────────────────────────────────────────────────

cells! {
    img_in_heading: r#"<h2><img src="i.png" alt="pic"></h2>"#
        => tree(r#"h2(image[i.png]("pic"))"#);
    strong_in_heading: r#"<h2><strong>b</strong></h2>"#
        => tree(r#"h2(strong("b"))"#);
    em_in_heading: r#"<h2><em>e</em></h2>"#
        => tree(r#"h2(em("e"))"#);
    code_in_heading: r#"<h2><code>c</code></h2>"#
        => tree(r#"h2(code("c"))"#);
    a_in_heading: r#"<h2><a href="/in">t</a></h2>"#
        => tree(r#"h2(link[/in]("t"))"#);
    text_in_heading: r#"<h2>a_b *c* [d] (e)</h2>"#
        => tree(r#"h2("a_b *c* [d] (e)")"#);
}
