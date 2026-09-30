//! RFC 049 — inline `style` emphasis: one rule, read independently for bold
//! (`font-weight`) and italic (`font-style`), on any element, gated by
//! `emphasis_from_style` (default off) except where it only *removes* a tag's
//! own default emphasis (RFC 028 Amendment 1, unconditional, both before and
//! after this RFC).

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

const MODES: [ConversionMode; 2] = [ConversionMode::Balanced, ConversionMode::Minimal];

fn on(mode: ConversionMode) -> ConversionOptions {
    ConversionOptions::for_mode(mode).emphasis_from_style(true)
}

fn off(mode: ConversionMode) -> ConversionOptions {
    ConversionOptions::for_mode(mode)
}

// ── handoff §2's decided cases, exactly as written (`<span>`/`<div>`) ──────
//
// `span`/`div` are wrapper tags (`utils::is_wrapper_tag`): Minimal unwraps
// them, discarding the tag -- and any `style` on it -- entirely, a pre-RFC
// 049 fact about Minimal mode's own compaction, not something this RFC
// changes or could change (there is no tag left to read a style from once
// unwrapped). So the decided cases, which use `<span>`/`<div>` as the
// standing example of "any element", are pinned in Balanced, where the tag
// survives; the block below them (`the_mechanism_itself_...`) proves the
// same rule holds in Minimal too, using a non-wrapper tag (`<u>`) that
// Minimal does not unwrap, and `wrapper_tags_carry_no_style_once_unwrapped`
// pins the Minimal-specific interaction explicitly, as a fact rather than a
// silent gap.

#[test]
fn inner_declaration_wins_for_its_own_content() {
    // `<b><span style="font-weight:400">x</span></b>` -> `x`
    assert_eq!(
        conv_with(
            r#"<b><span style="font-weight:400">x</span></b>"#,
            &on(ConversionMode::Balanced)
        ),
        "x\n"
    );
}

#[test]
fn un_bolded_then_re_bolded() {
    // `<b style="font-weight:400"><span style="font-weight:700">x</span></b>` -> `**x**`
    assert_eq!(
        conv_with(
            r#"<b style="font-weight:400"><span style="font-weight:700">x</span></b>"#,
            &on(ConversionMode::Balanced)
        ),
        "**x**\n"
    );
}

#[test]
fn emphasis_ends_where_the_declaration_is_overridden() {
    // `<span style="font-weight:700">a<span style="font-weight:400">b</span></span>` -> `**a**b`
    assert_eq!(
        conv_with(
            r#"<span style="font-weight:700">a<span style="font-weight:400">b</span></span>"#,
            &on(ConversionMode::Balanced)
        ),
        "**a**b\n"
    );
}

#[test]
fn one_level_not_two() {
    // `<span style="font-weight:700"><b>x</b></span>` -> `**x**` (RFC 037 collapse)
    for mode in MODES {
        // Holds in both modes regardless of the span/Minimal interaction:
        // `<b>`'s own tag-default bold gives the same answer either way.
        assert_eq!(
            conv_with(
                r#"<span style="font-weight:700"><b>x</b></span>"#,
                &on(mode)
            ),
            "**x**\n",
            "{mode}"
        );
    }
}

