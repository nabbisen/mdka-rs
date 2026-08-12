//! RFC 005 Slice A — structural-field characterisation.
//!
//! `drop_interactive_shell` and `unwrap_unknown_wrappers` are the two
//! `ConversionOptions` fields already known to be live (RFC 004's harvested
//! inventory, RFC 003's review). This file re-confirms both against the
//! public API on purpose-built fixtures, and records one thing the initial
//! discovery pass got wrong before correcting it: a naive wrapper fixture
//! (`<div><span>inner</span></div>` alone, or surrounded by `<p>` siblings)
//! shows *no* difference when `unwrap_unknown_wrappers` is toggled, because
//! neighbouring block elements' own blank-line spacing already dominates the
//! output either way. The fixture below uses bare sibling text nodes, which
//! do not contribute their own block spacing, so the wrapper's own
//! begin/end-block calls are the only source of separation - only then does
//! toggling the flag actually change anything observable.

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

const SHELL_HTML: &str = r#"<nav><a href="/">Home</a></nav><main><p>Content</p></main>"#;

/// No surrounding <p>/<div> to contribute independent block spacing - see
/// the module doc comment for why this shape is required to discriminate
/// unwrap_unknown_wrappers at all.
const WRAPPER_HTML: &str = r#"Before<div class="wrap"><span>inner</span></div>After"#;

#[test]
fn drop_interactive_shell_toggle_changes_output_in_every_mode() {
    let expected_baseline = [
        "[Home](/)\n\nContent\n", // balanced (default false)
        "[Home](/)\n\nContent\n", // strict (default false)
        "Content\n",              // minimal (default true)
        "[Home](/)\n\nContent\n", // semantic (default false)
        "[Home](/)\n\nContent\n", // preserve (default false)
    ];
    let expected_flipped = [
        "Content\n",              // balanced, flipped to true
        "Content\n",              // strict, flipped to true
        "[Home](/)\n\nContent\n", // minimal, flipped to false
        "Content\n",              // semantic, flipped to true
        "Content\n",              // preserve, flipped to true
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

#[test]
fn unwrap_unknown_wrappers_toggle_changes_output_in_every_mode() {
    let expected_baseline = [
        "Before\n\ninner\n\nAfter\n", // balanced (default false: div kept, own block spacing)
        "Before\n\ninner\n\nAfter\n", // strict (default false)
        "BeforeinnerAfter\n",         // minimal (default true: div unwrapped, no spacing at all)
        "BeforeinnerAfter\n",         // semantic (default true)
        "Before\n\ninner\n\nAfter\n", // preserve (default false)
    ];
    let expected_flipped = [
        "BeforeinnerAfter\n",         // balanced, flipped to true
        "BeforeinnerAfter\n",         // strict, flipped to true
        "Before\n\ninner\n\nAfter\n", // minimal, flipped to false
        "Before\n\ninner\n\nAfter\n", // semantic, flipped to false
        "BeforeinnerAfter\n",         // preserve, flipped to true
    ];

    for ((mode, exp_base), exp_flip) in MODES
        .iter()
        .zip(expected_baseline.iter())
        .zip(expected_flipped.iter())
    {
        let mut opts = ConversionOptions::for_mode(*mode);
        let base = conv_with(WRAPPER_HTML, &opts);
        assert_eq!(&base, exp_base, "{mode} baseline mismatch");

        let default_value = opts.unwrap_unknown_wrappers;
        opts.unwrap_unknown_wrappers = !default_value;
        let flipped = conv_with(WRAPPER_HTML, &opts);
        assert_eq!(&flipped, exp_flip, "{mode} flipped mismatch");
        assert_ne!(
            base, flipped,
            "{mode}: unwrap_unknown_wrappers toggle unexpectedly changed nothing"
        );
    }
}

#[test]
fn unwrap_unknown_wrappers_naive_fixture_shows_no_difference() {
    // Recorded deliberately: this is the fixture the discovery pass tried
    // first, and it demonstrates why WRAPPER_HTML above had to be different.
    // Surrounding <p> tags each enforce their own blank-line separation, so
    // the wrapper's own begin/end-block calls are redundant either way and
    // toggling the flag is invisible. This is not a bug; it is a property of
    // how neighbouring block elements interact, worth recording so nobody
    // "fixes" WRAPPER_HTML back to this simpler shape later.
    let html = r#"<p>Before</p><div class="wrap"><span>inner</span></div><p>After</p>"#;
    for mode in MODES {
        let mut opts = ConversionOptions::for_mode(mode);
        let base = conv_with(html, &opts);
        let default_value = opts.unwrap_unknown_wrappers;
        opts.unwrap_unknown_wrappers = !default_value;
        let flipped = conv_with(html, &opts);
        assert_eq!(
            base, flipped,
            "{mode}: expected the naive fixture to show no difference"
        );
        assert_eq!(base, "Before\n\ninner\n\nAfter\n", "{mode} baseline value");
    }
}

// ─── RFC 006: the three-mode identity claim, proven directly ───────────────
//
// docs/src/api/modes.md states that Balanced, Strict and Preserve currently
// produce identical output. That follows from the mode defaults table: the
// three share preserve_ids/drop_interactive_shell/unwrap_unknown_wrappers
// (the only fields with any effect) and differ only in the five deprecated
// no-op fields. This test proves it directly, on the fixtures most likely to
// discriminate a difference if one existed — the bare-sibling-text wrapper
// fixture (the one shape that discriminates unwrap_unknown_wrappers at all,
// per the module doc comment above) and an attribute-rich element (the one
// shape that discriminates the five deprecated fields, per
// characterisation_attributes.rs) — rather than leaving the claim to be
// inferred from scattered assert_matrix arrays elsewhere in the suite.

#[test]
fn balanced_strict_preserve_are_identical_on_the_wrapper_fixture() {
    let balanced = conv_with(
        WRAPPER_HTML,
        &ConversionOptions::for_mode(ConversionMode::Balanced),
    );
    let strict = conv_with(
        WRAPPER_HTML,
        &ConversionOptions::for_mode(ConversionMode::Strict),
    );
    let preserve = conv_with(
        WRAPPER_HTML,
        &ConversionOptions::for_mode(ConversionMode::Preserve),
    );
    assert_eq!(balanced, strict, "Balanced vs Strict on WRAPPER_HTML");
    assert_eq!(balanced, preserve, "Balanced vs Preserve on WRAPPER_HTML");
}

#[test]
fn balanced_strict_preserve_are_identical_on_an_attribute_rich_element() {
    let html = r#"<p id="pid" class="pclass" data-k="v" aria-label="lbl" style="color:red" foo="bar">Hi</p>"#;
    let balanced = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Balanced));
    let strict = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Strict));
    let preserve = conv_with(html, &ConversionOptions::for_mode(ConversionMode::Preserve));
    assert_eq!(
        balanced, strict,
        "Balanced vs Strict on an attribute-rich element"
    );
    assert_eq!(
        balanced, preserve,
        "Balanced vs Preserve on an attribute-rich element"
    );
}
