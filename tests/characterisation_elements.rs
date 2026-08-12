//! RFC 005 Slice A — element class x mode characterisation.
//!
//! Locks in current behaviour for one representative of each element class in
//! the RFC 005 handoff, across all five modes, using each mode's own default
//! `ConversionOptions` (no field overrides). Every value below was captured by
//! actually running `html_to_markdown_with` — none are inferred from reading
//! `src/`. Identical output across all five modes is recorded explicitly
//! (`[X; 5]`), not shortened into a comment, per the handoff's instruction
//! that "identity is the data."

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

const MODES: [ConversionMode; 5] = [
    ConversionMode::Balanced,
    ConversionMode::Strict,
    ConversionMode::Minimal,
    ConversionMode::Semantic,
    ConversionMode::Preserve,
];

/// Asserts `html`'s output under each of the five modes' own defaults,
/// in the fixed order Balanced, Strict, Minimal, Semantic, Preserve.
fn assert_matrix(label: &str, html: &str, expected: [&str; 5]) {
    for (mode, exp) in MODES.iter().zip(expected.iter()) {
        let opts = ConversionOptions::for_mode(*mode);
        let out = conv_with(html, &opts);
        assert_eq!(&out, exp, "{label} / {mode} mismatch");
    }
}

#[test]
fn block_with_markdown_form_p() {
    assert_matrix("block <p>", "<p>Hello</p>", ["Hello\n"; 5]);
}

#[test]
fn block_with_markdown_form_h2() {
    assert_matrix("block <h2>", "<h2>Hello</h2>", ["## Hello\n"; 5]);
}

#[test]
fn inline_with_markdown_form_a() {
    assert_matrix(
        "inline <a>",
        r#"<p><a href="https://example.com">link</a></p>"#,
        ["[link](https://example.com)\n"; 5],
    );
}

#[test]
fn inline_with_markdown_form_code() {
    assert_matrix("inline <code>", "<p><code>x</code></p>", ["`x`\n"; 5]);
}

#[test]
fn inline_with_markdown_form_strong() {
    assert_matrix(
        "inline <strong>",
        "<p><strong>x</strong></p>",
        ["**x**\n"; 5],
    );
}

#[test]
fn inline_without_markdown_form_span() {
    // <span> has no Markdown form of its own; it and its class attribute
    // disappear, leaving only the text content, in every mode.
    assert_matrix(
        "inline w/o form <span>",
        r#"<p><span class="hl">x</span></p>"#,
        ["x\n"; 5],
    );
}

#[test]
fn generic_block_container_div() {
    assert_matrix(
        "generic block <div>",
        r#"<div class="wrap"><p>Inner</p></div>"#,
        ["Inner\n"; 5],
    );
}

#[test]
fn unknown_tag() {
    assert_matrix(
        "unknown tag",
        "<custom-tag>Hello</custom-tag>",
        ["Hello\n"; 5],
    );
}

#[test]
fn void_element_br() {
    assert_matrix("void <br>", "<p>A<br>B</p>", ["A  \nB\n"; 5]);
}

#[test]
fn void_element_hr() {
    // This test previously asserted the buggy output "A\n\n---B\n": renderer.rs's
    // "hr" arm pushed "---" via `output.push_str`, which did not reset
    // `newlines_emitted` the way `push_raw`/other tag handlers do, so the
    // following `end_block()` saw a stale newline count and emitted nothing.
    // Reproduced whenever <hr> was not the very first element in the document.
    // Fixed by RFC 016: the arm now uses `push_raw`, which resets the state
    // correctly.
    assert_matrix("void <hr>", "<p>A</p><hr><p>B</p>", ["A\n\n---\n\nB\n"; 5]);
}

#[test]
fn void_element_img() {
    assert_matrix(
        "void <img>",
        r#"<p><img src="a.png" alt="A"></p>"#,
        ["![A](a.png)\n"; 5],
    );
}

#[test]
fn always_skipped_script() {
    assert_matrix(
        "always-skipped <script>",
        "<p>A</p><script>alert(1)</script><p>B</p>",
        ["A\n\nB\n"; 5],
    );
}

#[test]
fn always_skipped_svg() {
    assert_matrix(
        "always-skipped <svg>",
        "<p>A</p><svg><circle/></svg><p>B</p>",
        ["A\n\nB\n"; 5],
    );
}

#[test]
fn always_skipped_head() {
    assert_matrix(
        "always-skipped <head>",
        "<html><head><title>T</title></head><body><p>Z</p></body></html>",
        ["Z\n"; 5],
    );
}

#[test]
fn shell_element_nav() {
    // Differs only under Minimal, whose preset sets drop_interactive_shell
    // to true by default; every other mode leaves shell content in place.
    assert_matrix(
        "shell <nav>",
        r#"<nav><a href="/">Home</a></nav><main><p>Content</p></main>"#,
        [
            "[Home](/)\n\nContent\n", // balanced
            "[Home](/)\n\nContent\n", // strict
            "Content\n",              // minimal
            "[Home](/)\n\nContent\n", // semantic
            "[Home](/)\n\nContent\n", // preserve
        ],
    );
}

#[test]
fn shell_element_footer() {
    assert_matrix(
        "shell <footer>",
        "<footer>Foot</footer><p>Body</p>",
        [
            "Foot\n\nBody\n", // balanced
            "Foot\n\nBody\n", // strict
            "Body\n",         // minimal
            "Foot\n\nBody\n", // semantic
            "Foot\n\nBody\n", // preserve
        ],
    );
}

#[test]
fn figure_and_figcaption_never_unwrapped() {
    // Confirms the RFC 003 finding directly against the public API rather
    // than by source inspection: <figure>/<figcaption> are excluded from
    // is_wrapper_tag and are additionally listed in is_structural_tag, so
    // they are never unwrapped in any mode, including Minimal and Semantic
    // (unwrap_unknown_wrappers = true in both).
    assert_matrix(
        "figure/figcaption",
        r#"<figure><img src="a.png" alt="A"><figcaption>Cap</figcaption></figure>"#,
        ["![A](a.png)\n\nCap\n"; 5],
    );
}

#[test]
fn attribute_rich_element_is_identical_across_all_modes() {
    // The central finding of RFC 004/005 Slice A: an element carrying every
    // attribute the six preserve_*/drop_presentation_attrs fields could act
    // on produced byte-identical Markdown in all five modes, since none of
    // the six had any effect. Previously asserted ["Hi\n"; 5].
    //
    // RFC 005 Slice B1 changed this for `preserve_ids` specifically: it now
    // emits an anchor for a non-empty `id`, and this fixture's `id="pid"` is
    // non-empty, so modes where `preserve_ids` defaults true (all but
    // Minimal) now include the anchor. The other five fields are still
    // no-ops (now `#[deprecated]`, see characterisation_attributes.rs), so
    // this is no longer "identical across all modes" but "identical except
    // where preserve_ids's own default differs" -- name kept for history.
    assert_matrix(
        "attribute-rich <p>",
        r#"<p id="pid" class="pclass" data-k="v" aria-label="lbl" style="color:red" foo="bar">Hi</p>"#,
        [
            "<a id=\"pid\"></a>\n\nHi\n", // Balanced (preserve_ids: true)
            "<a id=\"pid\"></a>\n\nHi\n", // Strict   (preserve_ids: true)
            "Hi\n",                       // Minimal  (preserve_ids: false)
            "<a id=\"pid\"></a>\n\nHi\n", // Semantic (preserve_ids: true)
            "<a id=\"pid\"></a>\n\nHi\n", // Preserve (preserve_ids: true)
        ],
    );
}
