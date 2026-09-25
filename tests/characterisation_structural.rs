//! RFC 005 Slice A — structural-field characterisation.
//!
//! `drop_interactive_shell` is the one structural `ConversionOptions` field
//! that acts on the output, and this file confirms it against the public API
//! on purpose-built fixtures.
//!
//! The other one, `unwrap_unknown_wrappers`, was inert since RFC 036 §5.2 /
//! slice `036d` and was removed in 3.0. What it stood for is unchanged: `Minimal`
//! unwraps a wrapper element (`<div>`, `<section>`, `<article>`, `<main>`),
//! removing the tag but keeping the paragraph break it stood for, and `Balanced`
//! renders it, and the two write the same bytes. The fixtures below stay because
//! that is a documented behaviour with no option left to guard it: they assert
//! the output in each mode directly. `WRAPPER_HTML` uses bare sibling text nodes
//! because they contribute no block spacing of their own, so a wrapper's
//! separation is the only thing keeping "Before", "inner" and "After" apart -- the
//! shape that used to weld words together before 036d.

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

const MODES: [ConversionMode; 2] = [ConversionMode::Balanced, ConversionMode::Minimal];

const SHELL_HTML: &str = r#"<nav><a href="/">Home</a></nav><main><p>Content</p></main>"#;

/// No surrounding <p>/<div> to contribute independent block spacing - see
/// the module doc comment for why this shape is the one that can tell a
/// wrapper's separation from a neighbour's.
const WRAPPER_HTML: &str = r#"Before<div class="wrap"><span>inner</span></div>After"#;

/// `WRAPPER_HTML` with an `id` on the wrapper and on the span inside it
/// (2.4.1). The original fixture carried none, so it could not see
/// `preserve_ids` being dropped for a wrapper that a mode unwraps -- the one
/// axis on which the since-removed `Semantic` differed from `Balanced`, shipped
/// unnoticed in 2.4.0 because nothing here put an `id` where the difference
/// lived.
const WRAPPER_ID_HTML: &str =
    r#"Before<div class="wrap" id="w"><span id="s">inner</span></div>After"#;

#[test]
fn drop_interactive_shell_toggle_changes_output_in_every_mode() {
    let expected_baseline = [
        "[Home](/)\n\nContent\n", // balanced (default false)
        "Content\n",              // minimal (default true)
    ];
    let expected_flipped = [
        "Content\n",              // balanced, flipped to true
        "[Home](/)\n\nContent\n", // minimal, flipped to false
    ];

    for ((mode, exp_base), exp_flip) in MODES
        .iter()
        .zip(expected_baseline.iter())
        .zip(expected_flipped.iter())
    {
        let mut opts = ConversionOptions::for_mode(*mode);
        let base = conv_with(SHELL_HTML, &opts);
        assert_eq!(&base, exp_base, "{mode} baseline mismatch");

        let default_value = opts.drop_interactive_shell;
        opts.drop_interactive_shell = !default_value;
        let flipped = conv_with(SHELL_HTML, &opts);
        assert_eq!(&flipped, exp_flip, "{mode} flipped mismatch");
        assert_ne!(
            base, flipped,
            "{mode}: drop_interactive_shell toggle unexpectedly changed nothing"
        );
    }
}

/// Every mode reads the same, separated, on the bare-sibling-text fixture.
/// (Until 3.0 this flipped `unwrap_unknown_wrappers` and asserted nothing
/// changed. RFC 036 §5.2 / slice `036d` fixed the defect that once made that
/// flip matter: unwrapping now keeps the separation, so `Minimal`'s unwrapping
/// and `Balanced`'s rendering write the same bytes.)
#[test]
fn a_wrapper_keeps_its_separation_in_both_modes() {
    const SEPARATED: &str = "Before\n\ninner\n\nAfter\n";
    for mode in MODES {
        let got = conv_with(WRAPPER_HTML, &ConversionOptions::for_mode(mode));
        assert_eq!(got, SEPARATED, "{mode}");
    }
}

/// The same claim on the fixture that can falsify it for `preserve_ids`: an
/// unwrapped wrapper keeps its anchor, in the modes that emit anchors.
#[test]
fn a_wrapper_with_ids_keeps_its_anchors_in_the_modes_that_emit_them() {
    for mode in MODES {
        let opts = ConversionOptions::for_mode(mode);
        let got = conv_with(WRAPPER_ID_HTML, &opts);
        let expected = if opts.preserve_ids {
            "Before\n\n<a id=\"w\"></a><a id=\"s\"></a>inner\n\nAfter\n"
        } else {
            "Before\n\ninner\n\nAfter\n"
        };
        assert_eq!(got, expected, "{mode}");
    }
}

/// Div-per-line markup, without a `<p>` inside -- several wrapper elements in
/// a row rather than WRAPPER_HTML's one, and no marker-per-line the way a
/// hand-written fixture would add. This is precisely the shape RFC 036 §2.2
/// named as the trigger (ordinary CMS/SPA output), and the shape that used to
/// weld words together ("First.Second.Third.") in the unwrapping modes before
/// slice `036d`. `Minimal` unwraps and `Balanced` renders; both must separate.
const DIV_PER_LINE_HTML: &str = "<div>First.</div><div>Second.</div><div>Third.</div>";

