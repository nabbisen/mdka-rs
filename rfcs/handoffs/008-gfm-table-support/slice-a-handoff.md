# Developer Handoff — RFC 008 slice `008a` · Grid, pre-pass, and the expressible 27%

**Governing RFC.** [RFC 008](../../done/008-gfm-table-support.md) — §2 the measurement, §3 what GFM can and cannot do, §4 the mechanisms, §6 the size
**Milestone.** M4 · `2.4.0`
**Priority.** P1
**Prepared.** 2026-09-22
**Baseline.** `3110e6d` — seven workflows green; 488 Rust / 42 Node / 90 Python

---

## 0. Why this is a slice and not the whole RFC

RFC 008 §6 says I would not promise `2.4.0` on tables until two things are prototyped: **the expressibility
pre-pass** and **a cell renderer that cannot emit blocks**. An outside consumer has pinned `2.4.0` partly on
this, so the estimate matters more than usual.

**So `008a` is both a real slice and the spike.** It must deliver working value on its own and end with a
measured answer to *"how big is the rest?"*

| | |
|---|---|
| **In `008a`** | the grid model, the pre-pass, GFM emission for tables that are already expressible, and a **non-welding fallback** for those that are not |
| **In `008b`** | F1 flatten-cell-blocks, F2 synthesise-header, F3 expand-spans — scheduled once your §5 estimate lands |
| **Never automatic** | F4 raw-HTML passthrough (RFC 008 §4). Do not build it |

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. What to build

**A grid model.** Rows and cells from `<table>`/`<thead>`/`<tbody>`/`<tfoot>`/`<tr>`/`<th>`/`<td>`, with
`colspan`/`rowspan` **resolved into a rectangular grid** — `rowspan` carries down across subsequent `<tr>`s,
which is a real algorithm, not a flag. Build the grid even when the table turns out inexpressible; §2 needs
it and `008b` will need it.

**An expressibility pre-pass**, where `structure_hints` already computes `wrappers`, `loose_lists` and
`needs_disambiguation`. Per table, decide: is this emittable as a plain GFM table? The blocking conditions
are RFC 008 §3, each verified there — no header row, more than one header row, a cell containing a block, a
span, a nested table, a caption.

**GFM emission for the expressible case.** Alignment from `align=` and `text-align:` into `:--`/`--:`.
Escape `|` in cell content. Note from §3, verified: **ragged rows are not a blocker** — GFM pads short rows
and truncates long ones, so uneven `<tr>` lengths need no special handling.

## 2. The fallback for the rest — minimum bar, not the final answer

The 73% are `008b`'s work. But **today they weld**:

```
<table><tr><th>H1</th><th>H2</th></tr><tr><td>a</td><td>b</td></tr></table>  ->  "H1H2ab"
```

That is unacceptable in this slice regardless of what `008b` does later. **Emit the cells separated** —
whatever shape you judge best for a reader; a row per line reads better than a paragraph, but I am not
prescribing it. RFC 008 criterion 2 is absolute: **`H1H2ab` must be impossible when this lands.**

Say what you chose and why.

## 3. Things that will bite

- **The no-blocks cell context is the new machinery.** The sink has never had a context that forbids block
  output; every block path assumes it may write a newline. In `008a` you only need to *detect* that a cell
  contains a block (the pre-pass) — you do **not** need to render one inline yet. **But look hard enough to
  estimate it**, because that is §5's main question.
- **A table inside a list item or a blockquote** must carry the container prefix on every line. RFC 035's
  machinery should cover it. **Prove it; do not assume it** — RFC 008 criterion 5 exists for this.
- **A table inside `<pre>`** writes no table markup (RFC 024 rule 7). Every list slice this milestone has
  had to prove it does not write into a code fence; this is the same check.
- **`<caption>`** must not be silently dropped (criterion 7).
- **Parse, do not diff.** Use a GFM-enabled parser — `ENABLE_TABLES` — or you will be checking pipe
  characters rather than tables. There is a probe at `.git-exclude/tmp/gfmprobe` if it is still there;
  otherwise it is `mdprobe` with `Options::ENABLE_TABLES`.

## 4. Acceptance criteria

1. The three expressible tables from RFC 008 §2 become GFM tables whose **parse** matches the source grid.
2. **No table output welds cells**, for any input.
3. Alignment carried; `|` escaped and verified by parsing it back into one cell.
4. A table in a list item and in a blockquote carries the prefix on every line — proven.
5. A table inside `<pre>` writes no table markup.
6. `<caption>` content survives somewhere sensible.
7. Cells for every shape in RFC 008 §3, all five modes.
8. All three suites, reported separately; `fmt`/`clippy`; counts against 488 / 42 / 90.

## 5. The part I actually need from you

**An estimate for `008b`, grounded in what you found building `008a`:**

- What does a no-blocks cell context cost in the sink? Is it a flag on the destination, a separate
  destination, or something larger?
- Is span expansion harder than the grid model suggests once real `rowspan` shapes are in front of you?
- Anything in RFC 008 §3 or §4 that is wrong, now that you have the code open.

**Say plainly whether `2.4.0` looks achievable.** A consumer has pinned that version on tables, and I would
much rather tell them early than late. A "no" here costs nothing and is worth more than an optimistic yes.

## 6. Report back

`.git-exclude/review-request/008a-table-grid-and-prepass/README.md` — what moved, before/after **parses** for
every shape, the criterion-2 evidence that welding is gone, your fallback choice and its reasoning, the three
test counts, §5's estimate, and anything here I got wrong. Every package this milestone has corrected
something of mine.
