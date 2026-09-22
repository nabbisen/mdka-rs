# Developer Handoff — RFC 038 · Marker lines colliding with other CommonMark constructs

**Governing RFC.** [RFC 038](../../done/038-marker-line-collisions.md) — §2.5 the family, §2.6 the remedy, §3.1 the open question
**Milestone.** M4 · `2.4.0`
**Priority.** P1
**Prepared.** 2026-09-22
**Baseline.** `559e8ff` — **443 Rust, 39 Node, 80 Python** tests passing, seven CI workflows green
**Supersedes.** the `036c` scope in `rfcs/handoffs/036-whitespace-at-block-boundaries/addendum-b-2026-09-22.md`. That name is retired; nothing was built under it. RFC 036 is closed.

---

## 0. What changed between the addendum and this handoff

When I scoped this as `036c` I described **two** shapes and asked you to decide the remedy. I then re-derived
it at `559e8ff` before writing this, and both halves of that are now out of date:

- **The family is ten shapes, not two**, with **two distinct failure modes**.
- **The remedy is settled** — a blank line — and the obvious alternative is ruled out by measurement.

So this handoff asks you for something narrower and harder than the addendum did: not *what* to emit, but
**where the rule belongs** (§3).

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. The family, re-derived at `559e8ff`

Twelve shapes probed, **ten broken**. Full table in RFC 038 §2.5. The two failure modes:

**Setext** — an unordered empty child underlines the parent's text:
```
<ul><li>y<ul><li></li></ul></li></ul>   ->  "- y\n  -"   ->  ul(li(h2("y")))
```

**Lazy continuation** — an ordered empty child is absorbed as text, and takes its siblings with it:
```
<ul><li>y<ol><li></li><li>z</li></ol></li></ul>
   ->  "- y\n  1. \n  2. z"  ->  ul(li("y" SB "1." SB "2. z"))
```

The second destroys more: the whole nested list, including the non-empty `z`, stops being a list.

Depth **two** is enough. A list item with an empty nested item is ordinary CMS and editor output. All ten are
**pre-existing** — identical in `2.2.3` — so none is a regression.

**Two shapes are already correct**, and the first is the clue: a blockquote parent emits `> y\n>\n> -` — a
blank line — and parses correctly. A nested list whose first item has content is also fine.

## 2. The remedy — measured, not proposed

**A blank line before the nested list fixes all ten**, both failure modes, including the three-level shape
`036b`'s finding 2 deferred here:

```
- y\n\n  -               ->  ul(li(para("y"), ul(li())))
- y\n\n  1.\n  2. z      ->  ul(li(para("y"), ol(li(), li("z"))))
- x\n  - y\n\n    -      ->  ul(li("x", ul(li(para("y"), ul(li())))))
```

**The bullet swap is ruled out — do not reach for it.** `- y\n  *` gives `ul(li("y" SB "*"))`: the `*`
becomes literal text by lazy continuation. It never fixed the unordered case, and it cannot touch an ordered
child at all, since `1.` is not a setext underline. `c6b2ea7`'s accidental swap turned one wrong tree into a
different wrong tree — which is also why `036b`'s finding 2 could not be pinned with a plain cell.

## 3. The one thing you decide: looseness, or disambiguation?

The blank line makes the outer list **loose** — `"y"` becomes `para("y")`. That **contradicts RFC 035 §3.1**
as written: *"A list is loose if and only if some item contains two or more blocks … nested lists are not
counted."* This item has one block and is tight by that rule.

| Option | For | Against |
|---|---|---|
| **Amend RFC 035 §3.1** to count an empty-leading nested list | One rule, no second concept | Edits a settled rule that `api/elements.md` documents, and claims the *source* is loose when it is not |
| **Keep §3.1; add a separate disambiguation step** ⭐ | Looseness describes what the **source** means; this is about what the **output** can be misread as | Two concepts to hold |

**My recommendation is the second**, but it is yours to argue — you will be living in that code. **Say which
you chose and why** in the review package. The emitted bytes are identical either way; what differs is where
the rule lives and what the documentation says.

**Either way `api/elements.md`'s loose/tight section needs a sentence**, because a reader can currently
derive the wrong output from the documented rule. **No correct output changes under either choice** — only
the ten broken shapes move.

## 4. Acceptance criteria

1. All ten shapes in RFC 038 §2.5 parse to the structure the source describes — **verified by parsing**. The
   lazy-continuation ones are invisible to a string check that only inspects the marker.
2. The two currently-correct shapes — blockquote parent, and a nested list whose first item has content —
   stay **byte-identical** to `559e8ff`.
3. RFC 036's fixes untouched, byte-identical to `559e8ff`: `empty_li_d1`–`d5`, the empty-item-with-non-empty-
   sibling shape, both `<li><hr>` shapes, and every ordered variant.
4. `036b`'s finding 2 shape (`<ul><li>x<ul><li>y<ul><li></li></ul></li></ul></li></ul>`) now produces the
   correct tree. **It should need no `known_defect` marker** — §2 shows the remedy reaches it. If you find it
   does not, say so; that is a real finding, not a failure.
5. Cells for all ten shapes plus the two controls, in all five modes.
6. `api/elements.md`'s loose/tight section updated per your §3 choice.
7. **All three suites** — `cargo test`, `node test.js`, `pytest` — run and reported separately before commit.
   This slice changes list output, which every binding re-exposes; `cargo test` alone would not have caught
   the two binding tests that nearly turned CI red in `036d`.
8. `fmt` and `clippy` clean; report counts against the 443/39/80 baseline.

## 5. Things to watch

- **Do not widen the trigger.** Only a nested list whose **first** item is empty is at risk. A nested list
  starting with content is correct today and must stay byte-identical.
- **The `036b` and `036d` fixes are load-bearing.** If your change makes either look redundant, check twice
  before removing anything — they cover different members of this family.
- **Check `<pre>`.** Every list change this milestone has had to prove it does not write into a code fence;
  `036d` hit it, and `024c` shipped it once.
- **Parse, don't diff.** The lazy-continuation failures produce output that looks entirely reasonable in a
  diff and is structurally wrong.

## 6. Report back

`.git-exclude/review-request/038-marker-line-collisions/README.md`: what moved, before/after parses for all
twelve shapes, the criterion-2 and -3 byte-identical sets, your §3 decision and its argument, the three test
counts, and anything here I got wrong. Every package this milestone has corrected something of mine; that is
the standard, not the exception.
