//! RFC 043 — `<sup>` and `<sub>` that silently changed the meaning.
//!
//! `2<sup>n − 1</sup>` converted to `2n − 1`, which reads as "two times n
//! minus one"; `10<sup>−9</sup>` to `10−9`, "ten minus nine". Nothing in the
//! output said a value had been raised. Three tiers, in this order:
//!
//! 1. **The map** (§2): a real Unicode superscript/subscript wherever every
//!    character has one -- U+2212 counts as `-`, the lowercase letters
//!    Unicode provides are mapped, and emphasis-only content
//!    (`<sup><i>n</i></sup>`) is seen through.
//! 2. **Self-delimiting text** (§3): `[1]` and `(a b)` already say they are
//!    markers, and are left exactly as they were.
//! 3. **The marker** (§3), for what remains: `^(…)` and `_(…)`, always
//!    parenthesised.
//!
//! Like `elements::properties_exempt`, these are checked directly against
//! `structure()`, not through `properties()`: that check assumes the output's
//! words are the HTML's own, and a mapped superscript or an added marker
//! differs from the source on purpose. Every case runs in **all five modes**,
//! under **both readings** -- CommonMark and GFM -- since `_` is a CommonMark
//! character and the escaping differs by context.

use crate::harness::{MODES, Reading, mdka_convert, structure};
use mdka::options::ConversionOptions;

/// Asserts the structure under both readings, in every mode.
fn check(html: &str, commonmark: &str, gfm: &str) {
    for mode in MODES {
        let md = mdka_convert(html, &ConversionOptions::for_mode(mode));
        for (reading, want) in [(Reading::CommonMark, commonmark), (Reading::Gfm, gfm)] {
            let got = structure(&md, reading, true);
            assert_eq!(got, want, "mode {mode}, ({reading}) markdown: {md:?}");
        }
    }
}

/// The same structure under both readings.
fn check_both(html: &str, want: &str) {
    check(html, want, want);
}

/// The exact bytes, identical in every mode.
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

// ─── §2: the map ───────────────────────────────────────────────────────────