#[test]
fn balanced_and_minimal_write_the_same_bytes_on_div_per_line_markup() {
    let balanced = conv_with(
        DIV_PER_LINE_HTML,
        &ConversionOptions::for_mode(ConversionMode::Balanced),
    );
    let minimal = conv_with(
        DIV_PER_LINE_HTML,
        &ConversionOptions::for_mode(ConversionMode::Minimal),
    );
    assert_eq!(
        balanced, "First.\n\nSecond.\n\nThird.\n",
        "Balanced baseline"
    );
    assert_eq!(
        balanced, minimal,
        "Balanced vs Minimal on div-per-line markup (RFC 036 §5.2 / 036d)"
    );
}

/// Recorded deliberately: this is the fixture the discovery pass tried
/// first, and it demonstrates why WRAPPER_HTML above had to be different.
/// Surrounding <p> tags each enforce their own blank-line separation, so the
/// wrapper's own begin/end-block calls are redundant either way. This is not a
/// bug; it is a property of how neighbouring block elements interact, worth
/// recording so nobody "fixes" WRAPPER_HTML back to this simpler shape later.
#[test]
fn the_naive_wrapper_fixture_reads_the_same_in_both_modes() {
    let html = r#"<p>Before</p><div class="wrap"><span>inner</span></div><p>After</p>"#;
    for mode in MODES {
        let got = conv_with(html, &ConversionOptions::for_mode(mode));
        assert_eq!(got, "Before\n\ninner\n\nAfter\n", "{mode}");
    }
}

// ─── 2.4.1: fixtures that can actually discriminate ────────────────────────
//
// The two tests above passed while `Semantic` dropped the `id` anchor of every
// wrapper it unwrapped, because neither fixture put an `id` on a wrapper --
// `api/modes` cited them as *chosen to discriminate*, and they discriminated
// on none of the axes where the modes differed. These do, and each shape is
// here because it varies one thing the earlier fixtures held fixed:
//
// * every wrapper tag, not only `div`/`span` (`main`, `section`, `article`);
// * the wrapper alone, empty, and as the only content of a container (a
//   blockquote, a list item, a table cell) -- the anchor's line and prefix;
// * nested wrappers, each with its own id;
// * a wrapper around a block, a list and a heading, versus around bare text;
// * shell elements (`nav`/`header`/`footer`/`aside`) carrying an id, which
//   `drop_interactive_shell` touches in Minimal only;
// * an `id` that needs escaping, and an empty `id` (no anchor);
// * an inline wrapper (`span`) versus a block one, since only the latter
//   separates.

const ID_WRAPPER_SHAPES: &[&str] = &[
    r#"<main id="a"><p>hi</p></main>"#,
    r#"<div id="a"><p>hi</p></div>"#,
    r#"<section id="a"><h2>t</h2><p>x</p></section>"#,
    r#"<article id="a"><p>x</p><p>y</p></article>"#,
    r#"<div id="a"><div id="b"><p>x</p></div></div>"#,
    r#"<div id="a"><span id="b">x</span></div>"#,
    r#"<span id="a">x</span> tail"#,
    r#"before <span id="a"></span> after"#,
    r#"<div id="a"></div>"#,
    r#"<p>x</p><section id="a"></section><p>y</p>"#,
    r#"<blockquote><section id="a"><p>q</p></section></blockquote>"#,
    r#"<ol><li><article id="a"><p>i</p></article></li></ol>"#,
    r#"<ul><li><span id="a">i</span> tail</li></ul>"#,
    r#"<table><tr><th>H</th></tr><tr><td><div id="a">c</div></td></tr></table>"#,
    r#"<table><tr><th>H</th></tr><tr><td><span id="a">c</span></td></tr></table>"#,
    r#"<header id="h"><p>a</p></header><footer id="f"><p>b</p></footer><aside id="s"><p>c</p></aside>"#,
    r#"<div id="a" class="c" data-x="y"><p>x</p></div>"#,
    r#"<div id="">x</div>"#,
    r#"<div id="a &amp; b&quot;q">x</div>"#,
    r#"<h2><span id="a">t</span></h2>"#,
    r#"<div id="a"><ul><li>i</li></ul></div>"#,
    r#"<pre><span id="a">x</span></pre>"#,
];

#[test]
fn wrapper_with_an_id_emits_its_anchor_when_preserve_ids_is_on() {
    let html = r#"<main id="x"><p>hi</p></main>"#;
    let got = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Balanced));
    assert_eq!(got, "<a id=\"x\"></a>\n\nhi\n");
    // The control: Minimal has preserve_ids off, so no anchor -- the rule is
    // fine, its application was not.
    let minimal = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Minimal));
    assert_eq!(minimal, "hi\n");
}

#[test]
fn minimal_emits_no_anchor_for_any_id_wrapper_shape() {
    for html in ID_WRAPPER_SHAPES {
        let got = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Minimal));
        assert!(
            !got.contains("<a id="),
            "Minimal emitted an anchor for {html}: {got:?}"
        );
    }
}

#[test]
fn wrappers_without_an_id_are_unchanged() {
    for html in [
        "<div><p>x</p></div>",
        "<main><p>x</p></main>",
        "<span>a</span><span>b</span>",
        "Before<div><span>inner</span></div>After",
    ] {
        for mode in MODES {
            let got = conv_with(html, &ConversionOptions::for_mode(mode));
            assert!(
                !got.contains("<a id="),
                "{mode} invented an anchor for {html}: {got:?}"
            );
        }
    }
}
