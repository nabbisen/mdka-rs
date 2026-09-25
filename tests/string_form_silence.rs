//! RFC 048 §7 — the Rust string form of a deprecated mode is silent, on purpose.
//!
//! `"strict".parse::<ConversionMode>()` and `ConversionMode::parse_mode("strict")`
//! return the variant without naming it, so nothing warns a Rust caller who reads
//! a mode from a config file. That is a **known gap, closed by 3.0's removal
//! message, not an oversight**: a library must not print, and a string is
//! invisible to the compiler. The other surfaces differ (CLI and Node accept the
//! string and warn; Python raises `TypeError`) and are pinned in their own test
//! suites; 3.0 removes these names and its error text depends on knowing exactly
//! who was warned.
//!
//! `#![deny(deprecated)]` is what asserts the silence: if a later change made
//! `FromStr`, `parse_mode` or `as_str` deprecated, this file would stop
//! compiling. Nothing here names a deprecated variant.

#![deny(deprecated)]

use mdka::options::ConversionMode;

#[test]
fn parsing_a_deprecated_mode_from_a_string_is_accepted_and_compiles_without_a_warning() {
    for name in ["strict", "semantic", "preserve"] {
        let parsed: ConversionMode = name.parse().expect("the string form is still accepted");
        assert_eq!(parsed.as_str(), name);
        assert_eq!(ConversionMode::parse_mode(name), Some(parsed));
    }
}

#[test]
fn the_string_form_is_case_insensitive_and_an_unknown_name_is_still_a_hard_error() {
    assert_eq!(
        ConversionMode::parse_mode("STRICT").map(|m| m.as_str()),
        Some("strict")
    );
    assert!("bogus".parse::<ConversionMode>().is_err());
}
