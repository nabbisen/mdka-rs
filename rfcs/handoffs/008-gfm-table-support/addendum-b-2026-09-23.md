# Addendum — RFC 008 slice `008b`, 2026-09-23

**Amends.** `slice-b-handoff.md` (frozen; this addendum is the only correct channel)
**Issued after.** the `008b` review, `.git-exclude/reviewed/008b-table-fallback-mechanisms/README.md`
**Status.** Approved but for one fix. **27% → 90%** measured; §4.1's other seven F1 rules are exact.

---

## 1. 🛑 Required: the `<br>` before a nested list's first item

§4.1: *"one item per `<br>`-separated run, indented two spaces per level."*

```
<ul><li>a<ul><li>deep</li></ul></li></ul>   in a cell
  expected : - a<br>  - deep
  actual   : - a  - deep     parsed: TableCell("- a  - deep")  — one text run, no <br>
```

Only the **first** nested item is affected:

```
<ul><li>a<ul><li>d1</li><li>d2</li></ul></li><li>b</li></ul>
  ->  - a  - d1<br>  - d2<br>- b      missing before d1, present before d2 and b
```

A flat list is right. So the parent and the first nested item share a visual line, and the two-space indent —
genuinely present in the bytes — reads as nothing without a break before it.

**Almost certainly `cell_after_marker` firing once too often.** You reasoned that flag through for
`<li></li><li>two</li>` and caught that case; this is its neighbour.

**The test currently pins the bug.** `nested_list_in_cell_indents_two_spaces` asserts `td("- a - b")` and its
comment says the doubled indent *"is real in the Markdown bytes but not visible in the parsed tree"*. That is
true of the indent, and it is not what is happening here — with the `<br>` present the tree would read
`"- a" html"<br>" "  - b"`, exactly as it does for the second nested item. **A missing `<br>` and a collapsed
indent look identical in the tree; only one of them is present.** Correct the expectation and the comment
together.

**Criteria:** the three shapes above; flat-list and two-paragraph cells unchanged; non-table output still
byte-identical to `02c56e7`.

## 2. Optional, and worth it: pin the mode-independence your standalone tests rest on

`check()` runs one mode; `cells!` runs five. Thirteen tests now depend on table output being
mode-independent. **I verified it holds** — five F1/F3 shapes across all five modes, byte-identical — but
nothing in the suite says so, and if it ever stops being true those thirteen will not notice.

One test asserting five-mode agreement on a representative F1/F3 shape makes the other twelve honest.

## 3. The recount you declined to guess at — 27% → 90%

You were right not to invent a fraction, and right that a fresh MDN fetch does not reproduce §2's count. The
cause was the extraction: **§2 counted outermost tables**, and a plain `<table\b.*?</table>` scan also finds
the nested ones. Nesting-aware, I reproduce §2's eleven and its 27% exactly.

Measured by conversion rather than classification — each table through both binaries, each output parsed
with `ENABLE_TABLES`:

| | before | after |
|---|---|---|
| Rust Book ch03-02 | 2 / 2 | 2 / 2 |
| Wikipedia "Markdown" | **0 / 7** | **7 / 7** |
| MDN `<table>` | 1 / 2 | 1 / 2 |
| **Total** | **3 / 11 (27%)** | **10 / 11 (90%)** |

The one that remains is the **row-header pattern** — seven rows each beginning with `<th>` — which §4.1 says
must stay inexpressible. A decision, not a gap.

## 4. Everything else is approved

I re-derived criterion 6 independently: **84 non-table shapes × 5 modes = 420 comparisons against
`02c56e7`, zero differences.** 536 tests, 0 failed.

The stale-baseline catch is the best thing in the package — third time this milestone a leftover artifact has
nearly produced a false finding, and the first time the person holding it caught it before reporting.
Findings 4 and 6 name emergent behaviour rather than assuming it away, and Finding 2 fixed a genuine harness
gap rather than working around it.

## 5. After this lands

`2.4.0` is promisable on tables, and I will tell the owner so. bekoedit can then be given a date — which is
the first thing we will have been able to tell them that they did not have to ask for.
