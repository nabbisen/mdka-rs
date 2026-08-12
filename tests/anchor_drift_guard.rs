//! Integration test: RFC 006 Slice D — anchor drift guard.
//!
//! `enter_element`'s `anchor_before = matches!(tag, "a" | "pre")` duplicates
//! knowledge that lives at the two sites in `src/renderer.rs` that mutate
//! `capture_depth`/`in_pre` as part of entering that tag's own arm. If a
//! future tag starts setting either guard without `anchor_before` being
//! updated to include it, that tag's own `id` would silently stop getting
//! an anchor -- this happened twice already, for `"a"` and `"pre"`
//! themselves, before the placement fix.
//!
//! This test asserts the **observable**: every tag mdka handles specially,
//! given a non-empty `id` at the top level (not nested inside another
//! capturing element), produces an anchor somewhere in its output under a
//! mode where `preserve_ids` is on. A test that instead asserted
//! `matches!(tag, "a" | "pre")` is still the right set would pass forever
//! and catch nothing -- see the review that requested this file for why
//! that shape was explicitly rejected.

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

/// One fixture per tag `enter_element` handles specially in a `match tag`
/// arm (excluding the default `_ => {}`), built so the id-bearing element
/// is not nested inside another element that would legitimately suppress
/// its anchor (a link capture or a code block).
const FIXTURES: &[(&str, &str)] = &[
    ("h1", r#"<h1 id="t-h1">x</h1>"#),
    ("h2", r#"<h2 id="t-h2">x</h2>"#),
    ("h3", r#"<h3 id="t-h3">x</h3>"#),
    ("h4", r#"<h4 id="t-h4">x</h4>"#),
    ("h5", r#"<h5 id="t-h5">x</h5>"#),
    ("h6", r#"<h6 id="t-h6">x</h6>"#),
    ("p", r#"<p id="t-p">x</p>"#),
    ("div", r#"<div id="t-div">x</div>"#),
    ("article", r#"<article id="t-article">x</article>"#),
    ("section", r#"<section id="t-section">x</section>"#),
    ("main", r#"<main id="t-main">x</main>"#),
    ("header", r#"<header id="t-header">x</header>"#),
    ("footer", r#"<footer id="t-footer">x</footer>"#),
    ("nav", r#"<nav id="t-nav">x</nav>"#),
    ("aside", r#"<aside id="t-aside">x</aside>"#),
    ("figure", r#"<figure id="t-figure">x</figure>"#),
    (
        "figcaption",
        r#"<figcaption id="t-figcaption">x</figcaption>"#,
    ),
    ("ul", r#"<ul id="t-ul"><li>x</li></ul>"#),
    ("ol", r#"<ol id="t-ol"><li>x</li></ol>"#),
    ("li", r#"<ul><li id="t-li">x</li></ul>"#),
    (
        "blockquote",
        r#"<blockquote id="t-blockquote">x</blockquote>"#,
    ),
    ("pre", r#"<pre id="t-pre"><code>x</code></pre>"#),
    ("code", r#"<code id="t-code">x</code>"#),
    ("strong", r#"<strong id="t-strong">x</strong>"#),
    ("b", r#"<b id="t-b">x</b>"#),
    ("em", r#"<em id="t-em">x</em>"#),
    ("i", r#"<i id="t-i">x</i>"#),
    ("a", r#"<a id="t-a" href="/">x</a>"#),
    ("img", r#"<img id="t-img" src="a.png" alt="x">"#),
    ("hr", r#"<p>before</p><hr id="t-hr">"#),
    ("br", r#"<p>x<br id="t-br"></p>"#),
];

#[test]
fn every_specially_handled_tag_emits_an_anchor_for_its_own_id() {
    let opts = ConversionOptions::for_mode(ConversionMode::Balanced); // preserve_ids: true
    for (tag, html) in FIXTURES {
        let md = conv_with(html, &opts);
        let expected_id_fragment = format!("id=\"t-{tag}\"");
        assert!(
            md.contains(&expected_id_fragment),
            "tag `{tag}`: expected an anchor for its own id, found none. html={html:?} output={md:?}"
        );
    }
}
