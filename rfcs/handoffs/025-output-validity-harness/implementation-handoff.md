# Developer Handoff — RFC 025 · Markdown output-validity harness

**Governing RFC.** [RFC 025](../../accepted/025-output-validity-harness.md)
**Milestone.** M3 → `2.3.0`
**Priority.** P0 — **first work in M3, before RFC 024 and RFC 010**
**Do not start until M2b has shipped.**
**Prepared.** 2026-08-31

---

## 1. Purpose

No test verifies that mdka's output is Markdown. Every renderer assertion
compares against a string a human wrote. Add a harness that parses the output
with a real CommonMark parser and checks what comes back.

## 2. Why this is first

136 tests were green while all of this shipped:

```
<a href="/page"><img src="i.png" alt="pic"></a>  →  ![pic](i.png)[](/page)
<a href="/x"><strong>b</strong></a>              →  ****[b](/x)
<pre>plain</pre><p>After</p>                     →  plain\n```\n\nAfter
<code>snake_case</code>                          →  `snake\_case`
<a href="/a b.html">x</a>                        →  [x](/a b.html)
```

Each has a passing test asserting exactly that. The tests are **complete records
of the wrong answer** — the expected value was written by the same person as the
code, from the same misunderstanding.

**Your deliverable is not green tests. It is an honest inventory of where the
output is not Markdown.** Expect many failures. That is the product.

## 3. The parser

Add `pulldown-cmark` as a **dev-dependency only**. Verify with
`cargo tree -e normal` that it does not enter the published graph.

Assert on the **parsed event stream**, never on output bytes:

```
HTML → mdka → Markdown → pulldown-cmark → events → assert structure
```

`<a href="/page"><img alt="pic" src="i.png"></a>` must yield `Start(Link)`,
`Image` inside it, `End(Link)`. Today it yields an image then an empty link, and
the assertion fails regardless of spacing.

**A byte-comparison test is what we already have.** The point is to let the
parser tell us what the output means.

## 4. The composition matrix

Most defects are in composition, and no current fixture nests inline constructs.
Build the matrix — inline construct × container:

| | in `<a>` | in `<pre>` | in `<code>` | in `<li>` | in `<blockquote>` | in heading |
|---|---|---|---|---|---|---|
| `<img>` | | | | | | |
| `<strong>` | | | | | | |
| `<em>` | | | | | | |
| `<code>` | | | | | | |
| `<a>` | | | | | | |
| text with `_ * [ ] ( )` | | | | | | |

Each cell: convert, parse, assert the structure matches what the HTML meant.

Cells that fail get `#[ignore]` with the **owning RFC named in the attribute or
an adjacent comment** — 024 for inline-in-link and bare `<pre>`, 010 for
escaping. A cell whose owner is unclear: leave it ignored and **list it in the
review request as unowned**. Those are the interesting ones.

**Never delete a cell to make the suite pass.**

## 5. The escaping round-trip property

Whatever mdka escapes must parse back to the **original characters**.
`snake_case` inside a code span must emerge from the parser as `snake_case`, not
`snake\_case`.

The audit notes this alone catches three findings in roughly fifteen lines.

## 6. Well-formedness checks

- A fenced block's opening fence is longer than any backtick run in its content.
- A destination containing a space, `(`, `)` or `"` parses back to the original
  URL.

## 7. What this is not

Not a CommonMark conformance suite — mdka is a producer, not an implementation.
The obligation is that **what it emits parses to what it meant**.

Not a differential test against another converter. Peer output is not a spec.

**Scope boundary, per RFC 027 Rule 2.** This handoff builds the harness and
records failures. It **fixes nothing** — RFC 024 and RFC 010 do that. If you find
yourself fixing the renderer, stop: a harness written by the person fixing the
bugs is the trap that produced the current suite.

## 8. Required verification

Per RFC 027 Rule 3, state tree-vs-artifact for each.

1. `cargo tree -e normal` showing `pulldown-cmark` absent.
2. The matrix, with every cell asserting or ignored.
3. The full list of failing/ignored cells with owners — **the primary
   deliverable**.
4. `cargo test --workspace --locked` green (ignored cells do not fail CI).
5. fmt, clippy `-D warnings`.
6. Test count reconciled against 136 plus additions.

## 9. Prohibited shortcuts

- Do not fix any renderer defect. Report it.
- Do not delete or weaken a cell to get green.
- Do not assert on output bytes.
- Do not let `pulldown-cmark` become a runtime dependency.
- Do not skip a cell because you expect it to fail — that is the inventory.

## 10. Known risks

| Risk | If it happens |
|---|---|
| The "intended" structure for a cell is genuinely ambiguous | Record the question rather than picking. Ambiguity is a design input to RFC 024/010. |
| The matrix is bigger than expected | Fine. Small string conversions. |
| Some cells pass that you expected to fail | Interesting — say so. Our model of the defects may be wrong. |
| Ignored tests hide the inventory | The review request carries the list. That is why §8.3 is the deliverable. |

## 11. Acceptance checklist

- [ ] `pulldown-cmark` dev-only, absent from `cargo tree -e normal`
- [ ] Composition matrix exists; every cell asserts or is `#[ignore]`d with an owner
- [ ] Escaping round-trip property test exists
- [ ] Fence-width and destination well-formedness checks exist
- [ ] Assertions on parsed structure, not bytes
- [ ] **Full failing/ignored inventory in the review request**
- [ ] No renderer change whatsoever
- [ ] CI green; count reconciles

## 12. Escalate rather than decide

Stop and raise if: a cell's correct structure is genuinely undecidable; a failure
looks like it needs a renderer change to even express the test; or the inventory
is large enough to change M3's shape.
