//! Escaping by context (RFC 010 §3).
//!
//! There is no global character table. What a character needs depends on
//! where it is written:
//!
//! - **Prose** (§3.6, §3.7): [`decide`] classifies one literal character from
//!   the line it is on and the character before it. Some escapes depend on the
//!   character *after* it -- `1986.` is an ordered-list marker only when a
//!   space or the line end follows, `!` opens an image only before `[`. Those
//!   characters are written unescaped with a [`Wait`]; the sink settles it
//!   when the next byte is written, whatever writes it (text, markup, a line
//!   break), and inserts the backslash then. So the lookahead is exact across
//!   text nodes, markup and captures, and no escape is added "in case".
//! - **Code spans and fenced blocks** (§3.1, §3.2): no escapes; the delimiter
//!   is sized to the content ([`code_span`], and the sink's fence).
//! - **Destinations and titles** (§3.3, §3.4): [`destination`], [`title`].
//!
//! Every backslash written here precedes ASCII punctuation, which CommonMark
//! always honours as an escape outside code (§2.4), so an escape is never
//! left behind as a literal backslash.
//!
//! Flanking (CommonMark §6.2) reduces to two facts per delimiter character.
//! A single `*` or `~` can open or close unless whitespace is on both sides.
//! A `_` can open or close unless whitespace is on both sides or it is
//! intraword -- a letter or digit on both sides. A run of such characters is
//! checked character by character against its real neighbours, which escapes
//! at least every character a run-level check would.

#[cfg(test)]
mod tests;

/// Unicode whitespace as CommonMark defines it: space, tab, line feed, form
/// feed, carriage return, and the `Zs` category.
pub(super) fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\n' | '\u{0c}' | '\r' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

/// A neighbour of a delimiter character. The start or end of a line, and the
/// start or end of the destination, count as whitespace (§6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Class {
    Whitespace,
    /// A letter or digit.
    Word,
    /// Anything else, treated as punctuation. Treating a symbol as punctuation
    /// only ever adds an escape: a `_` stays unescaped only between two words.
    Punctuation,
}

pub(super) fn class(c: Option<char>) -> Class {
    match c {
        None => Class::Whitespace,
        Some(c) if is_whitespace(c) => Class::Whitespace,
        Some(c) if c.is_alphanumeric() => Class::Word,
        Some(_) => Class::Punctuation,
    }
}

/// Where the current line of prose stands, for block-start escaping (§3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Head {
    /// No literal text or markup yet on this line (container prefixes and
    /// list markers do not count: the block's content starts after them).
    Start,
    /// Only digits so far, this many: a `.` or `)` next may be a list marker.
    Digits(u8),
    /// Past the start.
    Inline,
}

/// The line state one destination keeps for escaping.
#[derive(Clone, Copy, Debug)]
pub(super) struct Line {
    pub(super) head: Head,
    /// The line continues a paragraph after a hard break, so a setext
    /// underline or a GFM table delimiter row could form here.
    pub(super) continuation: bool,
    /// The line above, in the same paragraph, contains `|`.
    pub(super) pipe_above: bool,
    /// This line contains `|`.
    pub(super) pipe_here: bool,
    /// The line is an ATX heading's content: a trailing `#` run would be read
    /// as its closing sequence.
    pub(super) heading: bool,
}

impl Line {
    pub(super) const START: Line = Line {
        head: Head::Start,
        continuation: false,
        pipe_above: false,
        pipe_here: false,
        heading: false,
    };

    pub(super) const INLINE: Line = Line {
        head: Head::Inline,
        continuation: false,
        pipe_above: false,
        pipe_here: false,
        heading: false,
    };

    /// A line break was written. `continuation`: a hard break inside a
    /// paragraph, not a block boundary.
    pub(super) fn newline(&mut self, continuation: bool) {
        *self = Line {
            head: Head::Start,
            continuation,
            pipe_above: continuation && self.pipe_here,
            pipe_here: false,
            heading: false,
        };
    }
}

/// An escape that depends on the next character written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Wait {
    /// `.` or `)` after 1–9 digits at a line start: an ordered-list marker
    /// when a space, a tab or the line end follows.
    ListDelimiter,
    /// `-` (`run`) or `+` at a line start: a bullet when a space or the line
    /// end follows; for `-`, also the start of a thematic break.
    Bullet { run: bool },
    /// `#` at a line start (an ATX heading) or after whitespace in a heading
    /// (its closing sequence): when `#`, whitespace or the line end follows.
    Hash,
    /// `*` or `~`: a delimiter unless whitespace is on both sides.
    Flank { prev: Class },
    /// `_`: a delimiter unless whitespace is on both sides or it is intraword.
    Underscore { prev: Class },
    /// `!`: an image before `[`.
    Bang,
    /// `<`: raw HTML, an HTML block or an autolink before a letter, `/`, `!`
    /// or `?`.
    Lt,
    /// `&`: an entity or numeric character reference before a letter, a digit
    /// or `#`.
    Amp,
    /// `\`: an escape before ASCII punctuation, a hard break before a line end.
    Backslash,
}

