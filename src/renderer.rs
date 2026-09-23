use std::fmt::Write;

use crate::utils::{self, Block};

mod escape;
mod sink;

use sink::{Capture, Sink};

#[derive(Debug, Clone)]
pub enum ListKind {
    Unordered,
    Ordered { counter: usize },
}

#[derive(Debug, Clone)]
pub struct ListContext {
    pub kind: ListKind,
    /// Loose per RFC 035 §3.1: items are separated by blank lines.
    pub loose: bool,
    /// An item of this list has been opened.
    pub started: bool,
    /// The bullet an unordered item writes (`ListKind::Ordered` ignores
    /// this). `'-'` unless a prior item of this same list collided with a
    /// thematic break and had its own marker swapped to something else
    /// (RFC 036 §5.5, `036b` §2): a sibling written after that must swap the
    /// same way, or the two would no longer share one bullet and CommonMark
    /// would read them as two lists, not one.
    pub bullet: char,
}

/// The opening fence of the current `<pre>` (RFC 024 A-02).
///
/// The fence belongs to `<pre>`, not to `<code>`, so a `<pre>` without a
/// `<code>` child still opens and closes one. It is held until the first
/// content so that a `<code>` preceded only by whitespace can still name the
/// language (`<pre>\n  <code class="language-x">`).
enum Fence {
    /// Not inside `<pre>`.
    None,
    /// Inside `<pre>`, fence not written yet. Whitespace-only text seen so far
    /// is held; it is code block content, written after the fence line.
    Pending {
        held: String,
    },
    Open,
}

/// What an open `<a>` did on entering.
enum LinkState {
    /// Opened a link capture.
    Captured,
    /// Wraps block content (RFC 028 criterion 7): each run of inline content
    /// between block boundaries is linked on its own -- `## [Title](/x)`.
    /// `capturing` is true while such a run's link capture is open.
    Distributed {
        href: String,
        title: Option<String>,
        capturing: bool,
    },
    /// Inside another link, a `<pre>` or a code span: text only.
    TextOnly,
}

pub struct MarkdownRenderer {
    sink: Sink,
    list_stack: Vec<ListContext>,
    in_pre: bool,
    /// Open `<pre>` elements. A `<pre>` nested in another opens no second
    /// fence and must not close the first.
    pre_depth: usize,
    /// Inside a `<pre>`: a block element began or ended since the last text,
    /// so the next text starts a new line (RFC 024 rule 7).
    pre_block_break: bool,
    /// Inside a `<pre>`: text has been written into the code block.
    pre_has_text: bool,
    fence: Fence,
    /// Per open `<a>`.
    links: Vec<LinkState>,
    /// Per open inline `<code>`: whether it opened a code-span capture.
    code_captures: Vec<bool>,
    /// Per open `<strong>`/`<b>`/`<em>`/`<i>`: `None` if it wrote no
    /// delimiters at all (inside code, around blocks, style-negated, or
    /// collapsed into a same-class ancestor, RFC 037); `Some` records what
    /// it wrote, so leaving can close it -- or, if it turns out empty,
    /// remove it instead (RFC 037).
    emphasis: Vec<Option<EmphasisFrame>>,
    /// Per open `<del>`/`<s>` (RFC 009 §4.2): `None` if it wrote no `~~` at
    /// all (inside code or around blocks -- the same two reasons emphasis
    /// suppresses its own delimiters); `Some` records what
    /// `Sink::strikethrough_open` wrote, so leaving can close it or, empty,
    /// remove it instead.
    strikethrough: Vec<Option<sink::StrikeMark>>,
    /// F1 (RFC 008 slice `008b`): a table cell's flattened list nesting --
    /// each open list's kind and next item number, indented two literal
    /// spaces per level, since a cell has no real lines for the
    /// container-prefix machinery to reapply to. Reset per cell.
    cell_lists: Vec<CellList>,
    /// F1: whether the content just written was a flattened list item's own
    /// marker (`- `/`N. `, indented) -- the item's first block (commonly a
    /// bare run of text, but a `<p>` is not unusual) follows it directly,
    /// with no `<br>` in between. Cleared by the next block boundary,
    /// whichever writes one. Reset per cell.
    cell_after_marker: bool,
    /// F1: an open `<pre>` inside a table cell, accumulating its raw text
    /// until it closes, when it becomes one code span per source line,
    /// `<br>`-joined -- not the normal fenced-block machinery, which
    /// assumes real lines to put the fence and the content on. Reset per
    /// cell.
    cell_pre: Option<String>,
}

/// One open list, flattened inside a table cell (F1): its kind and, for an
/// ordered list, the next item number.
struct CellList {
    ordered: bool,
    counter: usize,
}

/// `strong`/`b` vs `em`/`i`, by the delimiter they write -- the rule is
/// about the delimiter, not the tag name (RFC 037 §1.1 B: `<em><i>` collapses
/// the same as `<em><em>`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum EmphasisClass {
    Bold,
    Italic,
}

