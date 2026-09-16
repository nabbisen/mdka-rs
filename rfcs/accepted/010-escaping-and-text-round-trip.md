# RFC 010 — Escaping and text round-trip

**Status.** Accepted (2026-09-16, owner)
**Author.** Architect
**Created.** 2026-09-16 (number reserved in `ROADMAP.md` since M1)
**Milestone.** M3 · Output validity → `2.3.0` (M3 reshape accepted 2026-09-16)
**Sequencing.** After RFC 024 and RFC 028 — escaping writes through the sink RFC 024 builds, and RFC 028 changes the same renderer arms.
**Source.** The RFC 025 harness inventory (`7338b17`) — 24 cells; audit findings A-03, A-04, A-05, A-09, A-10, A-11, D-05.

---

## 1. Summary

Text mdka emits does not reliably parse back to the text the HTML contained. Some of it
adds visible backslashes; some of it **destroys content**: a line of `~~~` swallows the
rest of the document into a code block, `<div>` in text vanishes as an HTML block,
`1986.` at the start of a list item becomes a nested list and loses the number, and a
destination containing a space produces no link at all.

**The obligation:** every piece of text mdka emits — prose, code, destinations, titles,
alt text — parses back, under CommonMark **and** GFM, to exactly the source text, in
every context it can appear in. The RFC 025 harness is the measure.

## 2. Why it happens — one function, one context

All prose escaping is `write_normalised` in `src/utils.rs`, with a fixed per-character
table:

```rust
'\\' | '*' | '_' | '`' | '[' | ']' | '!' => escape always
'#' | '>' | '+' | '-' if line_start      => escape
'0'..='9' if line_start                  => escape      // the digit, not the delimiter
_ => push
```

Its only notion of context is `at_line_start`, and that has three consequences:

1. **The wrong character is escaped.** `\1986.` — CommonMark only honours a backslash
   before ASCII punctuation, so the backslash survives into the text. The delimiter
   (`.` or `)`) is what needs escaping.
2. **The context is lost after a prefix.** After `- ` or `> ` is written, `at_line_start`
   is false, so text at the start of a list item or quote gets **no** escape — and `1986.`
   becomes a list.
3. **The same table runs inside code.** Code spans and blocks honour no escapes, so
   `snake_case` in `<code>` becomes a literal `snake\_case`.

And some things are handled nowhere: `<` and `&` in text, `~~~` fences, destinations and
titles (written raw by the `<a>`/`<img>` arms), and code-span delimiter length.

## 3. Design — escape by context, not by character

Replace the single table with **escaping chosen by the context being written**. The
contexts, and the rule for each:

### 3.1 Code span (inline `<code>`)

- **No backslash escapes** inside. Text verbatim.
- Delimiter = a backtick run **one longer than the longest backtick run in the content**.
- If the content begins or ends with a backtick, **pad with one space** on both sides
  (CommonMark §6.1 strips exactly one).
- Cells: `underscore_in_code_span`, `markdown_in_code_span`, `backslash_in_code_span`,
  `text_in_code`, `code_span_with_backtick`, `…double_backtick`, `…starting_with_backtick`.

### 3.2 Fenced code block (`<pre>`)

- Content verbatim.
- Fence = backticks, length **max(3, longest backtick run at a line start in the content
  + 1)**. The info string must not contain a backtick; if the language hint does, drop the
  hint rather than break the fence.
- Cells: `fence_content_with_triple_backticks`, `…four_backticks`.

### 3.3 Link and image destination

- If the destination contains a space, a control character, or **unbalanced** parentheses,
  use the **angle-bracket form** `<…>`, escaping `<`, `>` and `\` inside it.
- Otherwise emit bare, escaping `\` and unbalanced `(`/`)`. Balanced parentheses — which
  the harness showed already parse — stay unescaped.
- Cells: `link_destination_with_space`, `…_unbalanced_paren`, `image_source_with_space`.

### 3.4 Link and image title

- Delimit with `"`; escape `"` and `\` inside. (Or choose `'` or `(…)` when that avoids
  escapes — implementer's choice, stated.)
- Cells: `link_title_with_quotes`, `image_title_with_quotes`.

### 3.5 Link text and image alt

