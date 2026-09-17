//! §6.1 — escaping round-trip. Text that looks like Markdown syntax must parse
//! back to exactly the original characters, in the construct the HTML meant.

use crate::harness::tree;

// ── line-leading syntax in a paragraph ─────────────────────────────────────

cells! {
    digit_period_at_line_start: "<p>1986. A great year</p>"
        => tree(r#"para("1986. A great year")"#);
    digit_paren_at_line_start: "<p>1) not a list</p>"
        => tree(r#"para("1) not a list")"#);
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
        => tree(r#"para("~~~")"#);
    backtick_fence_text: "<p>```</p>"
        => tree(r#"para("```")"#);
    html_tag_text: "<p>&lt;div&gt;</p>"
        => tree(r#"para("<div>")"#);
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
        => tree(r#"para("<http://example.com>")"#);
    entity_like_text: "<p>&amp;copy; &amp;#42;</p>"
        => tree(r#"para("&copy; &#42;")"#);
}

// ── code spans ─────────────────────────────────────────────────────────────

cells! {
    underscore_in_code_span: "<p><code>snake_case</code></p>"
        => tree(r#"para(code("snake_case"))"#);
    markdown_in_code_span: "<p><code>*a* [b](c) # d</code></p>"
        => tree(r##"para(code("*a* [b](c) # d"))"##);
    backslash_in_code_span: r"<p><code>C:\dir\*</code></p>"
        => tree(r#"para(code("C:\\dir\\*"))"#);
}

// ── other containers ───────────────────────────────────────────────────────

cells! {
    digit_period_in_list_item: "<ul><li>1986. A great year</li></ul>"
        => tree(r#"ul(li("1986. A great year"))"#);
    digit_period_in_heading: "<h2>1986. A great year</h2>"
        => tree(r#"h2("1986. A great year")"#);
    digit_period_in_blockquote: "<blockquote><p>1986. A great year</p></blockquote>"
        => tree(r#"quote(para("1986. A great year"))"#);
    brackets_in_link_text: r#"<a href="/x">a [b] c</a>"#
        => tree(r#"para(link[/x]("a [b] c"))"#);
    bracket_in_image_alt: r#"<img src="i.png" alt="a]b">"#
        => tree(r#"para(image[i.png]("a]b"))"#);
}

// ── added before the change (RFC 010 handoff §4.2): adjacent emphasis (§3.8),
// minimal escaping (A-10), and line-leading syntax the original cells did not
// reach ─────────────────────────────────────────────────────────────────────

cells! {
    adjacent_strong_then_em: "<p><strong>a</strong><em>b</em></p>"
        => tree(r#"para(strong("a"), em("b"))"#);
    adjacent_em_then_strong: "<p><em>a</em><strong>b</strong></p>"
        => tree(r#"para(em("a"), strong("b"))"#);
    adjacent_strong_then_strong: "<p><strong>a</strong><strong>b</strong></p>"
        => tree(r#"para(strong("a"), strong("b"))"#);
    adjacent_em_then_em: "<p><em>a</em><em>b</em></p>"
        => tree(r#"para(em("a"), em("b"))"#);
    snake_case_here: "<p>snake_case_here</p>"
        => tree(r#"para("snake_case_here")"#);
    bang_not_before_bracket: "<p>Wow! Really!</p>"
        => tree(r#"para("Wow! Really!")"#);
    setext_underline_after_line_break: "<p>Title<br>===</p>"
        => tree(r#"para("Title", br, "===")"#);
    thematic_break_after_line_break: "<p>a<br>---</p>"
        => tree(r#"para("a", br, "---")"#);
    digit_paren_in_blockquote: "<blockquote><p>1) x</p></blockquote>"
        => tree(r#"quote(para("1) x"))"#);
}

// ── added before the change: shapes the design must also cover ─────────────

cells! {
    digit_period_split_across_elements: "<p><span>1986</span>. A great year</p>"
        => tree(r#"para("1986. A great year")"#);
    bullet_after_line_break: "<p>a<br>- b</p>"
        => tree(r#"para("a", br, "- b")"#);
    tilde_fence_after_line_break: "<p>a<br>~~~</p><p>after</p>"
        => tree(r#"para("a", br, "~~~"), para("after")"#);
    html_block_after_line_break: "<p>a<br>&lt;div&gt;</p>"
        => tree(r#"para("a", br, "<div>")"#);
    gt_in_list_item: "<ul><li>&gt; q</li></ul>"
        => tree(r#"ul(li("> q"))"#);
    hash_in_list_item: "<ul><li># x</li></ul>"
        => tree(r##"ul(li("# x"))"##);
    hash_closing_sequence_in_heading: "<h2>C #</h2>"
        => tree(r##"h2("C #")"##);
    bang_before_link: r#"<p>Wow!<a href="/x">x</a></p>"#
        => tree(r#"para("Wow!", link[/x]("x"))"#);
    code_span_ending_with_backtick: "<p><code>a`</code></p>"
        => tree(r#"para(code("a`"))"#);
    link_title_with_both_quotes: r#"<a href="/x" title="a &quot;b&quot; 'c'">x</a>"#
        => tree(r##"para(link[/x "a \"b\" 'c'"]("x"))"##);
    fence_language_with_backtick: r#"<pre><code class="language-a`b">x</code></pre>"#
        => tree(r#"codeblock("x")"#);
}
