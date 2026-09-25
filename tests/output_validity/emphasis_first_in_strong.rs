//! RFC 044 — emphasis first inside `<strong>` is not lost.
//!
//! `<b><em>q</em>a</b>` was written `**_q_a**`, which parses as
//! `strong("_" "q_a")`: the emphasis is gone and an underscore that was not in
//! the source is left in the text. RFC 037's addendum swaps a *first-child*
//! emphasis to `_` (so `<strong><em>q</em></strong>` does not invert to
//! `em(strong("q"))`), and a closing `_` cannot close against a letter or digit
//! that follows it. The swap is now settled once that following character is
//! known: it reverts to `*` (`***q*a**`, which parses as
//! `strong(em("q") "a")`) and stays `_` everywhere else.
//!
//! One cell per row of RFC 044 §2's table, in all five modes and under both
//! readings. The three broken shapes are the fix; the four working shapes are
//! guards, and are also pinned byte for byte to what `2.6.0` wrote.

use crate::harness::{MODES, mdka_convert, tree};
use mdka::options::ConversionOptions;

cells! {
    // ── §2: the three broken shapes ────────────────────────────────────────
    em_first_then_word_in_b: "<b><em>q</em>a</b>"
        => tree(r#"para(strong(em("q"), "a"))"#);
    i_first_then_word_in_b: "<b><i>q</i>a</b>"
        => tree(r#"para(strong(em("q"), "a"))"#);
    em_first_then_digit_in_strong: "<strong><em>q</em>1</strong>"
        => tree(r#"para(strong(em("q"), "1"))"#);

    // ── §2: the four that already worked — guards ──────────────────────────
    em_first_then_space: "<b><em>q</em> a</b>"
        => tree(r#"para(strong(em("q"), " a"))"#);
    // §4 criterion 2: the inversion that the `_` swap exists to prevent must not return.
    em_only_content_stays_strong_of_em: "<b><em>q</em></b>"
        => tree(r#"para(strong(em("q")))"#);
    em_not_first_child: "<b>a<em>q</em>b</b>"
        => tree(r#"para(strong("a", em("q"), "b"))"#);
    the_other_nesting: "<em><b>q</b>a</em>"
        => tree(r#"para(em(strong("q"), "a"))"#);

    // ── the same fix in the contexts the harness always covers ─────────────
    in_a_sentence: "<p>see <b><em>note</em>worthy</b> here</p>"
        => tree(r#"para("see ", strong(em("note"), "worthy"), " here")"#);
    in_a_list_item: "<ul><li><b><em>q</em>a</b></li></ul>"
        => tree(r#"ul(li(strong(em("q"), "a")))"#);
    in_a_heading: "<h2><b><em>q</em>a</b></h2>"
        => tree(r#"h2(strong(em("q"), "a"))"#);
    a_longer_italic_run: "<b><em>hello world</em>foo</b>"
        => tree(r#"para(strong(em("hello world"), "foo"))"#);
    a_non_ascii_letter_follows: "<b><em>q</em>é</b>"
        => tree(r#"para(strong(em("q"), "é"))"#);
    a_collapsed_bold_after_it: "<b><em>q</em>a<b>z</b></b>"
        => tree(r#"para(strong(em("q"), "az"))"#);
}

fn markdown(html: &str) -> String {
    let first = mdka_convert(html, &ConversionOptions::for_mode(MODES[0]));
    for mode in MODES {
        assert_eq!(
            mdka_convert(html, &ConversionOptions::for_mode(mode)),
            first,
            "mode {mode} differs on {html:?}"
        );
    }
    first
}

/// The four shapes that were correct are byte-identical to what `2.6.0`
/// wrote (measured on the published binary), not merely parse the same.
#[test]
fn the_shapes_that_already_worked_are_byte_identical_to_2_6_0() {
    for (html, written) in [
        ("<b><em>q</em> a</b>", "**_q_ a**\n"),
        ("<b><em>q</em></b>", "**_q_**\n"),
        ("<b>a<em>q</em>b</b>", "**a*q*b**\n"),
        ("<em><b>q</b>a</em>", "***q**a*\n"),
    ] {
        assert_eq!(markdown(html), written, "{html}");
    }
}

#[test]
fn the_three_broken_shapes_now_write_the_forms_the_rfc_verified() {
    assert_eq!(markdown("<b><em>q</em>a</b>"), "***q*a**\n");
    assert_eq!(markdown("<b><i>q</i>a</b>"), "***q*a**\n");
    assert_eq!(markdown("<strong><em>q</em>1</strong>"), "***q*1**\n");
}

/// The revert is narrow, and where it does not apply the output is exactly
/// what it was: an italic run that ends in punctuation (a closing `*` cannot
/// close there either), and a second emphasis inside the same bold (where
/// `***q*a*r***` reads back worse than what it replaces). Both were broken
/// before and are recorded as such, not silently changed.
#[test]
fn shapes_the_revert_does_not_reach_are_unchanged() {
    assert_eq!(markdown("<b><em>q.</em>a</b>"), "**_q._a**\n");
    assert_eq!(markdown("<b><em>q</em>a<em>r</em></b>"), "**_q_a*r***\n");
    // The swap stays wherever no word character follows the italic run.
    assert_eq!(markdown("<b><em>q</em></b>x"), "***q***x\n");
    assert_eq!(markdown("<p>a <b><em>q</em>.</b></p>"), "a **_q_.**\n");
}
