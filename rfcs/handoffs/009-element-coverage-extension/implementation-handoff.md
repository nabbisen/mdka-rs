# Developer Handoff — RFC 009 · Element coverage extension

**Governing RFC.** [RFC 009](../../done/009-element-coverage-extension.md) — §2 the measurement, §3 what Markdown can express, §4 the decisions, §5 criteria
**Milestone.** M4 · `2.4.0`
**Priority.** P2
**Prepared.** 2026-09-23
**Baseline.** `d6db326` — seven workflows green; **537 Rust / 42 Node / 90 Python**
**Depends on.** RFC 008 — closed `7e7cec2`. §4 below is the part that touches it.

---

## 0. Four independent pieces, one of them a defect

| | |
|---|---|
| **§1** | `<dl>` welds — **the defect**, and the last one of its kind in the codebase |
| **§2** | `<del>`/`<s>` → `~~`; checkbox list items → `- [x]` / `- [ ]` — bekoedit items 5 and 6 |
| **§3** | `<sup>`/`<sub>` → Unicode where every character maps |
| **§4** | The RFC 008 interaction, which is where this could grow |

They are independent; land them in whatever order suits. **Once handed over, this file is frozen**; changes
arrive as dated addenda.

## 1. `<dl>` — the defect

```
<dl><dt>Term</dt><dd>Desc</dd><dt>T2</dt><dd>D2</dd></dl>   ->   "TermDescT2D2"
```

`<dt>` and `<dd>` each become their own block. RFC 008 did exactly this for `tr`/`td`/`th` via
`utils::block_kind`, and inexpressible tables then inherited container prefixes, `<pre>` suppression and
ordering for free. **The same route should work here** — but verify the inherited guarantees rather than
assuming them, as `008a` did.

**Nothing is invented.** No bold term, no `- ` prefix, no `—`. RFC 009 §4.1 says why, and the owner was
given the chance to overrule it and did not.

## 2. `~~` and task lists

`<del>` and `<s>` → `~~…~~`. `<ins>` and `<u>` stay plain text — GFM has no syntax for either, and `~~`
would say the opposite of what `<ins>` means.

An `<input type="checkbox">` as a list item's first content → `- [x]` or `- [ ]`. A list item **without** one
must be byte-identical to today.

**Emitting `~~` is already safe**: literal tildes in prose are escaped today (`a ~~b~~ c` → `a \~\~b\~\~ c`),
so this cannot collide with text that merely contains tildes. Confirm that still holds once you are emitting
real strikethrough — it is the one place this piece could bite.

## 3. `<sup>`/`<sub>` — Unicode, and only where it maps

**Read RFC 009 §4.3 before starting.** The measurement inverted the obvious priority: `<sup>` is the most
common element in scope at 201 occurrences, and **99% are citation markers that are already correct**. Only
the 1% mathematical case is wrong, and wrong in kind — `2<sup>7</sup>` → `27` is a different number.

If **every** character maps, emit Unicode; otherwise leave the content exactly as today.

```
2<sup>7</sup> -> 2⁷      H<sub>2</sub>O -> H₂O      x<sup>n</sup> -> xⁿ
<sup><a href="#c1">[1]</a></sup>  ->  unchanged — `[` and `]` do not map
```

Mappable: `0-9 + - = ( ) n i` superscript; `0-9 + - = ( ) a e o x h k l m n p s t` subscript.

**Criterion 4 is the important one here:** anything not fully mappable must be **byte-identical to
`7e7cec2`**. This rule exists to fix what is broken without disturbing what works.

## 4. 🛑 The RFC 008 interaction — this is the part to think about first

**§1 changes which cells RFC 008's pre-pass considers block-bearing.** Today a `<dl>` in a table cell is not
block content, so the table stays expressible and the cell welds:

```
<table><tr><th>H</th></tr><tr><td><dl><dt>T</dt><dd>D</dd></dl></td></tr></table>
  today ->  | H |
            | --- |
            | TD |          <- welded, but a real table
```

Once `dt`/`dd` are blocks, that cell **becomes block-bearing** and goes through RFC 008's F1 flattening.

**The table must stay expressible** — it must not regress to the one-paragraph-per-cell fallback — and the
cell should read `T<br>D`, consistent with how F1 already joins paragraphs.

**RFC 008 §4.1's F1 table does not list `<dl>`.** It names paragraphs, lists, code blocks, headings,
blockquotes and nested tables. Treating `<dt>`/`<dd>` as the paragraph case is the consistent reading and
what I intend; **if the code says otherwise once you are in it, stop and tell me** rather than inventing a
rule — §4.1 is amendable and that is my job, not yours.

Same question for the other three pieces in a cell: `~~`, a task marker, and a Unicode superscript should
all simply work as inline content, but criterion 7 asks you to prove it.

## 5. Acceptance criteria

RFC 009 §5, all eight. The three easiest to under-test:

- **§5.1** — `<dl>` cannot weld **for any input**, not just the example.
- **§5.4** — non-mappable `<sup>`/`<sub>` byte-identical to `7e7cec2`; the citation shape especially.
- **§5.7** — the RFC 008 interactions in §4 above, proven rather than assumed.

## 6. Report back

`.git-exclude/review-request/009-element-coverage/README.md`: what moved, before/after **parses** for every
rule, the §5.4 byte-identical set, the §4 table-cell evidence, the three test counts, and anything in RFC
009 I got wrong now that you have built it.

**One thing I would like your judgement on**, not a criterion: RFC 009 §4.1 accepts that a `<dl>` becomes
indistinguishable from a run of paragraphs, because Markdown has no definition-list syntax and inventing one
is the mistake RFC 037 exists to correct. If, having implemented it, you think that reads badly enough to be
worth revisiting, say so — the owner was offered the bolded-term alternative and declined, but an
implementer's view of the actual output is worth more than either of our predictions.
