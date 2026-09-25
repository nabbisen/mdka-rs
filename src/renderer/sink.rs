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
//! **A delimiter choice can depend on what has not been written yet
//! (RFC 037 addendum).** A `<strong><em>` swap to `_` is only safe once
//! nothing alphanumeric follows the whole nested span, which is not known
//! until later. It is written provisionally and settled the same way a
//! waiting escape is -- against whatever the destination writes next --
//! rather than deferring the write itself, since both possible delimiters
//! are one byte and patching one for the other later changes nothing else
//! already written.
//!
//! **Line breaks are written lazily.** A block boundary records how many
//! newlines it needs; they are written when the next content arrives, so a
//! blank line gets the prefix of the containers that stayed open across it,
//! and a container that closed in between leaves no `>` on the blank line
//! after it.
//!
//! **A marker line can be misread as something else it resembles (RFC 036,
//! RFC 038).** Three places adjust one to stop that: [`Sink::leave_item`]
//! swaps a bullet when a run of empty nested markers would read as a
//! thematic break; [`Sink::thematic_break`] moves a colliding `<hr>` to a
//! continuation line rather than change its character, since `---` is
//! documented and must stay `---`; [`Sink::force_disambiguating_blank_line`]
//! forces a blank line before a nested list whose own first item is empty,
//! so its bare marker is not read as the parent's setext underline or
//! absorbed as its lazy-continuation text. Each dodges a different
//! collision and none shares a mechanism with another -- landing on one
//! does not mean the other two do not exist.

use super::escape::{self, Decision, FenceScan, Head, Line, Wait};

#[cfg(test)]
mod tests;

