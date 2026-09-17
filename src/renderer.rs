use std::fmt::Write;

use crate::utils::{self, Block};

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
    /// Per open `<strong>`/`<b>`/`<em>`/`<i>`: the delimiter it wrote, if any,
    /// so leaving writes exactly what entering did.
    emphasis: Vec<Option<&'static str>>,
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
        }
    }

    /// Inside a `<pre>` -- with or without `<code>` -- or an inline code span,
    /// child elements contribute text only: no emphasis delimiters, image or
    /// link syntax, and an image contributes nothing (RFC 024 criteria 3 and 9,
    /// amended 2026-09-17). Markdown has no markup inside code.
    fn in_code(&self) -> bool {
        self.in_pre || self.sink.in_code_span()
    }

    fn begin_block(&mut self) {
        // The blockquote prefix is not written here; the sink writes it
        // before the next content byte.
        self.ensure_newlines(2);
    }

    fn end_block(&mut self) {
        self.ensure_newlines(2);
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
            let mut line = String::with_capacity(lang.len() + 4);
            line.push_str("```");
            line.push_str(lang);
            line.push('\n');
            self.sink.fence_line(&line);
            self.sink.code_block_content(&held);
        }
    }

    // ─── Text ──────────────────────────────────────────────────────────────

    pub fn process_text(&mut self, text: &str) {
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
    fn emit_id_anchor(&mut self, elem: &scraper::node::Element, preserve_ids: bool) {
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
    /// `loose_list`: the element is a list that is loose (RFC 035 §3.1). Both
    /// are computed once per document by the traversal.
    pub fn enter_element(
        &mut self,
        elem: &scraper::node::Element,
        preserve_ids: bool,
        wraps_blocks: bool,
        loose_list: bool,
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
            self.enter_block(block, elem, loose_list);
        } else {
            self.enter_inline(tag, elem, wraps_blocks);
        }
        if !anchor_before {
            self.emit_id_anchor(elem, preserve_ids);
        }
    }

    fn enter_block(&mut self, block: Block, elem: &scraper::node::Element, loose_list: bool) {
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
                self.sink.markup_closed(&marker);
            }
            Block::Paragraph => self.begin_block(),
            Block::UnorderedList => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                self.list_stack.push(ListContext {
                    kind: ListKind::Unordered,
                    loose: loose_list,
                    started: false,
                });
            }
            Block::OrderedList => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                let start = elem
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.list_stack.push(ListContext {
                    kind: ListKind::Ordered { counter: start },
                    loose: loose_list,
                    started: false,
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
                    match &mut ctx.kind {
                        ListKind::Unordered => marker.push_str("- "),
                        ListKind::Ordered { counter } => {
                            let n = *counter;
                            *counter += 1;
                            push_usize(&mut marker, n);
                            marker.push_str(". ");
                        }
                    }
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
                self.sink.block_raw("---");
                self.end_block();
            }
        }
    }

    fn enter_inline(&mut self, tag: &str, elem: &scraper::node::Element, wraps_blocks: bool) {
        match tag {
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
                // 1), or when the element's own style negates the emphasis
                // (criterion 4).
                let delimiter = if matches!(tag, "strong" | "b") {
                    "**"
                } else {
                    "*"
                };
                let write = !self.in_code()
                    && !wraps_blocks
                    && !utils::emphasis_negated_by_style(tag, elem.attr("style"));
                if write {
                    let _ = self.open_distributed_link();
                    self.sink.flush_space();
                    self.sink.markup(delimiter);
                }
                self.emphasis.push(write.then_some(delimiter));
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
                let src = elem.attr("src").unwrap_or("");
                let alt = elem.attr("alt").unwrap_or("");
                let mut image = String::with_capacity(src.len() + alt.len() + 8);
                image.push_str("![");
                image.push_str(alt);
                image.push_str("](");
                image.push_str(src);
                if let Some(t) = elem.attr("title") {
                    image.push_str(" \"");
                    image.push_str(t);
                    image.push('"');
                }
                image.push(')');
                let _ = self.open_distributed_link();
                self.sink.flush_space();
                self.sink.markup_closed(&image);
            }
            // Inside code a <br> is text, not a Markdown hard break, whose
            // trailing spaces would become code (RFC 024 rule 8): one line
            // break in a <pre>, one space in a code span.
            "br" if self.in_pre => self.process_text("\n"),
            "br" if self.sink.in_code_span() => self.sink.text(" "),
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
            "code" if !self.in_pre => {
                if self.code_captures.pop() == Some(true)
                    && let Some((_, content, trailing)) = self.sink.end_capture()
                {
                    // A code span with no content writes nothing.
                    let rendered = (!content.is_empty()).then(|| format!("`{content}`"));
                    self.sink.splice(rendered.as_deref(), trailing);
                }
            }
            "strong" | "b" | "em" | "i" => {
                if let Some(Some(delimiter)) = self.emphasis.pop() {
                    self.sink.markup(delimiter);
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
        if self.in_pre && (block != Block::Pre || self.pre_depth > 1) {
            self.pre_block_break = true;
            if block != Block::Pre {
                return;
            }
        }
        match block {
            Block::Heading(_) | Block::Paragraph => self.end_block(),
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
                self.sink.leave_item();
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
                self.sink.code_block_content("```");
                self.in_pre = false;
                self.pre_block_break = false;
                self.pre_has_text = false;
                self.fence = Fence::None;
                self.end_block();
            }
            Block::Rule => {}
        }
    }

    pub fn finish(self) -> String {
        self.sink.finish()
    }
}

/// `[text](href "title")`. Destinations and titles are written as given;
/// escaping them is RFC 010's.
fn link_syntax(text: &str, href: &str, title: Option<&str>) -> String {
    let mut link = String::with_capacity(text.len() + href.len() + 4);
    link.push('[');
    link.push_str(text);
    link.push_str("](");
    link.push_str(href);
    if let Some(t) = title {
        link.push_str(" \"");
        link.push_str(t);
        link.push('"');
    }
    link.push(')');
    link
}

/// Writes a usize without an intermediate allocation.
#[inline]
fn push_usize(s: &mut String, n: usize) {
    let _ = write!(s, "{n}");
}