struct EmphasisFrame {
    class: EmphasisClass,
    mark: sink::EmphasisMark,
}

impl MarkdownRenderer {
    pub fn new(capacity: usize) -> Self {
        Self {
            sink: Sink::new(capacity),
            list_stack: Vec::with_capacity(8),
            in_pre: false,
            pre_depth: 0,
            pre_block_break: false,
            pre_has_text: false,
            fence: Fence::None,
            links: Vec::new(),
            code_captures: Vec::new(),
            emphasis: Vec::new(),
            strikethrough: Vec::new(),
            cell_lists: Vec::new(),
            cell_after_marker: false,
            cell_pre: None,
        }
    }

    /// Inside a `<pre>` -- with or without `<code>` -- or an inline code span,
    /// child elements contribute text only: no emphasis delimiters, image or
    /// link syntax, and an image contributes nothing (RFC 024 criteria 3 and 9,
    /// amended 2026-09-17). Markdown has no markup inside code. A table cell's
    /// own flattened `<pre>` (F1, RFC 008 `008b`) is the same rule in a
    /// different destination: `cell_pre` accumulates raw text the same way,
    /// so nested markup must be suppressed here too.
    fn in_code(&self) -> bool {
        self.in_pre || self.sink.in_code_span() || self.cell_pre.is_some()
    }

    /// Whether a `<pre>` is currently open (RFC 008): an expressible table
    /// found inside one is not rendered specially -- its `tr`/`td`/`th`
    /// fall through to the ordinary `Block::Paragraph` dispatch, which
    /// `enter_block`'s own `in_pre` guard already empties of markup (RFC 024
    /// rule 7), the same as any other block inside `<pre>`.
    pub(crate) fn in_pre(&self) -> bool {
        self.in_pre
    }

    /// Whether a GFM table cell's content is being captured (RFC 008):
    /// `<br>` there is literal `<br>`, not a hard break.
    fn in_table_cell(&self) -> bool {
        self.sink.in_table_cell()
    }

    pub(crate) fn begin_block(&mut self) {
        // The blockquote prefix is not written here; the sink writes it
        // before the next content byte.
        self.ensure_newlines(2);
    }

    pub(crate) fn end_block(&mut self) {
        self.ensure_newlines(2);
    }

    // ─── Tables (RFC 008) ────────────────────────────────────────────────

    /// Starts capturing one GFM table cell's content: rendered the same as
    /// any other inline content -- escaping, `<br>`'s literal form,
    /// `<strong>`/`<a>`/`<code>` all behave exactly as they would anywhere
    /// else.
    pub(crate) fn begin_cell_capture(&mut self) {
        self.sink.begin_capture(Capture::Cell);
        self.cell_lists.clear();
        self.cell_after_marker = false;
        self.cell_pre = None;
    }

    /// Ends the innermost cell capture and returns what it collected, with
    /// every `|` escaped (RFC 008 addendum A). Escaped here, once, over the
    /// whole assembled cell -- not while writing -- because a nested capture
    /// (a code span, a link's text or destination, an image's alt) splices
    /// its own already-rendered string in whole, bypassing the per-character
    /// escaping a cell's own direct text already goes through; the cell
    /// boundary is the only point every source of a literal `|` passes
    /// through regardless of which path produced it.
    pub(crate) fn end_cell_capture(&mut self) -> String {
        let content = self
            .sink
            .end_capture()
            .map(|(_, content, _)| content)
            .unwrap_or_default();
        escape::escape_table_cell_pipes(&content)
    }

    /// Writes one already-assembled GFM row or delimiter line as raw markup
    /// -- the cells inside it are already escaped, from capture.
    pub(crate) fn write_table_line(&mut self, line: &str) {
        self.sink.markup_closed(line);
    }

    /// Requests the single newline between two table lines. Never a blank
    /// line: unlike every other block boundary in this renderer, one here
    /// would end the table (GFM has no concept of a table with a gap in it).
    pub(crate) fn next_table_line(&mut self) {
        self.ensure_newlines(1);
    }

    /// A block boundary for content whose own element contributed nothing:
    /// an unwrapped wrapper (`<div>`, `<section>`, `<article>`, `<main>`
    /// with `unwrap_unknown_wrappers` on) still separates its children from
    /// whatever surrounds them the way it would have as a rendered
    /// `Block::Paragraph`, even though the tag itself is never entered or
    /// left (RFC 036 §5.2, slice `036d`).
    ///
    /// It has to take the *same path a rendered block takes*, and that path
    /// starts with the table-cell check (`enter_block`/`leave_block`), before
    /// `in_pre` and before `begin_block`. `begin_block` alone is only what is
    /// left after those two guards -- reusing it directly wrote a real blank
    /// line into a GFM row, so an unwrapped wrapper inside a cell split the
    /// row in two (2.4.2; the comment that used to stand here said this
    /// reused "the same path any other block already takes", which was the
    /// bug). In a cell the separator is the cell's own `<br>`; inside a
    /// cell's flattened `<pre>` a wrapper contributes nothing, mirroring
    /// `enter_cell_block`.
    ///
    /// Inside a `<pre>`, a wrapper contributes text only, the same as any
    /// other block element would (RFC 024 rule 7): mirrors `enter_block`'s
    /// and `leave_block`'s own `in_pre` guard exactly, rather than let an
    /// unwrapped wrapper reach `ensure_newlines` unguarded and write a real
    /// blank line into what must stay verbatim content.
    pub fn begin_unwrapped_separator(&mut self) {
        if self.in_table_cell() {
            if self.cell_pre.is_none() {
                self.cell_block_separator();
            }
            return;
        }
        if self.in_pre {
            self.pre_block_break = true;
            return;
        }
        self.begin_block();
    }

