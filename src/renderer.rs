use std::fmt::Write;

use crate::utils;

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
    fence: Fence,
    /// Per open `<a>`.
    links: Vec<LinkState>,
    /// Per open inline `<code>`: whether it opened a code-span capture.
    code_captures: Vec<bool>,
}

impl MarkdownRenderer {
    pub fn new(capacity: usize) -> Self {
        Self {
            sink: Sink::new(capacity),
            list_stack: Vec::with_capacity(8),
            in_pre: false,
            pre_depth: 0,
            fence: Fence::None,
            links: Vec::new(),
            code_captures: Vec::new(),
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
        self.sink.ensure_newlines(2);
    }

    fn end_block(&mut self) {
        self.sink.ensure_newlines(2);
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
                    held.push_str(text);
                    return;
                }
                self.open_fence("");
            }
            self.sink.code_block_content(text);
            return;
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

    pub fn enter_element(&mut self, elem: &scraper::node::Element, preserve_ids: bool) {
        let tag = elem.name();
        // Tags whose own arm opens a capture or a code block: anchor first.
        // Adding a tag that sets either guard means adding it here too
        // (RFC 006 Slice D; tests/anchor_drift_guard.rs).
        let anchor_before = matches!(tag, "a" | "code" | "pre");
        if anchor_before {
            self.emit_id_anchor(elem, preserve_ids);
        }
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.begin_block();
                let level = (tag.as_bytes()[1] - b'0') as usize;
                let mut marker = "#".repeat(level);
                marker.push(' ');
                self.sink.markup_closed(&marker);
            }
            "p" | "div" | "article" | "section" | "main" | "header" | "footer" | "nav"
            | "aside" | "figure" | "figcaption" => {
                self.begin_block();
            }
            "ul" => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                self.list_stack.push(ListContext {
                    kind: ListKind::Unordered,
                });
            }
            "ol" => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                let start = elem
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.list_stack.push(ListContext {
                    kind: ListKind::Ordered { counter: start },
                });
            }
            "li" => {
                self.sink.ensure_newlines(1);
                let depth = self.list_stack.len().saturating_sub(1);
                let mut marker = "  ".repeat(depth);
                if let Some(ctx) = self.list_stack.last_mut() {
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
                self.sink.markup_closed(&marker);
            }
            "blockquote" => {
                self.begin_block();
                self.sink.enter_blockquote();
            }
            "pre" => {
                self.pre_depth += 1;
                if self.pre_depth == 1 {
                    self.begin_block();
                    self.in_pre = true;
                    self.fence = Fence::Pending {
                        held: String::new(),
                    };
                }
            }
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
                let capture = !self.sink.in_code_span();
                if capture {
                    self.sink.begin_capture(Capture::CodeSpan);
                }
                self.code_captures.push(capture);
            }
            "strong" | "b" if !self.in_code() => {
                self.sink.flush_space();
                self.sink.markup("**");
            }
            "em" | "i" if !self.in_code() => {
                self.sink.flush_space();
                self.sink.markup("*");
            }
            "a" => {
                let href = elem.attr("href").unwrap_or("").to_string();
                let title = elem.attr("title").map(str::to_string);
                let in_link = self.links.iter().any(|l| !matches!(l, LinkState::TextOnly));
                let state = if in_link || self.in_code() {
                    LinkState::TextOnly
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
                self.sink.flush_space();
                self.sink.markup_closed(&image);
            }
            "hr" => {
                self.begin_block();
                self.sink.block_raw("---");
                self.end_block();
            }
            "br" => {
                // The next content line gets the blockquote prefix, if any.
                self.sink.hard_break();
            }
            _ => {}
        }
        if !anchor_before {
            self.emit_id_anchor(elem, preserve_ids);
        }
    }

    // ─── Leave ─────────────────────────────────────────────────────────────

    pub fn leave_element(&mut self, elem: &scraper::node::Element) {
        let tag = elem.name();
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => self.end_block(),
            "p" | "div" | "article" | "section" | "main" | "header" | "footer" | "nav"
            | "aside" | "figure" | "figcaption" => self.end_block(),
            "ul" | "ol" => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.end_block();
                }
            }
            "li" => self.sink.ensure_newlines(1),
            "blockquote" => {
                self.sink.leave_blockquote();
                self.end_block();
            }
            "pre" => {
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
                self.fence = Fence::None;
                self.end_block();
            }
            "code" if !self.in_pre => {
                if self.code_captures.pop() == Some(true)
                    && let Some((_, content, trailing)) = self.sink.end_capture()
                {
                    // A code span with no content writes nothing.
                    let rendered = (!content.is_empty()).then(|| format!("`{content}`"));
                    self.sink.splice(rendered.as_deref(), trailing);
                }
            }
            "strong" | "b" if !self.in_code() => self.sink.markup("**"),
            "em" | "i" if !self.in_code() => self.sink.markup("*"),
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
                Some(LinkState::TextOnly) | None => {}
            },
            _ => {}
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
