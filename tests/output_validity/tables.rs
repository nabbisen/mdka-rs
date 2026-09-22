//! RFC 008 slice `008a` — GFM table support: the grid, the expressibility
//! pre-pass, and the non-welding fallback.
//!
//! A table's cells used to concatenate with no separator at all
//! (`<table><tr><th>H1</th><th>H2</th></tr>...</table>` -> `"H1H2ab"`). An
//! expressible table (one header row, first; no span; no cell holding more
//! than a single bare paragraph; no nested table; no caption) becomes a real
//! GFM table. Everything else falls back to `utils::block_kind` classifying
//! `tr`/`td`/`th`/`caption` as `Block::Paragraph` -- no special rendering at
//! all, just the same separation any other block already gets, which is
//! already enough to make `H1H2ab` impossible.
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

// ── inexpressible: the non-welding fallback (criterion 2) ──────────────────

cells! {
    no_header_row_falls_back: "<table><tr><td>a</td></tr></table>"
        => tree(r#"para("a")"#);
    two_header_rows_fall_back: "<table><tr><th>H1</th></tr><tr><th>H2</th></tr><tr><td>a</td></tr></table>"
        => tree(r#"para("H1"), para("H2"), para("a")"#);
    // The classic welding shape from RFC 008 §1 is now expressible (see
    // `classic_welding_shape_becomes_a_real_table` above); this is the
    // fallback's own equivalent -- three columns instead of two makes no
    // header/data distinction matter, but is still not expressible without
    // one: no `<th>` at all.
    header_free_multi_column_falls_back: "<table><tr><td>H1</td><td>H2</td></tr><tr><td>a</td><td>b</td></tr></table>"
        => tree(r#"para("H1"), para("H2"), para("a"), para("b")"#);
    list_in_cell_falls_back: "<table><tr><th>A</th></tr><tr><td><ul><li>x</li></ul></td></tr></table>"
        => tree(r#"para("A"), ul(li("x"))"#);
    two_paragraphs_in_cell_fall_back: "<table><tr><th>A</th></tr><tr><td><p>x</p><p>y</p></td></tr></table>"
        => tree(r#"para("A"), para("x"), para("y")"#);
    code_block_in_cell_falls_back: "<table><tr><th>A</th></tr><tr><td><pre><code>x</code></pre></td></tr></table>"
        => tree(r#"para("A"), codeblock("x")"#);
    colspan_falls_back: r#"<table><tr><th colspan="2">A</th></tr><tr><td>1</td><td>2</td></tr></table>"#
        => tree(r#"para("A"), para("1"), para("2")"#);
    rowspan_falls_back: r#"<table><tr><th>A</th><th>B</th></tr><tr><td rowspan="2">x</td><td>1</td></tr><tr><td>2</td></tr></table>"#
        => tree(r#"para("A"), para("B"), para("x"), para("1"), para("2")"#);
    // A nested table is its own table, analyzed on its own terms -- here
    // both the outer (no header row once the cell's own table is excluded
    // from its rows) and the inner (no header row either) fall back.
    nested_table_falls_back: "<table><tr><th>A</th></tr><tr><td><table><tr><td>inner</td></tr></table></td></tr></table>"
        => tree(r#"para("A"), para("inner")"#);

    // ── criterion 6/7: <caption> is not silently dropped ───────────────────

    caption_forces_fallback_but_survives: "<table><caption>My Table</caption><tr><th>H1</th></tr><tr><td>a</td></tr></table>"
        => tree(r#"para("My Table"), para("H1"), para("a")"#);

    // ── criteria 4/5 for the fallback too, not just the expressible case ───

    fallback_table_in_list_item_carries_prefix: "<ul><li><table><tr><td>a</td></tr></table></li></ul>"
        => tree(r#"ul(li("a"))"#);
    fallback_table_in_blockquote_carries_prefix: "<blockquote><table><tr><td>a</td></tr></table></blockquote>"
        => tree(r#"quote(para("a"))"#);
    fallback_table_inside_pre_writes_no_markup: "<pre><table><tr><td>a</td></tr></table></pre>"
        => tree(r#"codeblock("a")"#);
}

/// A long row's extra cell is truncated by GFM's own table parser once the
/// table has more cells than its header row (RFC 008 §3's own "Can" list) --
/// real, permanent data loss by design, not a defect, so the intent-free
/// properties `cells!` runs correctly cannot pass here. Checked directly,
/// under `Reading::Gfm` alone: the only reading this is read back as a table
/// at all (see `short_row_is_padded`, above, for the non-lossy padding half).
#[test]
fn long_row_is_truncated() {
    let html =
        r#"<table><tr><th>A</th><th>B</th></tr><tr><td>2</td><td>3</td><td>4</td></tr></table>"#;
    let md = crate::harness::mdka_convert(html, &mdka::options::ConversionOptions::default());
    let got = crate::harness::structure(&md, crate::harness::Reading::Gfm, false);
    let want = r#"table(thead(td("A"), td("B")), tr(td("2"), td("3")))"#;
    assert_eq!(got, want, "markdown: {md:?}");
}
