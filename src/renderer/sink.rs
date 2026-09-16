//! The output sink (RFC 024): the only way element handling writes Markdown.
//!
//! Every byte the renderer emits goes through [`Sink`]. Its fields are private
//! to this module, so a handler in `renderer.rs` cannot reach the output
//! buffer, the capture buffers or the line bookkeeping directly: writing to
//! the wrong destination, or writing without updating the bookkeeping, has to
//! be done deliberately inside this file.
//!
//! **Destinations.** While a link or an inline code span is open, content goes
//! into that construct's capture buffer; otherwise into the document. Captures
//! nest (a code span inside a link), and closing one hands its content back to
//! the destination that was current when it opened.
//!
//! **Bookkeeping travels with the destination.** Each destination keeps its
//! own `newlines_emitted`, `at_line_start` and pending-space state.
//!
//! **Containers (RFC 035).** Blockquotes and list items form a stack. Every
//! line written inside them in the document -- content lines, blank separator
//! lines and code-block content lines -- starts with the whole stack's prefix,
//! outermost first: `> ` for a quote, the item's content column in spaces for
//! a list item (`- ` -> 2, `1. ` -> 3, `10. ` -> 4). A blank line gets the
//! prefix with its trailing spaces trimmed (`>` alone). The prefix is written
//! by the sink before the first byte of a line, never by an element handler.
//! Inside a capture there is no line to prefix -- the captured content is
//! spliced into the document as a unit, and the prefix is written then.
//!
//! **Line breaks are written lazily.** A block boundary records how many
//! newlines it needs; they are written when the next content arrives, so a
//! blank line gets the prefix of the containers that stayed open across it,
//! and a container that closed in between leaves no `>` on the blank line
//! after it.

use crate::utils;

#[cfg(test)]
mod tests;

/// One destination and its line state.
struct Dest {
    buf: String,
    /// Consecutive newlines at the end of `buf`.
    newlines_emitted: usize,
    /// The next content byte starts a line.
    at_line_start: bool,
    /// Whitespace was seen and not yet written.
    last_was_space: bool,
    /// The number of trailing newlines a block boundary asked for and that
    /// are not yet written; written before the next content.
    pending_newlines: usize,
}

impl Dest {
    fn new(capacity: usize, at_line_start: bool) -> Self {
        Self {
            buf: String::with_capacity(capacity),
            newlines_emitted: 0,
            at_line_start,
            last_was_space: false,
            pending_newlines: 0,
        }
    }

    /// Writes `s` and derives the line state from its trailing newlines.
    fn push_raw(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.buf.push_str(s);
        let trailing = s.bytes().rev().take_while(|&b| b == b'\n').count();
        if trailing > 0 {
            self.newlines_emitted = trailing;
            self.at_line_start = true;
            self.last_was_space = false;
        } else {
            self.newlines_emitted = 0;
            self.at_line_start = false;
        }
    }
}

/// What a capture is collecting.
pub(super) enum Capture {
    Link { href: String, title: Option<String> },
    CodeSpan,
}

struct Open {
    kind: Capture,
    dest: Dest,
}

/// A container whose lines carry a prefix (RFC 035).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Container {
    Quote,
    /// A list item: `width` is its content column; `tight` is its list's.
    Item {
        width: usize,
        tight: bool,
    },
}

pub(super) struct Sink {
    document: Dest,
    captures: Vec<Open>,
    containers: Vec<Container>,
    /// The document's current line holds only container markers (a list
    /// item's `- `, and a quote's `> ` written after it): a block starting
    /// now shares the line instead of breaking it.
    only_markers: bool,
    /// The fewest containers open since content was last written: blank
    /// lines written before the next content belong to these.
    min_depth: usize,
}