/// RFC 044: see [`Dest::em_guard`].
#[derive(Clone, Copy)]
struct EmGuard {
    /// Offsets of the one-byte `_` opening and closing the swapped span.
    open: usize,
    close: usize,
    /// Opening offset of the Bold ancestor the swap was made against: the
    /// span whose own close settles this guard.
    strong_open: usize,
    /// Whether the character written right after the span's close is known.
    decided: bool,
    /// ... and was a letter or digit.
    word_follows: bool,
}

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
    /// The emphasis span closed last: its opening offset, delimiter length,
    /// end offset (RFC 010 §3.8), and whether it was itself an RFC 037
    /// addendum intraword `_`-swap candidate, pending confirmation (used by
    /// [`Sink::emphasis_close`] to tell an ordinary close from the outer
    /// Bold ancestor's own).
    last_emphasis: Option<(usize, usize, usize, bool)>,
    /// RFC 037 addendum: a `<strong><em>`-order `_`-swap whose safety is not
    /// yet confirmed -- the byte offsets of its one-byte opening and closing
    /// delimiters, patched back to `*` if the next character written
    /// anywhere after it turns out to be alphanumeric. Set only once the
    /// outer Bold ancestor itself closes (see [`Sink::emphasis_close`]),
    /// since only then is the position "whatever follows the whole nested
    /// span" fixed; resolved the same way [`wait`](Self::wait) is, by
    /// whatever writes next.
    flank_guard: Option<(usize, usize)>,
    /// RFC 044: a `<strong><em>`-order `_`-swapped span that has closed, whose
    /// closing `_` may land against a letter or digit that follows it *inside*
    /// the strong -- where an intraword `_` cannot close, so the emphasis
    /// would be lost and the underscore left literal. See
    /// [`Sink::emphasis_close`] for how it is settled.
    em_guard: Option<EmGuard>,
    /// RFC 009 addendum, 2026-09-23: the last `~~` span that closed with
    /// real delimiters, and nothing written since -- its opening offset and
    /// its closing offset (just past the closing `~~`). `~~` has no second
    /// delimiter form to alternate to the way `**`/`__` do (RFC 037's own
    /// answer for adjacent same-class emphasis), so two adjacent
    /// `<del>`/`<s>` spans merge into one instead: [`Sink::strikethrough_open`]
    /// checks this before opening a new span, and if it is still exactly
    /// where the next span would start, removes the previous close and
    /// continues that span rather than starting a second one -- the
    /// alternative, `~~~~`, degrades to literal text mid-line and destroys
    /// content outright at the start of one (a tilde code fence).
    last_strike: Option<(usize, usize)>,
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
            flank_guard: None,
            em_guard: None,
            last_strike: None,
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

    /// Settles a pending RFC 037 addendum flanking guard, now that the byte
    /// after it is known (`None`: nothing follows in this destination).
    /// Alphanumeric undoes the swap -- `**` cannot open at all once
    /// followed by `_` and preceded by a letter or digit, and followed by a
    /// letter or digit, `_` cannot close either, so the underscore reverts
    /// to `*` at both ends, exactly the pre-swap bytes. Anything else
    /// (whitespace, punctuation, or nothing) counts as safe: the swap
    /// stays.
    fn settle_flank(&mut self, next: Option<char>) {
        // RFC 044: the first thing written after a swapped span closes decides
        // whether its closing `_` would land against a word character.
        if let Some(guard) = self.em_guard.as_mut()
            && !guard.decided
        {
            if escape::class(next) == escape::Class::Word {
                guard.decided = true;
                guard.word_follows = true;
            } else {
                self.em_guard = None;
            }
        }
        if let Some((open, close)) = self.flank_guard.take()
            && escape::class(next) == escape::Class::Word
        {
            self.buf.replace_range(open..open + 1, "*");
            self.buf.replace_range(close..close + 1, "*");
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
        self.settle_flank(Some(first));
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
                if self.flank_guard.is_some() || self.em_guard.is_some() {
                    self.settle_flank(Some(c));
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
        if self.flank_guard.is_some() || self.em_guard.is_some() {
            self.settle_flank(Some(c));
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
    Link {
        href: String,
        title: Option<String>,
    },
    CodeSpan,
    /// A GFM table cell's content (RFC 008): rendered inline, then joined
    /// into the row line by the caller -- not spliced back through
    /// [`Sink::splice`] the way a link or code span is. `|` is escaped once
    /// the whole cell is assembled (`escape::escape_table_cell_pipes`,
    /// addendum A), not per character while writing: a nested capture (a
    /// code span, a link) splices its own already-rendered string in whole,
    /// bypassing per-character escaping entirely, so the cell boundary is
    /// the only point that sees the final text regardless of what produced
    /// it.
    Cell,
    /// `<sup>`/`<sub>` (RFC 009 §4.3): the whole subtree's own rendering is
    /// captured so the decision -- every character maps to a Unicode
    /// super/subscript, or none of it changes at all -- can be made once,
    /// against what it actually rendered to (a link's `[1](#c1)`, not just
    /// its own text), not assumed from the source HTML.
    SupSub,
}

/// What [`Sink::emphasis_open`] wrote, and enough of the destination's prior
/// state to undo it cleanly (RFC 037: an emphasis with no rendered content
/// emits no delimiters at all). Opaque outside this module -- the renderer
/// only ever holds one of these to hand back unchanged.
pub(super) struct EmphasisMark {
    /// Buffer position before `emphasis_open` did anything at all -- before
    /// even its own `flush_space`. Removing an empty span truncates back to
    /// here, not just past the delimiter: a space flushed only because this
    /// span was about to open must not become a real, separate byte once
    /// the span turns out to have nothing in it, or a still-pending space
    /// after it would double up instead of merging with it (RFC 037 finding
    /// C: `<p>a <b></b> b</p>` must collapse to one space, not two).
    pre_flush_start: usize,
    /// Whether `emphasis_open`'s own `flush_space` actually wrote a byte.
    flushed_space: bool,
    /// Buffer position right before the delimiter itself (after any flush).
    start: usize,
    delimiter: &'static str,
    at_line_start: bool,
    newlines_emitted: usize,
    line: Line,
    /// RFC 037 addendum: whether this span's delimiter is a tentative
    /// `<strong><em>`-order `_`-swap, not yet confirmed safe. Carried
    /// through, opaque, from [`Sink::emphasis_open`] to
    /// [`Sink::emphasis_close_or_remove`], which is the only place it is
    /// read.
    intraword_candidate: bool,
}

impl EmphasisMark {
    /// The offset just after the delimiter this mark records. Nothing
    /// written since means the span this mark belongs to is still exactly
    /// as it was on opening -- used both to decide emptiness on close, and
    /// by a nested emphasis deciding whether it would open touching this one
    /// (RFC 037's asymmetric `_`-swap: `<em><strong>` already parses back
    /// correctly as written, only `<strong><em>` needs it, so only that
    /// direction swaps).
    pub(super) fn end(&self) -> usize {
        self.start + self.delimiter.len()
    }

    /// The offset just before the delimiter this mark records -- used by the
    /// renderer's RFC 037 addendum flanking check to look at what comes
    /// before an open Bold ancestor's own delimiter, which is
    /// already-written history, not a lookahead.
    pub(super) fn start(&self) -> usize {
        self.start
    }
}

/// What [`Sink::strikethrough_open`] wrote, and enough of the destination's
/// prior state to undo it (RFC 009 §4.2): `EmphasisMark` without a delimiter
/// field or the RFC 037 addendum's swap bookkeeping, neither of which a
/// single-form delimiter like `~~` needs.
pub(super) struct StrikeMark {
    /// Buffer position before `strikethrough_open` did anything at all,
    /// including its own `flush_space` -- the same reason `EmphasisMark`
    /// keeps one (RFC 037 finding C).
    pre_flush_start: usize,
    flushed_space: bool,
    start: usize,
    at_line_start: bool,
    newlines_emitted: usize,
    line: Line,
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

    /// Whether a GFM table cell's content is currently being captured (RFC
    /// 008): `<br>` renders as literal `<br>` there, not a real hard break,
    /// since a cell's content is one line.
    pub(super) fn in_table_cell(&self) -> bool {
        self.captures
            .iter()
            .any(|open| matches!(open.kind, Capture::Cell))
    }

    /// Whether the next content byte in the current destination starts a line.
    pub(super) fn at_line_start(&self) -> bool {
        self.dest_ref().at_line_start
    }

    /// Whether the next byte written lands directly against a letter or
    /// digit, with nothing pending between (RFC 043): a `_` written there
    /// is flanked by a word on its left, so it cannot open emphasis -- the
    /// only thing an unescaped subscript marker has to avoid.
    pub(super) fn glued_to_word(&self) -> bool {
        let dest = self.dest_ref();
        !dest.last_was_space
            && !dest.at_line_start
            && escape::class(dest.prev_char()) == escape::Class::Word
    }

    /// Whether the current paragraph already holds an unescaped `_` that
    /// could be waiting for a partner (RFC 043) -- an emphasis delimiter, or
    /// a stray one left where a span's own delimiters did not pair. A `_`
    /// glued to a word on its left can *close* such an opener, so a
    /// subscript marker is escaped when one is there. Not counted: a `_`
    /// between two word characters (`snake_case`, which can neither open nor
    /// close) and one that is itself a safe subscript marker (`x_(`).
    ///
    /// Reads back to the last blank line, capped, since this runs only when
    /// a marker is about to be written and more escaping is always safe.
    pub(super) fn underscore_may_pair(&self) -> bool {
        let buf = &self.dest_ref().buf;
        let mut start = buf
            .rfind("\n\n")
            .map_or(0, |i| i + 2)
            .max(buf.len().saturating_sub(4096));
        while !buf.is_char_boundary(start) {
            start += 1;
        }
        let chars: Vec<char> = buf[start..].chars().collect();
        let word = |c: Option<&char>| c.is_some_and(|c| c.is_alphanumeric());
        (0..chars.len()).any(|i| {
            if chars[i] != '_' {
                return false;
            }
            // Escaped: an odd number of backslashes directly before it.
            let backslashes = chars[..i].iter().rev().take_while(|&&c| c == '\\').count();
            if backslashes % 2 == 1 {
                return false;
            }
            let before = i.checked_sub(1).and_then(|j| chars.get(j));
            let after = chars.get(i + 1);
            let intraword = word(before) && word(after);
            let safe_marker = word(before) && after == Some(&'(');
            !(intraword || safe_marker)
        })
    }

    pub(super) fn ends_with_newline(&self) -> bool {
        self.dest_ref().buf.ends_with('\n')
    }

    /// The current destination's length: used to tell whether a new
    /// emphasis span would open touching an already-open one (RFC 037), the
    /// same way [`EmphasisMark`] tells [`emphasis_close_or_remove`](Self::emphasis_close_or_remove)
    /// whether one closes touching nothing at all.
    pub(super) fn dest_len(&self) -> usize {
        self.dest_ref().buf.len()
    }

    /// The character in the current destination immediately before byte
    /// offset `pos` -- used by the renderer's RFC 037 addendum flanking
    /// check, which must look further back than the buffer's current end
    /// (already-written history, not a lookahead: `pos` is always an
    /// earlier [`EmphasisMark::start`]).
    pub(super) fn char_before(&self, pos: usize) -> Option<char> {
        self.dest_ref().buf[..pos].chars().next_back()
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

    /// Forces a blank line before content that would otherwise collide with
    /// a different CommonMark construct if it merely continued on the next
    /// line (RFC 038): a nested list whose own first item is empty would be
    /// read as the parent's own setext-heading underline (unordered child)
    /// or absorbed as the parent's own lazy-continuation text (ordered
    /// child) -- destroying the nested list and, for the ordered case, any
    /// non-empty siblings after it too.
    ///
    /// This is disambiguation, not looseness: RFC 035 §3.1's tight/loose
    /// rule and its own block-counting are unchanged by this call. A
    /// genuinely tight item's newline budget is `ensure_newlines`'s to
    /// enforce, and that clamp to one is deliberately bypassed here -- this
    /// blank line is not describing the source's structure, only keeping
    /// the output unambiguous. (CommonMark will still read the surrounding
    /// list as loose once it sees this blank line, since looseness is a
    /// property of the bytes, not of what emitted them; that consequence is
    /// unavoidable and does not need mirroring in `ensure_newlines`'s own
    /// accounting elsewhere.)
    ///
    /// Still suppressed when nothing but markers precedes on this line (an
    /// all-empty nested chain, RFC 036 §5.5, or an item with no real
    /// content of its own yet) -- the same as any other boundary; there is
    /// no parent text yet for a nested list to be confused with.
    pub(super) fn force_disambiguating_blank_line(&mut self) {
        if self.only_markers && !self.is_capturing() {
            return;
        }
        let dest = self.dest();
        if dest.buf.is_empty() {
            dest.at_line_start = true;
            return;
        }
        dest.pending_newlines = dest.pending_newlines.max(2);
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
    /// actually written (see below) plus an [`EmphasisMark`] the caller must
    /// hand back to [`emphasis_close_or_remove`](Self::emphasis_close_or_remove)
    /// when the element leaves.
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
    ///
    /// `intraword_candidate` (RFC 037 addendum) marks a caller-decided
    /// `<strong><em>`-order swap to `_` as not yet confirmed safe: the
    /// character before it was checked already (by the caller, since that is
    /// already-written history), but what follows the whole nested span's
    /// close is not known until later. Carried on the returned
    /// [`EmphasisMark`] to [`emphasis_close_or_remove`](Self::emphasis_close_or_remove),
    /// which is where the confirmation happens, once it can.
    pub(super) fn emphasis_open(
        &mut self,
        delimiter: &'static str,
        intraword_candidate: bool,
    ) -> (&'static str, EmphasisMark) {
        self.begin_content();
        self.end_leading_strip();
        // Flushing here, as part of this span's own opening, rather than
        // leaving the caller to do it first: an empty span (RFC 037) must be
        // able to undo the flush along with its delimiter, or a space
        // pending only because this span was about to open becomes a real,
        // separate byte that a later, still-pending one then duplicates.
        let pre_flush_start = self.dest().buf.len();
        self.flush_space();
        let dest = self.dest();
        let flushed_space = dest.buf.len() != pre_flush_start;
        let at_line_start = dest.at_line_start;
        let newlines_emitted = dest.newlines_emitted;
        let line = dest.line;
        let start = dest.buf.len();
        let mut written = delimiter;
        if let Some((open, len, end, _)) = dest.last_emphasis
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
        (
            written,
            EmphasisMark {
                pre_flush_start,
                flushed_space,
                start,
                delimiter: written,
                at_line_start,
                newlines_emitted,
                line,
                intraword_candidate,
            },
        )
    }

    /// Closes the innermost emphasis with `delimiter`.
    ///
    /// RFC 037 addendum: before writing this span's own closing delimiter,
    /// checks whether the span that closed immediately before it (nothing
    /// written since) was an unconfirmed intraword `_`-swap candidate. If
    /// so, this close is that swap's outer Bold ancestor's own -- the first
    /// point at which what follows the *whole* nested span is fixed, since
    /// nothing else can be written before this delimiter is. The check must
    /// happen before `markup` writes anything: `markup`'s own write would
    /// otherwise settle the guard against this delimiter's first character
    /// (always punctuation), never learning what genuinely comes after.
    fn emphasis_close(&mut self, delimiter: &str, intraword_candidate: bool) {
        let dest = self.dest();
        // RFC 044: the Bold a swapped span was made against is closing now, so
        // everything that follows the span inside it is written. If a word
        // character followed the span's close, and nothing between it and here
        // is another delimiter (a second emphasis inside the Bold makes
        // `***q*a*r***`, which reads back worse than what it replaces), the
        // swap reverts to `*` at both ends.
        if let Some(guard) = dest.em_guard
            && guard.decided
            && dest.emphasis_opens.last() == Some(&guard.strong_open)
        {
            dest.em_guard = None;
            if guard.word_follows && !dest.buf[guard.close + 1..].contains(['*', '_', '~']) {
                dest.buf.replace_range(guard.open..guard.open + 1, "*");
                dest.buf.replace_range(guard.close..guard.close + 1, "*");
            }
        }
        let pending = dest
            .last_emphasis
            .and_then(|(open, len, end, was_candidate)| {
                (was_candidate && end == dest.buf.len()).then_some((open, end - len))
            });
        self.markup(delimiter);
        let dest = self.dest();
        if let Some(guard) = pending {
            dest.flank_guard = Some(guard);
        }
        let opened_at = dest.emphasis_opens.pop();
        // RFC 044: this span's own close, if it is a `_`-swapped one. A closing
        // `_` cannot close against a letter or digit that follows it, so
        // `<b><em>q</em>a</b>` written `**_q_a**` loses the emphasis and leaves
        // the underscore literal. The swap was chosen when the span opened,
        // before that next character existed, so it is settled later: guard
        // the pair, let the next write say whether a word character follows,
        // and revert both `_` to `*` when the Bold closes (below) -- `***q*a**`
        // reads back as `strong(em("q") "a")`.
        //
        // Only where `*` would do better: the character before the close must
        // itself be a letter or digit, or a closing `*` cannot close either; and
        // the span must not start with whitespace, where no delimiter opens
        // (`*** a*b**` opens nothing, and would lose the Bold along with it).
        if intraword_candidate
            && dest.em_guard.is_none()
            && let Some(open) = opened_at
            && let Some(&strong_open) = dest.emphasis_opens.last()
            && escape::class(
                dest.buf[..dest.buf.len() - delimiter.len()]
                    .chars()
                    .next_back(),
            ) == escape::Class::Word
            && escape::class(dest.buf[open + 1..].chars().next()) != escape::Class::Whitespace
        {
            dest.em_guard = Some(EmGuard {
                open,
                close: dest.buf.len() - delimiter.len(),
                strong_open,
                decided: false,
                word_follows: false,
            });
        }
        dest.last_emphasis =
            opened_at.map(|open| (open, delimiter.len(), dest.buf.len(), intraword_candidate));
    }

    /// Closes the emphasis `mark` records, or -- if nothing was written
    /// since it opened -- removes it instead: no delimiters for an emphasis
    /// whose rendered content is empty (RFC 037), the same shape as the
    /// settled rule for a link with no text and no image (RFC 024 criterion
    /// 7). Restores the destination's line-start bookkeeping to what it was
    /// before the delimiter was written, so content after a removed,
    /// genuinely-empty span is not left thinking a line already started.
    ///
    /// `last_was_space` is deliberately left alone either way: content that
    /// collapsed to nothing but was whitespace (`<b> </b>`) must still leave
    /// its pending space for whatever comes next, exactly as if the element
    /// were not there at all.
    pub(super) fn emphasis_close_or_remove(&mut self, mark: EmphasisMark) {
        if self.dest().buf.len() == mark.start + mark.delimiter.len() {
            let dest = self.dest();
            dest.buf.truncate(mark.pre_flush_start);
            dest.emphasis_opens.pop();
            dest.at_line_start = mark.at_line_start;
            dest.newlines_emitted = mark.newlines_emitted;
            dest.line = mark.line;
            // The space this span's own opening flushed, if any, goes back
            // to being merely pending -- not gone, so it can still merge
            // with whatever comes next, and not a committed byte either, so
            // a still-pending one after this (now nonexistent) span does not
            // duplicate it.
            if mark.flushed_space {
                dest.last_was_space = true;
            }
        } else {
            self.emphasis_close(mark.delimiter, mark.intraword_candidate);
        }
    }

    /// Opens a `~~` span (RFC 009 §4.2): unlike `**`/`*`, GFM strikethrough
    /// has only one delimiter form, so none of [`emphasis_open`]'s
    /// interchange bookkeeping (`emphasis_opens`, `last_emphasis`, the RFC
    /// 037 addendum swap) applies -- reusing it here would risk that swap
    /// firing off an unrelated `~~` open and writing `__` instead. This is
    /// what `emphasis_open` would be without any of that: flush, remember
    /// enough to undo, write.
    pub(super) fn strikethrough_open(&mut self) -> StrikeMark {
        self.begin_content();
        self.end_leading_strip();
        let pre_flush_start = self.dest().buf.len();
        self.flush_space();
        let dest = self.dest();
        let now = dest.buf.len();
        // RFC 009 addendum, 2026-09-23: touching the previous `~~` span's own
        // close, with nothing written since -- merge into it instead of
        // opening a second one. `~~~~` mid-line reads back as ONE span whose
        // content includes the literal `~~~~` (RFC 037's collapse rule,
        // uncaught for this delimiter until the review found it); at the
        // start of a line it is a tilde code fence, destroying the content
        // outright. Removing the previous close and continuing its span
        // renders identically to the two adjacent sources and loses only
        // the element boundary between them.
        if let Some((open, end)) = dest.last_strike
            && end == now
        {
            dest.last_strike = None;
            dest.buf.truncate(end - 2);
            return StrikeMark {
                pre_flush_start,
                flushed_space: dest.buf.len() != pre_flush_start,
                start: open,
                at_line_start: dest.at_line_start,
                newlines_emitted: dest.newlines_emitted,
                line: dest.line,
            };
        }
        let flushed_space = now != pre_flush_start;
        let mark = StrikeMark {
            pre_flush_start,
            flushed_space,
            start: now,
            at_line_start: dest.at_line_start,
            newlines_emitted: dest.newlines_emitted,
            line: dest.line,
        };
        dest.put("~~");
        dest.line.head = Head::Inline;
        dest.newlines_emitted = 0;
        dest.at_line_start = false;
        mark
    }

    /// Closes `mark`, or -- if nothing was written since it opened --
    /// removes it instead: no delimiters for a `~~` span with no rendered
    /// content, the same rule [`emphasis_close_or_remove`] applies to an
    /// empty `<strong>`/`<em>` (RFC 037). A merged span (see
    /// [`strikethrough_open`](Self::strikethrough_open)) can never reach the
    /// empty branch -- `mark.start` is the first span's own start, and a
    /// merge only happens once that span already has real content past it.
    pub(super) fn strikethrough_close_or_remove(&mut self, mark: StrikeMark) {
        if self.dest().buf.len() == mark.start + 2 {
            let dest = self.dest();
            dest.buf.truncate(mark.pre_flush_start);
            dest.at_line_start = mark.at_line_start;
            dest.newlines_emitted = mark.newlines_emitted;
            dest.line = mark.line;
            if mark.flushed_space {
                dest.last_was_space = true;
            }
        } else {
            self.markup("~~");
            let dest = self.dest();
            dest.last_strike = Some((mark.start, dest.buf.len()));
        }
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
        open.dest.settle_flank(next);
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
        document.settle_flank(None);
        let mut out = document.buf;
        let end = out.trim_end().len();
        out.truncate(end);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }
}
