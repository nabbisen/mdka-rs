//! RFC 036 §5.2 / slice `036d`: an unwrapped wrapper keeps its block
//! separation. `docs/src/api/elements.md` documents `<div>`, `<section>`,
//! `<article>` and `<main>` as acting like paragraph breaks whether or not
//! `unwrap_unknown_wrappers` removes the tag; each cell here asserts exactly
//! that, in all five modes, on div-per-line markup with no `<p>` inside --
//! the shape that used to weld words together under Minimal and Semantic
//! (`- - -` sharing one line was the list-item analogue RFC 036's earlier
//! slices fixed; this is the same "the promise and the code disagreed"
//! defect, one level up, for a documented block rather than a marker).
//! `<figure>`/`<figcaption>` are never unwrapped in any mode and are not
//! this file's concern.

use crate::harness::tree;

cells! {
    div_per_line: "<div>First.</div><div>Second.</div><div>Third.</div>"
        => tree(r#"para("First."), para("Second."), para("Third.")"#);
    section_per_line: "<section>First.</section><section>Second.</section>"
        => tree(r#"para("First."), para("Second.")"#);
    article_per_line: "<article>First.</article><article>Second.</article>"
        => tree(r#"para("First."), para("Second.")"#);
    main_per_line: "<main>First.</main><main>Second.</main>"
        => tree(r#"para("First."), para("Second.")"#);
    mixed_wrapper_kinds_per_line: "<article>a</article><section>b</section><main>c</main>"
        => tree(r#"para("a"), para("b"), para("c")"#);
    // Nested wrappers separate each other too, not only top-level siblings.
    nested_wrappers_separate: "<div>a<div>b</div>c</div>"
        => tree(r#"para("a"), para("b"), para("c")"#);
    // A wrapper holding a real block (not bare text) already worked before
    // this slice -- the <p> supplied its own separation -- and must go on
    // working the same way now that the wrapper supplies one too.
    wrapper_holding_a_paragraph_still_separates: "<div><p>a</p></div><div><p>b</p></div>"
        => tree(r#"para("a"), para("b")"#);
    // 2.4.1: a wrapper carrying an `id` keeps its anchor in every mode that
    // has `preserve_ids` on, unwrapped or not. The harness skips id anchors
    // when comparing a tree (they are a documented option's output, not
    // content), so these cells assert the anchor does not disturb the
    // separation -- that the anchor is *there* is
    // `characterisation_structural.rs`'s job, byte for byte.
    id_wrapper_per_line: r#"<div id="a">First.</div><div id="b">Second.</div>"#
        => tree(r#"para("First."), para("Second.")"#);
    id_nested_wrappers_separate: r#"<div id="a">a<div id="b">b</div>c</div>"#
        => tree(r#"para("a"), para("b"), para("c")"#);
    id_wrapper_in_list_item: r#"<ul><li><div id="a">item</div></li></ul>"#
        => tree(r#"ul(li("item"))"#);
}
