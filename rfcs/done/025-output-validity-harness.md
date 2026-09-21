# RFC 025 — Markdown output-validity harness

**Status.** Implemented (2.3.0)
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

### ⚠ The matrix needs the other direction too — amended 2026-09-16

Every cell above puts an **inline construct inside a container**. That is one
direction, and as first specified the matrix had no cell for the inverse: a
**block inside an inline**.

A downstream consumer found a defect living exactly there —
`<strong><p>x</p></strong>` emits `**` around the blocks, producing stray
delimiter lines on essentially every Google Docs paste (RFC 028). The
56-finding external audit missed it too. The matrix had a direction and the
defect was on the other side of it.

Add the inverse rows: `<p>`, `<ul>`/`<li>`, `<blockquote>`, `<pre>` and a heading,
each inside `<strong>`, `<em>`, `<a>` and `<code>`.

Most of those combinations are invalid HTML, which is exactly why they need
covering: real clipboard HTML is full of invalid markup, and a converter meets it
far more often than a fixture author imagines.

### Real-world corpus

> **Corrected 2026-09-16.** bekoedit wrote that **the corpus does not exist yet** —
> their first letter implied it did. It will be captured by hand from documents
> they wrote themselves, under Apache-2.0, with **no date**
> (`.git-exclude/upstream/bekoedit/receive/2026-09-16-re-corpus-request.md`). The
> nine reproductions in their first letter were written by hand, not captured.
> This RFC's harness does not wait for it; corpus integration is slice `025b`,
> unscheduled until captures exist.

bekoedit has offered a corpus of real clipboard HTML from browsers, Google Docs,
Word and LibreOffice.

**Use it.** It is the input this project cannot generate for itself — the
fixtures here are composed from our model of what HTML looks like, and RFC 028 is
the proof that the model has gaps. Where the corpus and the hand-written matrix
disagree about what is worth testing, the corpus wins.

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
2b. **Both directions are covered** — inline-inside-container *and*
   block-inside-inline. A matrix with only the first direction is incomplete; see
   the 2026-09-16 amendment.
3. The escaping round-trip property test exists.
4. Assertions are on parsed structure, not on output bytes.
5. The review request lists **every failing or ignored cell** — this inventory is
   the primary output of this RFC.
6. CI is green: passing tests pass, known-broken cells are `#[ignore]`d, none
   deleted.

---

## Amendment — at handoff refresh, 2026-09-16: strict expected failures instead of `#[ignore]`

This RFC specifies that known-broken cells get `#[ignore]` with the owning RFC
named. **Replaced by strict expected failures** — a helper under which a marked
cell passes while its defect persists, **fails when the defect is fixed** (so the
marker must be removed), and fails on any panic or evaluation error.

**Why.** An ignored test never runs. When RFC 024 or RFC 028 fixes a defect, an
ignored cell cannot say so: the inventory goes stale, and "the fix removed the
defect" — this RFC's stated purpose for going first — is unverified. A strict
expected failure keeps the inventory executing on every CI run.

**Stricter, never looser:** CI stays green on known defects, as before, but now
also turns red when the recorded inventory stops being true.

The risk-table row and acceptance criteria 2 and 6 above read `#[ignore]`; read
them as "marked as a strict expected failure with the owner named".

---

## Amendment — review of the harness, 2026-09-16

Reasoning: `.git-exclude/reviewed/025-output-validity-harness/README.md`.

**Parse as GFM as well as CommonMark.** Most readers consumers use are GFM, where text
mdka leaves unescaped can become syntax (`~~x~~`, `|`-tables). Every property and tree
assertion runs under both option sets; a cell passes only if both readings are right.
Slice `025c`.

**Two properties beyond the original set are accepted:** `[blocks]` and
`[unterminated]`. **One is added:** `[link-content]` — a link keeps the text and images
the HTML put inside it. Slice `025c`.

**Known gap, recorded:** the properties normalise whitespace to words, so NBSP collapse
(audit A-14) is not caught.

**Specification for slice `025b` (the corpus), not built now.** Real clipboard HTML will
hit today's known defects, so the corpus cannot land green as data alone. Each corpus file
may carry a sidecar listing its **expected violations, each with an owner**, under the same
strict semantics as `known_defect`: a listed violation that no longer occurs **fails**
("remove it"), and an unlisted violation **fails**. Each file also carries the capture
metadata bekoedit described — application, version, operating system.

**Directory rule.** `corpus/` holds captured input only. Hand-written runner fixtures live
elsewhere, so the directory's provenance stays reliable.
