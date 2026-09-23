//! RFC 008 — GFM table support: the grid, the expressibility pre-pass, the
//! non-welding fallback (`008a`), and F1/F2/F3 (`008b`).
//!
//! A table's cells used to concatenate with no separator at all
//! (`<table><tr><th>H1</th><th>H2</th></tr>...</table>` -> `"H1H2ab"`).
//! `008a` made a narrow set of tables (one header row, first; no span; no
//! cell holding more than a single bare paragraph; no nested table; no
//! caption) into real GFM tables. `008b` (RFC 008 §4.1) widens that to the
//! 73% `008a` still fell back on: **F1** flattens a cell's block content to
//! inline (`<br>`-joined paragraphs, literal list markers, one code span per
//! source line, plain heading/blockquote text); **F2** synthesises an empty
//! header row when a table has no `<th>` at all; **F3** expands `colspan`/
//! `rowspan` by repeating the covered content, header included. What is
//! left inexpressible, and stays the `Block::Paragraph` fallback: more than
//! one whole-row header, the row-header pattern (`<th>` as each row's own
//! first cell -- not a span problem, per RFC 008 §4.1), a nested `<table>`
//! anywhere in a cell, and `<caption>` (no GFM syntax exists for any of
//! these).
//!
//! Expressible cells use [`tree_by_reading`]: GFM table syntax is still valid
//! Markdown without the extension, just read as a paragraph of literal pipe
//! text joined by soft breaks -- not garbage, but a different tree, so both
//! readings are asserted against what each actually means.

use crate::harness::{tree, tree_by_reading};

// ── expressible: a real GFM table ──────────────────────────────────────────

