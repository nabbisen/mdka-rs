//! The harness: HTML → mdka → Markdown → pulldown-cmark → events → assertions.
//!
//! Nothing here compares Markdown bytes. Every check reads the parsed event
//! stream, either as a structure tree (`structure`) or through the intent-free
//! properties (`properties`).

use std::panic::{AssertUnwindSafe, catch_unwind};

use mdka::options::{ConversionMode, ConversionOptions};

mod properties;
mod structure;

pub use properties::{emphasis_negated_by_own_style, properties, starts_markdown_block};
pub use structure::{READINGS, Reading, structure};

/// Every mode. A cell is evaluated in each, so a fix that lands in one mode
/// only is visible as a partial result rather than a silent pass.
pub const MODES: [ConversionMode; 2] = [ConversionMode::Balanced, ConversionMode::Minimal];

/// A converter under test. Real cells use [`mdka_convert`]; the helper proofs
/// substitute stubs.
pub type Convert = fn(&str, &ConversionOptions) -> String;

pub fn mdka_convert(html: &str, opts: &ConversionOptions) -> String {
    mdka::html_to_markdown_with(html, opts)
}

// ─── Expectations and owners ───────────────────────────────────────────────

/// What a cell asserts beyond the properties.
#[derive(Clone, Copy)]
pub enum Expect {
    /// The parsed structure, in `structure` notation, must equal this under
    /// every reading.
    Tree(&'static str),
    /// The parsed structure differs by reading: a GFM-only construct (RFC
    /// 008: a table) reads back as itself only once the extension is on;
    /// under plain CommonMark the same bytes are still valid Markdown, never
    /// garbage, just read as something else -- a paragraph holding the
    /// literal pipe text, joined by soft breaks. Both readings are still
    /// asserted; neither is exempt, only different.
    TreeByReading {
        commonmark: &'static str,
        gfm: &'static str,
    },
    /// The intended structure is a recorded question, not a decision. Only
    /// the intent-free properties are asserted.
    Undecided(&'static str),
}

pub const fn tree(s: &'static str) -> Expect {
    Expect::Tree(s)
}

pub const fn tree_by_reading(commonmark: &'static str, gfm: &'static str) -> Expect {
    Expect::TreeByReading { commonmark, gfm }
}

pub const fn undecided(question: &'static str) -> Expect {
    Expect::Undecided(question)
}

/// Who fixes a known defect. An owner stays listed after its cells pass, so a
/// regression can be marked against it again.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Owner {
    /// Inline composition: inline elements inside links and code spans,
    /// inline writers bypassing the blockquote prefix, bare `<pre>`.
    Rfc024,
    /// Emphasis (or another inline) around block content.
    Rfc028,
    /// Block structure inside list items and blockquotes.
    Rfc035,
    /// Escaping and text round-trip: prose, code spans, fences, destinations,
    /// titles.
    Rfc010,
    /// Whitespace and separators at block boundaries: leading whitespace in a
    /// list item or heading; degenerate nested-list markers colliding with a
    /// thematic break.
    Rfc036,
    /// Nobody. Listed in the review request.
    Unowned,
}

impl std::fmt::Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Owner::Rfc024 => "RFC 024",
            Owner::Rfc028 => "RFC 028",
            Owner::Rfc035 => "RFC 035",
            Owner::Rfc010 => "RFC 010",
            Owner::Rfc036 => "RFC 036",
            Owner::Unowned => "UNOWNED",
        })
    }
}

// ─── Evaluation ────────────────────────────────────────────────────────────

/// The result of one cell in one mode.
#[derive(Debug)]
pub enum ModeResult {
    /// Structure and properties as intended.
    Pass,
    /// Evaluated, and the output is not what the HTML meant.
    Mismatch(Vec<String>),
    /// Could not be evaluated: the conversion or the harness panicked. Never
    /// counts as "still defective".
    Error(String),
}

pub struct Evaluation {
    pub mode: ConversionMode,
    pub markdown: Option<String>,
    pub result: ModeResult,
}

