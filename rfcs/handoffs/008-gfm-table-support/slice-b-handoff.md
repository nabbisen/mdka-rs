# Developer Handoff — RFC 008 slice `008b` · F1 flatten, F2 header, F3 spans

**Governing RFC.** [RFC 008](../../done/008-gfm-table-support.md) — **§4.1 is new and settles what you were blocked on**; §4 for the mechanisms, §2 for why this matters
**Milestone.** M4 · `2.4.0`
**Priority.** P1
**Prepared.** 2026-09-22
**Baseline.** `02c56e7` — seven workflows green; **523 Rust / 42 Node / 90 Python**; `output_validity` 305

---

## 0. What changed since you asked

You were blocked on two design questions. **RFC 008 §4.1 answers both**, and one answer is the opposite of
what I would have written before measuring — read it before starting.

**This slice is where the value is.** `008a` rescued the 27% that were already expressible. `008b` is the
**73%**: every table with a span, a block in a cell, or no header. Today they fall back to one paragraph per
cell — lossless, but a seven-column table becomes a long vertical run with the grid gone.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. F2 — synthesise a header

A table with no `<th>` is not a GFM table at all. **Emit an empty header row** — verified to parse:
`|  |  |` over `| --- | --- |`.

**Not** promoting row 1: an empty header is visibly empty, where a promoted data row silently asserts a
heading the author never wrote. (RFC 008 §4, accepted.)

## 2. F3 — expand spans

Resolve through `008a`'s existing grid, then **repeat the content across every covered cell**, header
included.

**Read §4.1 for why the header is not a blocker.** I intended to make it one; the measurement — 18 of 18
span-using tables have a header span, 8 only there — showed that would have made F3 worthless. A `rowspan`
starting in the header and carrying into the body fills the covered body cell the same way.

**Row-header tables are not this slice's problem.** `<th>` as each row's first cell stays inexpressible;
§4.1 says why, and it must not be treated as a span to expand.

## 3. F1 — flatten cell blocks

The table in §4.1 is the whole specification, each row verified against a GFM parser. In summary: paragraphs
join with `<br>`; lists become `<br>`-separated `- `/`N. ` runs, two spaces of indent per nesting level; a
code block becomes **one code span per line, `<br>`-joined** — *not* one span containing `<br>`, where the
tag is literal text inside the code; a heading contributes its text **plain**, because bolding it would
invent emphasis the source never had; a blockquote contributes its text without the `>`.

A **nested table** remains a blocker. Do not try to flatten it.

**`<br>`-as-literal is already built.** `008a` did exactly this plumbing for a real `<br>` in a cell,
including the pending-space flush. Reuse it.

## 4. The machinery you predicted

Your `008a` §5 said the no-blocks cell context looks like *"one more branch in machinery already in place"* —
`begin_block`/`end_block` (or `ensure_newlines`) noticing `in_table_cell()` and writing an inline join
instead of a real newline — and that this was a plausible design you had not built.

**That is the design I am asking for.** If it turns out wrong once you are in it, stop and say what you
found rather than forcing it; a second `008a`-sized surprise is worth hearing early, not late.

**The guarantee `008a` relied on now changes.** There, `render_cell` was only ever called on cells the
pre-pass had proven block-free — a narrower guarantee than "a context that forbids blocks", as you said
yourself. `008b` removes that: cells with blocks now render. **A block boundary must not be able to write a
raw newline into a row's string**, or the table silently breaks. That is the slice's central risk.

## 5. Acceptance criteria

1. Every shape in §4.1's table produces the specified output, verified by **parsing** with `ENABLE_TABLES`.
2. The Wikipedia-class tables from RFC 008 §2 — spans plus blocks in cells — become GFM tables whose grid
   matches the source. Report how many of the eleven measured tables are expressible after this slice.
3. **No raw newline can enter a row.** Prove it for each F1 block type, not only the ones in §4.1.
4. Pipe escaping still holds for every context `02c56e7` fixed, now with flattened content in the cell.
5. `008a`'s guarantees re-proven, not assumed: container prefix in a list item and a blockquote, `<pre>`
   suppression, `<caption>` survival.
6. Non-table output **byte-identical to `02c56e7`**.
7. Cells for every rule, all five modes.
8. All three suites reported separately; `fmt`/`clippy`; counts against 523 / 42 / 90.

## 6. And the question I still need answered

**Does `2.4.0` look achievable now?** You declined to promise it at `008a`, rightly. bekoedit have pinned
that version on tables and have had no date from us. Tell me plainly when you know — an early "no" costs
nothing and is worth far more than a late yes.

## 7. Report back

`.git-exclude/review-request/008b-table-fallback-mechanisms/README.md`: what moved, before/after **parses**
for every §4.1 rule, criterion 2's count, criterion 3's evidence, the byte-identical set, the three test
counts, and anything in §4.1 I got wrong now that you have built it.
