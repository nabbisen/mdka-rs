//! Code holds text only (RFC 024 amendment of 2026-09-17, slice 024b):
//! inline markup inside `<pre><code>`, several `<code>` children in one
//! `<pre>`, and text beside them. Expectations were written by the architect
//! from the rules in the 024b addendum, not from output.

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
}