fn panic_message(p: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

pub fn evaluate(convert: Convert, html: &str, expect: Expect, mode: ConversionMode) -> Evaluation {
    let opts = ConversionOptions::for_mode(mode);
    let md = match catch_unwind(AssertUnwindSafe(|| convert(html, &opts))) {
        Ok(md) => md,
        Err(p) => {
            return Evaluation {
                mode,
                markdown: None,
                result: ModeResult::Error(format!("conversion panicked: {}", panic_message(p))),
            };
        }
    };
    // Every assertion under every reading; each problem names its reading.
    let checked = catch_unwind(AssertUnwindSafe(|| {
        let mut problems = Vec::new();
        for reading in READINGS {
            let want = match expect {
                Expect::Tree(want) => Some(want),
                Expect::TreeByReading { commonmark, gfm } => Some(match reading {
                    Reading::CommonMark => commonmark,
                    Reading::Gfm => gfm,
                }),
                Expect::Undecided(_) => None,
            };
            if let Some(want) = want {
                let got = structure(&md, reading, opts.preserve_ids);
                if got != want {
                    problems.push(format!(
                        "({reading}) [structure] expected {want}\n                           got      {got}"
                    ));
                }
            }
            // The intent-free properties assume the reading recovers the
            // HTML's own words and delimiter count. For GFM-only output (RFC
            // 008: a table) read as plain CommonMark, that assumption does
            // not hold by construction: the pipe/dash syntax the extension
            // would have consumed instead becomes literal prose, adding
            // words and punctuation the HTML never had. That is the accepted
            // cost of always emitting a real table (RFC 008 §4), not a text
            // loss or a stray delimiter -- so it is not checked under this
            // reading. The structure assertion above still runs for both:
            // the CommonMark reading must still be a sensible paragraph, not
            // garbage, and is asserted as one.
            if !(matches!(expect, Expect::TreeByReading { .. }) && reading == Reading::CommonMark) {
                problems.extend(
                    properties(html, &md, &opts, reading)
                        .into_iter()
                        .map(|p| format!("({reading}) {p}")),
                );
            }
        }
        problems
    }));
    let result = match checked {
        Ok(p) if p.is_empty() => ModeResult::Pass,
        Ok(p) => ModeResult::Mismatch(p),
        Err(p) => ModeResult::Error(format!("harness panicked: {}", panic_message(p))),
    };
    Evaluation {
        mode,
        markdown: Some(md),
        result,
    }
}

fn question(expect: Expect) -> String {
    match expect {
        Expect::Tree(_) | Expect::TreeByReading { .. } => String::new(),
        Expect::Undecided(q) => format!("\n  structure undecided: {q}"),
    }
}

fn describe(e: &Evaluation) -> String {
    let md = e
        .markdown
        .as_deref()
        .map_or("<none>".to_string(), |m| format!("{m:?}"));
    let body = match &e.result {
        ModeResult::Pass => "pass".to_string(),
        ModeResult::Mismatch(p) => p.join("\n    "),
        ModeResult::Error(err) => format!("ERROR: {err}"),
    };
    format!("  mode {}: markdown {md}\n    {body}", e.mode)
}

/// Asserts the cell is correct in every mode.
pub fn check(html: &str, expect: Expect) {
    let evals: Vec<_> = MODES
        .iter()
        .map(|&m| evaluate(mdka_convert, html, expect, m))
        .collect();
    if evals.iter().any(|e| !matches!(e.result, ModeResult::Pass)) {
        let report: Vec<_> = evals
            .iter()
            .filter(|e| !matches!(e.result, ModeResult::Pass))
            .map(describe)
            .collect();
        panic!(
            "cell failed\n  html {html:?}{}\n{}",
            question(expect),
            report.join("\n")
        );
    }
}

/// A strict expected failure. `Ok` only while the defect is present, as
/// recorded, in every mode. Fixed in any mode, or not evaluable in any mode:
/// `Err`.
pub fn known_defect_with(
    convert: Convert,
    owner: Owner,
    reason: &str,
    html: &str,
    expect: Expect,
) -> Result<String, String> {
    let evals: Vec<_> = MODES
        .iter()
        .map(|&m| evaluate(convert, html, expect, m))
        .collect();
    let errors: Vec<_> = evals
        .iter()
        .filter(|e| matches!(e.result, ModeResult::Error(_)))
        .map(describe)
        .collect();
    if !errors.is_empty() {
        return Err(format!(
            "known defect could not be evaluated -- this is not \"still defective\" (owner: {owner})\n  html {html:?}\n{}",
            errors.join("\n")
        ));
    }
    let passing: Vec<_> = evals
        .iter()
        .filter(|e| matches!(e.result, ModeResult::Pass))
        .map(|e| e.mode.to_string())
        .collect();
    if !passing.is_empty() {
        return Err(format!(
            "known defect now passes -- remove the known_defect marker (owner: {owner}); passing modes: {}\n  html {html:?}\n  recorded reason: {reason}",
            passing.join(", ")
        ));
    }
    let report: Vec<_> = evals.iter().map(describe).collect();
    Ok(format!(
        "KNOWN DEFECT (owner: {owner}): {reason}\n  html {html:?}{}\n{}",
        question(expect),
        report.join("\n")
    ))
}

// Reached only through `cells!` when a cell carries a marker; with none marked,
// the helper stays for the next regression (its behaviour is proven in
// `proofs.rs` through `known_defect_with`).
#[allow(dead_code)]
pub fn known_defect(owner: Owner, reason: &str, html: &str, expect: Expect) {
    match known_defect_with(mdka_convert, owner, reason, html, expect) {
        // Printed for the inventory capture (`-- --nocapture`); silent otherwise.
        Ok(report) => println!("{report}"),
        Err(e) => panic!("{e}"),
    }
}
