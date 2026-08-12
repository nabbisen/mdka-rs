//! RFC 005 Slice A/B/C — attribute-field characterisation.
//!
//! For each of the six attribute-related `ConversionOptions` fields
//! (`preserve_ids`, `preserve_classes`, `preserve_data_attrs`,
//! `preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs`),
//! flips it from each mode's own default and compares output against that
//! mode's baseline, on an element carrying every attribute kind these fields
//! could plausibly act on. Every comparison below was captured by actually
//! running `html_to_markdown_with` with the field flipped, not inferred.
//!
//! Slice A finding: flipping any of the six fields, in any mode, changed
//! nothing. **Slice B1 changed that for one field**: `preserve_ids` now
//! emits an anchor for a non-empty `id`, so toggling it changes output in
//! every mode (see `preserve_ids_toggle_changes_output_in_every_mode` below).
//! The other five remain no-ops and are now `#[deprecated]` (RFC 005 Slice
//! B2) rather than removed — this file deliberately keeps reading and
//! writing them to prove the no-op still holds, hence the file-level
//! `#![allow(deprecated)]`.

#![allow(deprecated)]

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
fn preserve_ids_toggle_changes_output_in_every_mode() {
    // Previously named preserve_ids_toggle_changes_nothing_in_any_mode and
    // asserted via assert_toggle_identical, like the five deprecated fields
    // below -- true under Slice A, before RFC 005 Slice B1 gave preserve_ids
    // a real effect: it now emits an anchor for a non-empty `id`, so
    // toggling it changes output in every mode. This is the one field
    // Option 3 makes real; the other five remain no-ops, proven below.
    for mode in MODES {
        let base = baseline(mode);
        let mut opts = ConversionOptions::for_mode(mode);
        let default_value = opts.preserve_ids;
        opts.preserve_ids = !default_value;
        let flipped = conv_with(ATTR_HTML, &opts);
        assert_ne!(
            flipped, base,
            "preserve_ids in {mode}: expected flipping to change output (RFC 005 Slice B1), but it matched the baseline"
        );
    }
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