cells! {
    two_column_table: "<table><tr><th>Name</th><th>Age</th></tr><tr><td>Alice</td><td>30</td></tr></table>"
        => tree_by_reading(
            r#"para("| Name | Age | | --- | --- | | Alice | 30 |")"#,
            r#"table(thead(td("Name"), td("Age")), tr(td("Alice"), td("30")))"#,
        );
    // Also the classic welding shape from RFC 008 §1 -- now a real table,
    // not merely non-welded (`H1H2ab` was already impossible before this
    // cell; this proves the *expressible* case too).
    classic_welding_shape_becomes_a_real_table: "<table><tr><th>H1</th><th>H2</th></tr><tr><td>a</td><td>b</td></tr></table>"
        => tree_by_reading(
            r#"para("| H1 | H2 | | --- | --- | | a | b |")"#,
            r#"table(thead(td("H1"), td("H2")), tr(td("a"), td("b")))"#,
        );
    // Alignment from `align=`.
    alignment_from_align_attribute: r#"<table><tr><th align="left">A</th><th align="center">B</th><th align="right">C</th></tr><tr><td>1</td><td>2</td><td>3</td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | B | C | | :-- | :-: | --: | | 1 | 2 | 3 |")"#,
            r#"table(thead(td("A"), td("B"), td("C")), tr(td("1"), td("2"), td("3")))"#,
        );
    // Alignment from `text-align:`, and `!important`/case are tolerated the
    // same way `emphasis_negated_by_style` already tolerates them
    // (`style_property` is shared).
    alignment_from_text_align_style: r#"<table><tr><th style="text-align: RIGHT !important">A</th></tr><tr><td>1</td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | | --: | | 1 |")"#,
            r#"table(thead(td("A")), tr(td("1")))"#,
        );
    // A ragged short row is not a blocker (RFC 008 §3's own "Can" list): GFM
    // pads it with an empty trailing cell, losing nothing. The long-row half
    // of this (a cell truncated away entirely) is `long_row_is_truncated`,
    // below, outside `cells!`: truncation is real, permanent data loss by
    // design, which the intent-free properties correctly cannot pass.
    short_row_is_padded: "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td></tr></table>"
        => tree_by_reading(
            r#"para("| A | B | | --- | --- | | 1 |")"#,
            r#"table(thead(td("A"), td("B")), tr(td("1"), td()))"#,
        );
    // `|` in cell content is escaped (criterion 3): plain backslash-escape
    // is valid CommonMark with or without table support, so it parses back
    // to one literal `|` under both readings, in one cell either way.
    pipe_in_cell_is_escaped: "<table><tr><th>A</th></tr><tr><td>a | b</td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | a | b |")"#,
            r#"table(thead(td("A")), tr(td("a | b")))"#,
        );
    // `|` escaping must reach a cell's content regardless of which path
    // produced it (RFC 008 addendum A): a code span writes verbatim (RFC
    // 024), and a link's text/destination and an image's alt are captured
    // and spliced back as an already-rendered string -- none of those go
    // through the per-character escaping a cell's own direct text does, so
    // each needs the cell-boundary escape (`escape::escape_table_cell_pipes`)
    // to reach it at all. Each still parses back into one cell, not two.
    // Under CommonMark, a code span's content is verbatim -- the backslash
    // itself is part of what the span holds, not consumed as an escape.
    // Under GFM, the pre-tokenization pipe-split still happens first, so the
    // same backslash protects the cell boundary either way.
    pipe_in_code_span_in_cell_is_escaped: "<table><tr><th>A</th></tr><tr><td><code>a | b</code></td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | ", code("a \\| b"), " |")"#,
            r#"table(thead(td("A")), tr(td(code("a | b"))))"#,
        );
    pipe_in_link_text_in_cell_is_escaped: r#"<table><tr><th>A</th></tr><tr><td><a href="/x">a | b</a></td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | | --- | | ", link[/x]("a | b"), " |")"#,
            r#"table(thead(td("A")), tr(td(link[/x]("a | b"))))"#,
        );
    pipe_in_link_destination_in_cell_is_escaped: r#"<table><tr><th>A</th></tr><tr><td><a href="/a|b">link</a></td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | | --- | | ", link[/a|b]("link"), " |")"#,
            r#"table(thead(td("A")), tr(td(link[/a|b]("link"))))"#,
        );
    pipe_in_image_alt_in_cell_is_escaped: r#"<table><tr><th>A</th></tr><tr><td><img src="i.png" alt="a | b"></td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | | --- | | ", image[i.png]("a | b"), " |")"#,
            r#"table(thead(td("A")), tr(td(image[i.png]("a | b"))))"#,
        );
    // Inline formatting inside a cell renders exactly as it would anywhere
    // else: bold, and a link -- both still plain CommonMark, so they stay
    // real nodes (not literal text) under the CommonMark reading too.
    inline_formatting_in_cell: r#"<table><tr><th>A</th></tr><tr><td><strong>bold</strong> and <a href="/x">link</a></td></tr></table>"#
        => tree_by_reading(
            r#"para("| A | | --- | | ", strong("bold"), " and ", link[/x]("link"), " |")"#,
            r#"table(thead(td("A")), tr(td(strong("bold"), " and ", link[/x]("link"))))"#,
        );
    // `<br>` survives as inline HTML rather than a real hard break, which
    // would end the cell (RFC 008 §3's "Can" list). A space on both sides in
    // the source keeps "one"/"two" apart regardless of how an inline HTML
    // tag affects word boundaries -- not what this cell is about.
    br_in_cell_stays_literal: "<table><tr><th>A</th></tr><tr><td>one <br> two</td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | one ", html("<br>"), " two |")"#,
            r#"table(thead(td("A")), tr(td("one ", html("<br>"), " two")))"#,
        );
    // A single bare paragraph in a cell flattens to inline trivially (RFC
    // 008 §2's method note) -- not a blocker.
    single_paragraph_cell_flattens: "<table><tr><th>A</th></tr><tr><td><p>x</p></td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | x |")"#,
            r#"table(thead(td("A")), tr(td("x")))"#,
        );
    // Empty cells are not a blocker.
    empty_cell_is_expressible: "<table><tr><th>A</th><th>B</th></tr><tr><td></td><td>x</td></tr></table>"
        => tree_by_reading(
            r#"para("| A | B | | --- | --- | | | x |")"#,
            r#"table(thead(td("A"), td("B")), tr(td(), td("x")))"#,
        );

    // ── F2: synthesise a header when there is no `<th>` at all ─────────────

    // A single headerless row: the old welding case (`no_header_row_falls_back`
    // in `008a`) is now expressible -- an empty header row over the data,
    // not a promoted row 1 (RFC 008 §4's own preference).
    no_header_row_synthesizes_header: "<table><tr><td>a</td></tr></table>"
        => tree_by_reading(
            r#"para("| | | --- | | a |")"#,
            r#"table(thead(td()), tr(td("a")))"#,
        );
    // The `008a` welding shape re-run without any `<th>` at all: still one
    // synthesized header, not the multi-header case below (no whole row is
    // ever all-`<th>` here).
    header_free_multi_column_synthesizes_header: "<table><tr><td>H1</td><td>H2</td></tr><tr><td>a</td><td>b</td></tr></table>"
        => tree_by_reading(
            r#"para("| | | | --- | --- | | H1 | H2 | | a | b |")"#,
            r#"table(thead(td(), td()), tr(td("H1"), td("H2")), tr(td("a"), td("b")))"#,
        );
    // Surfaced by the battery diff against `02c56e7`: on that baseline this
    // table (no `<th>`) was inexpressible under `008a`'s own rules and fell
    // back to `Block::Paragraph`, which RFC 028 already strips the enclosing
    // `<strong>` from, giving plain `x` -- not a defect, confirmed identical
    // to `<strong><p>x</p></strong>`. `008b`'s F2 now makes this table
    // expressible, so it renders for real instead; the enclosing `<strong>`
    // is still dropped, the same RFC 028 rule applying to the new output.
    table_nested_in_inline_emphasis_stays_unwoven: "<strong><table><tr><td>x</td></tr></table></strong>"
        => tree_by_reading(
            r#"para("| | | --- | | x |")"#,
            r#"table(thead(td()), tr(td("x")))"#,
        );

    // ── F1: flatten cell blocks to inline (RFC 008 §4.1's own table) ───────
    //
    // Every OTHER F1/F3 shape (a list, a code block, a heading, a blockquote,
    // a span repeating its content) is verified as a standalone test below
    // `cells!`, not here: `properties()` assumes a reading recovers the
    // HTML's own words and skeleton one-for-one, which does not hold for any
    // of them BY DESIGN -- flattening a list keeps its words but not its `ul`/
    // `li` nodes, and repeating a spanned label is a deliberate duplication
    // (RFC 008 §4's own "prefer repeating... over leaving neighbours empty").
    // That is the accepted cost, not a defect, the same reasoning
    // `long_row_is_truncated` already established for GFM's own truncation.
    // Two paragraphs joining with a literal `<br>` is the one F1 shape that
    // adds no new words and drops no skeleton entry (a `<p>` is not itself
    // tracked), so it is the one that can stay a normal cell here (was
    // `two_paragraphs_in_cell_fall_back` in `008a` -- GFM used to truncate
    // the row and leak the second paragraph below the table; now it stays in
    // the one cell).
    two_paragraphs_in_cell_join_with_br: "<table><tr><th>A</th></tr><tr><td><p>x</p><p>y</p></td></tr></table>"
        => tree_by_reading(
            r#"para("| A | | --- | | x", html("<br>"), "y |")"#,
            r#"table(thead(td("A")), tr(td("x", html("<br>"), "y")))"#,
        );

    // ── criterion 4: container prefix on every line ────────────────────────

    // A tight list item's sole paragraph is not itself wrapped (a CommonMark
    // parser convention, not mdka's choice) -- unlike a blockquote, below.
    expressible_table_in_list_item: "<ul><li><table><tr><th>A</th></tr><tr><td>1</td></tr></table></li></ul>"
        => tree_by_reading(
            r#"ul(li("| A | | --- | | 1 |"))"#,
            r#"ul(li(table(thead(td("A")), tr(td("1")))))"#,
        );
    expressible_table_in_blockquote: "<blockquote><table><tr><th>A</th></tr><tr><td>1</td></tr></table></blockquote>"
        => tree_by_reading(
            r#"quote(para("| A | | --- | | 1 |"))"#,
            r#"quote(table(thead(td("A")), tr(td("1"))))"#,
        );

    // ── criterion 5: no table markup inside <pre> ──────────────────────────

    expressible_table_inside_pre_writes_no_markup: "<pre><table><tr><th>A</th></tr><tr><td>1</td></tr></table></pre>"
        => tree(r#"codeblock("A\n1")"#);
}

