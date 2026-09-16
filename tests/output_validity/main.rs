//! RFC 025 — Markdown output-validity harness.
//!
//! No other test in this repository checks that mdka's output *is* Markdown:
//! they compare it against strings written by the same hands as the renderer.
//! This harness parses the output with a CommonMark parser (pulldown-cmark,
//! pinned in Cargo.toml) and asserts on what the parser reads.
//!
//! - `harness`: evaluation, the structure tree, the intent-free properties and
//!   the strict expected-failure helper.
//! - `inline_in_container`, `block_in_inline`: the composition matrix, both
//!   directions.
//! - `escaping`, `well_formedness`: §6.1 and §6.2 cells.
//! - `field_reports`: minimal reproductions from field reports and public
//!   issue reports.
//! - `gfm`: text that must stay text when read as GitHub Flavored Markdown.
//! - `corpus`: a directory runner that needs no per-file expectations.
//! - `proofs`: the helper and the properties shown failing.
//!
//! A cell marked `defect(...)` is a strict expected failure: it passes while
//! the recorded defect is present and FAILS once it is fixed, so the marker
//! has to be removed. It also fails if the cell cannot be evaluated. Run with
//! `-- --nocapture` to print every known defect with its evidence.
//!
//! This harness fixes nothing (RFC 027 Rule 2). Never delete a cell to make
//! the suite pass.

/// One `#[test]` per cell.
///
/// `name: html => expectation;` asserts the cell is correct in every mode.
/// `name: html => expectation, defect(Owner, "reason");` marks a known defect.
macro_rules! cells {
    ($( $name:ident : $html:expr => $expect:expr $(, defect($owner:ident, $reason:literal))? ; )*) => {
        $(
            #[test]
            fn $name() {
                cells!(@run $html, $expect $(, $owner, $reason)?)
            }
        )*
    };
    (@run $html:expr, $expect:expr) => {
        crate::harness::check($html, $expect)
    };
    (@run $html:expr, $expect:expr, $owner:ident, $reason:literal) => {
        crate::harness::known_defect(crate::harness::Owner::$owner, $reason, $html, $expect)
    };
}

mod harness;

mod block_in_inline;
mod code_context;
mod corpus;
mod escaping;
mod field_reports;
mod gfm;
mod inline_in_container;
mod proofs;
mod well_formedness;
