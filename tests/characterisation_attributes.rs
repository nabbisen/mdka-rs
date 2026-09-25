//! RFC 005 Slice A/B/C — attribute-field characterisation.
//!
//! `preserve_ids` is the one attribute-related `ConversionOptions` field that
//! acts on the output: it emits an anchor for a non-empty `id`, so toggling it
//! changes output in every mode (RFC 005 Slice B1). This file keeps that
//! assertion, on an element carrying every attribute kind.
//!
//! It used to prove the other five fields (`preserve_classes`,
//! `preserve_data_attrs`, `preserve_aria_attrs`, `preserve_unknown_attrs`,
//! `drop_presentation_attrs`) were no-ops, by flipping each from every mode's
//! default. They were removed in 3.0, so there is nothing left to flip; the
//! standing claim that Markdown has no attribute syntax to carry them is the
//! reason they went, and `tests/output_validity/mode_identity.rs` now guards
//! what matters after their removal: `Balanced` and `Minimal` convert exactly
//! as they did in 2.9.0.

mod common;
use common::conv_with;
use mdka::options::{ConversionMode, ConversionOptions};

const MODES: [ConversionMode; 2] = [ConversionMode::Balanced, ConversionMode::Minimal];

/// Carries id, class, data-*, aria-*, style, and an unknown attribute, so
/// every attribute-related field has something to act on, per the handoff's
/// §5 "Attributes" requirement.
const ATTR_HTML: &str =
    r#"<p id="pid" class="pclass" data-k="v" aria-label="lbl" style="color:red" foo="bar">Hi</p>"#;

/// Each mode's baseline on `ATTR_HTML`, re-derived per mode rather than
/// hard-coded, so this file does not silently depend on another file's
/// assertion still holding.
fn baseline(mode: ConversionMode) -> String {
    conv_with(ATTR_HTML, &ConversionOptions::for_mode(mode))
}

#[test]
fn preserve_ids_toggle_changes_output_in_every_mode() {
    // Previously named preserve_ids_toggle_changes_nothing_in_any_mode --
    // true under Slice A, before RFC 005 Slice B1 gave preserve_ids a real
    // effect: it now emits an anchor for a non-empty `id`, so toggling it
    // changes output in every mode.
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
