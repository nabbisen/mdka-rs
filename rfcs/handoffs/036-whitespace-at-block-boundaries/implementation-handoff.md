# Developer Handoff — RFC 036 · Whitespace and separators at block boundaries

**Governing RFC.** [RFC 036](../../done/036-whitespace-at-block-boundaries.md) — §3 design, §4 criteria, §4.1 re-derivation
**Milestone.** M4 · Coverage and durability → `2.4.0`
**Priority.** P1 — M4's first slice
**Prepared.** 2026-09-22
**Baseline.** `bec40bf` (tag `2.3.0`), **405 passing tests**, working tree otherwise clean.

---

## 0. Scope — three slices are handed over, one is NOT

RFC 036 covers four defects. **Slice 3 is withheld.** RFC 036 §6 asks the owner whether `<div>` unwrapping
should keep its block separation (change the code) or whether the documentation should change instead. That
question was **still open** when the RFC was accepted, and acceptance of the RFC is not acceptance of my
recommendation inside it.

| Slice | Covers | Handed over? |
|---|---|---|
| 1 | Leading whitespace in `<li>` — §5.1 | ✅ **yes** |
| 2 | Leading whitespace in a heading — §5.6 | ✅ **yes** |
| 3 | `<div>` unwrap drops the separator — §5.2 | 🛑 **no — blocked on RFC 036 §6** |
| 4 | Empty nested lists emit `- - -` — §5.5 | ✅ **yes** |

**Do not implement slice 3, and do not add harness cells that assert either answer for it.** If you finish
1, 2 and 4 and slice 3 is still unanswered, stop and say so; do not pick the direction yourselves. The
project has twice let an undecided question reach implementation as an implied decision, and this note
exists to stop a third.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Purpose

Four defects at the seam where a block's own text begins. The one that matters is slice 1: leading
whitespace inside `<li>` is not stripped, so the item's real content column is wider than the continuation
indent the sink writes, and CommonMark ends the item early. Lists split, nesting flattens, and code blocks
are ejected from their items.

**None of the four is a `2.3.0` regression** — all four reproduce identically in `2.2.3`, verified against
the published crate. Slice 1 is *less* damaging in `2.3.0` than it was before: `2.2.3` gave an empty bullet
with all content outside it. RFC 035 got most of the way; this finishes it.

**The trigger is pretty-printed HTML** — `<li>\n  <p>…</p>\n</li>`, which is what editors, CMSs and
formatters emit. One MDN page has 109 occurrences.

## 2. Current output — re-derived 2026-09-22 on a release build of `bec40bf`

Both columns are what the binary does today. The third column is the **parse** of the second, via
`.git-exclude/tmp/mdprobe`, because the damage in three of these rows is invisible to a string comparison.

### Slice 1 — leading whitespace in `<li>`

| Cell | Output today | Parses as |
|---|---|---|
| `li_ws_two_p` | `-  a\n\n  b\n\n- c\n` | `ul(li("a")) para("b") ul(li("c"))` — **two lists, paragraph outside both** |
| `li_ws_nested_ul` | `-  a\n  - b\n` | `ul(li("a") li("b"))` — **nesting lost** |
| `li_ws_pre` | `-  a\n\n  ```\n  x\n  ```\n\n- c\n` | `ul(li("a")) code("x\n") ul(li("c"))` — **code block ejected** |
| `ol_ws_two_p` | `1.  a\n\n   b\n\n2. c\n` | `ol[1](li("a")) para("b") ol[2](li("c"))` — split; numbering itself survives |
| `li_newline_pretty` | `-  a\n\n  b\n\n- c\n` | same as `li_ws_two_p` — **this is the realistic input** |
| `li_ws_quote` | `- >  q\n- c\n` | `ul(li(quote(para("q"))) li("c"))` — structure **survives**; only the stray space is wrong |

### Slice 2 — leading whitespace in a heading

| Cell | Output today | Parses as |
|---|---|---|
| `h1_ws` | `#  Quarterly Report\n` | `h1("Quarterly Report")` — renders correctly; cosmetic only |

### Slice 4 — empty nested lists

| Cell | Output today | Parses as |
|---|---|---|
| `empty_li_d1` | `-\n` | `ul(li())` — correct |
| `empty_li_d2` | `- -\n` | `ul(li(ul(li())))` — correct |
| `empty_li_d3` | `- - -\n` | **`hr`** — content type changed |
| `empty_li_d5` | `- - - - -\n` | **`hr`** |

