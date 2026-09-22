//! RFC 038 — marker lines colliding with other CommonMark constructs.
//!
//! A nested list whose own first item is empty sits one line below the
//! parent item's text, sharing that item's continuation column. Two
//! CommonMark constructs read that column differently than "an empty
//! nested list item": an unordered marker (`-`) alone on a line is a setext
//! H2 underline for the text above it, and an ordered marker (`1.`) alone
//! is absorbed as the parent paragraph's own lazy-continuation text --
//! taking any non-empty siblings after it along, since they are lazy
//! continuations of the same paragraph too.
//!
//! The remedy is one blank line before such a nested list, regardless of
//! whether the child is ordered or unordered (RFC 038 §2.6; a bullet swap,
//! RFC 036 §5.5's fix for a different member of this family, does not work
//! here and is not attempted). This is disambiguation, not looseness: RFC
//! 035 §3.1's tight/loose rule and its own counting are unchanged by this
//! slice (RFC 038 §3.1's decision, see `api/elements.md`) -- the blank line
//! is output safety, not a claim that the source was loose. CommonMark
//! itself will still read the immediately enclosing list as loose once it
//! sees the blank line (looseness is a property of the bytes), which is why
//! every cell below expects the un-emptied sibling wrapped in `para(...)`
//! per RFC 035's own already-established loose-list convention -- that is
//! not a new rule, just this one applying as it always does.

use crate::harness::tree;

// ── the family: an empty-leading nested list, both directions ─────────────

cells! {
    // Setext failure mode: an empty unordered child.
    ul_then_empty_ul: "<ul><li>y<ul><li></li></ul></li></ul>"
        => tree(r#"ul(li(para("y"), ul(li())))"#);
    ul_then_empty_ul_with_sibling: "<ul><li>y<ul><li></li><li>z</li></ul></li></ul>"
        => tree(r#"ul(li(para("y"), ul(li(), li("z"))))"#);
    ol_then_empty_ul: "<ol><li>y<ul><li></li></ul></li></ol>"
        => tree(r#"ol[1](li(para("y"), ul(li())))"#);
    ol_then_empty_ul_with_sibling: "<ol><li>y<ul><li></li><li>z</li></ul></li></ol>"
        => tree(r#"ol[1](li(para("y"), ul(li(), li("z"))))"#);

    // Lazy-continuation failure mode: an empty ordered child -- the one that
    // destroys more, taking non-empty siblings with it in the unfixed output.
    ul_then_empty_ol: "<ul><li>y<ol><li></li></ol></li></ul>"
        => tree(r#"ul(li(para("y"), ol[1](li())))"#);
    ul_then_empty_ol_with_sibling: "<ul><li>y<ol><li></li><li>z</li></ol></li></ul>"
        => tree(r#"ul(li(para("y"), ol[1](li(), li("z"))))"#);
    ol_then_empty_ol: "<ol><li>y<ol><li></li></ol></li></ol>"
        => tree(r#"ol[1](li(para("y"), ol[1](li())))"#);
    ol_then_empty_ol_with_sibling: "<ol><li>y<ol><li></li><li>z</li></ol></li></ol>"
        => tree(r#"ol[1](li(para("y"), ol[1](li(), li("z"))))"#);

    // Three-level variants: the disambiguation must reach an empty child no
    // matter how deep, and must not disturb a shallower level that is
    // itself correct (its own nested list's first item has content).
    three_level_finding_2: "<ul><li>x<ul><li>y<ul><li></li></ul></li></ul></li></ul>"
        => tree(r#"ul(li("x", ul(li(para("y"), ul(li())))))"#);
    three_level_middle_content: "<ul><li>y<ul><li>z<ul><li></li></ul></li></ul></li></ul>"
        => tree(r#"ul(li("y", ul(li(para("z"), ul(li())))))"#);
}

// ── controls: already correct, must stay byte-identical to `559e8ff` ──────

cells! {
    // A nested list whose first item has content was never at risk -- its
    // own marker line is never bare.
    nested_list_first_item_has_content: "<ul><li>y<ul><li>z</li></ul></li></ul>"
        => tree(r#"ul(li("y", ul(li("z"))))"#);
    // A blockquote parent already emits a blank line before any second
    // block (RFC 024/028; blockquotes have no "tight" clamp to bypass), so
    // this shape was already correct before RFC 038.
    blockquote_then_empty_ul: "<blockquote>y<ul><li></li></ul></blockquote>"
        => tree(r#"quote(para("y"), ul(li()))"#);
}
