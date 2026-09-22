//! GFM table support (RFC 008 slice `008a`).
//!
//! A pre-pass resolves each `<table>`'s `colspan`/`rowspan` into a
//! rectangular grid and decides whether it can become a plain GFM table (RFC
//! 008 §3): a blocker -- no header row, more than one header row, a cell
//! holding more than a single bare paragraph of content, any span, a nested
//! table, or a `<caption>` -- sends it to the fallback instead. An
//! expressible table is emitted as GFM here, in [`render`]. An inexpressible
//! one needs no special handling at all: `utils::block_kind` classifies
//! `tr`/`td`/`th`/`caption` as `Block::Paragraph`, so the ordinary traversal
//! already keeps every cell separated (criterion 2) with every existing
//! container-prefix, `<pre>`-suppression and `<caption>`-preservation
//! guarantee that classification already carries. `008b` (F1 flatten-cell-
//! blocks, F2 synthesise-header, F3 expand-spans) replaces that floor with a
//! real GFM table wherever it can.

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

pub(crate) struct HeaderCol<'a> {
    node: NodeRef<'a, Node>,
    align: Align,
}

pub(crate) struct ExpressibleGrid<'a> {
    header: Vec<HeaderCol<'a>>,
    /// Every row after the header, cells in document order. Ragged -- a
    /// short or long row relative to the header -- is not a blocker (RFC 008
    /// §3's own "Can" list): GFM pads and truncates gracefully, so each row
    /// is written with however many cells it actually has.
    rows: Vec<Vec<NodeRef<'a, Node>>>,
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

/// Whether `opts` would actually render `tag` at all: content that never
/// reaches the page is not a blocker (mirrors `traversal::disposition`'s
/// skip half, which this pre-pass cannot call directly without exposing it).
fn renders(tag: &str, opts: &ConversionOptions) -> bool {
    !(utils::is_skip_tag(tag) || (opts.drop_interactive_shell && utils::is_shell_tag(tag)))
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

/// Whether `cell` holds anything GFM cannot express (RFC 008 §3): any
/// rendered block descendant, unless it is exactly one direct-child `<p>`
/// holding no block of its own and nothing else beside it (whitespace-only
/// text aside) -- that flattens to inline trivially (§2's method note).  A
/// paragraph mixed with other sibling content does NOT qualify: rendering it
/// would still put a blank line inside the cell's own captured text.
fn cell_has_blocker(cell: NodeRef<'_, Node>, opts: &ConversionOptions) -> bool {
    let blocks: Vec<NodeRef<'_, Node>> = cell
        .descendants()
        .skip(1)
        .filter(|d| {
            element(*d)
                .is_some_and(|e| renders(e.name(), opts) && utils::block_kind(e.name()).is_some())
        })
        .collect();
    match blocks.as_slice() {
        [] => false,
        [only] => {
            let is_direct_child = only.parent().map(|p| p.id()) == Some(cell.id());
            let is_paragraph = element(*only)
                .is_some_and(|e| utils::block_kind(e.name()) == Some(utils::Block::Paragraph));
            let is_only_significant_child = cell.children().all(|c| {
                c.id() == only.id() || matches!(c.value(), Node::Text(t) if t.trim().is_empty())
            });
            !(is_direct_child && is_paragraph && is_only_significant_child)
        }
        _ => true,
    }
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

fn analyze_one<'a>(table: NodeRef<'a, Node>, opts: &ConversionOptions) -> TableInfo<'a> {
    let rows = collect_rows(table);
    // A `<caption>` has no GFM syntax (RFC 008 §3): send the whole table to
    // the fallback rather than emit a GFM table with the caption dropped.
    // `block_kind` still keeps its content, at its own document position
    // (module doc).
    if rows.is_empty() || table_has_caption(table) {
        return TableInfo { grid: None };
    }

    let row_cells: Vec<Vec<NodeRef<'a, Node>>> = rows.iter().map(|r| collect_cells(*r)).collect();

    // Exactly one header row, and it is the first (RFC 008 §3: "no header
    // row" and "two header rows" are both blockers).
    let header_rows: Vec<usize> = row_cells
        .iter()
        .enumerate()
        .filter(|(_, cells)| is_all_th(cells))
        .map(|(i, _)| i)
        .collect();
    if header_rows.len() != 1 || header_rows[0] != 0 {
        return TableInfo { grid: None };
    }

    // Any span at all is a blocker for `008a` (`008b`'s F3 owns expanding
    // them) -- resolved through the real grid algorithm, not a raw attribute
    // scan, so the decision and the estimate in the review package come from
    // the same code.
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
    let (placements, _col_count) = resolve_grid(&spans);
    if placements.iter().any(|p| p.colspan > 1 || p.rowspan > 1) {
        return TableInfo { grid: None };
    }

    for cells in &row_cells {
        for &cell in cells {
            if cell_has_blocker(cell, opts) || cell_has_nested_table(cell) {
                return TableInfo { grid: None };
            }
        }
    }

    let header: Vec<HeaderCol<'a>> = row_cells[0]
        .iter()
        .map(|&node| HeaderCol {
            node,
            align: cell_align(element(node).expect("header cell is an element")),
        })
        .collect();
    let body_rows: Vec<Vec<NodeRef<'a, Node>>> = row_cells[1..].to_vec();

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
pub(crate) fn analyze<'a>(document: &'a Html, opts: &ConversionOptions) -> Tables<'a> {
    document
        .tree
        .root()
        .descendants()
        .filter(|n| element(*n).is_some_and(|e| e.name() == "table"))
        .map(|n| (n.id(), analyze_one(n, opts)))
        .collect()
}

// ─── Rendering the expressible case ─────────────────────────────────────────

/// Captures `content`'s children as one GFM table cell: rendered through the
/// ordinary dispatch (so `<strong>`/`<a>`/`<code>`/`<img>` behave exactly as
/// they would anywhere else), with `|` escaped and `<br>` kept literal. Safe
/// to call precisely because the pre-pass already proved this cell holds no
/// block content -- there is no cell renderer here that *forbids* a block,
/// only one that is never asked to hold one (see the estimate in the review
/// package for what a real one would cost).
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

    let header_cells: Vec<String> = grid
        .header
        .iter()
        .map(|h| render_cell(renderer, h.node, hints, opts, tables))
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
        let cells: Vec<String> = row
            .iter()
            .map(|&cell| render_cell(renderer, cell, hints, opts, tables))
            .collect();
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
