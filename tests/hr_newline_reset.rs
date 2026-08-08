//! Integration tests: RFC 016 — `<hr>` newline reset regression coverage
//! Covers: all six positions from the RFC 016 handoff §6, guarding against the
//! state-desync defect where `<hr>` swallowed the newline before following
//! content (fixed by switching the "hr" arm from `output.push_str` to
//! `push_raw`).

mod common;
use common::conv;

#[test]
fn hr_first() {
    assert_eq!(conv("<hr><p>B</p>"), "---\n\nB\n");
}

#[test]
fn hr_last() {
    assert_eq!(conv("<p>A</p><hr>"), "A\n\n---\n");
}

#[test]
fn hr_middle() {
    assert_eq!(conv("<p>A</p><hr><p>B</p>"), "A\n\n---\n\nB\n");
}

#[test]
fn hr_consecutive() {
    assert_eq!(conv("<p>A</p><hr><hr><p>B</p>"), "A\n\n---\n\n---\n\nB\n");
}

#[test]
fn hr_in_blockquote() {
    assert_eq!(
        conv("<blockquote><p>Q</p><hr><p>R</p></blockquote>"),
        "> Q\n\n> ---\n\n> R\n"
    );
}

#[test]
fn hr_after_list() {
    assert_eq!(conv("<ul><li>a</li></ul><hr><p>B</p>"), "- a\n\n---\n\nB\n");
}