    /// See [`begin_unwrapped_separator`](Self::begin_unwrapped_separator).
    /// In a table cell nothing is written on the way out: like
    /// `leave_cell_block`, the `<br>` belongs to the *next* block's entry.
    pub fn end_unwrapped_separator(&mut self) {
        if self.in_table_cell() {
            return;
        }
        if self.in_pre {
            self.pre_block_break = true;
            return;
        }
        self.end_block();
    }

    /// Every block boundary goes through here: a distributed link's current
    /// run ends before the line does.
    fn ensure_newlines(&mut self, count: usize) {
        self.close_distributed_link();
        self.sink.ensure_newlines(count);
    }

    /// Before inline content: if the nearest enclosing link wraps blocks and
    /// has no run open, open one (RFC 028 criterion 7). Code holds text only,
    /// so nothing is linked inside it. Returns whether a run was opened.
    fn open_distributed_link(&mut self) -> bool {
        if self.in_code() {
            return false;
        }
        if let Some(LinkState::Distributed {
            href,
            title,
            capturing,
        }) = self
            .links
            .iter_mut()
            .rev()
            .find(|l| !matches!(l, LinkState::TextOnly))
            && !*capturing
        {
            self.sink.begin_capture(Capture::Link {
                href: href.clone(),
                title: title.clone(),
            });
            *capturing = true;
            return true;
        }
        false
    }

    /// Ends a distributed link's open run, writing `[run](href)` -- or nothing
    /// for a run with no text and no image.
    fn close_distributed_link(&mut self) {
        let Some(LinkState::Distributed { capturing, .. }) = self
            .links
            .iter_mut()
            .rev()
            .find(|l| !matches!(l, LinkState::TextOnly))
        else {
            return;
        };
        if !*capturing {
            return;
        }
        *capturing = false;
        if let Some((Capture::Link { href, title }, text, trailing)) = self.sink.end_capture() {
            let rendered =
                (!text.trim().is_empty()).then(|| link_syntax(&text, &href, title.as_deref()));
            self.sink.splice(rendered.as_deref(), trailing);
        }
    }

    /// Writes the pending opening fence of the current `<pre>`, if any.
    ///
    /// The fence starts its line; whitespace held before it is part of the
    /// code block's text and follows the fence line (RFC 024 rule 6). Written
    /// in front of the fence, four or more spaces would turn the fence into an
    /// indented code block's text, and the closing fence would then open a
    /// block that swallows the rest of the document.
    fn open_fence(&mut self, lang: &str) {
        if let Fence::Pending { held } = std::mem::replace(&mut self.fence, Fence::Open) {
            // An info string cannot hold a backtick: the hint is dropped
            // rather than break the fence (RFC 010 §3.2).
            let lang = if lang.contains('`') { "" } else { lang };
            self.sink.open_fence(lang);
            self.sink.code_block_content(&held);
        }
    }

    // ─── Text ──────────────────────────────────────────────────────────────

    pub fn process_text(&mut self, text: &str) {
        // F1 (RFC 008 `008b`): a table cell's own flattened `<pre>`
        // accumulates its text raw, exactly like the fenced-block case just
        // below, but into a plain buffer rather than the sink -- there is no
        // fence, and the whole thing becomes one code span per line only
        // once it closes (`leave_cell_block`).
        if let Some(held) = &mut self.cell_pre {
            held.push_str(text);
            return;
        }
        // A list item's marker only protects the separator immediately
        // before its OWN first content -- once that content is real text
        // (not whitespace), the protection is spent, so a block entered
        // afterwards (a nested list's first item, in particular) gets its
        // own `<br>` like any later sibling would, rather than running
        // together with this item's text on one visual line.
        if self.cell_after_marker && !text.trim().is_empty() {
            self.cell_after_marker = false;
        }
        if self.in_pre {
            if let Fence::Pending { held } = &mut self.fence {
                if text.trim().is_empty() {
                    // Held whitespace is the code block's text too (rule 6).
                    if std::mem::take(&mut self.pre_block_break)
                        && !held.is_empty()
                        && !text.starts_with('\n')
                        && !held.ends_with('\n')
                    {
                        held.push('\n');
                    }
                    if !text.is_empty() {
                        self.pre_has_text = true;
                    }
                    held.push_str(text);
                    return;
                }
                self.open_fence("");
            }
            // A block boundary inside the <pre> is one line break, unless the
            // text already provides one; none before the first text.
            if std::mem::take(&mut self.pre_block_break)
                && self.pre_has_text
                && !text.starts_with('\n')
                && !self.sink.ends_with_newline()
            {
                self.sink.code_block_content("\n");
            }
            if !text.is_empty() {
                self.pre_has_text = true;
            }
            self.sink.code_block_content(text);
            return;
        }
        if !text.trim().is_empty() {
            let at_line_start = self.sink.at_line_start();
            if self.open_distributed_link() && at_line_start {
                // A run opened at the start of a line: whitespace before its
                // first word would be dropped there, so it is not link text.
                self.sink.text(text.trim_start());
                return;
            }
        }
        self.sink.text(text);
    }

