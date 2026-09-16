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
        => tree(r#"para(link[/out](image[i.png]("pic")))"#),
        defect(Rfc024, "image escapes the link: an image followed by an empty link");
    strong_in_a: r#"<a href="/out"><strong>b</strong></a>"#
        => tree(r#"para(link[/out](strong("b")))"#),
        defect(Rfc024, "`**` delimiters escape the link: literal `****` before it");
    em_in_a: r#"<a href="/out"><em>e</em></a>"#
        => tree(r#"para(link[/out](em("e")))"#),
        defect(Rfc024, "`*` delimiters escape the link: literal `**` before it");
    code_in_a: r#"<a href="/out"><code>c</code></a>"#
        => tree(r#"para(link[/out](code("c")))"#),
        defect(Rfc024, "backticks escape the link: literal ``` `` ``` before it, code span lost");
    a_in_a: r#"<a href="/out"><a href="/in">t</a></a>"#
        => undecided("Q1: html5ever turns nested anchors into an empty /out link and a /in link; should the empty link be emitted?");
    text_in_a: r#"<a href="/out">a_b *c* [d] (e)</a>"#
        => tree(r#"para(link[/out]("a_b *c* [d] (e)"))"#);
}

// ── in <pre> ───────────────────────────────────────────────────────────────

cells! {
    img_in_pre: r#"<pre><img src="i.png" alt="pic"></pre>"#
        => undecided("Q2: inline markup, links and images inside <pre> without <code>"),
        defect(Rfc024, "bare <pre>: no opening fence, so the closing fence opens an unterminated code block");
    strong_in_pre: r#"<pre><strong>b</strong></pre>"#
        => undecided("Q2"),
        defect(Rfc024, "bare <pre>: no opening fence; `b` becomes bold prose and the fence is left open");
    em_in_pre: r#"<pre><em>e</em></pre>"#
        => undecided("Q2"),
        defect(Rfc024, "bare <pre>: no opening fence; `e` becomes emphasis and the fence is left open");
    code_in_pre: r#"<pre><code>c</code></pre>"#
        => tree(r#"codeblock("c")"#);
    a_in_pre: r#"<pre><a href="/in">t</a></pre>"#
        => undecided("Q2"),
        defect(Rfc024, "bare <pre>: no opening fence; the text is prose, the link emptied, the fence left open");
    text_in_pre: r#"<pre>a_b *c* [d] (e)</pre>"#
        => tree(r#"codeblock("a_b *c* [d] (e)")"#),
        defect(Rfc024, "bare <pre>: no opening fence; the content is parsed as Markdown (`*c*` becomes emphasis)");
}

// ── in <code> ──────────────────────────────────────────────────────────────

cells! {
    img_in_code: r#"<code><img src="i.png" alt="pic"></code>"#
        => undecided("Q3: inline markup, links and images inside inline <code>"),
        defect(Unowned, "image syntax emitted inside the code span: reads as literal `![pic](i.png)`");
    strong_in_code: r#"<code><strong>b</strong></code>"#
        => undecided("Q3"),
        defect(Unowned, "`**` emitted inside the code span: reads as literal `**b**`");
    em_in_code: r#"<code><em>e</em></code>"#
        => undecided("Q3"),
        defect(Unowned, "`*` emitted inside the code span: reads as literal `*e*`");
    code_in_code: r#"<code><code>c</code></code>"#
        => tree(r#"para(code("c"))"#);
    a_in_code: r#"<code><a href="/in">t</a></code>"#
        => undecided("Q3"),
        defect(Unowned, "link syntax emitted inside the code span: reads as literal `[t](/in)`, link lost");
    text_in_code: r#"<code>a_b *c* [d] (e)</code>"#
        => tree(r#"para(code("a_b *c* [d] (e)"))"#),
        defect(Rfc010Planned, "escaping applied inside the code span, where CommonMark does not honour it: literal backslashes (audit A-03)");
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
        => tree(r#"quote(para(image[i.png]("pic")))"#),
        defect(Unowned, "blockquote whose content starts with an inline element loses its `>` prefix entirely");
    strong_in_blockquote: r#"<blockquote><strong>b</strong></blockquote>"#
        => tree(r#"quote(para(strong("b")))"#),
        defect(Unowned, "blockquote whose content starts with an inline element loses its `>` prefix entirely");
    em_in_blockquote: r#"<blockquote><em>e</em></blockquote>"#
        => tree(r#"quote(para(em("e")))"#),
        defect(Unowned, "blockquote whose content starts with an inline element loses its `>` prefix entirely");
    code_in_blockquote: r#"<blockquote><code>c</code></blockquote>"#
        => tree(r#"quote(para(code("c")))"#),
        defect(Unowned, "blockquote whose content starts with an inline element loses its `>` prefix entirely");
    a_in_blockquote: r#"<blockquote><a href="/in">t</a></blockquote>"#
        => tree(r#"quote(para(link[/in]("t")))"#),
        defect(Unowned, "blockquote whose content starts with an inline element loses its `>` prefix entirely");
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
