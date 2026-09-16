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
//! own `newlines_emitted`, `at_line_start` and pending-space state. The pending
//! blockquote prefix belongs to the document destination only: it is emitted
//! by the sink before the first content byte written at a line start, never by
//! an element handler. Inside a capture there is no line to prefix -- the
//! captured content is spliced into the document as a unit, and the prefix is
//! emitted then.

use crate::utils;

/// One destination and its line state.
struct Dest {
    buf: String,
    /// Consecutive newlines at the end of `buf`.
    newlines_emitted: usize,
    /// The next content byte starts a line.
    at_line_start: bool,
    /// Whitespace was seen and not yet written.
    last_was_space: bool,
}

impl Dest {
    fn new(capacity: usize, at_line_start: bool) -> Self {
        Self {
            buf: String::with_capacity(capacity),
            newlines_emitted: 0,
            at_line_start,
            last_was_space: false,
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

pub(super) struct Sink {
    document: Dest,
    captures: Vec<Open>,
    blockquote_depth: usize,
}

impl Sink {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            document: Dest::new(capacity, true),
            captures: Vec::new(),
            blockquote_depth: 0,
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

    // ─── Blockquote prefix ─────────────────────────────────────────────────

    pub(super) fn enter_blockquote(&mut self) {
        self.blockquote_depth += 1;
    }

    pub(super) fn leave_blockquote(&mut self) {
        self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
    }

    /// Emits `> ` × depth if the document is at a line start inside a
    /// blockquote. Called by every content-writing method below; nothing
    /// outside this module emits the prefix.
    fn emit_pending_prefix(&mut self) {
        if self.is_capturing() {
            return;
        }
        let depth = self.blockquote_depth;
        let doc = &mut self.document;
        if doc.at_line_start && depth > 0 {
            for _ in 0..depth {
                doc.buf.push_str("> ");
            }
            doc.at_line_start = false;
        }
    }

    // ─── Line structure ────────────────────────────────────────────────────

    /// Ends the current line with up to `count` newlines in total.
    ///
    /// Inside a capture this writes into the capture like any other content.
    /// A block inside a link or code span therefore puts line breaks into
    /// inline text, which is not valid Markdown -- but where blocks inside
    /// inline elements belong is RFC 028's to define, not this sink's.
    pub(super) fn ensure_newlines(&mut self, count: usize) {
        let dest = self.dest();
        // Never lead a destination with newlines (e.g. the implicit
        // <html><body> that scraper adds opens a block first).
        if dest.buf.is_empty() {
            dest.at_line_start = true;
            return;
        }
        while dest.newlines_emitted < count {
            dest.buf.push('\n');
            dest.newlines_emitted += 1;
        }
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
        self.emit_pending_prefix();
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
        self.emit_pending_prefix();
        let dest = self.dest();
        dest.buf.push_str(s);
        dest.newlines_emitted = 0;
        dest.at_line_start = true;
    }

    /// A block-level literal (a thematic break, an id anchor). Emits the
    /// pending prefix first; line state follows the string's newlines.
    pub(super) fn block_raw(&mut self, s: &str) {
        self.emit_pending_prefix();
        self.dest().push_raw(s);
    }

    /// An `<a id="…"></a>` anchor: the pending prefix, then any pending space,
    /// then the anchor.
    pub(super) fn id_anchor(&mut self, anchor: &str) {
        self.emit_pending_prefix();
        self.flush_space();
        self.dest().push_raw(anchor);
    }

    /// Code block content, written verbatim. **No blockquote prefix**: a
    /// multi-line code block inside a quote losing its prefix from line 2 is
    /// audit A-08 (blockquote continuity, RFC 009), and `<pre><code>` output
    /// must not move in RFC 024.
    pub(super) fn code_block_content(&mut self, s: &str) {
        self.dest().push_raw(s);
    }

    /// A hard line break. No prefix: the next content line gets one.
    pub(super) fn hard_break(&mut self) {
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
            self.emit_pending_prefix();
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
        let mut out = self.document.buf;
        let end = out.trim_end().len();
        out.truncate(end);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }
}