    // ─── Id anchors ────────────────────────────────────────────────────────

    /// Emits an `<a id="…"></a>` anchor for an element with a non-empty `id`
    /// when `preserve_ids` is on. It is normally placed as the element's
    /// leading content -- after a heading's `# `, a list item's `- `, a
    /// blockquote's `> ` -- so `enter_element` calls it after the tag's arm.
    ///
    /// The exception is the tags whose own arm opens a capture or a code
    /// block (`a`, `code`, `pre`): called afterwards, their own `id` would be
    /// suppressed by the guard below. For those, `enter_element` calls it
    /// before the arm, with only the inherited state. An anchor is never
    /// written into a capture or a code block, where it would corrupt the
    /// content.
    pub(crate) fn emit_id_anchor(&mut self, elem: &scraper::node::Element, preserve_ids: bool) {
        if !preserve_ids || self.sink.is_capturing() || self.in_pre {
            return;
        }
        let Some(id) = elem.attr("id") else { return };
        if id.is_empty() {
            return;
        }
        let mut anchor = String::with_capacity(id.len() + 10);
        anchor.push_str("<a id=\"");
        for c in id.chars() {
            match c {
                '&' => anchor.push_str("&amp;"),
                '"' => anchor.push_str("&quot;"),
                other => anchor.push(other),
            }
        }
        anchor.push_str("\"></a>");
        self.sink.id_anchor(&anchor);
    }

    // ─── Enter ─────────────────────────────────────────────────────────────

    /// `wraps_blocks`: the element has a rendered block among its descendants.
    /// `loose_list`: the element is a list that is loose (RFC 035 §3.1).
    /// `needs_disambiguation`: the element is a list whose own first item is
    /// empty, so entering it nested may need a blank line first (RFC 038).
    /// `checkbox`: the element is a `<li>` whose own first rendered child is
    /// an `<input type="checkbox">` -- `Some(checked)` writes a task marker
    /// instead of a plain one (RFC 009 §4.2); meaningless for anything else.
    /// All four are computed once per document by the traversal.
    pub fn enter_element(
        &mut self,
        elem: &scraper::node::Element,
        preserve_ids: bool,
        wraps_blocks: bool,
        loose_list: bool,
        needs_disambiguation: bool,
        checkbox: Option<bool>,
    ) {
        let tag = elem.name();
        // Tags whose own arm opens a capture or a code block: anchor first.
        // Adding a tag that sets either guard means adding it here too
        // (RFC 006 Slice D; tests/anchor_drift_guard.rs).
        let anchor_before = matches!(tag, "a" | "code" | "pre");
        if anchor_before {
            self.emit_id_anchor(elem, preserve_ids);
        }
        if let Some(block) = utils::block_kind(tag) {
            self.enter_block(block, elem, loose_list, needs_disambiguation, checkbox);
        } else {
            self.enter_inline(tag, elem, wraps_blocks);
        }
        if !anchor_before {
            self.emit_id_anchor(elem, preserve_ids);
        }
    }

