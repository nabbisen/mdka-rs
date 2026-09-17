//! §6.2 — fences longer than their content's backtick runs; destinations with
//! spaces, parentheses and quotes that parse back to the original URL.

use crate::harness::tree;

// ── fences and code spans ──────────────────────────────────────────────────

cells! {
    fence_content_with_triple_backticks: "<pre><code>a\n```\nb</code></pre>"
        => tree(r#"codeblock("a\n```\nb")"#),
        defect(Rfc010, "fence fixed at three backticks: the content's ``` closes it early and a stray fence is left open (audit A-05)");
    fence_content_with_four_backticks: "<pre><code>````</code></pre>"
        => tree(r#"codeblock("````")"#),
        defect(Rfc010, "fence fixed at three backticks: content ```` is read as a fence; the block is emptied and one left open (audit A-05)");
    fence_content_with_tildes: "<pre><code>~~~\nx</code></pre>"
        => tree(r#"codeblock("~~~\nx")"#);
    fence_with_language: r#"<pre><code class="language-rust">fn main() {}</code></pre>"#
        => tree(r#"codeblock[rust]("fn main() {}")"#);
    code_span_with_backtick: "<p><code>a`b</code></p>"
        => tree(r#"para(code("a`b"))"#),
        defect(Rfc010, "backtick in a code span is backslash-escaped instead of widening the delimiter: the span ends early (audits A-03, A-05)");
    code_span_with_double_backtick: "<p><code>a``b</code></p>"
        => tree(r#"para(code("a``b"))"#),
        defect(Rfc010, "backticks in a code span are backslash-escaped instead of widening the delimiter: the span ends early (audits A-03, A-05)");
    code_span_starting_with_backtick: "<p><code>`a</code></p>"
        => tree(r#"para(code("`a"))"#),
        defect(Rfc010, "leading backtick in a code span is backslash-escaped: the span ends after the backslash (audits A-03, A-05)");
}

// ── destinations and titles ────────────────────────────────────────────────

cells! {
    link_destination_with_space: r#"<a href="/a b.html">x</a>"#
        => tree(r#"para(link[/a b.html]("x"))"#),
        defect(Rfc010, "destination with a space emitted bare: not a link at all, literal `[x](/a b.html)` (audit A-04)");
    link_destination_with_parentheses: r#"<a href="/wiki/A_(b)">x</a>"#
        => tree(r#"para(link[/wiki/A_(b)]("x"))"#);
    link_destination_with_unbalanced_paren: r#"<a href="/a)b">x</a>"#
        => tree(r#"para(link[/a)b]("x"))"#),
        defect(Rfc010, "unbalanced `)` in the destination is not escaped: the link ends early at `/a` (audit A-04)");
    link_destination_with_quote: r#"<a href='/a"b'>x</a>"#
        => tree(r#"para(link[/a"b]("x"))"#);
    link_destination_with_angle_brackets: r#"<a href="/a&lt;b&gt;">x</a>"#
        => tree(r#"para(link[/a<b>]("x"))"#);
    link_title_with_quotes: r#"<a href="/x" title='say "hi"'>x</a>"#
        => tree(r#"para(link[/x "say \"hi\""]("x"))"#),
        defect(Rfc010, "a double quote in the title is not escaped: not a link at all (audit A-04)");
    image_source_with_space: r#"<img src="a b.png" alt="x">"#
        => tree(r#"para(image[a b.png]("x"))"#),
        defect(Rfc010, "source with a space emitted bare: not an image at all (audit A-04)");
    image_title_with_quotes: r#"<img src="i.png" alt="x" title='a "b"'>"#
        => tree(r#"para(image[i.png "a \"b\""]("x"))"#),
        defect(Rfc010, "a double quote in the title is not escaped: not an image at all (audit A-04)");
}
