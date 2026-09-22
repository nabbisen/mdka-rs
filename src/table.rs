//! GFM table support (RFC 008, slices `008a` and `008b`).
//!
//! A pre-pass resolves each `<table>`'s `colspan`/`rowspan` into a
//! rectangular grid and decides whether it can become a GFM table. Three
//! blockers remain (RFC 008 §3, §4.1): more than one whole-row header, a
//! row-header table (`<th>` as each row's own first cell rather than as a
//! whole row -- RFC 008 amendment, "not a span problem"), and a nested
//! `<table>` anywhere in a cell. A `<caption>` still has no GFM syntax and
//! also sends the whole table to the fallback. Everything else is now
//! expressible: a genuinely absent header is synthesised empty (F2); a span
//! is resolved through the grid and its content repeated across every
//! covered cell, header included (F3); a cell holding block content is
//! flattened to inline, per block type (F1, `MarkdownRenderer`'s
//! `enter_cell_block`/`leave_cell_block`). An inexpressible table needs no
//! special fallback rendering at all: `utils::block_kind` classifies
//! `tr`/`td`/`th`/`caption` as `Block::Paragraph`, so the ordinary traversal
//! already keeps every cell separated (criterion 2) with every existing
//! container-prefix, `<pre>`-suppression and `<caption>`-preservation
//! guarantee that classification already carries.

use std::collections::HashMap;

use ego_tree::{NodeId, NodeRef};
use scraper::{Html, Node};

use crate::options::ConversionOptions;
use crate::renderer::MarkdownRenderer;
use crate::traversal::{self, Hints};
use crate::utils;

// ─── Grid resolution: colspan/rowspan -> a rectangle ───────────────────────

/// One authored cell's span, before grid resolution.
#[derive(Clone, Copy)]
pub(crate) struct SpanIn {
    pub(crate) colspan: usize,
    pub(crate) rowspan: usize,
}

/// Where one authored cell landed in the resolved grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Placement {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) colspan: usize,
    pub(crate) rowspan: usize,
}

/// Resolves `colspan`/`rowspan` into a rectangular grid: `rowspan` carries a
/// cell down across the following rows, occupying their columns until it is
/// exhausted, exactly as a browser lays the table out. Returns each input
/// cell's placement, row-major in the same order as `rows`, and the grid's
/// total column count.
///
/// A real algorithm, not a flag (handoff §1): `occupied[col]` tracks the row
/// index at which that column is next free, so a row with cells carried in
/// from above places its own new cells in the first columns actually free,
/// not the first columns numerically -- the case a flag cannot distinguish.
pub(crate) fn resolve_grid(rows: &[Vec<SpanIn>]) -> (Vec<Placement>, usize) {
    let mut placements = Vec::with_capacity(rows.iter().map(Vec::len).sum());
    // occupied[col]: the row index at which this column becomes free again.
    let mut occupied: Vec<usize> = Vec::new();
    let mut col_count = 0usize;

    for (r, row) in rows.iter().enumerate() {
        let mut col = 0usize;
        for span in row {
            while occupied.get(col).copied().unwrap_or(0) > r {
                col += 1;
            }
            let colspan = span.colspan.max(1);
            let rowspan = span.rowspan.max(1);
            placements.push(Placement {
                row: r,
                col,
                colspan,
                rowspan,
            });
            if occupied.len() < col + colspan {
                occupied.resize(col + colspan, 0);
            }
            for slot in &mut occupied[col..col + colspan] {
                *slot = r + rowspan;
            }
            col += colspan;
        }
        col_count = col_count.max(col).max(occupied.len());
    }
    (placements, col_count)
}

// ─── Alignment: align= and text-align: -> :--/--: ──────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Align {
    None,
    Left,
    Center,
    Right,
}

impl Align {
    /// The GFM delimiter cell for this alignment.
    fn delimiter(self) -> &'static str {
        match self {
            Align::None => "---",
            Align::Left => ":--",
            Align::Center => ":-:",
            Align::Right => "--:",
        }
    }
}