impl Wait {
    /// Whether the waiting character needs its backslash, now that `next` is
    /// known (`None`: nothing follows in this destination).
    pub(super) fn escapes(self, next: Option<char>) -> bool {
        let ws = class(next) == Class::Whitespace;
        match self {
            Wait::ListDelimiter => ws,
            Wait::Bullet { run } => ws || (run && next == Some('-')),
            Wait::Hash => ws || next == Some('#'),
            Wait::Flank { prev } => !(prev == Class::Whitespace && ws),
            Wait::Underscore { prev } => {
                let next = class(next);
                !((prev == Class::Whitespace && next == Class::Whitespace)
                    || (prev == Class::Word && next == Class::Word))
            }
            Wait::Bang => next == Some('['),
            Wait::Lt => {
                next.is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'))
            }
            Wait::Amp => next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '#'),
            Wait::Backslash => next.is_some_and(|c| c.is_ascii_punctuation() || c == '\n'),
        }
    }
}

/// Whether any context can escape `c` past the start of a line. Every other
/// character is written as it is without consulting [`decide`].
pub(super) fn may_escape(c: char) -> bool {
    matches!(
        c,
        '`' | '[' | ']' | '*' | '~' | '_' | '!' | '<' | '&' | '\\' | '#'
    )
}

/// What to do with one literal prose character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Decision {
    Plain,
    Escape,
    Wait(Wait),
}

/// Decides one literal prose character `c`, written after `prev`, and moves
/// the line state past it. `link_text`: inside a link's text or an image's
/// alt, where `]` would end it.
pub(super) fn decide(c: char, prev: Option<char>, line: &mut Line, link_text: bool) -> Decision {
    let head = line.head;
    line.head = match head {
        Head::Start if c.is_ascii_digit() => Head::Digits(1),
        Head::Digits(n) if c.is_ascii_digit() && n < 9 => Head::Digits(n + 1),
        _ => Head::Inline,
    };
    match head {
        Head::Start => match c {
            '>' | '*' | '_' => Decision::Escape,
            '-' if line.continuation => Decision::Escape,
            '-' => Decision::Wait(Wait::Bullet { run: true }),
            '+' => Decision::Wait(Wait::Bullet { run: false }),
            '#' => Decision::Wait(Wait::Hash),
            '=' if line.continuation => Decision::Escape,
            '|' | ':' if line.continuation && line.pipe_above => Decision::Escape,
            _ => inline(c, prev, line, link_text),
        },
        Head::Digits(_) if matches!(c, '.' | ')') => Decision::Wait(Wait::ListDelimiter),
        _ => inline(c, prev, line, link_text),
    }
}

fn inline(c: char, prev: Option<char>, line: &Line, link_text: bool) -> Decision {
    match c {
        '`' | '[' => Decision::Escape,
        ']' if link_text => Decision::Escape,
        '*' | '~' => Decision::Wait(Wait::Flank { prev: class(prev) }),
        '_' => Decision::Wait(Wait::Underscore { prev: class(prev) }),
        '!' => Decision::Wait(Wait::Bang),
        '<' => Decision::Wait(Wait::Lt),
        '&' => Decision::Wait(Wait::Amp),
        '\\' => Decision::Wait(Wait::Backslash),
        '#' if line.heading && class(prev) == Class::Whitespace => Decision::Wait(Wait::Hash),
        _ => Decision::Plain,
    }
}

// ─── Code ───────────────────────────────────────────────────────────────────

/// A code span holding `content` verbatim (§3.1): the delimiter is one
/// backtick longer than the longest backtick run inside, and content that
/// begins or ends with a backtick is padded with one space on each side,
/// which CommonMark strips.
pub(super) fn code_span(content: &str) -> String {
    let fence = "`".repeat(longest_run(content, '`') + 1);
    let pad = if content.starts_with('`') || content.ends_with('`') {
        " "
    } else {
        ""
    };
    let mut out = String::with_capacity(content.len() + 2 * (fence.len() + 1));
    out.push_str(&fence);
    out.push_str(pad);
    out.push_str(content);
    out.push_str(pad);
    out.push_str(&fence);
    out
}

