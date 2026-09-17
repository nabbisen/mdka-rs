//! §6.2 — fences longer than their content's backtick runs; destinations with
//! spaces, parentheses and quotes that parse back to the original URL.

use crate::harness::tree;

// ── fences and code spans ──────────────────────────────────────────────────

cells! {
    fence_content_with_triple_backticks: "<pre><code>a\n```\nb</code></pre>"
        => tree(r#"codeblock("a\n```\nb")"#);
    fence_content_with_four_backticks: "<pre><code>````</code></pre>"
        => tree(r#"codeblock("````")"#);
    fence_content_with_tildes: "<pre><code>~~~\nx</code></pre>"
        => tree(r#"codeblock("~~~\nx")"#);
    fence_with_language: r#"<pre><code class="language-rust">fn main() {}</code></pre>"#
        => tree(r#"codeblock[rust]("fn main() {}")"#);
    code_span_with_backtick: "<p><code>a`b</code></p>"
        => tree(r#"para(code("a`b"))"#);
    code_span_with_double_backtick: "<p><code>a``b</code></p>"
        => tree(r#"para(code("a``b"))"#);
    code_span_starting_with_backtick: "<p><code>`a</code></p>"
        => tree(r#"para(code("`a"))"#);
}

// ── destinations and titles ────────────────────────────────────────────────

cells! {
    link_destination_with_space: r#"<a href="/a b.html">x</a>"#
        => tree(r#"para(link[/a b.html]("x"))"#);
    link_destination_with_parentheses: r#"<a href="/wiki/A_(b)">x</a>"#
        => tree(r#"para(link[/wiki/A_(b)]("x"))"#);
    link_destination_with_unbalanced_paren: r#"<a href="/a)b">x</a>"#
        => tree(r#"para(link[/a)b]("x"))"#);
    link_destination_with_quote: r#"<a href='/a"b'>x</a>"#
        => tree(r#"para(link[/a"b]("x"))"#);
    link_destination_with_angle_brackets: r#"<a href="/a&lt;b&gt;">x</a>"#
        => tree(r#"para(link[/a<b>]("x"))"#);
    link_title_with_quotes: r#"<a href="/x" title='say "hi"'>x</a>"#
        => tree(r#"para(link[/x "say \"hi\""]("x"))"#);
    image_source_with_space: r#"<img src="a b.png" alt="x">"#
        => tree(r#"para(image[a b.png]("x"))"#);
    image_title_with_quotes: r#"<img src="i.png" alt="x" title='a "b"'>"#
        => tree(r#"para(image[i.png "a \"b\""]("x"))"#);
}
