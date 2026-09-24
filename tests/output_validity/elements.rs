//! RFC 009 — element coverage extension.
//!
//! Four independent pieces. `<dl>` no longer welds -- the last element in
//! the codebase that concatenated content with no separator at all, now
//! that RFC 008 fixed tables; `<dt>`/`<dd>` each become their own
//! `Block::Paragraph`, exactly the route RFC 008 took for `tr`/`td`/`th`, so
//! `<dl>` inherits container prefixes and `<pre>` suppression for free.
//! `<del>`/`<s>` become `~~struck~~`; a checkbox list item's first content
//! becomes `- [x]`/`- [ ]`; both are plain GFM, both verified by parsing.
//! `<sup>`/`<sub>` become Unicode where **every** character of the content
//! maps, and are left exactly as they were otherwise -- the citation-marker
//! case (already correct today) especially. And all four interact correctly
//! with RFC 008's table cells: an expressible table stays expressible once
//! flattening a `<dl>`/emitting `~~`/a task marker/a mapped superscript is
//! involved, proven rather than assumed (RFC 009 §5.7).

use crate::harness::{tree, tree_by_reading};

cells! {
    // ── §4.1: <dl> no longer welds ──────────────────────────────────────────

    dl_no_longer_welds: "<dl><dt>Term</dt><dd>Desc</dd><dt>T2</dt><dd>D2</dd></dl>"
        => tree(r#"para("Term"), para("Desc"), para("T2"), para("D2")"#);
    dl_single_term: "<dl><dt>Term</dt><dd>Desc</dd></dl>"
        => tree(r#"para("Term"), para("Desc")"#);
    dl_in_list_item_carries_prefix: "<ul><li><dl><dt>Term</dt><dd>Desc</dd></dl></li></ul>"
        => tree(r#"ul(li(para("Term"), para("Desc")))"#);
    dl_in_blockquote_carries_prefix: "<blockquote><dl><dt>Term</dt><dd>Desc</dd></dl></blockquote>"
        => tree(r#"quote(para("Term"), para("Desc"))"#);
    dl_inside_pre_writes_no_markup: "<pre><dl><dt>Term</dt><dd>Desc</dd></dl></pre>"
        => tree(r#"codeblock("Term\nDesc")"#);

    // ── §4.2: <del>/<s> -> ~~struck~~; <ins>/<u> stay plain text ───────────

    del_becomes_strikethrough: "<del>gone</del>"
        => tree_by_reading(r#"para("~~gone~~")"#, r#"para(del("gone"))"#);
    s_becomes_strikethrough: "<s>old</s>"
        => tree_by_reading(r#"para("~~old~~")"#, r#"para(del("old"))"#);
    // GFM strikethrough is inline syntax; `~~` spanning a block boundary is
    // no more valid than `**` would be (RFC 028's own reasoning for
    // emphasis, extended to `~~`), so a <del>/<s> wrapping a block writes no
    // delimiters at all -- the content flows through as if it were not
    // there.
    strikethrough_around_block_writes_no_delimiters: "<del><p>x</p></del>"
        => tree(r#"para("x")"#);
    ins_and_u_stay_plain_text: "<ins>added</ins> and <u>under</u>"
        => tree(r#"para("added and under")"#);
    // RFC 037 §3, review-found for this delimiter (addendum, 2026-09-23):
    // nested same-class emphasis collapses to one level, and `~~` is a
    // delimiter class too -- `~~~~` at the start of a line is a tilde code
    // fence (content destroyed) and mid-line reads back as ONE span whose
    // content includes the literal `~~~~`, neither of which this can ever
    // produce once collapse applies. Mixed tags collapse the same way: the
    // rule is about the delimiter, not the tag (RFC 037's own point).
    nested_strikethrough_collapses_to_one_level: "<del><del>x</del></del>"
        => tree_by_reading(r#"para("~~x~~")"#, r#"para(del("x"))"#);
    nested_strikethrough_collapses_across_tags: "<del><s>x</s></del>"
        => tree_by_reading(r#"para("~~x~~")"#, r#"para(del("x"))"#);
    // No alternate delimiter form exists to swap to the way RFC 037 solved
    // adjacent same-class emphasis (`**`/`__`) -- `~~` has only one form, so
    // two adjacent spans merge into one instead of touching (the same
    // `~~~~` problem as nesting, addendum 2026-09-23). Renders identically
    // to the two adjacent sources; only the element boundary between them
    // is lost, not any content.
    adjacent_strikethrough_spans_merge: "<del>a</del><del>b</del>"
        => tree_by_reading(r#"para("~~ab~~")"#, r#"para(del("ab"))"#);
    three_adjacent_strikethrough_spans_merge: "<del>a</del><del>b</del><del>c</del>"
        => tree_by_reading(r#"para("~~abc~~")"#, r#"para(del("abc"))"#);
    // A genuine space between two spans is not adjacency -- GFM has no
    // trouble with `~~a~~ ~~b~~`, so nothing should merge here.
    strikethrough_spans_separated_by_space_do_not_merge: "<del>a</del> <del>b</del>"
        => tree_by_reading(
            r#"para("~~a~~ ~~b~~")"#,
            r#"para(del("a"), " ", del("b"))"#,
        );

    // ── §4.2: a checkbox list item's first content -> - [x] / - [ ] ────────

    checked_checkbox_becomes_task_marker: r#"<ul><li><input type="checkbox" checked>done</li></ul>"#
        => tree_by_reading(
            r#"ul(li("[x] done"))"#,
            r#"ul(li(task[x], "done"))"#,
        );
    unchecked_checkbox_becomes_task_marker: r#"<ul><li><input type="checkbox">todo</li></ul>"#
        => tree_by_reading(
            r#"ul(li("[ ] todo"))"#,
            r#"ul(li(task[ ], "todo"))"#,
        );
    // A list item with no checkbox at all is unaffected -- byte-identical to
    // today, not merely structurally equivalent (criterion 3).
    list_item_without_checkbox_is_unaffected: "<ul><li>plain</li></ul>"
        => tree(r#"ul(li("plain"))"#);
    // An `<input>` that is not a checkbox (or not the item's own first
    // content) is not mistaken for one.
    non_checkbox_input_is_not_a_task_marker: r#"<ul><li><input type="text" value="x">label</li></ul>"#
        => tree(r#"ul(li("label"))"#);

    // ── §4.3: <sup>/<sub> -> Unicode where every character maps ────────────
    //
    // The mappable shapes themselves (a digit, `n`/`i`) are standalone tests
    // below `cells!`, not entries here -- see the module doc comment there.

    // The 99% case (RFC 009 §4.3): a Wikipedia-style citation marker is
    // already correct today (a link whose text is the literal `[1]`), and
    // must stay exactly that -- not because it fails to map (it does not
    // even reach the mapping check: `[`/`]` are not in the mappable set),
    // but because changing it at all would be the defect, not fixing one.
    non_mappable_citation_marker_is_unchanged: r##"<sup><a href="#c1">[1]</a></sup>"##
        => tree(r#"para(link[#c1]("[1]"))"#);
    // The partial-content rule this cell held -- not every character maps, so
    // the whole thing stays as it was -- was narrowed by RFC 043: `a` now has
    // a superscript form, so `<sup>7a</sup>` maps whole, and content that
    // still cannot map takes a visible marker instead of being left to read
    // as a different statement. The all-or-nothing rule itself is unchanged
    // and is asserted, with the new shapes, in `sup_sub.rs`
    // (`a_run_with_one_unmappable_character_is_not_half_mapped`). This cell
    // is not among `cells!` any more because the marker adds characters the
    // HTML never had, which `properties()` -- by design -- does not accept.
    empty_superscript_writes_nothing: "a<sup></sup>b"
        => tree(r#"para("ab")"#);

    // ── §5.7 / RFC 008 §4: all of it inside a table cell ────────────────────
    //
    // Each must not regress an otherwise-expressible table to the fallback
    // (RFC 008 handoff §4's own warning) -- proven per mechanism, not
    // assumed from the non-cell cells above.

    dl_in_table_cell_flattens_with_br: "<table><tr><th>H</th></tr><tr><td><dl><dt>T</dt><dd>D</dd></dl></td></tr></table>"
        => tree_by_reading(
            r#"para("| H | | --- | | T", html("<br>"), "D |")"#,
            r#"table(thead(td("H")), tr(td("T", html("<br>"), "D")))"#,
        );
    strikethrough_in_table_cell_stays_expressible: "<table><tr><th>A</th></tr><tr><td><del>gone</del></td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | ~~gone~~ |")"#,
            r#"table(thead(td("A")), tr(td(del("gone"))))"#,
        );
    // A checkbox and a mapped superscript, each in a cell, are standalone
    // tests below `cells!`, not entries here -- see the module doc comment
    // there.
}

/// Criterion 4's "byte-identical" is checked at the byte level too, not only
/// against the parsed tree (two different byte sequences could parse the
/// same): the citation-marker shape, named explicitly in RFC 009 §4.3/§5.4,
/// unchanged from `7e7cec2`.
#[test]
fn non_mappable_citation_marker_is_byte_identical_to_baseline() {
    let html = r##"<sup><a href="#c1">[1]</a></sup>"##;
    let md = crate::harness::mdka_convert(html, &mdka::options::ConversionOptions::default());
    assert_eq!(md, "[\\[1\\]](#c1)\n");
}

/// Shapes whose `properties()` promise does not hold BY DESIGN, the same
/// reasoning RFC 008's own `f1_f3_structural_cost` module already
/// established: a mapped superscript/subscript is not "text survives
/// word-for-word" by construction (RFC 009 §4.3's whole point is that `7`
/// becomes `⁷`, a different character on purpose), and a checkbox in a
/// table cell is F1's own "words kept, structure not" trade-off, applied to
/// a new block type. Neither is a defect `properties()` should catch; both
/// are checked directly against `structure()`.
mod properties_exempt {
    use crate::harness::{Reading, mdka_convert, structure};

    fn check(html: &str, commonmark: &str, gfm: &str) {
        let md = mdka_convert(html, &mdka::options::ConversionOptions::default());
        for (reading, want) in [(Reading::CommonMark, commonmark), (Reading::Gfm, gfm)] {
            let got = structure(&md, reading, false);
            assert_eq!(got, want, "({reading}) markdown: {md:?}");
        }
    }

    #[test]
    fn superscript_digit_maps_to_unicode() {
        check("2<sup>7</sup>", r#"para("2⁷")"#, r#"para("2⁷")"#);
    }

    #[test]
    fn subscript_digits_map_to_unicode() {
        check("H<sub>2</sub>O", r#"para("H₂O")"#, r#"para("H₂O")"#);
    }

    #[test]
    fn superscript_letters_n_and_i_map() {
        check(
            "x<sup>n</sup> and y<sup>i</sup>",
            r#"para("xⁿ and yⁱ")"#,
            r#"para("xⁿ and yⁱ")"#,
        );
    }

    #[test]
    fn superscript_in_table_cell_stays_expressible() {
        check(
            "<table><tr><th>A</th></tr><tr><td>2<sup>7</sup></td></tr></table>",
            r#"para("| A | | --- | | 2⁷ |")"#,
            r#"table(thead(td("A")), tr(td("2⁷")))"#,
        );
    }

    // A checkbox inside a cell is literal `- [x]` text, not a real task
    // marker -- a GFM table cell holds inline content only, and task-list
    // parsing only ever applies to a real list item, which a cell is not
    // (the same reason a flattened list's own `- `/`N. ` markers, RFC 008
    // §4.1, are literal text there too, under both readings alike).
    #[test]
    fn checkbox_in_table_cell_stays_literal_text() {
        check(
            r#"<table><tr><th>A</th></tr><tr><td><ul><li><input type="checkbox" checked>done</li></ul></td></tr></table>"#,
            r#"para("| A | | --- | | - [x] done |")"#,
            r#"table(thead(td("A")), tr(td("- [x] done")))"#,
        );
    }
}
