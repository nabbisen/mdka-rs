//! RFC 048 §6 — a removed mode name says *removed*, not *unknown*.
//!
//! `Strict`, `Semantic` and `Preserve` were removed in 3.0. Without a message of
//! their own, `"strict"` would fall into the generic *unknown conversion mode*
//! error, which tells a user their configuration is **wrong** when it is
//! **obsolete** -- the one outcome RFC 041 §10 called unacceptable.
//!
//! `FromStr` is the single place the CLI, the Node.js binding and `parse_mode`
//! all route a mode string through, so this is asserted here on `FromStr` and
//! `parse_mode` directly, and in the CLI's and Node.js's own tests. It is the
//! only notice a Rust caller who parses a mode from a config file will ever get:
//! no compiler warning could reach them, and the 2.9.0 `FromStr` accepted the
//! name silently.

use mdka::options::ConversionMode;

const REMOVED: [&str; 3] = ["strict", "semantic", "preserve"];

#[test]
fn a_removed_name_gets_the_removed_message_and_says_what_to_use() {
    for name in REMOVED {
        let err = name.parse::<ConversionMode>().unwrap_err();
        assert_eq!(
            err,
            format!(
                "conversion mode '{name}' was removed in 3.0; it was an alias of 'balanced'. \
                 Use 'balanced'."
            )
        );
        assert!(
            !err.contains("unknown"),
            "{name}: a removed name is not an unknown one: {err}"
        );
    }
}

#[test]
fn the_removed_message_is_case_insensitive_and_names_the_canonical_form() {
    let err = "STRICT".parse::<ConversionMode>().unwrap_err();
    assert!(
        err.starts_with("conversion mode 'strict' was removed in 3.0"),
        "{err}"
    );
    let err = "Semantic".parse::<ConversionMode>().unwrap_err();
    assert!(
        err.starts_with("conversion mode 'semantic' was removed in 3.0"),
        "{err}"
    );
}

#[test]
fn a_name_that_never_existed_still_gets_the_unknown_message() {
    for name in ["bogus", "", "balance", "strictly", "minimal2"] {
        let err = name.parse::<ConversionMode>().unwrap_err();
        assert_eq!(
            err,
            format!("unknown conversion mode: {}", name.to_ascii_lowercase())
        );
        assert!(!err.contains("removed"), "{name:?}: {err}");
    }
}

#[test]
fn the_two_surviving_modes_still_parse_in_any_case() {
    assert_eq!(
        "balanced".parse::<ConversionMode>(),
        Ok(ConversionMode::Balanced)
    );
    assert_eq!(
        "Minimal".parse::<ConversionMode>(),
        Ok(ConversionMode::Minimal)
    );
    assert_eq!(
        "BALANCED".parse::<ConversionMode>(),
        Ok(ConversionMode::Balanced)
    );
}

/// `parse_mode` returns an `Option`, so it cannot carry either message: it
/// answers `None` for a removed name and for an unknown one alike. Callers who
/// want the message use `FromStr` (`.parse()`), which is what the CLI and Node.js
/// do. Asserted so the difference is deliberate rather than discovered.
#[test]
fn parse_mode_is_none_for_a_removed_name_and_carries_no_message() {
    for name in REMOVED {
        assert_eq!(ConversionMode::parse_mode(name), None, "{name}");
    }
    assert_eq!(
        ConversionMode::parse_mode("balanced"),
        Some(ConversionMode::Balanced)
    );
}
