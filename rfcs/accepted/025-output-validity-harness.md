# RFC 025 — Markdown output-validity harness

**Status.** Accepted 2026-08-31 — implementer may start
**Tracks.** M3 · Conversion fidelity → `2.3.0`
**Priority.** P0 — **lands before RFC 024, 010 and 008**
**Touches.** `tests/`, `Cargo.toml` dev-dependencies.
**Source.** External audit 2026-08-31, `C-08` (High).
**Prepared.** 2026-08-31

## Summary

No test verifies that mdka's output is Markdown. Every renderer assertion compares
against a string a human wrote by hand. Add a harness that parses the output with
a real CommonMark parser and checks the structure that comes back.

## Why this is P0 and why it is first

136 tests were green while all of the following shipped:

```
<a href="/page"><img src="i.png" alt="pic"></a>  →  ![pic](i.png)[](/page)
<a href="/x"><strong>b</strong></a>              →  ****[b](/x)
<pre>plain</pre><p>After</p>                     →  plain\n```\n\nAfter
<code>snake_case</code>                          →  `snake\_case`
<a href="/a b.html">x</a>                        →  [x](/a b.html)
```

Every one of these has a passing test asserting exactly that output. The tests
are not wrong about what the renderer does — they are **complete records of the
wrong answer**, because the expected value was authored by the same person who
wrote the code, from the same misunderstanding.

**A suite that only compares output to expectations written by its own author
cannot discover that the output is not Markdown.** That is the control gap, and
fixing it before the renderer fixes means those fixes are verified by something
other than a second hand-written string.

This is the same lesson as `verify-ci` and RFC 020's install gate, applied to the
product itself: check what the consumer receives, not what we intended to emit.

## Design

### 1 · Round-trip through a CommonMark parser

Add `pulldown-cmark` as a **dev-dependency only** — it must not enter the
published dependency graph.

For a fixture, assert on the **parsed event stream**, not on the Markdown text:

```
HTML  →  mdka  →  Markdown  →  pulldown-cmark  →  events  →  assert structure
```

`<a href="/page"><img alt="pic" src="i.png"></a>` must yield a link containing an
image — one `Start(Link)`, one `Image` inside it, one `End(Link)`. Today it
yields an image followed by an empty link, and the assertion fails whatever the
exact spacing.

**Assert on structure, not on bytes.** A byte-comparison test is what we already
have. The point is to let the parser tell us what the output *means*.

### 2 · The composition matrix

Most defects are in **composition** — an element inside another element — and
none of the current fixtures nest inline constructs. Build a matrix of inline
constructs inside every container that captures or transforms:

| | in `<a>` | in `<pre>` | in `<code>` | in `<li>` | in `<blockquote>` | in heading |
|---|---|---|---|---|---|---|
| `<img>` | | | | | | |
| `<strong>` | | | | | | |
| `<em>` | | | | | | |
| `<code>` | | | | | | |
| `<a>` | | | | | | |
| text with `_ * [ ] ( )` | | | | | | |

Each cell: convert, parse, assert the structure is what the HTML meant. Cells
that are known-unsupported get an explicit `#[ignore]` with the RFC that owns
them — visible as unfinished rather than absent.

### 3 · The escaping round-trip property

For text content: whatever mdka escapes must parse back to the **original
characters**. `snake_case` in a code span must come out of the parser as
`snake_case`, not `snake\_case`.

The audit notes this alone catches three findings in roughly fifteen lines.

### 4 · Fence and destination well-formedness

- A fenced block's opening fence must be longer than any backtick run inside it.
- A link destination containing a space, `(`, `)` or `"` must parse back to the
  original URL — either angle-bracketed or escaped.

## What this does not do

It is not a full CommonMark conformance suite. mdka is not a CommonMark
implementation; it is a producer. The obligation is that **what it emits parses
to what it meant**, not that it handles every spec case.

It is also not a differential test against another converter. Peer output is not
a specification.

## Sequencing

**Land this first, with the failing cells recorded.** RFC 024 and RFC 010 then
turn cells green. If the harness is written after the fixes, it can only confirm
the fixer's own belief — the exact trap that produced the current suite.

Expect a substantial number of failures on landing. That is the deliverable, not
a problem: an honest inventory of where the output is not Markdown.

## Compatibility

Tests only. No API change, no runtime dependency.

## Risks

| Risk | Mitigation |
|---|---|
| `pulldown-cmark` leaks into published deps | Dev-dependency only. Verify with `cargo tree -e normal`. |
| The matrix is large and slow | It is small string conversions; if it becomes slow, that is a finding. |
| Disagreement over the "intended" structure for a cell | Where genuinely ambiguous, record the question in the review request rather than picking. Ambiguity is a design question for RFC 024/010. |
| Failing tests land on `main` | Use `#[ignore]` with the owning RFC named, so CI stays green and the inventory stays visible. Do **not** delete a cell to make the suite pass. |

## Acceptance criteria

1. `pulldown-cmark` is a dev-dependency and absent from `cargo tree -e normal`.
2. The composition matrix exists, every cell either asserting or `#[ignore]`d
   with an owning RFC.
3. The escaping round-trip property test exists.
4. Assertions are on parsed structure, not on output bytes.
5. The review request lists **every failing or ignored cell** — this inventory is
   the primary output of this RFC.
6. CI is green: passing tests pass, known-broken cells are `#[ignore]`d, none
   deleted.
