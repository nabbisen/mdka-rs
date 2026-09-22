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
//! **Escaping by context (RFC 010).** Prose goes through
//! [`escape::decide`], with the line state and the character before it; an
//! escape that depends on the character after it waits, and is settled when
//! the destination's next byte is written, by whatever writes it. Code-span
//! captures are not escaped, and fenced blocks are sized to their content.
//! Every write to a destination therefore goes through [`Dest::put`].
//!
//! **Line breaks are written lazily.** A block boundary records how many
//! newlines it needs; they are written when the next content arrives, so a
//! blank line gets the prefix of the containers that stayed open across it,
//! and a container that closed in between leaves no `>` on the blank line
//! after it.

use super::escape::{self, Decision, FenceScan, Head, Line, Wait};

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
    /// The current line, for escaping.
    line: Line,
    /// A written character whose escape waits on the next byte: its offset in
    /// `buf` and what it waits for.
    wait: Option<(usize, Wait)>,
    /// A link's text or an image's alt: it follows `[`, and `]` ends it.
    link_text: bool,
    /// Offsets of the emphasis delimiters opened and not yet closed.
    emphasis_opens: Vec<usize>,
    /// The emphasis span closed last: its opening offset, delimiter length
    /// and end offset (RFC 010 §3.8).
    last_emphasis: Option<(usize, usize, usize)>,
}

impl Dest {
    fn new(capacity: usize, at_line_start: bool) -> Self {
        Self {
            buf: String::with_capacity(capacity),
            newlines_emitted: 0,
            at_line_start,
            last_was_space: false,
            pending_newlines: 0,
            line: if at_line_start {
                Line::START
            } else {
                Line::INLINE
            },
            wait: None,
            link_text: false,
            emphasis_opens: Vec::new(),
            last_emphasis: None,
        }
    }

    /// Settles the waiting escape, now that the byte after it is known
    /// (`None`: nothing follows in this destination).
    fn settle(&mut self, next: Option<char>) {
        if let Some((at, wait)) = self.wait.take()
            && wait.escapes(next)
        {
            self.buf.insert(at, '\\');
        }
    }

    /// Every write to `buf` goes through here: the waiting escape is settled
    /// against the first character, and a line break starts a new line -- a
    /// block boundary's; a hard break says otherwise afterwards.
    fn put(&mut self, s: &str) {
        let Some(first) = s.chars().next() else {
            return;
        };
        self.settle(Some(first));
        self.buf.push_str(s);
        if s.contains('|') {
            self.line.pipe_here = true;
        }
        if s.contains('\n') {
            self.line.newline(false);
            self.last_emphasis = None;
        }
    }

    /// The character before the next one written: the end of `buf`, or `[`
    /// at the start of a link's text.
    fn prev_char(&self) -> Option<char> {
        self.buf
            .chars()
            .next_back()
            .or(if self.link_text { Some('[') } else { None })
    }

    /// Text: whitespace collapsed, and escaped by context unless `escape` is
    /// false (a code span, whose content is verbatim).
    fn write_text(&mut self, text: &str, escape: bool) {
        // Whitespace at the start of a line is dropped until the first
        // character; inside, a run of whitespace is one space, written only
        // before the next character.
        let mut line_start = self.at_line_start;
        for c in text.chars() {
            if c.is_ascii_whitespace() || c == '\u{a0}' {
                if !line_start {
                    self.last_was_space = true;
                }
                continue;
            }
            if self.last_was_space && !line_start {
                self.put_char(' ');
            }
            self.last_was_space = false;
            line_start = false;
            self.at_line_start = false;
            // Past the start of the line, a character that no context escapes
            // (nor `|`, which the line state notes) is written as it is: the
            // common case, kept cheap.
            if !escape {
                self.line.head = Head::Inline;
                self.put_char(c);
            } else if self.line.head == Head::Inline && !escape::may_escape(c) && c != '|' {
                if self.wait.is_some() {
                    self.settle(Some(c));
                }
                self.buf.push(c);
            } else {
                self.write_escaped(c);
            }
        }
    }