fn longest_run(s: &str, c: char) -> usize {
    let (mut longest, mut run) = (0, 0);
    for ch in s.chars() {
        run = if ch == c { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    longest
}

/// Tracks the longest backtick run that starts a line of a fenced block's
/// content -- after at most three spaces, where it could close the fence
/// (§3.2).
#[derive(Clone, Copy, Debug)]
pub(super) struct FenceScan {
    pub(super) longest: usize,
    at_line_start: bool,
    indent: u8,
    run: usize,
}

impl FenceScan {
    pub(super) const NEW: FenceScan = FenceScan {
        longest: 0,
        at_line_start: true,
        indent: 0,
        run: 0,
    };

    pub(super) fn feed(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.at_line_start = true;
                    self.indent = 0;
                    self.run = 0;
                }
                _ if !self.at_line_start => {}
                ' ' if self.run == 0 && self.indent < 3 => self.indent += 1,
                '`' => {
                    self.run += 1;
                    self.longest = self.longest.max(self.run);
                }
                _ => self.at_line_start = false,
            }
        }
    }

    /// The fence length: three, or one more than the longest run.
    pub(super) fn fence_len(&self) -> usize {
        (self.longest + 1).max(3)
    }
}

// ─── Destinations and titles ────────────────────────────────────────────────

/// A link or image destination (§3.3). The `<…>` form when the destination
/// holds a space or a control character, has unbalanced parentheses, or
/// starts with `<`; otherwise bare. A line break can appear in neither form
/// and is written as a character reference (`&#10;`), which CommonMark decodes
/// in destinations; so a literal `&` before an entity shape is escaped.
pub(super) fn destination(href: &str) -> String {
    let angle = href.starts_with('<')
        || href
            .chars()
            .any(|c| c == ' ' || (c.is_ascii_control() && c != '\n' && c != '\r'))
        || !parens_balanced(href);
    let mut out = String::with_capacity(href.len() + 2);
    if angle {
        out.push('<');
    }
    let mut chars = href.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            _ => {
                let next = chars.peek();
                let escape = match c {
                    '\\' => next.is_none_or(|n| n.is_ascii_punctuation()),
                    '&' => next.is_some_and(|n| n.is_ascii_alphanumeric() || *n == '#'),
                    '<' | '>' => angle,
                    _ => false,
                };
                if escape {
                    out.push('\\');
                }
                out.push(c);
            }
        }
    }
    if angle {
        out.push('>');
    }
    out
}

fn parens_balanced(s: &str) -> bool {
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return false,
            ')' => depth -= 1,
            _ => {}
        }
    }
    depth == 0
}

/// A link or image title with its delimiters (§3.4): `"…"`, or `'…'` when the
/// title holds `"` but not `'`, or `(…)` when it holds both but no
/// parenthesis -- so a title needs no escape unless it holds all three kinds.
/// Then `"…"`, with `"` escaped. In every form a backslash before ASCII
/// punctuation or the closing delimiter, and `&` before an entity shape, are
/// escaped, and a line break is a character reference: a blank line would end
/// the paragraph.
pub(super) fn title(t: &str) -> String {
    let (open, close) = if !t.contains('"') {
        ('"', '"')
    } else if !t.contains('\'') {
        ('\'', '\'')
    } else if !t.contains('(') && !t.contains(')') {
        ('(', ')')
    } else {
        ('"', '"')
    };
    let mut out = String::with_capacity(t.len() + 2);
    out.push(open);
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek();
        match c {
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            _ => {
                let escape = c == close
                    || c == open
                    || (c == '\\' && next.is_none_or(|n| n.is_ascii_punctuation()))
                    || (c == '&' && next.is_some_and(|n| n.is_ascii_alphanumeric() || *n == '#'));
                if escape {
                    out.push('\\');
                }
                out.push(c);
            }
        }
    }
    out.push(close);
    out
}

// ─── Table cells ────────────────────────────────────────────────────────────

/// Escapes every `|` in `s` not already escaped, so it cannot end a GFM
/// table cell early (RFC 008, addendum A). Run once over a cell's *whole*
/// assembled content -- the cell is the escaping boundary, not any one path
/// through it. A cell's own direct text and inline-formatting content is
/// already correctly escaped by [`decide`] as it is written; what that
/// cannot reach is a nested capture's own already-rendered string spliced in
/// whole (a code span's verbatim backticks, a link's captured text and
/// destination, an image's alt) -- each bypasses per-character escaping
/// entirely by design (RFC 024), so their `|` is untouched until this runs.
///
/// Safe to run unconditionally over the whole string: a `|` already preceded
/// by an escaping `\` is left alone (no double escape), and a literal `\`
/// the source itself contained before punctuation is already doubled by
/// [`decide`]'s own `Wait::Backslash` rule before this ever sees it, so a
/// single `\` immediately before `|` here is always an escape, never raw
/// content -- tracked by parity, so `\\|` (an escaped backslash, then a
/// genuinely unescaped `|`) still gets the `|` escaped.
pub(super) fn escape_table_cell_pipes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut escaped = false;
    for c in s.chars() {
        if c == '|' && !escaped {
            out.push('\\');
        }
        out.push(c);
        escaped = c == '\\' && !escaped;
    }
    out
}
