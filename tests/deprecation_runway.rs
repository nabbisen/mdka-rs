//! RFC 041 §9 — the deprecation runway for `3.0`.
//!
//! `ConversionMode::Strict`, `Semantic` and `Preserve` are aliases of
//! `Balanced`, and the owner's decision is to deprecate them in a minor and
//! remove them at `3.0`. A `3.0` that deletes a public name nobody was warned
//! about is the break this project does not make, so the warning is asserted
//! here: if the attribute is dropped, or a variant is deleted before `3.0`
//! deliberately deletes this file, a test fails.
//!
//! `#[deprecated]` is a compile-time lint with no runtime footprint, so the
//! attribute is asserted by reading it out of `src/options.rs`: a source check
//! by necessity. The last test names the variants directly, so deleting one
//! also fails to compile here.

use mdka::options::{ConversionMode, ConversionOptions};

const OPTIONS_RS: &str = include_str!("../src/options.rs");

/// The attribute block (doc comments and attributes) immediately above the
/// enum variant `name`.
fn attrs_above_variant(name: &str) -> String {
    let lines: Vec<&str> = OPTIONS_RS.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim() == format!("{name},"))
        .unwrap_or_else(|| panic!("variant `{name}` not found in src/options.rs"));
    let mut start = at;
    while start > 0 {
        let above = lines[start - 1].trim_start();
        if above.starts_with("///")
            || above.starts_with("#[")
            || above.starts_with(')')
            || above.starts_with("since")
            || above.starts_with("note")
        {
            start -= 1;
        } else {
            break;
        }
    }
    lines[start..at].join("\n")
}

#[test]
fn the_three_alias_variants_carry_a_deprecation_since_2_8_0() {
    for name in ["Strict", "Semantic", "Preserve"] {
        let attrs = attrs_above_variant(name);
        assert!(
            attrs.contains("#[deprecated("),
            "`ConversionMode::{name}` lost its #[deprecated] attribute; 3.0 cannot remove \
             what was never deprecated:\n{attrs}"
        );
        assert!(
            attrs.contains(r#"since = "2.8.0""#),
            "`ConversionMode::{name}`'s deprecation must say since = \"2.8.0\":\n{attrs}"
        );
    }
}

/// The note must say what to do, not only that something is wrong.
#[test]
fn the_deprecation_notes_say_what_to_use_and_when_it_goes() {
    for name in ["Strict", "Semantic", "Preserve"] {
        let attrs = attrs_above_variant(name);
        for needle in [
            "alias of `Balanced`",
            "use `Balanced`",
            "removed in 3.0",
            "api/modes.html",
        ] {
            assert!(
                attrs.contains(needle),
                "`ConversionMode::{name}`'s note must contain {needle:?}:\n{attrs}"
            );
        }
    }
}

/// `Balanced` is the replacement and `Minimal` converts differently; neither
/// is deprecated.
#[test]
fn balanced_and_minimal_are_not_deprecated() {
    for name in ["Balanced", "Minimal"] {
        let attrs = attrs_above_variant(name);
        assert!(
            !attrs.contains("deprecated"),
            "`ConversionMode::{name}` must not be deprecated:\n{attrs}"
        );
    }
}

/// Every name keeps working and keeps its output until 3.0. Naming the
/// variants here is the one place a removal would fail to compile, so it
/// cannot happen without this file being deleted on purpose.
#[test]
#[allow(deprecated)] // Internal: this asserts the deprecated aliases still exist and still alias Balanced.
fn the_variants_still_exist_and_still_alias_balanced() {
    let html = r#"<h1 id="t">T</h1><p class="c">a <strong>b</strong></p>"#;
    let balanced =
        mdka::html_to_markdown_with(html, &ConversionOptions::for_mode(ConversionMode::Balanced));
    for mode in [
        ConversionMode::Strict,
        ConversionMode::Semantic,
        ConversionMode::Preserve,
    ] {
        let got = mdka::html_to_markdown_with(html, &ConversionOptions::for_mode(mode));
        assert_eq!(got, balanced, "{mode} must stay identical to Balanced");
    }
}