    /// A character that may need escaping, decided by context. Kept out of
    /// [`write_text`](Self::write_text)'s loop, which it would otherwise bloat
    /// for the rare characters that reach it.
    #[inline(never)]
    fn write_escaped(&mut self, c: char) {
        match escape::decide(c, self.prev_char(), &mut self.line, self.link_text) {
            Decision::Plain => self.put_char(c),
            Decision::Escape => {
                self.put_char('\\');
                self.put_char(c);
            }
            Decision::Wait(wait) => {
                self.put_char(c);
                self.wait = Some((self.buf.len() - c.len_utf8(), wait));
            }
        }
    }

    /// [`put`](Self::put) for one character that is not a line break.
    fn put_char(&mut self, c: char) {
        if self.wait.is_some() {
            self.settle(Some(c));
        }
        self.buf.push(c);
        if c == '|' {
            self.line.pipe_here = true;
        }
    }

    /// Writes `s` and derives the line state from its trailing newlines.
    fn push_raw(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.put(s);
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
    /// The open fenced block in the document: the offset just after its
    /// opening backticks, and its content scanned so far (RFC 010 §3.2).
    fence: Option<(usize, FenceScan)>,
    /// A list item's or heading's marker was just written, and no real
    /// content has appeared for the block yet: the next text carries the
    /// block's own leading whitespace and must have it stripped, not
    /// collapsed, the way a fresh line's would be (RFC 036 §5.1, §5.6, and
    /// slice `036b` §1). Set by [`item_marker`](Self::item_marker) and
    /// [`heading_marker`](Self::heading_marker); cleared by
    /// [`end_leading_strip`](Self::end_leading_strip), called at every other
    /// write of real content -- so a future content producer inherits the
    /// clearing for free, with no call site of its own to remember, the way
    /// `consumed_leading()` at the renderer layer had to be (slice 4's
    /// review found three more of those to add: `<br>`, empty emphasis, a
    /// fenced block). Not cleared by `id_anchor`: an anchor is metadata, not
    /// the block's own content, and whitespace after it is still leading.
    pending_leading_strip: bool,
}

impl Sink {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            document: Dest::new(capacity, true),
            captures: Vec::new(),
            containers: Vec::new(),
            only_markers: false,
            min_depth: 0,
            fence: None,
            pending_leading_strip: false,
        }
    }

    /// Ends the block's leading-whitespace-strip window, if one is open: no
    /// further text in this block is the block's own leading whitespace.
    /// Called at every write of real content, and (`pub(super)`, for a
    /// heading -- a list item calls it itself, from
    /// [`leave_item`](Self::leave_item)) at a block's end for one that had
    /// none at all.
    pub(super) fn end_leading_strip(&mut self) {
        self.pending_leading_strip = false;
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
            self.document.put("> ");
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
        // Like a container prefix, a marker leaves the line at its start: the
        // item's content begins a block.
        dest.put(marker);
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
        dest.last_was_space = false;
        self.containers.push(Container::Item {
            width: marker.len(),
            tight,
        });
        self.only_markers = true;
        self.pending_leading_strip = true;
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

    /// `Some(bullet)` when this item's own marker was swapped to dodge a
    /// thematic break: the caller must carry `bullet` to this same list's
    /// later items too (`036b` §2), or they would no longer share a bullet
    /// with it and CommonMark would read two lists where the source had one.
    pub(super) fn leave_item(&mut self) -> Option<char> {
        debug_assert!(matches!(
            self.containers.last(),
            Some(Container::Item { .. })
        ));
        // A chain of nested items that are all otherwise empty writes only
        // their own markers, sharing one line (RFC 035's "only markers"
        // rule): "- - -" for three levels. Once a run of plain `- ` markers
        // (width 2: the unordered bullet, not an ordered `N. `) actually on
        // the current line -- not merely open ancestors, which may have
        // broken onto earlier lines of their own if an earlier sibling had
        // real content (`036b` §2's fix) -- reaches three, the line reads as
        // a CommonMark thematic break, not nested empty list items (RFC 036
        // §5.5). Swap this item's own marker to a different, equally valid
        // bullet character: a thematic break needs every character in the
        // run to match, so one different bullet breaks that reading while
        // every level still nests as a list, each still honouring the width
        // RFC 035 assumed.
        let mut swapped = None;
        if !self.is_capturing() && self.only_markers && self.document.buf.ends_with("- ") {
            let line_start = self.document.buf.rfind('\n').map_or(0, |i| i + 1);
            let dash_run = self.document.buf.as_bytes()[line_start..]
                .rchunks(2)
                .take_while(|chunk| *chunk == b"- ")
                .count();
            if dash_run >= 3 {
                let at = self.document.buf.len() - 2;
                self.document.buf.replace_range(at..at + 1, "*");
                swapped = Some('*');
            }
        }
        // An item with no content at all leaves no dangling strip request
        // past its own end.
        self.end_leading_strip();
        self.pop_container();
        swapped
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
                    dest.put(&blank);
                }
                dest.put("\n");
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
            self.document.put(&prefix);
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
            dest.put(" ");
            dest.last_was_space = false;
        }
    }

    // ─── Content ───────────────────────────────────────────────────────────

    /// Markdown syntax: heading and list markers, emphasis delimiters, image
    /// syntax. Emits the pending prefix first.
    pub(super) fn markup(&mut self, s: &str) {
        self.begin_content();
        self.end_leading_strip();
        let dest = self.dest();
        dest.put(s);
        dest.line.head = Head::Inline;
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
    }

    /// Like [`markup`](Self::markup), and leaves no space pending.
    pub(super) fn markup_closed(&mut self, s: &str) {
        self.markup(s);
        self.dest().last_was_space = false;
    }

    /// An ATX heading's marker (`## `). Its content is inline, and a trailing
    /// `#` run in it would be read as the closing sequence.
    pub(super) fn heading_marker(&mut self, marker: &str) {
        self.markup_closed(marker);
        self.dest().line.heading = true;
        self.pending_leading_strip = true;
    }

    /// Opens emphasis with `delimiter` (`*` or `**`) and returns the delimiter
    /// written, which the closing run must repeat.
    ///
    /// When the emphasis closed last ends exactly here with a `*` run, its
    /// closing run would touch this opening run, and CommonMark would resolve
    /// the joined run by its own rules (RFC 010 §3.8: `*a**b*` is not two
    /// emphases). One of the two spans is written with `_` instead:
    ///
    /// - the earlier span, if `_` can still open where it starts -- after
    ///   whitespace, punctuation or nothing -- and no `_` touches its runs;
    ///   its closing run is followed by this `*`, so it can close;
    /// - otherwise this span: its opening run follows the earlier `*` run, so
    ///   it can open; it can close unless a letter or digit follows it.
    ///
    /// Between two letters or digits (`x*a**b*y`) neither can: delimiters
    /// cannot express that, and it is left as it is.
    pub(super) fn emphasis_open(&mut self, delimiter: &'static str) -> &'static str {
        self.begin_content();
        self.end_leading_strip();
        let dest = self.dest();
        let mut written = delimiter;
        if let Some((open, len, end)) = dest.last_emphasis
            && end == dest.buf.len()
            && dest.buf[end - len..end].starts_with('*')
        {
            let before = dest.buf[..open].chars().next_back();
            let first = dest.buf[open + len..].chars().next();
            let last = dest.buf[..end - len].chars().next_back();
            if escape::class(before) != escape::Class::Word
                && ![before, first, last].contains(&Some('_'))
            {
                let run = "_".repeat(len);
                dest.buf.replace_range(open..open + len, &run);
                dest.buf.replace_range(end - len..end, &run);
            } else if escape::class(before) != escape::Class::Word {
                written = if delimiter.len() == 2 { "__" } else { "_" };
            }
        }
        dest.put(written);
        dest.line.head = Head::Inline;
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
        dest.emphasis_opens.push(dest.buf.len() - written.len());
        written
    }

    /// Closes the innermost emphasis with `delimiter`.
    pub(super) fn emphasis_close(&mut self, delimiter: &str) {
        self.markup(delimiter);
        let dest = self.dest();
        dest.last_emphasis = dest
            .emphasis_opens
            .pop()
            .map(|open| (open, delimiter.len(), dest.buf.len()));
    }

    /// The opening fence of a code block, with its info string. Emits the
    /// pending prefix first; the next byte is at a line start. The fence is
    /// three backticks for now: [`close_fence`](Self::close_fence) lengthens
    /// it if the content needs more.
    pub(super) fn open_fence(&mut self, info: &str) {
        self.begin_content();
        self.end_leading_strip();
        let capturing = self.is_capturing();
        let dest = self.dest();
        dest.put("```");
        let at = dest.buf.len();
        dest.put(info);
        dest.put("\n");
        dest.newlines_emitted = 0;
        dest.at_line_start = true;
        if !capturing {
            self.fence = Some((at, FenceScan::NEW));
        }
    }

    /// The closing fence: one backtick longer than the longest backtick run at
    /// the start of a content line, at least three, with the opening fence
    /// lengthened to match (RFC 010 §3.2).
    pub(super) fn close_fence(&mut self) {
        let len = match self.fence.take() {
            Some((at, scan)) => {
                let len = scan.fence_len();
                if len > 3 {
                    self.document.buf.insert_str(at, &"`".repeat(len - 3));
                }
                len
            }
            None => 3,
        };
        self.code_block_content(&"`".repeat(len));
    }

    /// A thematic break (`<hr>`). Emits the pending prefix first; line state
    /// follows the string's newlines.
    ///
    /// Sharing its line with an item's own marker -- the "only markers"
    /// idiom (RFC 035) -- collides exactly the way slice 4's nested empty
    /// markers did (RFC 036 §5.5): `- ---` is the marker's `-` plus this
    /// break's `---`, four homogeneous dashes that CommonMark reads as one
    /// break for the whole line, not a list item containing one (`036b`
    /// §3). Unlike an empty marker, there is no substitute bullet character
    /// to reach for without the break itself stopping being `---` -- and
    /// `docs/src/api/elements.md` documents `<hr>` as `---` unconditionally
    /// (addendum B §1: writing `___` here would be exactly the
    /// documented-intent violation this RFC exists to close). Instead, the
    /// break moves to a continuation line, the same place any other block
    /// content of an item already lives once something precedes it (`<p>`,
    /// `<pre>`, a blockquote) -- sharing the marker's line was the anomaly
    /// only `<hr>` had, not a property worth keeping for it alone.
    pub(super) fn thematic_break(&mut self) {
        if !self.is_capturing()
            && self.only_markers
            && matches!(
                self.containers.last(),
                Some(Container::Item { width: 2, .. })
            )
        {
            // Clearing `only_markers` first makes `ensure_newlines` treat
            // this exactly like the boundary before any other block inside
            // the item, instead of sharing the marker's line.
            self.only_markers = false;
            self.ensure_newlines(1);
        }
        self.begin_content();
        self.end_leading_strip();
        self.dest().push_raw("---");
    }

    /// An `<a id="…"></a>` anchor: the pending prefix, then any pending space,
    /// then the anchor.
    pub(super) fn id_anchor(&mut self, anchor: &str) {
        self.begin_content();
        self.flush_space();
        let dest = self.dest();
        dest.push_raw(anchor);
        dest.line.head = Head::Inline;
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
        if let Some((_, scan)) = &mut self.fence {
            scan.feed(s);
        }
        self.only_markers = false;
        self.min_depth = self.containers.len();
        for line in s.split_inclusive('\n') {
            if self.document.at_line_start && !self.containers.is_empty() {
                let prefix = self.prefix(self.containers.len());
                if line == "\n" {
                    self.document.put(prefix.trim_end());
                } else {
                    self.document.put(&prefix);
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
        self.end_leading_strip();
        let dest = self.dest();
        let pipe = dest.line.pipe_here;
        dest.put("  \n");
        // The paragraph continues on the next line.
        dest.line.pipe_here = pipe;
        dest.line.newline(true);
        dest.newlines_emitted = 1;
        dest.at_line_start = true;
        dest.last_was_space = false;
    }

    /// Text: whitespace collapsed, and escaped by context -- not at all inside
    /// a code span.
    ///
    /// While a leading-whitespace strip is pending (RFC 036 §5.1, §5.6), it is
    /// applied here first: a list item's or heading's marker leaves the
    /// destination's line-start bookkeeping consumed (RFC 035's content
    /// column already begins after it), so the usual leading-whitespace
    /// collapse -- correct for any other fresh line -- would not reach the
    /// block's own text. Leading whitespace is stripped instead, wherever in
    /// the block's descendants it falls; a whitespace-only text node before
    /// the first real content is dropped entirely here, not forwarded as a
    /// space.
    pub(super) fn text(&mut self, text: &str) {
        let text = if self.pending_leading_strip {
            let stripped =
                text.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\u{a0}');
            if stripped.is_empty() {
                return;
            }
            self.pending_leading_strip = false;
            stripped
        } else {
            text
        };
        let has_content = !text.trim().is_empty();
        if has_content {
            self.begin_content();
        }
        let escape = !matches!(
            self.captures.last(),
            Some(Open {
                kind: Capture::CodeSpan,
                ..
            })
        );
        let dest = self.dest();
        dest.write_text(text, escape);
        if has_content {
            dest.newlines_emitted = 0;
        }
    }

    /// `![alt](src "title")`, with the alt text escaped as a link's text and
    /// the destination and title by their own rules (RFC 010 §3.3–3.5).
    pub(super) fn image_syntax(alt: &str, src: &str, title: Option<&str>) -> String {
        let mut text = Dest::new(alt.len(), true);
        text.line = Line::INLINE;
        text.link_text = true;
        text.write_text(alt, true);
        text.settle(Some(']'));
        let destination = escape::destination(src);
        let mut image = String::with_capacity(text.buf.len() + destination.len() + 8);
        image.push_str("![");
        image.push_str(&text.buf);
        image.push_str("](");
        image.push_str(&destination);
        if let Some(t) = title {
            image.push(' ');
            image.push_str(&escape::title(t));
        }
        image.push(')');
        image
    }

    // ─── Captures ──────────────────────────────────────────────────────────

    /// Starts collecting into a new buffer. Whitespace pending in the current
    /// destination stays pending there until the capture closes.
    pub(super) fn begin_capture(&mut self, kind: Capture) {
        let mut dest = Dest::new(0, false);
        dest.link_text = matches!(kind, Capture::Link { .. });
        self.captures.push(Open { kind, dest });
    }

    /// Closes the innermost capture and returns what it collected, after
    /// restoring the destination it opened in. The caller decides what to
    /// write with [`splice`](Self::splice).
    pub(super) fn end_capture(&mut self) -> Option<(Capture, String, bool)> {
        let mut open = self.captures.pop()?;
        // A link's text is followed by `]`.
        let next = matches!(open.kind, Capture::Link { .. }).then_some(']');
        open.dest.settle(next);
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
        let mut document = self.document;
        document.settle(None);
        let mut out = document.buf;
        let end = out.trim_end().len();
        out.truncate(end);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }
}