- Escape `[`, `]` and `\` when they would unbalance the brackets; prose rules otherwise.
- Cell: `bracket_in_image_alt`.

### 3.6 Prose — the start of a block's content

"Block start" means the **first content of a paragraph, heading, list item, table cell
(RFC 008), or line inside a blockquote** — after any `- `, `1. ` or `> ` prefix has been
written. The sink RFC 024 builds knows this; `at_line_start` alone does not.

At block start, escape what would otherwise open a construct:

| Text begins with | Would become | Escape |
|---|---|---|
| digits then `.` or `)` then space/end | ordered list | the **delimiter**: `1986\.`, `1\)` |
| `-`, `+`, `*` then space/end | bullet list | `\-` |
| `#`…`######` then space/end | heading | `\#` |
| `>` | blockquote | `\>` |
| `` ``` `` or `~~~` | fenced code block | first character |
| `---`, `***`, `___` (thematic break) | thematic break | first character |
| `<` followed by an HTML block start condition | HTML block | `\<` |
| a line of `=` or `-` directly under a paragraph line | setext heading | first character |
| **GFM:** a table delimiter row — cells of `-` with optional leading/trailing `:`, separated by `\|`, with or without outer pipes — directly under a line containing `\|` | GFM table | escape so the row is not a delimiter row (e.g. the first `-`, `:` or `\|`) — *added 2026-09-16 from `025c`* |

Cells: `digit_period_at_line_start`, `digit_paren_at_line_start`,
`digit_period_in_list_item`, `digit_period_in_blockquote`, `tilde_fence_text`,
`html_tag_text`, `bekoedit_digit_period_escape`.

### 3.7 Prose — inline

| Text | Would become | Rule |
|---|---|---|
| `*`, `_` | emphasis | escape **only where they could form a delimiter run** — CommonMark's flanking rules. **Intraword `_` needs no escape** (A-10) |
| `` ` `` | code span | escape |
| `[`, `]` | link | escape when they could pair |
| `!` | image | escape **only before `[`** (A-10) |
| `<` followed by a tag or autolink shape | raw HTML / autolink | `\<` |
| `&` followed by an entity-reference shape | entity | `\&` |
| `\` | escape | escape when followed by ASCII punctuation |
| **GFM:** `~~` **and single `~`** around text | strikethrough | escape — `025c` found both |

Cells: `autolink_like_text`, `entity_like_text`, plus the GFM cells `025c` adds.

**Minimal escaping is a requirement, not a nicety** (A-10): noisy source costs every
consumer who diffs, commits or hand-edits the output. The harness guards the other side
— an escape removed incorrectly breaks round-trip and turns a cell red.

### 3.8 Adjacent emphasis (A-11)

`<strong>a</strong><em>b</em>` → `**a***b*`: a `***` run CommonMark resolves by its own
flanking rules. Not a character-escaping defect, but the same obligation — emitted
delimiters must parse as meant. Rule: when a closing delimiter run would touch an opening
one of the same character, **use `_` for one of them** where flanking allows, or separate
them otherwise. **RFC 025 has no cell for this** — add them (adjacent strong/em, em/strong,
strong/strong).

## 4. Documentation — D-05

`docs/src/api/text-processing.md`'s escaping table is wrong in the audit's three ways.
Rewrite it to match §3 **after** the implementation lands, describing contexts rather
than a character list.

## 5. Not in scope

- **Where bytes go** — RFC 024.
- **Blocks inside inline elements** — RFC 028.
- NBSP collapse (A-14) — a whitespace question, not escaping; recorded as a harness gap.
- Emitting GFM constructs (tables, strikethrough) — RFC 008, RFC 009.

## 6. Compatibility

Output bytes change widely: escapes removed (`snake_case`), moved (`1986\.`), added (`\<`,
`\&`, `\~~~`), and destinations re-delimited. **Rendered meaning becomes correct**; source
bytes are not stable across this release. A minor-release change, stated plainly in the
CHANGELOG with before/after examples.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Removing an escape breaks round-trip somewhere untested | The harness: every property runs under CommonMark and GFM in all five modes. Add a cell before removing any escape the harness does not already cover. |
| Flanking-rule escaping is subtle | Implement against CommonMark §6.2's definitions, cite them in comments, and test left-/right-flanking cases explicitly. |
| Block-start context is mis-tracked after prefixes | RFC 024's sink owns prefix state; this RFC consumes it. If the sink does not expose block start, raise it against RFC 024. |
| Performance — context checks on the hot path | Single pass, no allocation per character, as today. Benchmark before/after with the existing bench suite. |

## 8. Acceptance criteria

1. Every RFC 025 cell owned by RFC 010 has its `known_defect` marker removed and passes —
   under CommonMark **and** GFM, in all five modes.
2. No other RFC 025 cell changes state, or each change is reported with its owner.
3. §3.8 adjacent-emphasis cells added and passing.
4. Code spans and fenced blocks contain **no** backslash escapes.
5. `snake_case_here` emits no escapes (A-10).
6. `text-processing.md` rewritten to §3 (D-05).
7. Benchmarks within noise of 2.2.3, or the regression stated and accepted.
8. CHANGELOG with before/after examples.