fn cell_align(elem: &scraper::node::Element) -> Align {
    if let Some(v) = elem.attr("align") {
        match v.trim().to_ascii_lowercase().as_str() {
            "left" => return Align::Left,
            "center" => return Align::Center,
            "right" => return Align::Right,
            _ => {}
        }
    }
    if let Some(style) = elem.attr("style")
        && let Some(v) = utils::style_property(style, "text-align")
    {
        return match v.as_str() {
            "left" => Align::Left,
            "center" => Align::Center,
            "right" => Align::Right,
            _ => Align::None,
        };
    }
    Align::None
}

// ─── Pre-pass: per-table expressibility (RFC 008 §3) ───────────────────────

/// One resolved header column. `node: None` is F2's synthesised empty
/// header (no `<th>` anywhere in the source) -- rendered as an empty cell,
/// never `<th>` text that does not exist.
pub(crate) struct HeaderCol<'a> {
    node: Option<NodeRef<'a, Node>>,
    align: Align,
}

pub(crate) struct ExpressibleGrid<'a> {
    header: Vec<HeaderCol<'a>>,
    /// Every row after the header, one entry per grid column. `None` is an
    /// uncovered position -- a genuinely ragged row, not a span artifact
    /// (`render` trims a row's own trailing `None`s: GFM already pads a
    /// short row, so nothing is gained by writing empty cells GFM would add
    /// on its own). A span's covered cells are `Some`, holding the
    /// *originating* cell repeated -- F3 renders it more than once, not a
    /// second, different node.
    rows: Vec<Vec<Option<NodeRef<'a, Node>>>>,
}

/// What the pre-pass decided about one `<table>`. `grid: None` covers every
/// blocker in one outcome -- the fallback does not need to know *which* one
/// applied, only that it did.
pub(crate) struct TableInfo<'a> {
    pub(crate) grid: Option<ExpressibleGrid<'a>>,
}

pub(crate) type Tables<'a> = HashMap<NodeId, TableInfo<'a>>;

fn element(node: NodeRef<'_, Node>) -> Option<&scraper::node::Element> {
    match node.value() {
        Node::Element(e) => Some(e),
        _ => None,
    }
}

fn parse_span(elem: &scraper::node::Element, attr: &str) -> usize {
    elem.attr(attr)
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1)
}

/// Every `<tr>` inside `table`, in document order, never crossing into a
/// nested `<table>` -- its rows are analyzed and rendered as their own table
/// (RFC 008 §3's "nested table" blocker is about a table appearing *inside a
/// cell* of this one, not about this function accidentally absorbing its
/// rows).
fn collect_rows(table: NodeRef<'_, Node>) -> Vec<NodeRef<'_, Node>> {
    fn walk<'a>(node: NodeRef<'a, Node>, out: &mut Vec<NodeRef<'a, Node>>) {
        for child in node.children() {
            let Some(e) = element(child) else { continue };
            match e.name() {
                "tr" => out.push(child),
                "table" => {}
                _ => walk(child, out),
            }
        }
    }
    let mut out = Vec::new();
    walk(table, &mut out);
    out
}

