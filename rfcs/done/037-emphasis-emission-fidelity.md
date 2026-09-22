# RFC 037 — Emphasis emission fidelity: empty and nested

**Status.** Implemented (2.4.0) — closed 2026-09-23 at `81f0ec5`
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0` (owner, 2026-09-22)
**Sequencing.** Independent of RFC 036; either order.
**Source.** `2.3.0` consumer pass, findings §5.3 and §5.4 — `.git-exclude/reviewed/2.3.0-consumer-pass/README.md`.
**Touches.** `src/renderer.rs`, `tests/output_validity/`.

---

## 0. Closed, 2026-09-23 at `81f0ec5`

Empty emphasis no longer writes delimiters around nothing: `a<b></b>b` → `ab`, and a paragraph containing
only an empty emphasis emits **nothing at all** rather than `****`, which read as a thematic break. Nested
same-class emphasis collapses, so `<em><em>x</em></em>` is italic again instead of bold.

**§4 criterion 2 was relaxed mid-slice, and that was my error to fix.** Preserving nesting *order* is only
possible where the delimiters can flank: intraword, `strong(em)` is **inexpressible in CommonMark** — I
tested `***x***`, `**_x_**`, `__*x*__` and `*__x__*`, and only `em(strong)` parses. The first implementation
swapped to `_` unconditionally and **lost the bold entirely** intraword, which was worse than the inverted
order it replaced. The shipped rule: **never lose an emphasis level to gain ordering** — swap only when the
surrounding characters are absent or non-alphanumeric, otherwise emit `***x***`.

Implemented with a two-sided flanking check: the preceding character is already-written history and is
checked at open time; what follows the whole nested span is not, so the swap is written provisionally and a
deferred guard patches it back if the next character turns out alphanumeric. I probed it across block ends,
punctuation, adjacent emphasis, code spans, images, `<br>`, **link captures** (correct both inside and
outside), container prefixes and **non-ASCII** — `日本`, `é` and digits all correctly suppress the swap,
where an ASCII-only check would have failed.

## 1. Summary

Neither is a `2.3.0` regression; both reproduce identically in `2.2.3`.

| # | HTML | Output | Reader gets |
|---|---|---|---|
| §5.3 | `<p>a<b></b>b</p>` | `a****b` | literal `a****b` |
| §5.3 | `<p>Hello <b></b>world</p>` | `Hello ****world` | literal asterisks mid-sentence |
| §5.3 | `<p>a<em></em>b</p>` | `a**b` | literal `a**b` |
| §5.4 | `<p><em><em>x</em></em></p>` | `**x**` | **bold** — source has no bold |
| §5.4 | `<p><em><i>x</i></em></p>` | `**x**` | **bold** — mixed tags do it too |

## 1.1 Re-derived at `2e9af49`, 2026-09-22 — three additions before handover

Probed beyond §1's shapes after acceptance. All of the following are **pre-existing in `2.2.3`**; none is a
regression.

**A. Two shapes change content type — the RFC 038 family, in emphasis clothing.**

```
<p><b></b></p>              ->  "****"  ->  hr        an empty paragraph becomes a thematic break
<p><em><em></em></em></p>   ->  "****"  ->  hr
```

A paragraph whose only content is empty emphasis emits four asterisks on their own line, which CommonMark
reads as a thematic break. **In scope**: it is the same defect as §5.3 — delimiters emitted around nothing —
and it is the more damaging half, because the content type changes rather than merely showing junk.

**B. Nesting order is lost, and §4's criterion 2 does not hold today.**

```
<p><em><strong>x</strong></em></p>   ->  "***x***"  ->  em(strong("x"))
<p><strong><em>x</em></strong></p>   ->  "***x***"  ->  em(strong("x"))     order inverted
```

Both orders produce identical bytes, and both parse as `em(strong(…))`. Criterion 2 as written — *"still
parse back to both, nested, in the source's order"* — therefore **fails at the baseline**. It is not
unreachable: order is expressible by mixing delimiters, verified —

```
**_x_**   ->  strong(em("x"))          _**x**_   ->  em(strong("x"))
```

— and **mdka already alternates to `_`** when adjacent emphasis would otherwise merge
(`src/renderer/sink.rs:634–640`; `<p><b>a</b><b>b</b></p>` → `__a__**b**`). So this is the existing
mechanism applied to a new position, not a new concept. Keep criterion 2.

**C. Whitespace-only emphasis leaks its whitespace.**

```
<p>a<b> </b>b</p>   ->  "a**** b"
```

Criterion 3 already requires whitespace-only emphasis to be treated as empty; recorded here because the
space currently survives *outside* the delimiters, so the expected result is `a b`, not `ab`.

**Also observed, out of scope:** `<b><i><b>x</b></i></b>` → `*****x*****` → `em(strong(strong("x")))`
(criterion 2's collapse rule should reach it); `<del><del>x</del></del>` → `x`, strikethrough dropped
entirely — that is RFC 009's element coverage, not this.

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
