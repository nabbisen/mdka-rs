//! §6.1 — escaping round-trip. Text that looks like Markdown syntax must parse
//! back to exactly the original characters, in the construct the HTML meant.

use crate::harness::tree;

// ── line-leading syntax in a paragraph ─────────────────────────────────────

cells! {
    digit_period_at_line_start: "<p>1986. A great year</p>"
        => tree(r#"para("1986. A great year")"#),
        defect(Rfc010Planned, "the backslash goes before the digit, where CommonMark does not honour it: literal `\\1986.` (audit A-09)");
    digit_paren_at_line_start: "<p>1) not a list</p>"
        => tree(r#"para("1) not a list")"#),
        defect(Rfc010Planned, "same misplaced escape for `1)`: literal `\\1)` (audit A-09)");
    hash_at_line_start: "<p># not a heading</p>"
        => tree(r##"para("# not a heading")"##);
    dash_at_line_start: "<p>- not a list</p>"
        => tree(r#"para("- not a list")"#);
    plus_at_line_start: "<p>+ not a list</p>"
        => tree(r#"para("+ not a list")"#);
    gt_at_line_start: "<p>&gt; not a quote</p>"
        => tree(r#"para("> not a quote")"#);
    thematic_break_text: "<p>---</p>"
        => tree(r#"para("---")"#);
    star_break_text: "<p>***</p>"
        => tree(r#"para("***")"#);
    setext_underline_text: "<p>Title</p><p>===</p>"
        => tree(r#"para("Title"), para("===")"#);
    tilde_fence_text: "<p>~~~</p>"
        => tree(r#"para("~~~")"#),
        defect(Rfc010Planned, "`~~~` text is not escaped (the backtick form is): it opens a code block that swallows the rest of the document");
    backtick_fence_text: "<p>```</p>"
        => tree(r#"para("```")"#);
    html_tag_text: "<p>&lt;div&gt;</p>"
        => tree(r#"para("<div>")"#),
        defect(Rfc010Planned, "text `<div>` is emitted raw and parses as an HTML block: the text disappears from rendered output");
    link_reference_definition_text: "<p>[a]: /b</p>"
        => tree(r#"para("[a]: /b")"#);
}

// ── inline syntax in a paragraph ───────────────────────────────────────────

cells! {
    underscore_in_word: "<p>snake_case</p>"
        => tree(r#"para("snake_case")"#);
    emphasis_like_text: "<p>a *b* _c_ d</p>"
        => tree(r#"para("a *b* _c_ d")"#);
    lone_backtick: "<p>a ` b</p>"
        => tree(r#"para("a ` b")"#);
    backslash_text: r"<p>a \ b \* c</p>"
        => tree(r#"para("a \\ b \\* c")"#);
    image_like_text: "<p>![x](y)</p>"
        => tree(r#"para("![x](y)")"#);
    link_like_text: "<p>[x](y)</p>"
        => tree(r#"para("[x](y)")"#);
    autolink_like_text: "<p>&lt;http://example.com&gt;</p>"
        => tree(r#"para("<http://example.com>")"#),
        defect(Rfc010Planned, "text `<http://example.com>` is emitted raw and parses as an autolink: the angle brackets are lost and a link appears");
    entity_like_text: "<p>&amp;copy; &amp;#42;</p>"
        => tree(r#"para("&copy; &#42;")"#),
        defect(Rfc010Planned, "text `&copy; &#42;` is emitted raw and parses as entity references: `(c) *`");
}

// ── code spans ─────────────────────────────────────────────────────────────

cells! {
    underscore_in_code_span: "<p><code>snake_case</code></p>"
        => tree(r#"para(code("snake_case"))"#),
        defect(Rfc010Planned, "escaping inside a code span: literal `snake\\_case` (audit A-03)");
    markdown_in_code_span: "<p><code>*a* [b](c) # d</code></p>"
        => tree(r##"para(code("*a* [b](c) # d"))"##),
        defect(Rfc010Planned, "escaping inside a code span: literal backslashes before `*` and `[` (audit A-03)");
    backslash_in_code_span: r"<p><code>C:\dir\*</code></p>"
        => tree(r#"para(code("C:\\dir\\*"))"#),
        defect(Rfc010Planned, "backslashes doubled inside a code span, where CommonMark does not unescape them (audit A-03)");
}

// ── other containers ───────────────────────────────────────────────────────

cells! {
    digit_period_in_list_item: "<ul><li>1986. A great year</li></ul>"
        => tree(r#"ul(li("1986. A great year"))"#),
        defect(Rfc010Planned, "no escape at the start of a list item's text: `1986.` becomes a nested ordered list starting at 1986 and the number is lost");
    digit_period_in_heading: "<h2>1986. A great year</h2>"
        => tree(r#"h2("1986. A great year")"#);
    digit_period_in_blockquote: "<blockquote><p>1986. A great year</p></blockquote>"
        => tree(r#"quote(para("1986. A great year"))"#),
        defect(Rfc010Planned, "no escape after `> `: `1986.` becomes an ordered list inside the quote and the number is lost");
    brackets_in_link_text: r#"<a href="/x">a [b] c</a>"#
        => tree(r#"para(link[/x]("a [b] c"))"#);
    bracket_in_image_alt: r#"<img src="i.png" alt="a]b">"#
        => tree(r#"para(image[i.png]("a]b"))"#),
        defect(Rfc010Planned, "`]` in alt text is not escaped: the image syntax breaks and is read as literal text (audit A-04)");
}
