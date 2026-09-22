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

// ── slice `036b` §2: the same collision, with a non-empty sibling ──────────
//
// The swap above lands on the one item whose own line collides; a sibling
// after it, in the same list, must swap the same way, or the two would no
// longer share a bullet and CommonMark would read two lists where the
// source had one. Two empty siblings already swapped identically (each hits
// the collision on its own turn); this is the sibling that does not, and
// must still be made to match.

cells! {
    empty_li_sibling_nonempty: "<ul><li><ul><li><ul><li></li><li>x</li></ul></li></ul></li></ul>"
        => tree(r#"ul(li(ul(li(ul(li(), li("x"))))))"#);
    // Control: two empty siblings, already correct before this slice.
    empty_li_all_empty_siblings: "<ul><li><ul><li><ul><li></li><li></li></ul></li></ul></li></ul>"
        => tree(r#"ul(li(ul(li(ul(li(), li())))))"#);
}

// ── slice `036b` §3: a thematic break collides with its own item's marker ──
//
// `<hr>` shares its line with an item's marker the same way an empty nested
// item does (RFC 035's "only markers" idiom): `- ---` is the marker's `-`
// plus the break's `---`, four homogeneous dashes CommonMark reads as one
// break for the whole line, destroying the item and splitting the list.
// Moved to a continuation line instead -- where any other block content of
// an item already lives once something precedes it -- the break keeps its
// documented `---` unconditionally (addendum B §1: a substitute character
// here would itself violate `docs/src/api/elements.md`'s promise). An
// ordered item's `1. ` is never homogeneous with a break either way, so it
// is a control here.

cells! {
    li_hr_ws_then_text: "<ul><li> <hr> text</li></ul>"
        => tree(r#"ul(li(rule, para("text")))"#);
    li_hr_only_then_sibling: "<ul><li><hr></li><li>b</li></ul>"
        => tree(r#"ul(li(rule), li("b"))"#);
    ol_hr_ws_then_text: "<ol><li> <hr> text</li></ol>"
        => tree(r#"ol[1](li(rule, para("text")))"#);
    ol_hr_only_then_sibling: "<ol><li><hr></li><li>b</li></ol>"
        => tree(r#"ol[1](li(rule), li("b"))"#);
}
