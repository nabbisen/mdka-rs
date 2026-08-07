//! RFC 005 Slice A — attribute-field characterisation.
//!
//! For each of the six attribute-related `ConversionOptions` fields
//! (`preserve_ids`, `preserve_classes`, `preserve_data_attrs`,
//! `preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs`),
//! flips it from each mode's own default and compares output against that
//! mode's baseline, on an element carrying every attribute kind these fields
//! could plausibly act on. Every comparison below was captured by actually
//! running `html_to_markdown_with` with the field flipped, not inferred.
//!
//! Finding, confirmed here rather than assumed from `src/`: flipping any of
//! these six fields, in any mode, changes nothing. All 30 (6 fields x 5
//! modes) toggle comparisons are identical to their mode's baseline. See the
//! review request for the full count reconciliation against
//! `characterisation_elements.rs`'s independent confirmation of the same fact.

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

/// Carries id, class, data-*, aria-*, style, and an unknown attribute, so
/// every attribute-related field has something to act on, per the handoff's
/// §5 "Attributes" requirement.
const ATTR_HTML: &str =
    r#"<p id="pid" class="pclass" data-k="v" aria-label="lbl" style="color:red" foo="bar">Hi</p>"#;

/// Every mode's baseline on `ATTR_HTML` is "Hi\n" (see
/// characterisation_elements.rs::attribute_rich_element_is_identical_across_all_modes).
/// Re-derived here per mode rather than hard-coded, so this file does not
/// silently depend on that other file's assertion still holding.
fn baseline(mode: ConversionMode) -> String {
    conv_with(ATTR_HTML, &ConversionOptions::for_mode(mode))
}

/// Flips `field` from its mode default and asserts the output versus that
/// mode's own baseline. `field` is applied via a closure so each of the six
/// fields can share this one assertion body.
fn assert_toggle_identical(field_name: &str, set_field: impl Fn(&mut ConversionOptions, bool)) {
    for mode in MODES {
        let base = baseline(mode);
        let mut opts = ConversionOptions::for_mode(mode);
        let default_value = field_name_default(field_name, &opts);
        set_field(&mut opts, !default_value);
        let flipped = conv_with(ATTR_HTML, &opts);
        assert_eq!(
            flipped, base,
            "{field_name} in {mode}: expected flipping to change nothing, but output differed"
        );
    }
}

// Small helper so assert_toggle_identical can report the default value it
// flipped from without each call site having to know it.
fn field_name_default(field_name: &str, opts: &ConversionOptions) -> bool {
    match field_name {
        "preserve_ids" => opts.preserve_ids,
        "preserve_classes" => opts.preserve_classes,
        "preserve_data_attrs" => opts.preserve_data_attrs,
        "preserve_aria_attrs" => opts.preserve_aria_attrs,
        "preserve_unknown_attrs" => opts.preserve_unknown_attrs,
        "drop_presentation_attrs" => opts.drop_presentation_attrs,
        other => panic!("unknown field in test: {other}"),
    }
}

#[test]
fn preserve_ids_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("preserve_ids", |o, v| o.preserve_ids = v);
}

#[test]
fn preserve_classes_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("preserve_classes", |o, v| o.preserve_classes = v);
}

#[test]
fn preserve_data_attrs_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("preserve_data_attrs", |o, v| o.preserve_data_attrs = v);
}

#[test]
fn preserve_aria_attrs_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("preserve_aria_attrs", |o, v| o.preserve_aria_attrs = v);
}

#[test]
fn preserve_unknown_attrs_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("preserve_unknown_attrs", |o, v| {
        o.preserve_unknown_attrs = v
    });
}

#[test]
fn drop_presentation_attrs_toggle_changes_nothing_in_any_mode() {
    assert_toggle_identical("drop_presentation_attrs", |o, v| {
        o.drop_presentation_attrs = v
    });
}
