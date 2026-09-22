# RFC 037 — Emphasis emission fidelity: empty and nested

**Status.** Proposed
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0` (proposed)
**Sequencing.** Independent of RFC 036; either order.
**Source.** `2.3.0` consumer pass, findings §5.3 and §5.4 — `.git-exclude/reviewed/2.3.0-consumer-pass/README.md`.
**Touches.** `src/renderer.rs`, `tests/output_validity/`.

---

## 1. Summary

Neither is a `2.3.0` regression; both reproduce identically in `2.2.3`.

| # | HTML | Output | Reader gets |
|---|---|---|---|
| §5.3 | `<p>a<b></b>b</p>` | `a****b` | literal `a****b` |
| §5.3 | `<p>Hello <b></b>world</p>` | `Hello ****world` | literal asterisks mid-sentence |
| §5.3 | `<p>a<em></em>b</p>` | `a**b` | literal `a**b` |
| §5.4 | `<p><em><em>x</em></em></p>` | `**x**` | **bold** — source has no bold |
| §5.4 | `<p><em><i>x</i></em></p>` | `**x**` | **bold** — mixed tags do it too |

## 2. Why

**§5.3 — delimiters are emitted before the content is known to exist.** An empty emphasis element writes its
open and close delimiters around nothing; `**` `**` becomes the literal text `****`. Empty
`<b></b>`/`<strong></strong>` is routine WYSIWYG and CMS output — an editor leaves the element behind when
its text is deleted.

The damage is bounded: CommonMark's flanking rules stop the stray `**` pairing with a later real one in the
cases tried, so it shows as visible junk rather than a swallowed span. Bounded, not acceptable.

**§5.4 — two nested single-delimiter emphases concatenate into a double.** `*` + `*x*` + `*` is `**x**`,
which is `<strong>`. The italic-family case is a **meaning change**: the source says italic twice and the
output says bold. `<b><b>x</b></b>` → `****x****` parses as nested `<strong>`, ugly but not a meaning change.

The consumer pass reported the identical-tag cases; the mixed case `<em><i>` is mine, and it means the rule
is about the *delimiter*, not about matching tag names.

## 3. Design

1. **Emit no delimiters for emphasis whose rendered content is empty.** The element contributes nothing.
   This is the same shape as the settled rule for a link with no text and no image (RFC 024 criterion 7) and
   should read like it.
2. **Collapse nested emphasis of the same delimiter class to one level.** Italic inside italic is italic;
   bold inside bold is bold. Bold inside italic and italic inside bold keep both, and must still produce
   output that parses back to both.

## 4. Acceptance criteria

1. Every row in §1 produces what the source means, verified by **parsing the output**: `a b` for the empty
   cases, `para(em("x"))` for nested italic, `para(strong("x"))` for nested bold.
2. `<em><strong>x</strong></em>` and `<strong><em>x</em></strong>` still parse back to both, nested, in the
   source's order.
3. Emphasis containing only whitespace, and emphasis containing only a dropped element, are treated as
   empty and tested as such.
4. Non-empty emphasis output is **byte-identical to `2.3.0`** for the shapes named here — the narrow form,
   not the corpus.
5. Harness cells for empty and doubly-nested emphasis, all five modes.

## 5. Not in scope

Reading `style` to *add* emphasis — `<span style="font-weight:700">` is still flattened (consumer pass
§6.2). That is a separate candidate, already disclosed to bekoedit as a candidate and not a commitment, and
it needs its own decision because it turns mdka into a partial CSS consumer.
