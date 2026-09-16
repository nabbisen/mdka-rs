//! RFC 024 — inline composition through the output sink.
//!
//! The RFC 025 harness (`tests/output_validity/`) checks these constructs by
//! parsing the output. The tests here pin the bytes the acceptance criteria
//! name, so a change to them is a visible, reviewed change.

mod common;
use common::conv;

// ── criteria 1, 2: inline elements stay inside the link ────────────────────

#[test]
fn linked_image_is_a_link_containing_an_image() {
    assert_eq!(
        conv(r#"<a href="/page"><img src="i.png" alt="pic"></a>"#),
        "[![pic](i.png)](/page)\n"
    );
}

#[test]
fn strong_inside_a_link_stays_inside_it() {
    assert_eq!(
        conv(r#"<a href="/x"><strong>b</strong></a>"#),
        "[**b**](/x)\n"
    );
}

#[test]
fn code_span_inside_a_link_stays_inside_it() {
    assert_eq!(conv(r#"<a href="/x"><code>c</code></a>"#), "[`c`](/x)\n");
}

// ── criterion 8: spaces inside a link, as outside one ──────────────────────

const INSIDE: &str = r#"<a href="/x">Read <strong>more</strong> now</a>"#;
const OUTSIDE: &str = "<p>Read <strong>more</strong> now</p>";

#[test]
fn delimiters_inside_a_link_are_in_the_right_place() {
    assert_eq!(conv(INSIDE), "[Read **more** now](/x)\n");
}

#[test]
fn text_inside_a_link_is_byte_identical_to_the_same_markup_outside() {
    let outside = conv(OUTSIDE);
    let inside = conv(INSIDE);
    let link_text = inside
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix("](/x)\n"))
        .expect("a single link");
    assert_eq!(link_text, outside.trim_end());
}

#[test]
fn whitespace_at_the_end_of_link_text_is_kept_after_the_link() {
    assert_eq!(conv(r#"<p><a href="/x">x </a>y</p>"#), "[x](/x) y\n");
}

// ── criterion 7: nested and empty links ────────────────────────────────────

#[test]
fn link_with_no_text_and_no_image_emits_nothing() {
    assert_eq!(conv(r#"<p>a <a href="/x"></a> b</p>"#), "a b\n");
    assert_eq!(conv(r#"<p>a <a href="/x"> </a> b</p>"#), "a b\n");
}

#[test]
fn nested_links_keep_the_inner_link_and_drop_the_empty_outer_one() {
    // html5ever closes the outer <a> when the inner one opens, leaving an
    // empty outer link and the inner link as siblings.
    assert_eq!(
        conv(r#"<a href="/out"><a href="/in">t</a></a>"#),
        "[t](/in)\n"
    );
}

// ── criteria 3, 4: <pre> owns its fence ────────────────────────────────────

#[test]
fn bare_pre_has_a_balanced_fence_and_the_next_paragraph_is_outside() {
    assert_eq!(
        conv("<pre>plain</pre><p>After</p>"),
        "```\nplain\n```\n\nAfter\n"
    );
}

#[test]
fn bare_pre_holds_text_only() {
    assert_eq!(
        conv(r#"<pre>a <strong>b</strong> <a href="/x">c</a><img src="i.png" alt="i"></pre>"#),
        "```\na b c\n```\n"
    );
}

#[test]
fn empty_pre_is_an_empty_balanced_code_block() {
    assert_eq!(conv("<pre></pre>"), "```\n```\n");
}

#[test]
fn pre_code_keeps_its_language() {
    assert_eq!(
        conv(r#"<pre><code class="language-rust">fn main() {}</code></pre>"#),
        "```rust\nfn main() {}\n```\n"
    );
}

#[test]
fn whitespace_before_code_in_pre_is_content_after_the_fence() {
    // Rule 6: the fence starts its line and keeps the language; the whitespace
    // is the code block's text, as a browser shows it.
    assert_eq!(
        conv("<pre>\n  <code class=\"language-js\">x()</code>\n</pre>"),
        "```js\n  x()\n```\n"
    );
}

// ── criterion 9: an inline code span holds text only ───────────────────────

#[test]
fn code_span_holds_text_only() {
    assert_eq!(conv("<code><strong>b</strong></code>"), "`b`\n");
    assert_eq!(conv("<code><em>e</em></code>"), "`e`\n");
    assert_eq!(conv(r#"<code><a href="/in">t</a></code>"#), "`t`\n");
    assert_eq!(conv("<code><code>c</code></code>"), "`c`\n");
}

#[test]
fn code_span_with_no_text_emits_nothing() {
    assert_eq!(conv("<p>a <code></code> b</p>"), "a b\n");
    assert_eq!(conv(r#"<code><img src="i.png" alt="pic"></code>"#), "");
}

// ── criterion 10: the sink writes the blockquote prefix ────────────────────

#[test]
fn blockquote_starting_with_an_inline_element_keeps_its_prefix() {
    assert_eq!(
        conv("<blockquote><strong>b</strong> rest</blockquote>"),
        "> **b** rest\n"
    );
    assert_eq!(conv("<blockquote><em>e</em></blockquote>"), "> *e*\n");
    assert_eq!(conv("<blockquote><code>c</code></blockquote>"), "> `c`\n");
    assert_eq!(
        conv(r#"<blockquote><img src="i.png" alt="pic"></blockquote>"#),
        "> ![pic](i.png)\n"
    );
    assert_eq!(
        conv(r#"<blockquote><a href="/in">t</a></blockquote>"#),
        "> [t](/in)\n"
    );
}

#[test]
fn nested_blockquote_starting_with_an_inline_element_keeps_both_prefixes() {
    assert_eq!(
        conv("<blockquote><blockquote><em>x</em></blockquote></blockquote>"),
        "> > *x*\n"
    );
}