#[test]
fn u2212_minus_sign_maps_like_a_hyphen() {
    check_both("10<sup>−9</sup>", r#"para("10⁻⁹")"#);
    check_both("a<sub>n−1</sub>", r#"para("aₙ₋₁")"#);
    // The ASCII hyphen still does, unchanged.
    check_both("10<sup>-9</sup>", r#"para("10⁻⁹")"#);
}

#[test]
fn emphasis_only_content_is_seen_through() {
    for inner in [
        "<i>n</i>",
        "<em>n</em>",
        "<b>n</b>",
        "<strong>n</strong>",
        "<var>n</var>",
        "<b><i>n</i></b>",
        "<span><em>n</em></span>",
    ] {
        check_both(&format!("x<sup>{inner}</sup>"), r#"para("xⁿ")"#);
    }
    // Text beside the emphasis is part of the same content.
    check_both("x<sup><i>n</i>+1</sup>", r#"para("xⁿ⁺¹")"#);
    check_both("x<sub><i>i</i></sub>", r#"para("xᵢ")"#);
}

#[test]
fn letters_added_to_the_superscript_map() {
    check_both("x<sup>y</sup>", r#"para("xʸ")"#);
    check_both(
        "x<sup>abcdefghijklmnoprstuvwxyz</sup>",
        r#"para("xᵃᵇᶜᵈᵉᶠᵍʰⁱʲᵏˡᵐⁿᵒᵖʳˢᵗᵘᵛʷˣʸᶻ")"#,
    );
}

#[test]
fn letters_added_to_the_subscript_map() {
    check_both("x<sub>i</sub>", r#"para("xᵢ")"#);
    check_both("x<sub>max</sub>", r#"para("xₘₐₓ")"#);
    check_both(
        "x<sub>aehijklmnoprstuvx</sub>",
        r#"para("xₐₑₕᵢⱼₖₗₘₙₒₚᵣₛₜᵤᵥₓ")"#,
    );
}

/// Criterion 6: symmetric wherever Unicode allows, and the asymmetry is exactly
/// the letters Unicode lacks -- 25 of 26 for `<sup>` (no `q`), 17 for `<sub>`
/// (no `b c d f g q w y z`). Checked letter by letter so a table typo in either
/// direction names the letter.
#[test]
fn superscript_and_subscript_agree_wherever_unicode_allows() {
    let no_superscript = "q";
    let no_subscript = "bcdfgqwyz";
    for letter in 'a'..='z' {
        let sup = markdown(&format!("x<sup>{letter}</sup>"));
        let sub = markdown(&format!("x<sub>{letter}</sub>"));
        let sup_maps = !sup.contains("^(");
        let sub_maps = !sub.contains("_(");
        assert_eq!(
            sup_maps,
            !no_superscript.contains(letter),
            "<sup>{letter}</sup> -> {sup:?}"
        );
        assert_eq!(
            sub_maps,
            !no_subscript.contains(letter),
            "<sub>{letter}</sub> -> {sub:?}"
        );
    }
    assert_eq!(
        (b'a'..=b'z')
            .filter(|l| !no_superscript.contains(*l as char))
            .count(),
        25
    );
    assert_eq!(
        (b'a'..=b'z')
            .filter(|l| !no_subscript.contains(*l as char))
            .count(),
        17
    );
}

#[test]
fn every_mapped_character_is_the_letter_it_stands_for() {
    // A typo in the tables is silent otherwise: NFKD folds each superscript
    // and subscript letter back to its plain one.
    for letter in ('a'..='z').chain('0'..='9') {
        for tag in ["sup", "sub"] {
            let md = markdown(&format!("<{tag}>{letter}</{tag}>"));
            let out = md.trim_end();
            if out.contains("^(") || out.contains("_(") {
                continue; // no Unicode form: the marker fallback, tested below
            }
            let folded: String = nfkd_fold(out);
            assert_eq!(
                folded,
                letter.to_string(),
                "<{tag}>{letter}</{tag}> -> {out:?}"
            );
        }
    }
}

/// Compatibility folding for the few characters used, without a dependency:
/// each superscript/subscript form's plain letter or digit.
fn nfkd_fold(s: &str) -> String {
    const PAIRS: &[(char, char)] = &[
        ('⁰', '0'),
        ('¹', '1'),
        ('²', '2'),
        ('³', '3'),
        ('⁴', '4'),
        ('⁵', '5'),
        ('⁶', '6'),
        ('⁷', '7'),
        ('⁸', '8'),
        ('⁹', '9'),
        ('₀', '0'),
        ('₁', '1'),
        ('₂', '2'),
        ('₃', '3'),
        ('₄', '4'),
        ('₅', '5'),
        ('₆', '6'),
        ('₇', '7'),
        ('₈', '8'),
        ('₉', '9'),
        ('ᵃ', 'a'),
        ('ᵇ', 'b'),
        ('ᶜ', 'c'),
        ('ᵈ', 'd'),
        ('ᵉ', 'e'),
        ('ᶠ', 'f'),
        ('ᵍ', 'g'),
        ('ʰ', 'h'),
        ('ⁱ', 'i'),
        ('ʲ', 'j'),
        ('ᵏ', 'k'),
        ('ˡ', 'l'),
        ('ᵐ', 'm'),
        ('ⁿ', 'n'),
        ('ᵒ', 'o'),
        ('ᵖ', 'p'),
        ('ʳ', 'r'),
        ('ˢ', 's'),
        ('ᵗ', 't'),
        ('ᵘ', 'u'),
        ('ᵛ', 'v'),
        ('ʷ', 'w'),
        ('ˣ', 'x'),
        ('ʸ', 'y'),
        ('ᶻ', 'z'),
        ('ₐ', 'a'),
        ('ₑ', 'e'),
        ('ₕ', 'h'),
        ('ᵢ', 'i'),
        ('ⱼ', 'j'),
        ('ₖ', 'k'),
        ('ₗ', 'l'),
        ('ₘ', 'm'),
        ('ₙ', 'n'),
        ('ₒ', 'o'),
        ('ₚ', 'p'),
        ('ᵣ', 'r'),
        ('ₛ', 's'),
        ('ₜ', 't'),
        ('ᵤ', 'u'),
        ('ᵥ', 'v'),
        ('ₓ', 'x'),
    ];
    s.chars()
        .map(|c| {
            PAIRS
                .iter()
                .find(|(from, _)| *from == c)
                .map_or(c, |(_, to)| *to)
        })
        .collect()
}

// ─── The all-or-nothing rule survives ──────────────────────────────────────

#[test]
fn a_run_with_one_unmappable_character_is_not_half_mapped() {
    // Uppercase mostly has no form, so `N` falls back to the marker rather
    // than produce `xⁿ` beside a plain `N`.
    check_both("x<sup>N</sup>", r#"para("x^(N)")"#);
    check_both("x<sup>2N</sup>", r#"para("x^(2N)")"#);
    // A slash, a space, a comma: none has a form either.
    check_both("x<sup>1/2</sup>", r#"para("x^(1/2)")"#);
    check_both("2<sup>a b</sup>", r#"para("2^(a b)")"#);
    check_both("x<sup>q</sup>", r#"para("x^(q)")"#);
}

// ─── §3: the marker ────────────────────────────────────────────────────────

/// Criterion 3: parsed, not read -- base and marker are one text run.
#[test]
fn superscript_marker_parses_as_one_text_run() {
    check_both("2<sup>n − 1</sup>", r#"para("2^(n − 1)")"#);
    assert_eq!(markdown("2<sup>n − 1</sup>"), "2^(n − 1)\n");
}

#[test]
fn subscript_marker_parses_as_text() {
    check_both("x<sub>y</sub>", r#"para("x_(y)")"#);
    assert_eq!(markdown("x<sub>y</sub>"), "x_(y)\n");
}

/// Criterion 4: `_` is a CommonMark character and the escaping differs by
/// context, so the marker is parsed in all four -- a paragraph, a table cell,
/// a list item and a heading -- and read as text every time.
#[test]
fn subscript_marker_parses_as_text_in_a_paragraph() {
    check_both(
        "<p>base x<sub>y</sub> and z<sub>b</sub> end</p>",
        r#"para("base x_(y) and z_(b) end")"#,
    );
}

#[test]
fn subscript_marker_parses_as_text_in_a_table_cell() {
    check(
        "<table><tr><th>A</th></tr><tr><td>x<sub>y</sub></td></tr></table>",
        r#"para("| A | | --- | | x_(y) |")"#,
        r#"table(thead(td("A")), tr(td("x_(y)")))"#,
    );
    check(
        "<table><tr><th>A</th></tr><tr><td>2<sup>n − 1</sup></td></tr></table>",
        r#"para("| A | | --- | | 2^(n − 1) |")"#,
        r#"table(thead(td("A")), tr(td("2^(n − 1)")))"#,
    );
}

#[test]
fn subscript_marker_parses_as_text_in_a_list_item() {
    check_both("<ul><li>x<sub>y</sub></li></ul>", r#"ul(li("x_(y)"))"#);
    check_both(
        "<ol><li>2<sup>n − 1</sup></li></ol>",
        r#"ol[1](li("2^(n − 1)"))"#,
    );
}

#[test]
fn subscript_marker_parses_as_text_in_a_heading() {
    check_both("<h2>x<sub>y</sub></h2>", r#"h2("x_(y)")"#);
    check_both("<h2>2<sup>n − 1</sup></h2>", r#"h2("2^(n − 1)")"#);
}

#[test]
fn subscript_marker_parses_as_text_in_a_blockquote_and_a_link() {
    check_both(
        "<blockquote>x<sub>y</sub></blockquote>",
        r#"quote(para("x_(y)"))"#,
    );
    check_both(
        r#"<a href="/u">x<sub>y</sub></a>"#,
        r#"para(link[/u]("x_(y)"))"#,
    );
}

/// The one way an unescaped `_` could go wrong: with no word to its left it can
/// open emphasis, and a later marker would then close it. Escaped there, and
/// the pair `_(y) b_` is text, not an `<em>`.
#[test]
fn subscript_marker_with_no_word_before_it_cannot_open_emphasis() {
    check_both("a <sub>y</sub> b<sub>z</sub>", r#"para("a _(y) b_(z)")"#);
    assert_eq!(markdown("a <sub>y</sub> b<sub>z</sub>"), "a \\_(y) b_(z)\n");
    // Start of the block, and after punctuation.
    check_both(
        "<p><sub>y</sub> and z<sub>b</sub></p>",
        r#"para("_(y) and z_(b)")"#,
    );
    check_both(
        "<p>(<sub>y</sub>) and z<sub>b</sub></p>",
        r#"para("(_(y)) and z_(b)")"#,
    );
}

/// Inside emphasis the marker could close an enclosing `_` delimiter (which
/// delimiter an emphasis span writes is settled late, RFC 037 addendum), so it
/// is escaped there too, and the emphasis survives whole.
#[test]
fn subscript_marker_inside_emphasis_does_not_close_it() {
    check_both(
        "<b><i>x<sub>y</sub></i></b>",
        r#"para(strong(em("x_(y)")))"#,
    );
    check_both("<em>x<sub>y</sub> z</em>", r#"para(em("x_(y) z"))"#);
}

/// Found by fuzzing, and not caused by this RFC: an emphasis span whose closing
/// `_` cannot close leaves a stray `_` opener in the paragraph, and an
/// unescaped subscript marker glued to a word could then *close* it, eating the
/// marker and italicising what lay between. The marker is escaped whenever an
/// unescaped `_` earlier in the paragraph could still be waiting.
///
/// The original trigger, `<b><em>q</em>a</b>` (written `**_q_a**`), is fixed by
/// RFC 044; `<b><em>q.</em>a</b>` -- an italic run ending in punctuation, which
/// no delimiter choice can close against a letter -- still writes one, and is
/// the input here.
#[test]
fn subscript_marker_cannot_close_a_stray_underscore_opener() {
    let html = "<b><em>q.</em>a</b>snake_case<sub>1/2</sub>x";
    assert_eq!(markdown(html), "**_q._a**snake_case\\_(1/2)x\n");
    for mode in MODES {
        let md = mdka_convert(html, &ConversionOptions::for_mode(mode));
        let tree = structure(&md, Reading::CommonMark, true);
        // The parsed text keeps the marker: no `em` swallowed its `_`.
        let text: String = tree
            .split('"')
            .enumerate()
            .filter_map(|(i, part)| (i % 2 == 1).then_some(part))
            .collect();
        assert!(
            text.contains("_(1/2)x"),
            "mode {mode}: {md:?} parses as {tree}"
        );
    }
}

#[test]
fn emphasis_survives_when_the_fallback_fires() {
    // §2.3 strips emphasis only when mapping succeeds.
    check_both(
        "x<sup><i>n</i> + N</sup>",
        r#"para("x^(", em("n"), " + N)")"#,
    );
    assert_eq!(markdown("x<sup><i>n</i> + N</sup>"), "x^(*n* + N)\n");
}

// ─── §3: self-delimiting text is left exactly as it was ────────────────────

/// The single most important rule: 44% of real occurrences are citation
/// markers and must not change by a byte.
#[test]
fn bracketed_citation_markers_are_byte_identical() {
    assert_eq!(
        markdown(r##"<sup><a href="#c1">[1]</a></sup>"##),
        "[\\[1\\]](#c1)\n"
    );
    check_both(
        r##"a<sup><a href="#c1">[1]</a></sup>"##,
        r#"para("a", link[#c1]("[1]"))"#,
    );
    assert_eq!(markdown("a<sup>[note 2]</sup>"), "a\\[note 2]\n");
    assert_eq!(markdown("a<sup>[N]</sup>"), "a\\[N]\n");
}

#[test]
fn parenthesised_text_that_cannot_map_is_left_alone() {
    // `(N)` cannot map (uppercase) and closes itself: no second pair of
    // parentheses.
    check_both("a<sup>(N)</sup>", r#"para("a(N)")"#);
    assert_eq!(markdown("a<sup>(N)</sup>"), "a(N)\n");
    // A parenthesised run that does map still maps, as before.
    assert_eq!(markdown("a<sup>(1)</sup>"), "a⁽¹⁾\n");
    assert_eq!(markdown("a<sup>(n+1)</sup>"), "a⁽ⁿ⁺¹⁾\n");
}

// ─── Never inside code ─────────────────────────────────────────────────────

#[test]
fn no_marker_and_no_mapping_inside_code() {
    for html in [
        "<pre>a<sup>N</sup>b<sub>y</sub></pre>",
        "<pre><code>a<sup>N</sup>b<sub>y</sub></code></pre>",
    ] {
        let md = markdown(html);
        assert!(!md.contains("^(") && !md.contains("_("), "{html} -> {md:?}");
        assert!(md.contains("aNby"), "{html} -> {md:?}");
    }
    let md = markdown("<p><code>a<sup>N</sup>b<sub>y</sub></code></p>");
    assert_eq!(md, "`aNby`\n");
    // A table cell's own flattened <pre> is the same rule in another
    // destination.
    let md =
        markdown("<table><tr><th>A</th></tr><tr><td><pre>a<sup>N</sup>b</pre></td></tr></table>");
    assert!(!md.contains("^("), "{md:?}");
}

// ─── Nothing else moved ────────────────────────────────────────────────────

#[test]
fn shapes_that_already_mapped_are_unchanged() {
    assert_eq!(markdown("2<sup>7</sup>"), "2⁷\n");
    assert_eq!(markdown("H<sub>2</sub>O"), "H₂O\n");
    assert_eq!(markdown("x<sup>n</sup> and y<sup>i</sup>"), "xⁿ and yⁱ\n");
    assert_eq!(markdown("a<sup></sup>b"), "ab\n");
}

#[test]
fn mixed_or_structural_content_takes_the_marker() {
    // A link that is not bracketed, an image and inline code are not
    // emphasis-only, and none maps.
    assert_eq!(
        markdown(r#"a<sup><a href="/u">note</a></sup>"#),
        "a^([note](/u))\n"
    );
    assert_eq!(markdown("a<sup><code>x</code></sup>"), "a^(`x`)\n");
}