/// A row's own `<th>`/`<td>` children, in order.
fn collect_cells(row: NodeRef<'_, Node>) -> Vec<NodeRef<'_, Node>> {
    row.children()
        .filter(|c| element(*c).is_some_and(|e| matches!(e.name(), "th" | "td")))
        .collect()
}

fn is_all_th(cells: &[NodeRef<'_, Node>]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|c| element(*c).is_some_and(|e| e.name() == "th"))
}

/// Whether any cell in the table is a `<th>`, even though no *row* is
/// all-`<th>` (RFC 008 amendment: a row-header table, `<th>` as each row's
/// own first cell, is not a missing-header table -- it must stay
/// inexpressible, not be handed to F2). `opts` is unused here deliberately:
/// a dropped `<th>` (inside a skipped shell tag, say) still marks the
/// source as row-header-shaped, since F2's synthesised header is about
/// tables that never had `<th>` at all, not ones where it happened not to
/// render.
fn table_has_any_th(row_cells: &[Vec<NodeRef<'_, Node>>]) -> bool {
    row_cells
        .iter()
        .flatten()
        .any(|&c| element(c).is_some_and(|e| e.name() == "th"))
}

fn cell_has_nested_table(cell: NodeRef<'_, Node>) -> bool {
    cell.descendants()
        .skip(1)
        .any(|d| element(d).is_some_and(|e| e.name() == "table"))
}

fn table_has_caption(table: NodeRef<'_, Node>) -> bool {
    table
        .children()
        .any(|c| element(c).is_some_and(|e| e.name() == "caption"))
}

fn analyze_one<'a>(table: NodeRef<'a, Node>) -> TableInfo<'a> {
    let rows = collect_rows(table);
    // A `<caption>` has no GFM syntax (RFC 008 §3): send the whole table to
    // the fallback rather than emit a GFM table with the caption dropped.
    // `block_kind` still keeps its content, at its own document position
    // (module doc).
    if rows.is_empty() || table_has_caption(table) {
        return TableInfo { grid: None };
    }

    let row_cells: Vec<Vec<NodeRef<'a, Node>>> = rows.iter().map(|r| collect_cells(*r)).collect();

    // Nested table anywhere blocks the whole outer table: GFM cannot
    // express a table inside a cell at all, and it is the one shape F1
    // cannot flatten (RFC 008 §4.1: "a nested table remains a blocker").
    // Every other block shape a cell might hold is F1's, not this pre-pass's,
    // to reject.
    if row_cells
        .iter()
        .flatten()
        .any(|&c| cell_has_nested_table(c))
    {
        return TableInfo { grid: None };
    }

    // A whole-row header, in document order.
    let all_th_rows: Vec<usize> = row_cells
        .iter()
        .enumerate()
        .filter(|(_, cells)| is_all_th(cells))
        .map(|(i, _)| i)
        .collect();
    // More than one whole-row header (RFC 008 §3, unchanged by the
    // amendment): the first becomes a stray paragraph above the table.
    if all_th_rows.len() > 1 {
        return TableInfo { grid: None };
    }
    // A single whole-row header exists but is not the table's first row:
    // this slice does not attempt to reorder it, so the table is still not
    // directly expressible.
    if all_th_rows.len() == 1 && all_th_rows[0] != 0 {
        return TableInfo { grid: None };
    }
    // No whole-row header, but `<th>` appears somewhere: a row-header table
    // (`<th>` as each row's own first cell) -- the RFC 008 amendment's own
    // finding. Not a missing header for F2 to synthesise, and not a span for
    // F3 to expand; it must stay inexpressible.
    if all_th_rows.is_empty() && table_has_any_th(&row_cells) {
        return TableInfo { grid: None };
    }
    let has_header = all_th_rows == [0];

    // Resolve every row -- header included, when one exists -- through one
    // grid, so a span starting in the header and carrying into the body (or
    // vice versa) is placed consistently (RFC 008 §4.1).
    let spans: Vec<Vec<SpanIn>> = row_cells
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|&c| {
                    let e = element(c).expect("collect_cells only returns elements");
                    SpanIn {
                        colspan: parse_span(e, "colspan"),
                        rowspan: parse_span(e, "rowspan"),
                    }
                })
                .collect()
        })
        .collect();
    let (placements, col_count) = resolve_grid(&spans);
    if col_count == 0 {
        // Every row is empty (no cells at all): nothing to render.
        return TableInfo { grid: None };
    }

    // The full logical grid: `grid[r][c]` is the cell occupying that
    // position, repeated across every position its span covers (F3) --
    // `None` where nothing does (a genuinely ragged row).
    let mut grid: Vec<Vec<Option<NodeRef<'a, Node>>>> =
        vec![vec![None; col_count]; row_cells.len()];
    let mut placement_idx = 0;
    for cells in &row_cells {
        for &cell in cells {
            let p = placements[placement_idx];
            placement_idx += 1;
            let row_end = (p.row + p.rowspan).min(row_cells.len());
            let col_end = (p.col + p.colspan).min(col_count);
            for row in &mut grid[p.row..row_end] {
                for slot in &mut row[p.col..col_end] {
                    *slot = Some(cell);
                }
            }
        }
    }

    let (header, body_start) = if has_header {
        let header: Vec<HeaderCol<'a>> = grid[0]
            .iter()
            .map(|&node| HeaderCol {
                node,
                align: node
                    .and_then(element)
                    .map(cell_align)
                    .unwrap_or(Align::None),
            })
            .collect();
        (header, 1)
    } else {
        // F2: no `<th>` anywhere, so no row was consumed as a header --
        // synthesise an empty one rather than promote a data row, which
        // would silently assert a heading the source never wrote (RFC 008
        // §4).
        let header: Vec<HeaderCol<'a>> = (0..col_count)
            .map(|_| HeaderCol {
                node: None,
                align: Align::None,
            })
            .collect();
        (header, 0)
    };
    let body_rows: Vec<Vec<Option<NodeRef<'a, Node>>>> = grid[body_start..].to_vec();

    TableInfo {
        grid: Some(ExpressibleGrid {
            header,
            rows: body_rows,
        }),
    }
}

