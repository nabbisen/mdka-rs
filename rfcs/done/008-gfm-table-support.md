# RFC 008 — GFM table support

**Status.** Implemented (2.4.0) — closed 2026-09-23 at `7e7cec2`
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0`
**Source.** The roadmap's oldest open item (P1, moved from M3 on 2026-09-16) and bekoedit's item 1. Written now because they have **pinned `2.4.0`** partly on it: `.git-exclude/review-request/bekoedit-2.4.0-commitment/README.md`.
**Touches.** `src/traversal.rs`, `src/renderer.rs`, `src/renderer/sink.rs`, `src/utils.rs`, `docs/src/api/elements.md`, `tests/output_validity/`.
**Depends on.** Nothing. **Depended on by** RFC 009 (element coverage) and RFC 012 (benchmark regeneration).

---

## 0. Closed, 2026-09-23

Two slices, three commits, all seven CI workflows green on each landing.

| Slice | Commit | Delivered |
|---|---|---|
| `008a` | `02c56e7` | grid resolution, the expressibility pre-pass, GFM emission for the already-expressible tables, a non-welding fallback for the rest |
| `008b` | `7e7cec2` | F1 flatten cell blocks, F2 synthesise a header, F3 expand spans |

**Measured outcome: 27% → 90%.** Of the eleven tables in §2's corpus, ten now emit a real GFM table —
measured by conversion and parsed with `ENABLE_TABLES`, not by classification. Wikipedia's "Markdown"
article went from **0 of 7** to **7 of 7**.

**The one that remains is the row-header pattern** — `<th>` as each row's first cell — which §4.1 says must
stay inexpressible, because GFM has no row-header concept to express it with. A decision, not a gap.

**Welding is impossible on either path**, which is where this started: `H1H2ab`, in the element the
documentation called *"the largest gap"*.

Four shapes are permanently inexpressible and fall back to one paragraph per cell, losslessly: more than one
whole-row header, the row-header pattern, a nested `<table>`, and `<caption>`. No GFM syntax exists for any
of them.

**F4, raw-HTML passthrough, was never built** — §4 left it as a future opt-in and nothing required it.

### What the work actually cost, against §6's estimate

§6 declined to promise `2.4.0` until two things were prototyped, and was right to. Both came in **smaller**
than feared: the no-blocks cell context turned out to be hooks on the existing renderer rather than a
parallel one, and span expansion was correct on its first attempt. The size estimate of **L** held; the
weight was in the pre-pass and the cell context, as predicted, not in the grid.

**`2.4.0` is promisable on tables.**

## 1. Summary

Today a table's cells are concatenated **with no separator at all**:

```
<table><tr><th>H1</th><th>H2</th></tr><tr><td>a</td><td>b</td></tr></table>   ->  "H1H2ab"
```

That is the `<div>` welding defect of RFC 036 §5.2, in the element the documentation calls *"the largest
gap"*.

**The happy path is the small part of this RFC.** The decision it exists to make is what happens to the
tables GFM cannot express — which, measured, is most of them.

## 2. The measurement

Eleven tables from five real pages, classified by whether they can become a plain GFM table:

| Page | Tables | Expressible | Blocked by |
|---|---|---|---|
| Rust Book ch03-02 | 2 | **2** | — |
| MDN `<table>` element | 2 | 1 | block content in a cell |
| Wikipedia "Markdown" | 7 | **0** | block content 5, colspan/rowspan 5, no `<th>` 1 |
| MDN HTTP status, Python datastructures | 0 | — | no tables |
| **Total** | **11** | **3 (27%)** | block in cell 6, colspan/rowspan 5, no `<th>` 1 |

**27%.** The remaining 73% is the RFC.

*(Method note: an earlier count said 0% because the "block content" regex matched each table's own `<table>`
tag. Corrected by testing cell contents only. `<p>` in a cell is **not** counted as a blocker — it flattens
to inline trivially.)*

## 3. What GFM can and cannot do — verified, not assumed

Every row below was checked against `pulldown-cmark` with `ENABLE_TABLES`.

**Can:** alignment (`:--`, `--:`), escaped pipes (`a \| b`), ragged rows — **short rows are padded and long
rows truncated, gracefully**, so uneven `<tr>` lengths are not a blocker. `<br>` inside a cell survives as
inline HTML.

**Cannot, each one verified to break the table:**

| Shape | What GFM does with it |
|---|---|
| No header row | **Not a table at all** — the whole thing becomes a paragraph |
| Two header rows | The first becomes a stray paragraph above the table |
| Two paragraphs in a cell | The row truncates; the second paragraph escapes the table |
| A code block in a cell | The fence splits the table and leaks an empty code block |
| A list in a cell | The row truncates and the list escapes below it |
| colspan / rowspan | No syntax exists |
| Nested table | No syntax exists (a cell holds inline content only) |
| `<caption>` | No syntax exists |

## 4. The decision: what to emit for the 73%

Four mechanisms, all verified to work; the question is which to apply and in what order.

| # | Mechanism | Fidelity | Cost |
|---|---|---|---|
| **F1** | **Flatten cell blocks to inline** — a list becomes `x, y, z`; paragraphs join with `<br>` (verified: survives in a cell) | Content kept, structure inside the cell lost | Cheap. Covers the 6 "block in cell" cases |
| **F2** | **Synthesise a header** when there is no `<th>` — an empty header row (verified: parses) or promote row 1 | Grid kept; an empty header is ugly, promoting row 1 invents a claim the source did not make | Cheap. Covers 1 case |
| **F3** | **Expand spans** — repeat the content across covered cells, or fill the neighbours empty (both verified) | Grid kept, the span itself lost. Repeating duplicates data; empty loses it | Moderate. Covers 5 cases |
| **F4** | **Raw HTML passthrough** — emit the `<table>` verbatim (verified: parses as one `HtmlBlock`, intact, and stays a separate block between paragraphs) | **Total** | Cheap to implement, but the output stops being Markdown |

> **Accepted 2026-09-22 with §4's F4 question left open — deliberately, and it blocks nothing.** Unlike
> RFC 036 §6, the two readings of F4 ("never" vs "parked as a future opt-in") produce **the same
> implementation for `2.4.0`**: F1+F2+F3, no F4. It is a question about a *future* option, and RFC 039 Half B
> owns the option surface. The sub-preferences below — empty header over promoted row 1, repeat over empty
> neighbours — **do** affect the work, are stated here, and are accepted with the RFC.

**F4 is the one that needs an owner decision**, because it changes what "conversion" means. Arguments both
ways, honestly:

- **For:** it is the only option that loses nothing, CommonMark permits it, and most renderers pass it
  through. A consumer archiving pages keeps their tables.
- **Against:** a paste-as-Markdown feature — bekoedit's exact use — cannot put raw HTML into a user's
  document. Sanitising consumers will strip it and get *nothing* where today they at least get the text. And
  `Minimal` exists for "LLM input", where an HTML blob is the wrong answer.

**My recommendation: F1 + F2 + F3 as the default, F4 never automatic.** Always emit a GFM table, degrading
the cell contents rather than abandoning the grid, because the grid is the part a reader needs and the
part we lose entirely today. Offer F4 later as an **opt-in option** if a consumer asks for archival
fidelity — but not in this RFC, and not as a mode default, because no current consumer has asked for it and
RFC 039 is in the middle of reducing the option surface, not growing it.

**Within F2, prefer the empty header row** over promoting row 1: an empty header is visibly empty, whereas
promoting a data row silently asserts a heading the author never wrote. **Within F3, prefer repeating** the
spanned content over leaving neighbours empty — a reader can see a repeat is a span artifact; a blank cell
reads as missing data.

## 4.1 Amendment, 2026-09-22 — F1 and F3 settled

`008a`'s review raised two questions §4 did not answer. Both are architect decisions within the accepted
F1/F2/F3 framework, settled here so `008b` can be scoped. Every rule below was verified against
`pulldown-cmark` with `ENABLE_TABLES`, not reasoned about.

### F3 — header spans: repeat the label. **A measurement reversed my intuition.**

My instinct was to make a span in the header a blocker: repeating a header label across columns asserts two
columns are both called "Group", where the source said one merged label. For *body* cells a repeat reads as
a visible artifact; for a header it changes what the columns are named.

**Then I measured it.** Across four span-heavy pages: **18 tables use spans, and all 18 have a span in the
header** — 8 have one *only* there.

So treating a header span as a blocker would send **every** span-using table to the fallback and make F3
worth exactly nothing. **Rule: repeat the label across the columns it spanned**, the same rule as body
cells, header included. A `rowspan` starting in the header and carrying into the body fills the covered body
cell with the same content, for the same reason.

*(Method: the header region is `<thead>` where present, else the first `<tr>`. On a table whose first row is
data this mislabels — but 18 of 18 is lopsided enough that the direction holds.)*

### F1 — flattening, per block type

A GFM cell holds inline content only. Each rule keeps the words and gives up only the block structure:

| In a cell | Emit | Verified |
|---|---|---|
| Two or more paragraphs | join with `<br>` | `a<br>b` → two text runs, one cell |
| `<ul>` / `<ol>` | one item per `<br>`-separated run, prefixed `- ` or `N. ` | `- one<br>- two` → literal markers, reads as a list |
| A **nested** list | the same, indented two spaces per level | `  - nested` — **leading spaces survive** in the cell |
| `<pre><code>` | **one code span per line**, `<br>`-joined | `` `line1`<br>`line2` `` ✔. **Not** `` `line1<br>line2` `` — inside a code span the tag is literal text |
| A heading | its text, **plain** | Bold would invent emphasis the source never had — the defect RFC 037 exists to fix |
| A blockquote | its text, **without** the `>` | `> quoted` in a cell is literal text, not a quote |
| A nested table | unchanged: a **blocker**, the whole outer table falls back | GFM has no nesting |

**One honest limit:** the nested-list indent survives in the Markdown but most HTML renderers collapse
consecutive spaces, so the depth may not be visible once rendered. `&nbsp;` would render but pollutes the
cell's text with non-breaking spaces that every downstream consumer then carries. **Plain spaces, and accept
the limit.**

### Row-header tables stay inexpressible

From `008a`'s finding 5: a table using `<th>` as each row's *first cell* rather than as a whole first row is
correctly inexpressible under "no header row", and must stay so — GFM has no row-header concept to express
it with. **This is not a span problem** and `008b` must not try to solve it as one.

## 5. Acceptance criteria

1. All eleven measured tables convert; the three expressible ones become GFM tables whose parse matches the
   source's grid, verified by **parsing** with `ENABLE_TABLES`.
2. No table output ever welds cells. `H1H2ab` must be impossible.
3. Alignment from `align=` and `text-align:` is carried into `:--`/`--:`.
4. `|` in cell content is escaped; verified by parsing it back into one cell.
5. A table **inside a list item and inside a blockquote** carries the container prefix on every line — RFC
   035's machinery should cover it, but it must be proven, not assumed.
6. A table inside `<pre>` writes no table markup (RFC 024 rule 7).
7. `<caption>` content is not lost — emitted adjacent to the table, not silently dropped.
8. Cells for every shape in §3, every mechanism in §4, all five modes.
9. All three suites; `fmt`/`clippy`; counts reported.

## 6. Size — re-estimated honestly

**The roadmap says L. L is right, and the weight is not where the roadmap implies.**

Building the grid and emitting a well-formed GFM table is perhaps a third of it. The rest:

- **The expressibility pre-pass** — deciding per table which mechanisms apply, in the same place
  `structure_hints` already computes `wrappers`, `loose_lists` and `needs_disambiguation`.
- **A cell renderer that cannot emit blocks** — the sink has never had a context that forbids block output;
  today every block path assumes it may write a newline. This is the genuinely new machinery.
- **Span expansion** into a rectangular grid, which is a real algorithm (`rowspan` carries down across
  subsequent `<tr>`s), not a flag.
- **Interaction with everything M3 and M4 built** — container prefixes, escaping, the line model. Criterion
  5 exists because a table in a list item touches all of it.

**I would not promise `2.4.0` on this until the pre-pass and the no-blocks cell renderer are prototyped.**
That is the honest answer to the question that prompted this RFC, and it is why it is written before being
scheduled rather than after.

## 7. Not in scope

`<del>`/`<s>`, `<dl>`, `<sup>`/`<sub>`, task-list checkboxes — RFC 009, which depends on this. Any new
public option — RFC 039 Half B owns the option surface. F4 as a shipped feature (§4).