impl Sink {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            document: Dest::new(capacity, true),
            captures: Vec::new(),
            containers: Vec::new(),
            only_markers: false,
            min_depth: 0,
        }
    }

    fn dest(&mut self) -> &mut Dest {
        match self.captures.last_mut() {
            Some(open) => &mut open.dest,
            None => &mut self.document,
        }
    }

    fn dest_ref(&self) -> &Dest {
        match self.captures.last() {
            Some(open) => &open.dest,
            None => &self.document,
        }
    }

    // ─── State queries ─────────────────────────────────────────────────────

    pub(super) fn is_capturing(&self) -> bool {
        !self.captures.is_empty()
    }

    pub(super) fn in_code_span(&self) -> bool {
        self.captures
            .iter()
            .any(|open| matches!(open.kind, Capture::CodeSpan))
    }

    /// Whether the next content byte in the current destination starts a line.
    pub(super) fn at_line_start(&self) -> bool {
        self.dest_ref().at_line_start
    }

    pub(super) fn ends_with_newline(&self) -> bool {
        self.dest_ref().buf.ends_with('\n')
    }

    // ─── Containers ────────────────────────────────────────────────────────

    /// The prefix of the outermost `depth` containers.
    fn prefix(&self, depth: usize) -> String {
        let mut prefix = String::new();
        for container in &self.containers[..depth] {
            match container {
                Container::Quote => prefix.push_str("> "),
                Container::Item { width, .. } => {
                    prefix.extend(std::iter::repeat_n(' ', *width));
                }
            }
        }
        prefix
    }

    fn pop_container(&mut self) {
        self.containers.pop();
        self.only_markers = false;
        self.min_depth = self.min_depth.min(self.containers.len());
        // Back directly inside a tight list item: no blank line may follow.
        if let Some(Container::Item { tight: true, .. }) = self.containers.last() {
            let doc = &mut self.document;
            doc.pending_newlines = doc.pending_newlines.min(1);
        }
    }

    pub(super) fn enter_blockquote(&mut self) {
        // On a line holding only an item marker, the quote starts on it.
        if self.only_markers && !self.document.at_line_start && !self.is_capturing() {
            self.document.buf.push_str("> ");
        }
        self.containers.push(Container::Quote);
    }

    pub(super) fn leave_blockquote(&mut self) {
        debug_assert_eq!(self.containers.last(), Some(&Container::Quote));
        self.pop_container();
    }

    /// A list item's marker (`- `, `1. `), written after the enclosing
    /// containers' prefix; the item's later lines continue at its width.
    pub(super) fn item_marker(&mut self, marker: &str, tight: bool) {
        self.flush_newlines();
        self.emit_pending_prefix();
        let dest = self.dest();
        dest.buf.push_str(marker);
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
        dest.last_was_space = false;
        self.containers.push(Container::Item {
            width: marker.len(),
            tight,
        });
        self.only_markers = true;
    }

    /// Between two items of a loose list: a blank line, even directly inside
    /// a tight list item. The blank line belongs to the loose list, which is
    /// nested in the tight item's content, so it does not loosen that item's
    /// list (RFC 035 §3.1).
    pub(super) fn loose_item_separator(&mut self) {
        let dest = self.dest();
        if dest.buf.is_empty() {
            return;
        }
        dest.pending_newlines = dest.pending_newlines.max(2);
        dest.last_was_space = false;
        dest.at_line_start = true;
    }

    pub(super) fn leave_item(&mut self) {
        debug_assert!(matches!(
            self.containers.last(),
            Some(Container::Item { .. })
        ));
        self.pop_container();
    }

    /// Writes the newlines a block boundary asked for. Each blank line gets
    /// the prefix of the containers that stayed open across it, trailing
    /// spaces trimmed.
    fn flush_newlines(&mut self) {
        let blank = if self.is_capturing() {
            String::new()
        } else {
            self.prefix(self.min_depth).trim_end().to_string()
        };
        let dest = self.dest();
        if dest.pending_newlines > dest.newlines_emitted {
            for i in dest.newlines_emitted..dest.pending_newlines {
                if i >= 1 {
                    dest.buf.push_str(&blank);
                }
                dest.buf.push('\n');
            }
            dest.newlines_emitted = dest.pending_newlines;
        }
        dest.pending_newlines = 0;
    }

    /// Writes the containers' prefix if the document is at a line start.
    /// Every content write reaches it; nothing outside this module writes a
    /// prefix.
    fn emit_pending_prefix(&mut self) {
        if self.is_capturing() {
            return;
        }
        self.min_depth = self.containers.len();
        if self.document.at_line_start && !self.containers.is_empty() {
            let prefix = self.prefix(self.containers.len());
            self.document.buf.push_str(&prefix);
            self.document.at_line_start = false;
        }
    }

    /// Before content: pending newlines, then the prefix.
    fn begin_content(&mut self) {
        self.flush_newlines();
        self.emit_pending_prefix();
        if !self.is_capturing() {
            self.only_markers = false;
        }
    }

    // ─── Line structure ────────────────────────────────────────────────────

    /// Ends the current line with up to `count` newlines in total.
    ///
    /// Inside a capture this writes into the capture like any other content.
    /// A block inside a link or code span therefore puts line breaks into
    /// inline text, which is not valid Markdown -- but where blocks inside
    /// inline elements belong is RFC 028's to define, not this sink's.
    ///
    /// Recorded, not written: see "Line breaks are written lazily" above. On
    /// a line holding only container markers nothing is recorded -- the block
    /// shares the marker's line (`- > q`). Directly inside a tight list item
    /// at most one newline is recorded: a blank line there would make the
    /// list loose (RFC 035 §3.1).
    pub(super) fn ensure_newlines(&mut self, count: usize) {
        let capturing = self.is_capturing();
        if self.only_markers && !capturing {
            return;
        }
        let count = match self.containers.last() {
            Some(Container::Item { tight: true, .. }) if !capturing => count.min(1),
            _ => count,
        };
        let dest = self.dest();
        // Never lead a destination with newlines (e.g. the implicit
        // <html><body> that scraper adds opens a block first).
        if dest.buf.is_empty() {
            dest.at_line_start = true;
            return;
        }
        dest.pending_newlines = dest.pending_newlines.max(count);
        dest.last_was_space = false;
        dest.at_line_start = true;
    }

    /// Writes a pending space, unless at a line start.
    pub(super) fn flush_space(&mut self) {
        let dest = self.dest();
        if dest.last_was_space && !dest.at_line_start {
            dest.buf.push(' ');
            dest.last_was_space = false;
        }
    }

    // ─── Content ───────────────────────────────────────────────────────────

    /// Markdown syntax: heading and list markers, emphasis delimiters, image
    /// syntax. Emits the pending prefix first.
    pub(super) fn markup(&mut self, s: &str) {
        self.begin_content();
        let dest = self.dest();
        dest.buf.push_str(s);
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
    }

    /// Like [`markup`](Self::markup), and leaves no space pending.
    pub(super) fn markup_closed(&mut self, s: &str) {
        self.markup(s);
        self.dest().last_was_space = false;
    }

    /// A code fence line (ends with `\n`). Emits the pending prefix first; the
    /// next byte is at a line start.
    pub(super) fn fence_line(&mut self, s: &str) {
        self.begin_content();
        let dest = self.dest();
        dest.buf.push_str(s);
        dest.newlines_emitted = 0;
        dest.at_line_start = true;
    }

    /// A block-level literal (a thematic break, an id anchor). Emits the
    /// pending prefix first; line state follows the string's newlines.
    pub(super) fn block_raw(&mut self, s: &str) {
        self.begin_content();
        self.dest().push_raw(s);
    }

    /// An `<a id="…"></a>` anchor: the pending prefix, then any pending space,
    /// then the anchor.
    pub(super) fn id_anchor(&mut self, anchor: &str) {
        self.begin_content();
        self.flush_space();
        self.dest().push_raw(anchor);
    }

    /// Code block content, written verbatim. Every line inside a container
    /// gets the containers' prefix, an empty line the prefix with trailing
    /// spaces trimmed (RFC 035; this reverses RFC 024's "no prefix", which
    /// deferred exactly this to audit A-08).
    pub(super) fn code_block_content(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.flush_newlines();
        if self.is_capturing() {
            self.dest().push_raw(s);
            return;
        }
        self.only_markers = false;
        self.min_depth = self.containers.len();
        for line in s.split_inclusive('\n') {
            if self.document.at_line_start && !self.containers.is_empty() {
                let prefix = self.prefix(self.containers.len());
                if line == "\n" {
                    self.document.buf.push_str(prefix.trim_end());
                } else {
                    self.document.buf.push_str(&prefix);
                }
            }
            self.document.push_raw(line);
        }
    }

    /// A hard line break. No prefix: the next content line gets one.
    pub(super) fn hard_break(&mut self) {
        self.flush_newlines();
        if !self.is_capturing() {
            self.only_markers = false;
        }
        let dest = self.dest();
        dest.buf.push_str("  \n");
        dest.newlines_emitted = 1;
        dest.at_line_start = true;
        dest.last_was_space = false;
    }

    /// Text: whitespace collapsed and Markdown-escaped (`utils::write_normalised`).
    pub(super) fn text(&mut self, text: &str) {
        let has_content = !text.trim().is_empty();
        if has_content {
            self.begin_content();
        }
        let dest = self.dest();
        let at_block = dest.at_line_start;
        utils::write_normalised(
            text,
            &mut dest.buf,
            &mut dest.last_was_space,
            at_block,
            &mut dest.at_line_start,
        );
        if has_content {
            dest.newlines_emitted = 0;
        }
    }

    // ─── Captures ──────────────────────────────────────────────────────────

    /// Starts collecting into a new buffer. Whitespace pending in the current
    /// destination stays pending there until the capture closes.
    pub(super) fn begin_capture(&mut self, kind: Capture) {
        self.captures.push(Open {
            kind,
            dest: Dest::new(0, false),
        });
    }

    /// Closes the innermost capture and returns what it collected, after
    /// restoring the destination it opened in. The caller decides what to
    /// write with [`splice`](Self::splice).
    pub(super) fn end_capture(&mut self) -> Option<(Capture, String, bool)> {
        let open = self.captures.pop()?;
        Some((open.kind, open.dest.buf, open.dest.last_was_space))
    }

    /// Writes a closed capture's rendering (`None` writes nothing) into the
    /// current destination, then carries over whitespace that was pending at
    /// the end of the captured content -- so `<a>x </a>y` keeps its space,
    /// exactly as it would outside a link.
    pub(super) fn splice(&mut self, rendered: Option<&str>, trailing_space: bool) {
        match rendered {
            Some(s) => {
                self.flush_space();
                self.markup_closed(s);
                self.dest().last_was_space = trailing_space;
            }
            None => {
                let dest = self.dest();
                dest.last_was_space |= trailing_space;
            }
        }
    }

    // ─── Result ────────────────────────────────────────────────────────────

    /// The document, with trailing whitespace trimmed and one final newline.
    /// Unclosed captures (not produced by the traversal) are discarded.
    pub(super) fn finish(self) -> String {
        debug_assert!(
            self.containers.is_empty(),
            "container prefix stack unbalanced at document end: {:?}",
            self.containers
        );
        let mut out = self.document.buf;
        let end = out.trim_end().len();
        out.truncate(end);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }
}