/// Analyzes every `<table>` in the document, independently -- a nested
/// table (already excluded from its parent's own rows by `collect_rows`)
/// gets its own entry here and is expressible or not on its own terms.
pub(crate) fn analyze(document: &Html) -> Tables<'_> {
    document
        .tree
        .root()
        .descendants()
        .filter(|n| element(*n).is_some_and(|e| e.name() == "table"))
        .map(|n| (n.id(), analyze_one(n)))
        .collect()
}

// ─── Rendering the expressible case ─────────────────────────────────────────

/// Captures `content`'s children as one GFM table cell: rendered through the
/// ordinary dispatch (so `<strong>`/`<a>`/`<code>`/`<img>` behave exactly as
/// they would anywhere else), with `|` escaped at the cell boundary and
/// `<br>` kept literal. A cell holding block content -- a list, a code
/// block, a heading, a blockquote, two or more paragraphs -- flattens to
/// inline via `MarkdownRenderer::enter_cell_block`/`leave_cell_block` (F1),
/// which `enter_block`/`leave_block` reach automatically once
/// `Sink::in_table_cell` is true; nothing here needs to know that happened.
fn render_cell(
    renderer: &mut MarkdownRenderer,
    content: NodeRef<'_, Node>,
    hints: &Hints,
    opts: &ConversionOptions,
    tables: &Tables<'_>,
) -> String {
    renderer.begin_cell_capture();
    traversal::drive(renderer, content.children(), hints, opts, tables);
    renderer.end_cell_capture()
}

/// [`render_cell`] for a possibly-uncovered grid position: `None` (a
/// genuinely ragged row, or F2's synthesised header) renders as an empty
/// cell, never `<th>`/`<td>` text that was never authored.
fn render_cell_opt(
    renderer: &mut MarkdownRenderer,
    content: Option<NodeRef<'_, Node>>,
    hints: &Hints,
    opts: &ConversionOptions,
    tables: &Tables<'_>,
) -> String {
    match content {
        Some(node) => render_cell(renderer, node, hints, opts, tables),
        None => String::new(),
    }
}

fn row_line(cells: &[String]) -> String {
    let mut line =
        String::with_capacity(cells.iter().map(String::len).sum::<usize>() + cells.len() * 3 + 2);
    line.push('|');
    for cell in cells {
        line.push(' ');
        line.push_str(cell);
        line.push_str(" |");
    }
    line
}

