//! §6.2 — fences longer than their content's backtick runs; destinations with
//! spaces, parentheses and quotes that parse back to the original URL.
//! Also RFC 036: leading whitespace in a heading (§5.6), and a run of
//! otherwise-empty nested list item markers colliding with a thematic
//! break (§5.5).

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

// ── RFC 036 §5.6: leading whitespace in a heading ──────────────────────────
//
// A heading's marker leaves the same bookkeeping consumed as a list item's
// (RFC 036 §5.1); the stray leading space was cosmetic here, not a validity
// defect, but it is the same bug and moves with the same fix.

cells! {
    h1_ws: "<h1> Quarterly Report</h1>"
        => tree(r#"h1("Quarterly Report")"#);
    // Control: trailing whitespace was already stripped before RFC 036.
    h2_ws_trailing: "<h2>T </h2>"
        => tree(r#"h2("T")"#);
    // Found while implementing the fix: an image inside the heading, right
    // after the marker, must not have the space AFTER it mistaken for the
    // heading's own leading whitespace -- that space separates the image
    // from "t" and must survive.
    h2_image_then_text: r#"<h2><img src="i.png" alt="i"> t</h2>"#
        => tree(r#"h2(image[i.png]("i"), " t")"#);
}

// ── RFC 036 §5.5: empty nested list items vs a thematic break ──────────────
//
// A chain of otherwise-empty nested list items writes only its own markers,
// sharing one line: `- - -` for three levels. That line, once nothing else
// follows it, also matches CommonMark's thematic break -- three or more of
// the same character, optionally spaced -- so the run must not read as one
// once it reaches three. One and two levels were already correct and are
// controls here.

cells! {
    empty_li_d1: "<ul><li></li></ul>"
        => tree(r#"ul(li())"#);
    empty_li_d2: "<ul><li><ul><li></li></ul></li></ul>"
        => tree(r#"ul(li(ul(li())))"#);
    empty_li_d3: "<ul><li><ul><li><ul><li></li></ul></li></ul></li></ul>"
        => tree(r#"ul(li(ul(li(ul(li())))))"#);
    empty_li_d5: "<ul><li><ul><li><ul><li><ul><li><ul><li></li></ul></li></ul></li></ul></li></ul></li></ul>"
        => tree(r#"ul(li(ul(li(ul(li(ul(li(ul(li())))))))))"#);
}