    fn enter_block(
        &mut self,
        block: Block,
        elem: &scraper::node::Element,
        loose_list: bool,
        needs_disambiguation: bool,
        checkbox: Option<bool>,
    ) {
        // A GFM cell holds inline content only (F1, RFC 008 `008b`): a block
        // that reaches here is flattened, never given the normal container/
        // prefix/blank-line treatment, which assumes real lines a cell does
        // not have.
        if self.in_table_cell() {
            self.enter_cell_block(block, elem, checkbox);
            return;
        }
        // Inside a <pre>, a block element -- a nested <pre> too -- contributes
        // its text only: no container, prefix, marker or blank line, and its
        // boundaries are line breaks (RFC 024 rule 7).
        if self.in_pre {
            self.pre_block_break = true;
            if block != Block::Pre {
                return;
            }
        }
        match block {
            Block::Heading(level) => {
                self.begin_block();
                let mut marker = "#".repeat(level);
                marker.push(' ');
                self.sink.heading_marker(&marker);
            }
            Block::Paragraph => self.begin_block(),
            Block::UnorderedList => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                } else if needs_disambiguation {
                    self.sink.force_disambiguating_blank_line();
                }
                self.list_stack.push(ListContext {
                    kind: ListKind::Unordered,
                    loose: loose_list,
                    started: false,
                    bullet: '-',
                });
            }
            Block::OrderedList => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                } else if needs_disambiguation {
                    self.sink.force_disambiguating_blank_line();
                }
                let start = elem
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.list_stack.push(ListContext {
                    kind: ListKind::Ordered { counter: start },
                    loose: loose_list,
                    started: false,
                    bullet: '-', // unused: an ordered item's marker ignores it
                });
            }
            Block::ListItem => {
                // A nested list's indent is not written here: the enclosing
                // item's continuation prefix (its content column) is, by the
                // sink.
                let loose = self.list_stack.last().is_some_and(|ctx| ctx.loose);
                let started = self.list_stack.last().is_some_and(|ctx| ctx.started);
                self.ensure_newlines(if loose { 2 } else { 1 });
                if loose && started {
                    self.sink.loose_item_separator();
                }
                let mut marker = String::new();
                if let Some(ctx) = self.list_stack.last_mut() {
                    ctx.started = true;
                    let bullet = ctx.bullet;
                    match &mut ctx.kind {
                        ListKind::Unordered => {
                            marker.push(bullet);
                            marker.push(' ');
                        }
                        ListKind::Ordered { counter } => {
                            let n = *counter;
                            *counter += 1;
                            push_usize(&mut marker, n);
                            marker.push_str(". ");
                        }
                    }
                }
                // A task marker (RFC 009 §4.2) follows the bullet/number
                // exactly the way it would follow "- "/"N. " if typed by
                // hand -- `checkbox` is only ever `Some` for a `<li>` whose
                // own first rendered child was the checkbox, so this cannot
                // fire on a later, unrelated one.
                if let Some(checked) = checkbox {
                    marker.push_str(if checked { "[x] " } else { "[ ] " });
                }
                self.close_distributed_link();
                self.sink.item_marker(&marker, !loose);
            }
            Block::Quote => {
                self.begin_block();
                self.sink.enter_blockquote();
            }
            Block::Pre => {
                self.pre_depth += 1;
                if self.pre_depth == 1 {
                    self.begin_block();
                    self.in_pre = true;
                    self.fence = Fence::Pending {
                        held: String::new(),
                    };
                }
            }
            Block::Rule => {
                self.begin_block();
                self.sink.thematic_break();
                self.end_block();
            }
        }
    }

    fn enter_inline(&mut self, tag: &str, elem: &scraper::node::Element, wraps_blocks: bool) {
        match tag {
            // A cell's own flattened `<pre>` (F1) has no fence and opens no
            // capture: its `<code>`, like anything else inside it, is text
            // that `process_text`'s `cell_pre` redirect already collects.
            "code" if self.cell_pre.is_some() => {}
            "code" if self.in_pre => {
                // One <pre>, one code block: only the first thing inside the
                // <pre> opens the fence, and only a <code> that opens it names
                // the language. A later <code> adds its text to the same block.
                if matches!(self.fence, Fence::Pending { .. }) {
                    let lang = elem
                        .attr("class")
                        .and_then(|cls| utils::extract_code_lang(Some(cls)))
                        .unwrap_or("");
                    self.open_fence(lang);
                }
            }
            "code" => {
                // A code span around blocks writes no backticks and keeps the
                // blocks (RFC 028 criterion 6).
                let capture = !self.sink.in_code_span() && !wraps_blocks;
                if capture {
                    let _ = self.open_distributed_link();
                    self.sink.begin_capture(Capture::CodeSpan);
                }
                self.code_captures.push(capture);
            }
            "strong" | "b" | "em" | "i" => {
                // No delimiters inside code, around blocks (RFC 028 criterion
                // 1), when the element's own style negates the emphasis
                // (criterion 4), or when a same-class ancestor is already
                // open (RFC 037: italic in italic is italic, bold in bold is
                // bold) -- this element's content flows through as if it
                // were not there, the same as any of the other cases.
                let class = if matches!(tag, "strong" | "b") {
                    EmphasisClass::Bold
                } else {
                    EmphasisClass::Italic
                };
                let write = !self.in_code()
                    && !wraps_blocks
                    && !utils::emphasis_negated_by_style(tag, elem.attr("style"));
                let collapsed = write && self.emphasis.iter().flatten().any(|f| f.class == class);
                let frame = (write && !collapsed).then(|| {
                    let _ = self.open_distributed_link();
                    // `emphasis_open` flushes any pending space itself, as
                    // part of what it can undo if this span turns out empty
                    // (RFC 037 finding C) -- not done here first.
                    // A same-class ancestor aside, an *open*, differently
                    // classed ancestor immediately adjacent (nothing written
                    // since its own delimiter) risks the opposite problem:
                    // `**` then `*` (or vice versa) concatenate into one run
                    // that reparses the same way regardless of which order
                    // wrote it. `<em><strong>` already reparses correctly as
                    // plain `*`/`**`; only `<strong><em>` does not, so only
                    // that one direction swaps to `_` here.
                    let start = self.sink.dest_len();
                    let bold_ancestor = self
                        .emphasis
                        .iter()
                        .rev()
                        .find_map(|f| f.as_ref())
                        .filter(|f| f.class == EmphasisClass::Bold);
                    let touching_bold_ancestor = class == EmphasisClass::Italic
                        && bold_ancestor.is_some_and(|f| f.mark.end() == start);
                    // RFC 037 addendum: swapping to `_` only helps if `**`
                    // can still open where it is -- CommonMark requires a
                    // delimiter run followed by punctuation (here, the `_`
                    // this swap is about to write) to be preceded by
                    // whitespace, punctuation, or nothing. Preceded by a
                    // letter or digit instead, `**` cannot open at all, and
                    // the bold is lost outright: worse than the
                    // order-inverted bug this swap exists to fix (RFC 037
                    // review, 2026-09-22). Checked against the character
                    // before the Bold ancestor's own opening delimiter --
                    // already-written history, not a lookahead. What follows
                    // the whole nested span's *close* is not knowable here;
                    // `Sink::emphasis_close` confirms that half once it can.
                    let intraword_candidate = touching_bold_ancestor
                        && bold_ancestor.is_some_and(|f| {
                            escape::class(self.sink.char_before(f.mark.start()))
                                != escape::Class::Word
                        });
                    let delimiter = if intraword_candidate {
                        "_"
                    } else if class == EmphasisClass::Bold {
                        "**"
                    } else {
                        "*"
                    };
                    let (_, mark) = self.sink.emphasis_open(delimiter, intraword_candidate);
                    EmphasisFrame { class, mark }
                });
                self.emphasis.push(frame);
            }
            "del" | "s" => {
                // No delimiters inside code or around blocks (RFC 009 §4.2),
                // the same two reasons `strong`/`em` suppress theirs -- GFM
                // strikethrough is inline syntax, and `~~` spanning a block
                // boundary is no more valid than `**` would be. Nested
                // `<del>`/`<s>` collapses to one level too (RFC 037 §3,
                // review-found for this delimiter, 2026-09-23): unlike
                // `**`/`*` there is only one `~~` class at all, so ANY
                // already-open span -- `<del>` or `<s>`, RFC 037's own point
                // that the rule is about the delimiter, not the tag --
                // suppresses this one.
                let write = !self.in_code() && !wraps_blocks;
                let collapsed = write && self.strikethrough.iter().any(Option::is_some);
                let frame = (write && !collapsed).then(|| {
                    let _ = self.open_distributed_link();
                    self.sink.strikethrough_open()
                });
                self.strikethrough.push(frame);
            }
            // A `<sup>`/`<sub>` inside code (a real `<pre>`, a cell's own
            // flattened one, or an inline code span) contributes its text
            // only, unmapped and undecorated -- the same rule any other
            // markup suppresses inside code by (RFC 009 §4.3 has no mapping
            // rule to apply there; the content is already verbatim).
            "sup" | "sub" if self.in_code() => {}
            "sup" | "sub" => {
                let _ = self.open_distributed_link();
                self.sink.begin_capture(Capture::SupSub);
            }
            "a" => {
                let href = elem.attr("href").unwrap_or("").to_string();
                let title = elem.attr("title").map(str::to_string);
                let in_link = self.links.iter().any(|l| !matches!(l, LinkState::TextOnly));
                let state = if in_link || self.in_code() {
                    LinkState::TextOnly
                } else if wraps_blocks {
                    LinkState::Distributed {
                        href,
                        title,
                        capturing: false,
                    }
                } else {
                    self.sink.begin_capture(Capture::Link { href, title });
                    LinkState::Captured
                };
                self.links.push(state);
            }
            "img" if !self.in_code() => {
                let image = Sink::image_syntax(
                    elem.attr("alt").unwrap_or(""),
                    elem.attr("src").unwrap_or(""),
                    elem.attr("title"),
                );
                let _ = self.open_distributed_link();
                self.sink.flush_space();
                self.sink.markup_closed(&image);
            }
            // Inside code a <br> is text, not a Markdown hard break, whose
            // trailing spaces would become code (RFC 024 rule 8): one line
            // break in a <pre>, one space in a code span -- including a
            // table cell's own flattened <pre> (F1), a real line break in
            // the code its buffer accumulates, checked before the general
            // in_pre case since neither `self.in_pre` nor `self.fence` is
            // set for it. Inside a GFM table cell otherwise, a hard break's
            // own newline would end the cell -- and survives as inline HTML
            // instead (RFC 008 §3's own "Can" list).
            "br" if self.in_pre => self.process_text("\n"),
            "br" if self.cell_pre.is_some() => self.process_text("\n"),
            "br" if self.sink.in_code_span() => self.sink.text(" "),
            "br" if self.in_table_cell() => {
                self.sink.flush_space();
                self.sink.markup("<br>");
            }
            "br" => {
                // The next content line gets the blockquote prefix, if any.
                self.sink.hard_break();
            }
            _ => {}
        }
    }

    // ─── Leave ─────────────────────────────────────────────────────────────

    pub fn leave_element(&mut self, elem: &scraper::node::Element) {
        let tag = elem.name();
        if let Some(block) = utils::block_kind(tag) {
            self.leave_block(block);
            return;
        }
        match tag {
            "code" if self.cell_pre.is_some() => {}
            "code" if !self.in_pre => {
                if self.code_captures.pop() == Some(true)
                    && let Some((_, content, trailing)) = self.sink.end_capture()
                {
                    // A code span with no content writes nothing.
                    let rendered = (!content.is_empty()).then(|| escape::code_span(&content));
                    self.sink.splice(rendered.as_deref(), trailing);
                }
            }
            "strong" | "b" | "em" | "i" => {
                if let Some(Some(frame)) = self.emphasis.pop() {
                    self.sink.emphasis_close_or_remove(frame.mark);
                }
            }
            "del" | "s" => {
                if let Some(Some(mark)) = self.strikethrough.pop() {
                    self.sink.strikethrough_close_or_remove(mark);
                }
            }
            "sup" | "sub" if self.in_code() => {}
            "sup" | "sub" => {
                if let Some((_, content, trailing)) = self.sink.end_capture() {
                    // Empty content maps vacuously (RFC 037's own rule for
                    // an empty `<strong>`: nothing to write). Otherwise,
                    // every character mapping wins; falling short of that,
                    // the captured content is spliced back exactly as
                    // captured -- unchanged, the citation-marker case above
                    // everything else here (RFC 009 §5.4).
                    let rendered = if content.is_empty() {
                        None
                    } else {
                        utils::map_script(&content, tag == "sup").or(Some(content))
                    };
                    self.sink.splice(rendered.as_deref(), trailing);
                }
            }
            "a" => match self.links.pop() {
                Some(LinkState::Captured) => {
                    if let Some((Capture::Link { href, title }, text, trailing)) =
                        self.sink.end_capture()
                    {
                        // A link with no text and no image writes nothing
                        // (RFC 024 criterion 7): `[](/x)` renders as nothing.
                        let rendered = (!text.trim().is_empty())
                            .then(|| link_syntax(&text, &href, title.as_deref()));
                        self.sink.splice(rendered.as_deref(), trailing);
                    }
                }
                Some(state @ LinkState::Distributed { .. }) => {
                    // Close the last run while this link is still the nearest.
                    self.links.push(state);
                    self.close_distributed_link();
                    self.links.pop();
                }
                Some(LinkState::TextOnly) | None => {}
            },
            _ => {}
        }
    }

    fn leave_block(&mut self, block: Block) {
        if self.in_table_cell() {
            self.leave_cell_block(block);
            return;
        }
        if self.in_pre && (block != Block::Pre || self.pre_depth > 1) {
            self.pre_block_break = true;
            if block != Block::Pre {
                return;
            }
        }
        match block {
            Block::Heading(_) => {
                self.end_block();
                // A heading with no real content (all whitespace) leaves no
                // dangling strip request past its own end.
                self.sink.end_leading_strip();
            }
            Block::Paragraph => self.end_block(),
            Block::UnorderedList | Block::OrderedList => {
                self.list_stack.pop();
                // A nested list ending inside an item is a block boundary too:
                // content after it must not continue the sublist's last item.
                // Inside a tight item the sink keeps this to one newline.
                self.end_block();
            }
            Block::ListItem => {
                // End a link run before the item's prefix goes away.
                self.close_distributed_link();
                // A swap to dodge a thematic break (RFC 036 §5.5) must carry
                // to this same list's later items too, or they would no
                // longer share this item's bullet and CommonMark would read
                // two lists where the source had one (`036b` §2).
                if let Some(bullet) = self.sink.leave_item()
                    && let Some(ctx) = self.list_stack.last_mut()
                {
                    ctx.bullet = bullet;
                }
                self.ensure_newlines(1);
            }
            Block::Quote => {
                // End a link run before the quote's prefix goes away.
                self.close_distributed_link();
                self.sink.leave_blockquote();
                self.end_block();
            }
            Block::Pre => {
                self.pre_depth = self.pre_depth.saturating_sub(1);
                if self.pre_depth > 0 {
                    return;
                }
                // An empty <pre> still gets a balanced, empty code block.
                self.open_fence("");
                if !self.sink.ends_with_newline() {
                    self.sink.code_block_content("\n");
                }
                self.sink.close_fence();
                self.in_pre = false;
                self.pre_block_break = false;
                self.pre_has_text = false;
                self.fence = Fence::None;
                self.end_block();
            }
            Block::Rule => {}
        }
    }

    // ─── Table cells: flattening blocks to inline (F1, RFC 008 `008b`) ─────

    /// A literal `<br>` before the next flattened block's content, unless
    /// the cell holds nothing yet, or what was just written is this same
    /// item's own marker (`cell_after_marker`) -- a fresh item's first block
    /// follows its `- `/`N. ` directly, the way plain text after it already
    /// would. Consumes `cell_after_marker` unconditionally: it protects
    /// exactly one following block, never the one after that (a sibling
    /// item, or this cell's very next block, still gets its own separator).
    fn cell_block_separator(&mut self) {
        if self.sink.dest_len() > 0 && !self.cell_after_marker {
            self.sink.flush_space();
            self.sink.markup("<br>");
        }
        self.cell_after_marker = false;
    }

    /// A GFM cell holds inline content only. Each block keeps its words and
    /// gives up only the structure (RFC 008 §4.1's own table): a nested
    /// `<table>` is not reachable here at all -- the pre-pass already sent
    /// the whole outer table to the fallback before any cell is rendered.
    fn enter_cell_block(
        &mut self,
        block: Block,
        elem: &scraper::node::Element,
        checkbox: Option<bool>,
    ) {
        // Inside this cell's own flattened `<pre>`, a further block
        // contributes text only -- mirrors the outer `in_pre` guard in
        // `enter_block`, for the same reason: a fenced block's content is
        // exactly its text, whatever markup momentarily wraps a line of it.
        if self.cell_pre.is_some() && block != Block::Pre {
            return;
        }
        match block {
            Block::ListItem => {
                self.cell_block_separator();
                let indent = "  ".repeat(self.cell_lists.len().saturating_sub(1));
                let mut marker = match self.cell_lists.last_mut() {
                    Some(CellList {
                        ordered: true,
                        counter,
                    }) => {
                        let n = *counter;
                        *counter += 1;
                        let mut m = String::new();
                        push_usize(&mut m, n);
                        m.push_str(". ");
                        m
                    }
                    _ => "- ".to_string(),
                };
                // A task marker (RFC 009 §4.2) works the same way inside a
                // flattened cell as it does in the main document.
                if let Some(checked) = checkbox {
                    marker.push_str(if checked { "[x] " } else { "[ ] " });
                }
                self.sink.markup(&format!("{indent}{marker}"));
                self.cell_after_marker = true;
            }
            Block::UnorderedList => {
                self.cell_lists.push(CellList {
                    ordered: false,
                    counter: 0,
                });
            }
            Block::OrderedList => {
                let start = elem
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.cell_lists.push(CellList {
                    ordered: true,
                    counter: start,
                });
            }
            Block::Pre => {
                self.cell_block_separator();
                self.cell_pre = Some(String::new());
            }
            Block::Heading(_) | Block::Paragraph | Block::Quote | Block::Rule => {
                self.cell_block_separator();
            }
        }
    }

    fn leave_cell_block(&mut self, block: Block) {
        if self.cell_pre.is_some() && block != Block::Pre {
            return;
        }
        match block {
            Block::ListItem => {
                // A later sibling item always gets its own separator, empty
                // or not -- this is not the one `cell_block_separator` call
                // `cell_after_marker` protects.
                self.cell_after_marker = false;
            }
            Block::UnorderedList | Block::OrderedList => {
                self.cell_lists.pop();
            }
            Block::Pre => {
                // One code span per source line, `<br>`-joined -- not one
                // span holding a literal `<br>`, where the tag would be
                // verbatim text inside the span (RFC 008 §4.1).
                if let Some(code) = self.cell_pre.take() {
                    // A blank source line is a bare `<br>`, not an empty code
                    // span: ` `` ` is a stray delimiter run, not an empty span,
                    // and pairs with the next run, pulling the following line
                    // into a span it was never in (2.4.1).
                    let spans: Vec<String> = code
                        .lines()
                        .map(|line| {
                            if line.is_empty() {
                                String::new()
                            } else {
                                escape::code_span(line)
                            }
                        })
                        .collect();
                    self.sink.markup(&spans.join("<br>"));
                }
            }
            Block::Heading(_) | Block::Paragraph | Block::Quote | Block::Rule => {}
        }
    }

    pub fn finish(self) -> String {
        self.sink.finish()
    }
}

/// `[text](href "title")`. `text` was escaped as it was captured; the
/// destination and title are escaped here (RFC 010 §3.3, §3.4).
fn link_syntax(text: &str, href: &str, title: Option<&str>) -> String {
    let destination = escape::destination(href);
    let mut link = String::with_capacity(text.len() + destination.len() + 4);
    link.push('[');
    link.push_str(text);
    link.push_str("](");
    link.push_str(&destination);
    if let Some(t) = title {
        link.push(' ');
        link.push_str(&escape::title(t));
    }
    link.push(')');
    link
}

/// Writes a usize without an intermediate allocation.
#[inline]
fn push_usize(s: &mut String, n: usize) {
    let _ = write!(s, "{n}");
}