/// Renders one expressible table as GFM: the header row, the delimiter row,
/// then every data row -- each written as a single line, newline-separated
/// (never blank-line separated: a blank line would end the table), so
/// `Sink`'s own per-line container-prefix machinery covers a table nested in
/// a list item or a blockquote (criterion 4) exactly as it already does for
/// a fenced code block's lines.
pub(crate) fn render(
    renderer: &mut MarkdownRenderer,
    info: &TableInfo<'_>,
    hints: &Hints,
    opts: &ConversionOptions,
    tables: &Tables<'_>,
) {
    let Some(grid) = &info.grid else {
        return;
    };
    renderer.begin_block();

    // The header is never trimmed, even when F2 synthesised it empty: its
    // width is what fixes the table's own column count, since ragged body
    // rows are tolerated against it (RFC 008 §3's own "Can" list), not the
    // other way around.
    let header_cells: Vec<String> = grid
        .header
        .iter()
        .map(|h| render_cell_opt(renderer, h.node, hints, opts, tables))
        .collect();
    renderer.write_table_line(&row_line(&header_cells));

    let delimiters: Vec<&'static str> = grid.header.iter().map(|h| h.align.delimiter()).collect();
    let delimiter_line = row_line(
        &delimiters
            .iter()
            .map(|d| (*d).to_string())
            .collect::<Vec<_>>(),
    );
    renderer.next_table_line();
    renderer.write_table_line(&delimiter_line);

    for row in &grid.rows {
        // Trim a row's own trailing `None`s -- an uncovered position with
        // nothing after it is a genuinely ragged row, not a span artifact,
        // and GFM already pads it (§3's own "Can" list); writing the empty
        // cells ourselves would only bloat the line for no visible
        // difference. An uncovered position with *something* after it
        // (should not happen for a well-formed table, but is not this
        // renderer's place to assume) still gets an empty cell, since
        // trimming only the end keeps every row's own column positions
        // aligned with the header's.
        let last_covered = row.iter().rposition(Option::is_some);
        let cells: Vec<String> = match last_covered {
            Some(last) => row[..=last]
                .iter()
                .map(|&cell| render_cell_opt(renderer, cell, hints, opts, tables))
                .collect(),
            // A row with no cells of its own at all (malformed input): keep
            // the table well-formed with one empty cell rather than a
            // pipe-less blank line.
            None => vec![String::new()],
        };
        renderer.next_table_line();
        renderer.write_table_line(&row_line(&cells));
    }

    renderer.end_block();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(colspan: usize, rowspan: usize) -> SpanIn {
        SpanIn { colspan, rowspan }
    }

    #[test]
    fn plain_grid_no_spans() {
        let rows = vec![vec![span(1, 1), span(1, 1)], vec![span(1, 1), span(1, 1)]];
        let (placements, cols) = resolve_grid(&rows);
        assert_eq!(cols, 2);
        assert_eq!(
            placements,
            vec![
                Placement {
                    row: 0,
                    col: 0,
                    colspan: 1,
                    rowspan: 1
                },
                Placement {
                    row: 0,
                    col: 1,
                    colspan: 1,
                    rowspan: 1
                },
                Placement {
                    row: 1,
                    col: 0,
                    colspan: 1,
                    rowspan: 1
                },
                Placement {
                    row: 1,
                    col: 1,
                    colspan: 1,
                    rowspan: 1
                },
            ]
        );
    }

    /// A cell spanning two rows carries down, so the next row's own cell
    /// lands in the first column actually free, not column 0.
    #[test]
    fn rowspan_carries_down() {
        let rows = vec![vec![span(1, 2), span(1, 1)], vec![span(1, 1)]];
        let (placements, cols) = resolve_grid(&rows);
        assert_eq!(cols, 2);
        assert_eq!(
            placements,
            vec![
                Placement {
                    row: 0,
                    col: 0,
                    colspan: 1,
                    rowspan: 2
                },
                Placement {
                    row: 0,
                    col: 1,
                    colspan: 1,
                    rowspan: 1
                },
                Placement {
                    row: 1,
                    col: 1,
                    colspan: 1,
                    rowspan: 1
                },
            ]
        );
    }

    /// A colspan-2 cell pushes a later cell on the same row past both the
    /// columns it covers.
    #[test]
    fn colspan_pushes_later_cells() {
        let rows = vec![vec![span(2, 1), span(1, 1)]];
        let (placements, cols) = resolve_grid(&rows);
        assert_eq!(cols, 3);
        assert_eq!(
            placements,
            vec![
                Placement {
                    row: 0,
                    col: 0,
                    colspan: 2,
                    rowspan: 1
                },
                Placement {
                    row: 0,
                    col: 2,
                    colspan: 1,
                    rowspan: 1
                },
            ]
        );
    }

    /// Combining both: a rowspan-2 colspan-2 cell occupies a 2x2 block, and
    /// the row below must skip past all of it before placing its own cells.
    #[test]
    fn combined_span_occupies_a_block() {
        let rows = vec![vec![span(2, 2), span(1, 1)], vec![span(1, 1)]];
        let (placements, cols) = resolve_grid(&rows);
        assert_eq!(cols, 3);
        assert_eq!(
            placements,
            vec![
                Placement {
                    row: 0,
                    col: 0,
                    colspan: 2,
                    rowspan: 2
                },
                Placement {
                    row: 0,
                    col: 2,
                    colspan: 1,
                    rowspan: 1
                },
                Placement {
                    row: 1,
                    col: 2,
                    colspan: 1,
                    rowspan: 1
                },
            ]
        );
    }
}
