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
//! - `wrappers`: an unwrapped wrapper (`<div>`, `<section>`, `<article>`,
//!   `<main>`) keeps its block separation (RFC 036 §5.2, slice `036d`).
//! - `marker_collisions`: a nested list's own marker line, colliding with a
//!   setext heading underline or lazy-continuation text (RFC 038).
//! - `emphasis_fidelity`: empty emphasis writes no delimiters; nested
//!   same-class emphasis collapses to one level (RFC 037).
//! - `tables`: an expressible table becomes GFM; everything else falls back
//!   to a shape that never welds cells (RFC 008 slice `008a`).
//! - `elements`: `<dl>` no longer welds, `<del>`/`<s>`/checkbox lists become
//!   real GFM, `<sup>`/`<sub>` become Unicode where every character maps
//!   (RFC 009).
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

mod block_in_container;
mod block_in_inline;
mod code_context;
mod corpus;
mod elements;
mod emphasis_fidelity;
mod escaping;
mod field_reports;
mod gfm;
mod inline_in_container;
mod marker_collisions;
mod proofs;
mod tables;
mod well_formedness;
mod wrappers;
