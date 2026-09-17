//! Code holds text only (RFC 024 amendment of 2026-09-17, slice 024b):
//! inline markup inside `<pre><code>`, several `<code>` children in one
//! `<pre>`, and text beside them, and whitespace around `<code>` (024c, rule 6),
//! block elements inside `<pre>` (024d, rule 7), and `<br>` inside code (024e,
//! rule 8).
//! Expectations were written by the architect from the rules in the 024b,
//! 024c, 024d and 024e addenda, not from output; the two container cells at the end
//! enter the structures 024d §3 states.

use crate::harness::tree;

cells! {
    strong_in_pre_code: "<pre><code><b>kw</b> fn</code></pre>"
        => tree(r#"codeblock("kw fn")"#);
    em_in_pre_code_with_language: r#"<pre><code class="language-rs"><em>let</em> x</code></pre>"#
        => tree(r#"codeblock[rs]("let x")"#);
    a_in_pre_code: r#"<pre><code><a href="/x">t</a></code></pre>"#
        => tree(r#"codeblock("t")"#);
    img_in_pre_code: r#"<pre><code><img src="i.png" alt="pic"></code></pre>"#
        => tree(r#"codeblock()"#);
    two_codes_in_pre: "<pre><code>a</code><code>b</code></pre>"
        => tree(r#"codeblock("ab")"#);
    language_from_first_code_only: r#"<pre><code>a</code><code class="language-js">b</code></pre>"#
        => tree(r#"codeblock("ab")"#);
    text_before_code_in_pre: "<pre>text<code>x</code></pre>"
        => tree(r#"codeblock("textx")"#);
    text_after_code_in_pre: "<pre><code>x</code> tail</pre>"
        => tree(r#"codeblock("x tail")"#);
    four_spaces_before_code_in_pre: r#"<pre>    <code class="language-js">x</code></pre>"#
        => tree(r#"codeblock[js]("    x")"#);
    two_spaces_before_code_in_pre: r#"<pre>  <code class="language-js">x</code></pre>"#
        => tree(r#"codeblock[js]("  x")"#);
    pretty_printed_pre_code_then_content: "<pre>\n    <code class=\"language-js\">x</code>\n</pre><p>after</p><h2>later</h2>"
        => tree(r#"codeblock[js]("    x"), para("after"), h2("later")"#);
    newline_before_code_in_pre: "<pre>\n<code>x</code></pre>"
        => tree(r#"codeblock("x")"#);
    newline_after_code_in_pre: "<pre><code>x</code>\n</pre>"
        => tree(r#"codeblock("x")"#);
    language_not_taken_after_text: r#"<pre>text<code class="language-js">x</code></pre>"#
        => tree(r#"codeblock("textx")"#);
}

// ── blocks inside code (024d, rule 7) ──────────────────────────────────────

cells! {
    blockquote_and_ul_in_pre: "<pre><blockquote><p>q</p></blockquote><ul><li>a</li></ul></pre><p>after</p>"
        => tree(r#"codeblock("q\na"), para("after")"#);
    p_in_pre: "<pre><p>one</p><p>two</p></pre><p>after</p>"
        => tree(r#"codeblock("one\ntwo"), para("after")"#);
    div_in_pre: "<pre><div>x</div></pre><p>after</p>"
        => tree(r#"codeblock("x"), para("after")"#);
    div_in_pre_code: "<pre><code><div>x</div></code></pre><p>after</p>"
        => tree(r#"codeblock("x"), para("after")"#);
}

// ── blocks inside code inside a container (024d §3): one code block, the
// container's prefix on every line (RFC 035), `a\nb` as its text ─────────────

cells! {
    pre_blocks_in_li: "<ul><li><pre><p>a</p><p>b</p></pre></li></ul>"
        => tree(r#"ul(li(codeblock("a\nb")))"#);
    pre_blocks_in_blockquote: "<blockquote><pre><p>a</p><p>b</p></pre></blockquote>"
        => tree(r#"quote(codeblock("a\nb"))"#);
}

// ── <br> inside code (024e, rule 8); the pretty-printed reading of rule 7 ──

cells! {
    br_in_pre: "<pre>line1<br>line2<br>line3</pre>"
        => tree(r#"codeblock("line1\nline2\nline3")"#),
        defect(Rfc024, "the <br> is written as a Markdown hard break: two trailing spaces added to every code line");
    br_in_pre_code: "<pre><code>x<br>y</code></pre>"
        => tree(r#"codeblock("x\ny")"#),
        defect(Rfc024, "the <br> is written as a Markdown hard break: trailing spaces added to the code line");
    br_in_code_span: "<p><code>a<br>b</code></p>"
        => tree(r#"para(code("a b"))"#),
        defect(Rfc024, "the <br> is written as a Markdown hard break inside the code span: three spaces instead of one");
    pretty_printed_blocks_in_pre: "<pre>\n  <p>a</p>\n  <p>b</p>\n</pre><p>after</p>"
        => tree(r#"codeblock("  \na\n  \nb"), para("after")"#);
}