### Controls — these must not move

| Cell | Output today | Parses as |
|---|---|---|
| `li_nows_two_p` | `- a\n\n  b\n\n- c\n` | `ul(li(para("a") para("b")) li(para("c")))` — correct |
| `li_nows_nested` | `- a\n  - b\n` | `ul(li("a" ul(li("b"))))` — correct |
| `tight_control` | `- a\n- b\n` | `ul(li("a") li("b"))` — correct |
| `li_ws_trailing` | `- a\n- b\n` | correct — **trailing whitespace is already stripped** |
| `h2_ws_trailing` | `## T\n` | correct — same |

## 3. What the re-derivation narrowed

Three things that are not in the consumer pass report, and each one makes the work smaller:

1. **The fix is one-sided.** Trailing whitespace is already stripped, in both list items and headings. Only
   leading whitespace is at fault. Do not touch the trailing path.
2. **`li_ws_quote` is cosmetic, not structural.** A blockquote inside a whitespace-led item still parses
   correctly. It carries the same stray space and should move with the rest, but if it resists, it is not a
   validity defect and can be reported rather than forced.
3. **Ordered-list numbering survives the split** — the second list comes back with `start=2`. So slice 1 is
   purely a structural fix; there is no renumbering bug hiding underneath it.

## 4. Design — from RFC 036 §3

**Slices 1 and 2.** Strip leading whitespace from a block's own text before emitting its marker or prefix.
The content column then matches the sink's continuation prefix by construction, which is what RFC 035 §3
already assumed. **Whitespace inside the text is untouched, and whitespace anywhere inside `<pre>` is
untouched** — that is the rule `024c` was written to protect, and breaking it is how this release's worst
near-miss happened.

**Slice 4.** An item with empty content emits its marker and nothing else. A run of such markers must not
produce a line that parses as a thematic break. A single empty item already emits `-` safely, so the
narrowest sufficient change is at the point where nested empty markers are concatenated onto one line —
find out *why* they land on one line before choosing how to stop it, and say what you found.

## 5. Acceptance criteria

1. Every row in §2's slice 1, 2 and 4 tables produces the structure the source describes, **verified by
   parsing the output**, not by string comparison. For `li_ws_nested_ul` and `li_ws_pre` a string check
   that only inspects the marker will pass while the defect remains.
2. The five **control** rows in §2 are **byte-identical to `2.3.0`**. This is the narrow form: it names five
   shapes, it does not protect the corpus wholesale. (A blanket byte-identity criterion of mine locked in a
   defect at `024c` — that is why this one is enumerated.)
3. Harness cells for every row in §2, in all five modes, in `tests/output_validity/block_in_container.rs`
   for the list cells and `well_formedness.rs` for the heading and empty-list cells — or say why a different
   file is the right home.
4. Cells assert **trees written from the HTML's meaning**, in the style already used in
   `block_in_container.rs` — `=> tree(r#"ul(li(para("a"), para("b")))"#)`. No `known_defect` markers should
   remain for these shapes when the slice lands.
5. `cargo test` passes with **more than 405** tests and no ignored ones.
6. Report the test count and the diff of what moved, as usual.

## 6. Things to watch

- **Do not reach for slice 3.** §0.
- **`<pre>` whitespace is sacred.** If your strip runs anywhere near a code path that touches `<pre>` or
  `<code>` content, stop and re-read RFC 024c. The `024c` defect wrote a fence after four spaces, which
  turned it into an indented code block that swallowed the rest of the document.
- **The sink is not the bug.** RFC 035's continuation-prefix logic is correct and should not need changing.
  The marker writer is what violates it. If you find yourself changing the sink, explain why before you do.
- **Check the nesting case by parsing.** `li_ws_nested_ul` produces output that *looks* right in a diff and
  is structurally wrong. `.git-exclude/tmp/mdprobe/target/release/mdprobe` reads Markdown on stdin and
  prints the tree; use it while developing, not only at the end.

## 7. Report back

A review-request package at `.git-exclude/review-request/036-whitespace-at-block-boundaries/README.md`,
with: what moved in `src/`, the before/after parse of every row in §2, the control rows shown byte-identical,
the test count, and anything you found that this handoff got wrong. **Say so plainly if a row resisted** —
`li_ws_quote` is the likeliest, and reporting it beats forcing it.