#[test]
fn bold_and_italic_combo_matches_the_nested_tag_emission() {
    // `<span style="font-weight:700;font-style:italic">x</span>` -> same as `<b><i>x</i></b>`
    let combo = conv_with(
        r#"<span style="font-weight:700;font-style:italic">x</span>"#,
        &on(ConversionMode::Balanced),
    );
    let nested = conv_with(r#"<b><i>x</i></b>"#, &on(ConversionMode::Balanced));
    assert_eq!(combo, nested);
    assert_eq!(combo, "**_x_**\n");
}

#[test]
fn todays_negation_preserved() {
    // `<em style="font-style:normal">x</em>` -> `x`, option on or off, every
    // mode -- `em` is not a wrapper tag, so this holds everywhere.
    for mode in MODES {
        for opts in [on(mode), off(mode)] {
            assert_eq!(
                conv_with(r#"<em style="font-style:normal">x</em>"#, &opts),
                "x\n",
                "{mode}"
            );
        }
    }
}

#[test]
fn a_style_on_a_container_reaches_every_descendant_block() {
    // `<div style="font-weight:700"><p>a</p><p>b</p></div>` -> `**a**` / `**b**`
    assert_eq!(
        conv_with(
            r#"<div style="font-weight:700"><p>a</p><p>b</p></div>"#,
            &on(ConversionMode::Balanced)
        ),
        "**a**\n\n**b**\n"
    );
}

// ── the same mechanism, every mode, via a tag Minimal never unwraps ────────

#[test]
fn the_mechanism_itself_holds_in_every_mode() {
    for mode in MODES {
        // The negation-through-a-nested-non-default-element case, row 1's
        // point, restated with `<u>` (not a wrapper tag) instead of `<span>`.
        assert_eq!(
            conv_with(r#"<b><u style="font-weight:400">x</u></b>"#, &on(mode)),
            "x\n",
            "{mode}"
        );
        // The addition case, row 3's point: any non-default element gains
        // emphasis from its own style, independent of Minimal's unwrapping.
        assert_eq!(
            conv_with(r#"<u style="font-weight:700">x</u>"#, &on(mode)),
            "**x**\n",
            "{mode}"
        );
        // A container (block-kind, no tag default either) reaching a leaf
        // block descendant: `<blockquote>` is not a wrapper tag.
        assert_eq!(
            conv_with(
                r#"<blockquote style="font-weight:700"><p>a</p></blockquote>"#,
                &on(mode)
            ),
            "> **a**\n",
            "{mode}"
        );
    }
}

#[test]
fn wrapper_tags_carry_no_style_once_unwrapped_in_minimal() {
    // Pre-RFC 049, pre-Minimal-even-having-this-option fact, pinned rather
    // than left as a silent gap: `span`/`div`/`section`/`article`/`main`
    // (`utils::is_wrapper_tag`) lose their tag, and with it any `style`,
    // when Minimal unwraps them -- there is nothing left to read a
    // declaration from. `emphasis_from_style` cannot reach past that; no
    // implementation of "read the element" can, once the element is gone.
    let opts = on(ConversionMode::Minimal);
    assert_eq!(
        conv_with(r#"<b><span style="font-weight:400">x</span></b>"#, &opts),
        "**x**\n",
        "the negating span is unwrapped, so <b>'s own default bold survives"
    );
    assert_eq!(
        conv_with(r#"<span style="font-weight:700">x</span>"#, &opts),
        "x\n",
        "the declaring span is unwrapped before its style can be read"
    );
    assert_eq!(
        conv_with(r#"<div style="font-weight:700"><p>a</p></div>"#, &opts),
        "a\n",
        "the declaring div is unwrapped before its style can be read"
    );
}

// ── the option is off by default, and off means byte-identical to 3.0.0 ───

#[test]
fn off_by_default_a_span_gains_no_emphasis() {
    for mode in MODES {
        assert_eq!(
            conv_with(r#"<span style="font-weight:700">x</span>"#, &off(mode)),
            "x\n",
            "{mode}"
        );
        assert_eq!(
            conv_with(r#"<div style="font-weight:700"><p>a</p></div>"#, &off(mode)),
            "a\n",
            "{mode}"
        );
    }
}

#[test]
fn negation_path_passes_with_the_option_on_and_off() {
    for mode in MODES {
        for opts in [on(mode), off(mode)] {
            assert_eq!(
                conv_with(r#"<b style="font-weight:400">x</b>"#, &opts),
                "x\n",
                "{mode}"
            );
            assert_eq!(
                conv_with(r#"<strong style="font-weight:normal">x</strong>"#, &opts),
                "x\n",
                "{mode}"
            );
            assert_eq!(
                conv_with(r#"<i style="font-style:normal">x</i>"#, &opts),
                "x\n",
                "{mode}"
            );
        }
    }
}

// ── the value grammar (handoff criterion 3 / RFC 049 §6.4) ────────────────

#[test]
fn font_weight_value_grammar_adds_bold_on_a_non_default_tag() {
    for (value, bold) in [
        ("bold", true),
        ("700", true),
        ("900", true),
        ("normal", false),
        ("400", false),
        ("500", false),
        // relative to the parent's own computed weight, which mdka does not
        // track -- unresolved, so a `<span>` (no tag default) stays plain.
        ("bolder", false),
        ("lighter", false),
        // absent / malformed
        ("not-a-number", false),
    ] {
        let html = format!(r#"<span style="font-weight:{value}">x</span>"#);
        let expected = if bold { "**x**\n" } else { "x\n" };
        assert_eq!(
            conv_with(&html, &on(ConversionMode::Balanced)),
            expected,
            "{value}"
        );
    }
    assert_eq!(
        conv_with("<span>x</span>", &on(ConversionMode::Balanced)),
        "x\n",
        "no style attribute at all"
    );
}

#[test]
fn font_style_value_grammar_adds_italic_on_a_non_default_tag() {
    for (value, italic) in [
        ("italic", true),
        ("oblique", true),
        ("normal", false),
        ("not-a-value", false),
    ] {
        let html = format!(r#"<span style="font-style:{value}">x</span>"#);
        let expected = if italic { "*x*\n" } else { "x\n" };
        assert_eq!(
            conv_with(&html, &on(ConversionMode::Balanced)),
            expected,
            "{value}"
        );
    }
}

#[test]
fn a_declaration_among_others_is_still_read() {
    assert_eq!(
        conv_with(
            r#"<span style="color:red;font-weight:700;text-decoration:underline">x</span>"#,
            &on(ConversionMode::Balanced)
        ),
        "**x**\n"
    );
}

// ── the option can affect output (RFC 049 §6.6 / handoff §3) ──────────────

#[test]
fn the_option_can_affect_output() {
    let html = r#"<span style="font-weight:700">x</span>"#;
    assert_ne!(
        conv_with(html, &off(ConversionMode::Balanced)),
        conv_with(html, &on(ConversionMode::Balanced))
    );
}

// ── `class` is never read, on or off (RFC 049 §2) ─────────────────────────

#[test]
fn class_never_adds_emphasis_even_with_the_option_on() {
    for mode in MODES {
        assert_eq!(
            conv_with(r#"<span class="bold-ish">x</span>"#, &on(mode)),
            "x\n",
            "{mode}"
        );
    }
}