// ── inexpressible: the three `008b` blockers, plus <caption> (criterion 2) ─
//
// `008a`'s own blockers (no header, block in a cell, colspan/rowspan) are all
// expressible now -- see the F1/F2/F3 cells above. What is left is RFC 008
// §4.1's own list: more than one whole-row header, the row-header pattern,
// a nested table anywhere in a cell, and <caption> (unchanged: no GFM syntax
// exists for it).

cells! {
    two_header_rows_fall_back: "<table><tr><th>H1</th></tr><tr><th>H2</th></tr><tr><td>a</td></tr></table>"
        => tree(r#"para("H1"), para("H2"), para("a")"#);
    // `<th>` as each row's own first cell, not as a whole first row -- RFC
    // 008 §4.1: "not a span problem", must stay inexpressible under "no
    // header row" rather than be treated as F3's to expand.
    row_header_pattern_stays_inexpressible: "<table><tr><th>R1</th><td>1</td></tr><tr><th>R2</th><td>2</td></tr></table>"
        => tree(r#"para("R1"), para("1"), para("R2"), para("2")"#);
    // A nested table remains a blocker for the OUTER table (RFC 008 §4.1:
    // "do not try to flatten it") -- but the inner table is analyzed on its
    // own terms once the outer has fallen back, and this one has no `<th>`
    // of its own, so F2 applies to it independently: an emergent case RFC
    // 008 §4.1 does not name, worth flagging back rather than assuming away.
    nested_table_blocks_outer_but_inner_can_still_synthesize_its_own_header:
        "<table><tr><th>A</th></tr><tr><td><table><tr><td>inner</td></tr></table></td></tr></table>"
        => tree_by_reading(
            r#"para("A"), para("| | | --- | | inner |")"#,
            r#"para("A"), table(thead(td()), tr(td("inner")))"#,
        );

    // ── <caption> is not silently dropped (unchanged from `008a`) ──────────

    caption_forces_fallback_but_survives: "<table><caption>My Table</caption><tr><th>H1</th></tr><tr><td>a</td></tr></table>"
        => tree(r#"para("My Table"), para("H1"), para("a")"#);

    // ── criteria 4/5 for the fallback too, not just the expressible case ───
    // (a single headerless `<td>` is F2's own case now, tested above -- a
    // genuine blocker is needed here, so the row-header pattern stands in.)

    fallback_table_in_list_item_carries_prefix: "<ul><li><table><tr><th>R1</th><td>1</td></tr><tr><th>R2</th><td>2</td></tr></table></li></ul>"
        => tree(r#"ul(li(para("R1"), para("1"), para("R2"), para("2")))"#);
    fallback_table_in_blockquote_carries_prefix: "<blockquote><table><tr><th>R1</th><td>1</td></tr><tr><th>R2</th><td>2</td></tr></table></blockquote>"
        => tree(r#"quote(para("R1"), para("1"), para("R2"), para("2"))"#);
    fallback_table_inside_pre_writes_no_markup: "<pre><table><tr><td>a</td></tr></table></pre>"
        => tree(r#"codeblock("a")"#);

    // ── criterion 5 restated for `008b`: a container prefix must survive
    // even when the cell content is itself flattened, multi-block F1 output,
    // not just a bare inline cell (`expressible_table_in_list_item`/
    // `_in_blockquote`, above, only ever tested the simple case). ──────────

    expressible_table_with_flattened_cell_in_list_item:
        "<ul><li><table><tr><th>A</th></tr><tr><td><p>x</p><p>y</p></td></tr></table></li></ul>"
        => tree_by_reading(
            r#"ul(li("| A | | --- | | x", html("<br>"), "y |"))"#,
            r#"ul(li(table(thead(td("A")), tr(td("x", html("<br>"), "y")))))"#,
        );
    expressible_table_with_flattened_cell_in_blockquote:
        "<blockquote><table><tr><th>A</th></tr><tr><td><p>x</p><p>y</p></td></tr></table></blockquote>"
        => tree_by_reading(
            r#"quote(para("| A | | --- | | x", html("<br>"), "y |"))"#,
            r#"quote(table(thead(td("A")), tr(td("x", html("<br>"), "y"))))"#,
        );
}

/// F3's grid is resolved over every row, header and body together (RFC 008
/// §4.1), not the header row alone as `008a` implicitly assumed: a body row
/// longer than the header now widens the whole grid, with an empty
/// synthesized column in the header, rather than being silently truncated by
/// GFM's own table parser. An emergent, welcome side effect of the unified
/// grid, not something RFC 008 §4.1 asked for -- worth reporting back as a
/// found improvement. (The short-row, non-lossy padding half of raggedness
/// is `short_row_is_padded`, above, unaffected either way.)
#[test]
fn long_row_widens_the_grid_instead_of_being_truncated() {
    let html =
        r#"<table><tr><th>A</th><th>B</th></tr><tr><td>2</td><td>3</td><td>4</td></tr></table>"#;
    for (reading, want) in [
        (
            crate::harness::Reading::CommonMark,
            r#"para("| A | B | | | --- | --- | --- | | 2 | 3 | 4 |")"#,
        ),
        (
            crate::harness::Reading::Gfm,
            r#"table(thead(td("A"), td("B"), td()), tr(td("2"), td("3"), td("4")))"#,
        ),
    ] {
        let md = crate::harness::mdka_convert(html, &mdka::options::ConversionOptions::default());
        let got = crate::harness::structure(&md, reading, false);
        assert_eq!(got, want, "({reading}) markdown: {md:?}");
    }
}

/// F1/F3 shapes whose `properties()` promise does not hold BY DESIGN:
/// flattening a cell's blocks to inline keeps the words but not the `ul`/
/// `li`/`codeblock`/heading/blockquote skeleton those words came from, and
/// repeating a spanned label duplicates a word GFM has no other syntax to
/// carry (RFC 008 §4's own "prefer repeating... over leaving neighbours
/// empty"). Neither is a defect -- both are the accepted mechanism itself --
/// so each is checked directly against `structure()`, the same way
/// `long_row_widens_the_grid_instead_of_being_truncated` already must, one
/// `#[test]` per shape rather than a `cells!` entry `properties()` could
/// never pass.
mod f1_f3_structural_cost {
    use crate::harness::{MODES, Reading, mdka_convert, structure};

    fn check(html: &str, commonmark: &str, gfm: &str) {
        let md = mdka_convert(html, &mdka::options::ConversionOptions::default());
        for (reading, want) in [(Reading::CommonMark, commonmark), (Reading::Gfm, gfm)] {
            let got = structure(&md, reading, false);
            assert_eq!(got, want, "({reading}) markdown: {md:?}");
        }
    }

    /// `check`, above, converts with `ConversionOptions::default()` -- one
    /// mode -- unlike `cells!`, which runs all five. Every `#[test]` in this
    /// module rests on F1/F3 output being mode-independent (RFC 008 §4.1's
    /// own mechanisms never read `ConversionOptions`, `008a`'s
    /// `evidence/d-before-after-parses.txt` showed the same for its own
    /// expressible path), but nothing before this pinned it -- found in
    /// `008b`'s review. One representative shape combining F1 (a flattened
    /// list) and F3 (a repeated span) makes the other twelve honest: if a
    /// future change ever makes table rendering mode-sensitive, this is
    /// what catches it.
    #[test]
    fn f1_f3_output_is_mode_independent() {
        let html = r#"<table><tr><th colspan="2">A</th></tr><tr><td><ul><li>x</li><li>y</li></ul></td><td>1</td></tr></table>"#;
        let mut outputs = MODES.iter().map(|&mode| {
            let opts = mdka::options::ConversionOptions::for_mode(mode);
            (mode, mdka_convert(html, &opts))
        });
        let (first_mode, first_md) = outputs.next().expect("MODES is non-empty");
        for (mode, md) in outputs {
            assert_eq!(
                md, first_md,
                "mode {mode} produced different table output than {first_mode}: {md:?} vs {first_md:?}"
            );
        }
    }

    #[test]
    fn colspan_repeats_the_label() {
        check(
            r#"<table><tr><th colspan="2">A</th></tr><tr><td>1</td><td>2</td></tr></table>"#,
            r#"para("| A | A | | --- | --- | | 1 | 2 |")"#,
            r#"table(thead(td("A"), td("A")), tr(td("1"), td("2")))"#,
        );
    }

    #[test]
    fn rowspan_repeats_down() {
        check(
            r#"<table><tr><th>A</th><th>B</th></tr><tr><td rowspan="2">x</td><td>1</td></tr><tr><td>2</td></tr></table>"#,
            r#"para("| A | B | | --- | --- | | x | 1 | | x | 2 |")"#,
            r#"table(thead(td("A"), td("B")), tr(td("x"), td("1")), tr(td("x"), td("2")))"#,
        );
    }

    // RFC 008 §4.1: a measurement (18/18 span-using tables have a header
    // span, 8 only there) reversed the original intuition that a header
    // span should be a blocker -- it is repeated exactly like a body cell.
    #[test]
    fn header_colspan_repeats_the_label() {
        check(
            r#"<table><tr><th colspan="2">Group</th></tr><tr><td>1</td><td>2</td></tr></table>"#,
            r#"para("| Group | Group | | --- | --- | | 1 | 2 |")"#,
            r#"table(thead(td("Group"), td("Group")), tr(td("1"), td("2")))"#,
        );
    }

    // A rowspan starting in the header carries its label into the covered
    // body cell, the same rule, for the same reason (RFC 008 §4.1).
    #[test]
    fn header_rowspan_carries_into_body() {
        check(
            r#"<table><tr><th rowspan="2">A</th><th>B</th></tr><tr><td>1</td></tr><tr><td>x</td><td>2</td></tr></table>"#,
            r#"para("| A | B | | --- | --- | | A | 1 | | x | 2 |")"#,
            r#"table(thead(td("A"), td("B")), tr(td("A"), td("1")), tr(td("x"), td("2")))"#,
        );
    }

    // A list becomes literal `- `-prefixed marker text, `<br>`-joined --
    // reads as a list under CommonMark's own soft-break-joined paragraph,
    // not real list nodes (RFC 008 §4.1: "literal markers, reads as a list").
    #[test]
    fn unordered_list_in_cell_becomes_literal_markers() {
        check(
            "<table><tr><th>A</th></tr><tr><td><ul><li>x</li></ul></td></tr></table>",
            r#"para("| A | | --- | | - x |")"#,
            r#"table(thead(td("A")), tr(td("- x")))"#,
        );
    }

    #[test]
    fn ordered_list_in_cell_becomes_literal_markers() {
        check(
            "<table><tr><th>A</th></tr><tr><td><ol><li>x</li><li>y</li></ol></td></tr></table>",
            r#"para("| A | | --- | | 1. x", html("<br>"), "2. y |")"#,
            r#"table(thead(td("A")), tr(td("1. x", html("<br>"), "2. y")))"#,
        );
    }

    // A nested list item gets its own `<br>` before its marker, the same as
    // any later sibling item would (RFC 008 addendum-b, 2026-09-23: found by
    // review -- `cell_after_marker` stayed set across the parent item's own
    // text, so the first nested item ran together with it on one line, only
    // fixed by clearing the flag once real text has been written for the
    // marker's own item, not just when its enclosing item closes). The two-
    // space indent is real in the Markdown bytes (`markdown:` in a failure
    // would show it), but `structure()` collapses any run of whitespace to
    // one space (the same normalization every other cell in this file
    // already goes through), so it reads back as a single leading space.
    #[test]
    fn nested_list_in_cell_indents_two_spaces() {
        check(
            "<table><tr><th>A</th></tr><tr><td><ul><li>a<ul><li>b</li></ul></li></ul></td></tr></table>",
            r#"para("| A | | --- | | - a", html("<br>"), " - b |")"#,
            r#"table(thead(td("A")), tr(td("- a", html("<br>"), " - b")))"#,
        );
    }

    // A code block becomes one code span per source line, `<br>`-joined --
    // *not* one span holding a literal `<br>` (RFC 008 §4.1's explicit "not"
    // case): inside a code span the tag would be literal text instead of a
    // separator.
    #[test]
    fn code_block_in_cell_becomes_one_span_per_line() {
        check(
            "<table><tr><th>A</th></tr><tr><td><pre><code>x</code></pre></td></tr></table>",
            r#"para("| A | | --- | | ", code("x"), " |")"#,
            r#"table(thead(td("A")), tr(td(code("x"))))"#,
        );
    }

    #[test]
    fn multiline_code_block_in_cell_one_span_per_line() {
        check(
            "<table><tr><th>A</th></tr><tr><td><pre><code>line1\nline2</code></pre></td></tr></table>",
            r#"para("| A | | --- | | ", code("line1"), html("<br>"), code("line2"), " |")"#,
            r#"table(thead(td("A")), tr(td(code("line1"), html("<br>"), code("line2"))))"#,
        );
    }

    // 2.4.1: a blank source line inside a code block in a cell is a bare
    // `<br>`, not an empty code span. ` `` ` is a stray delimiter run that
    // paired with the next one, pulling the following line into a span it
    // was never in and turning the `<br>`s into literal text -- 24 cells of
    // Wikipedia's *Markdown* article, the syntax-reference table that
    // article exists for. Each line is now its own span, the blank lines
    // are the `<br><br>` between them.
    #[test]
    fn code_block_with_blank_lines_in_cell_keeps_every_line_in_its_own_span() {
        check(
            "<table><tr><th>A</th></tr><tr><td><pre>a\n\nb\n\nc</pre></td></tr></table>",
            r#"para("| A | | --- | | ", code("a"), html("<br>"), html("<br>"), code("b"), html("<br>"), html("<br>"), code("c"), " |")"#,
            r#"table(thead(td("A")), tr(td(code("a"), html("<br>"), html("<br>"), code("b"), html("<br>"), html("<br>"), code("c"))))"#,
        );
    }

    #[test]
    fn code_block_with_consecutive_blank_lines_in_cell() {
        check(
            "<table><tr><th>A</th></tr><tr><td><pre>a\n\n\nb</pre></td></tr></table>",
            r#"para("| A | | --- | | ", code("a"), html("<br>"), html("<br>"), html("<br>"), code("b"), " |")"#,
            r#"table(thead(td("A")), tr(td(code("a"), html("<br>"), html("<br>"), html("<br>"), code("b"))))"#,
        );
    }

    /// Byte-identical to 2.4.0: no blank line, so no change.
    #[test]
    fn code_block_cells_without_blank_lines_are_byte_identical() {
        let one = mdka_convert(
            "<table><tr><th>A</th></tr><tr><td><pre>x</pre></td></tr></table>",
            &mdka::options::ConversionOptions::default(),
        );
        assert_eq!(one, "| A |\n| --- |\n| `x` |\n");
        let two = mdka_convert(
            "<table><tr><th>A</th></tr><tr><td><pre>a\nb</pre></td></tr></table>",
            &mdka::options::ConversionOptions::default(),
        );
        assert_eq!(two, "| A |\n| --- |\n| `a`<br>`b` |\n");
    }

    // A heading contributes its text plain -- bolding it would invent
    // emphasis the source never had (RFC 037's own defect, RFC 008 §4.1).
    #[test]
    fn heading_in_cell_becomes_plain_text() {
        check(
            "<table><tr><th>A</th></tr><tr><td><h2>Title</h2></td></tr></table>",
            r#"para("| A | | --- | | Title |")"#,
            r#"table(thead(td("A")), tr(td("Title")))"#,
        );
    }

    // A blockquote contributes its text without the `>` -- `> quoted` in a
    // cell would be literal text, not a quote (RFC 008 §4.1).
    #[test]
    fn blockquote_in_cell_drops_the_marker() {
        check(
            "<table><tr><th>A</th></tr><tr><td><blockquote>quoted</blockquote></td></tr></table>",
            r#"para("| A | | --- | | quoted |")"#,
            r#"table(thead(td("A")), tr(td("quoted")))"#,
        );
    }

    // Pipe escaping still holds once F1 has produced flattened, non-trivial
    // cell content (criterion 4 restated for `008b`'s own new shapes).
    #[test]
    fn pipe_in_list_item_in_cell_is_escaped() {
        check(
            "<table><tr><th>A</th></tr><tr><td><ul><li>a | b</li></ul></td></tr></table>",
            r#"para("| A | | --- | | - a | b |")"#,
            r#"table(thead(td("A")), tr(td("- a | b")))"#,
        );
    }

    // A container prefix must survive even when the cell content is itself
    // flattened, multi-block, structure-losing F1 output, not just a bare
    // inline cell or the `<p><p>` case `cells!` above already covers.
    #[test]
    fn flattened_list_cell_in_blockquote_still_carries_the_prefix() {
        check(
            "<blockquote><table><tr><th>A</th></tr><tr><td><ul><li>x</li><li>y</li></ul></td></tr></table></blockquote>",
            r#"quote(para("| A | | --- | | - x", html("<br>"), "- y |"))"#,
            r#"quote(table(thead(td("A")), tr(td("- x", html("<br>"), "- y"))))"#,
        );
    }
}
