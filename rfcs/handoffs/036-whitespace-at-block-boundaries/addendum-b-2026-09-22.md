# Addendum B — RFC 036 slice `036b`, 2026-09-22

**Amends.** `slice-b-handoff.md` (frozen; this addendum is the only correct channel)
**Issued after.** the `036b` review, `.git-exclude/reviewed/036b-sink-choke-point-and-marker-collisions/README.md`

---

## 1. Required before committing `036b`: §3 emits `---`, not `___`

`slice-b-handoff.md` §3 said *"decide what the item should emit and argue for it."* You did, and the
argument for `___` (never a valid bullet, so it can never collide) is correct. I am overriding it on
consistency grounds, not because it is wrong.

**The problem.** The same element now emits two different characters depending on its position:

```
<ul><li><p>p</p><hr></li></ul>  ->  "- p\n\n  ---"   not first -> the documented ---
<ul><li><hr></li></ul>          ->  "- ___"          first     -> ___
```

And `docs/src/api/elements.md:18` states `` | `<hr>` | `---` | ``. Shipping `___` would create a
documented-intent violation inside the slice that closes one, and then require a documentation exception to
cover it.

**The remedy the renderer already uses.** Put the break on a continuation line, exactly as it does when the
`<hr>` is not first. Verified for all three shapes:

```
"-\n  ---"            -> ul(li(hr))
"-\n  ---\n- b"       -> ul(li(hr), li("b"))
"-\n  ---\n\n  text"  -> ul(li(hr, para("text")))
```

RFC 035 already put `<p>`, `<pre>` and `<blockquote>` inside an item on continuation lines. An `<hr>`
sharing the marker's line is the anomaly, and it is the anomaly that caused the collision. This removes a
special case instead of adding one.

**Criteria.** All three shapes above. `<hr>` not-first byte-identical. Ordered variants byte-identical. Top
level, blockquote and `hr_in_blockquote` byte-identical. **No `_` in any output.**

## 2. Also fold into the `036b` commit: two `id_anchor` cells

You offered these and you were right that the gap exists. Your exclusion is correct — I probed eight
combinations of `preserve_ids` with leading whitespace, including a newline-led heading, an `id` on a list
item, and an anchor followed by `<b>` or `<img>`; all correct. Pin it: one heading cell and one list-item
cell, each with an `id` **and** leading whitespace.

## 3. Your three questions

**Q1 — no standalone cell for the over-triggering dash run.** A tree cell cannot express it: the correct
tree is `ul(li("x", ul(li("y", ul(li())))))` and neither the old nor the new output produces it, so the cell
fails either way. It needs `known_defect`, and it belongs with `036c` below. The mechanism fix stands on its
own.

**Q2 — the setext collision joins the family; see `036c`.**

**Q3 — yes; see §2 above.**

## 4. `036c` — the setext collision

**It is worse than you judged, so do not carry it as backlog.**

```
<ul><li>y<ul><li></li></ul></li></ul>  ->  "- y\n  -"  ->  ul(li(h2("y")))
```

Depth **two**, one empty sub-item, and the parent's text becomes an H2 heading while the sub-list vanishes.
A list item with an empty nested item is ordinary CMS output, not a degenerate shape.

Your classification as pre-existing is right — I confirmed it is byte-and-parse identical in `2.2.3`.

One thing worth knowing before you start: **your finding-2 fix restored this collision** in the three-level
shape, because `c6b2ea7`'s over-triggering `*` happened to break the setext underline by accident. That is
not an argument against your fix — an accidental no-op is not a defence — but the family's severity was
partly masked by a bug, which is why it looked smaller than it is.

**Scope.** Both shapes (depth two, and the three-level one from your finding 2), the sibling variants, and a
`known_defect`-marked cell for finding 2's exact shape if the fix does not reach it. Ordered variants stay
byte-identical.

**Raise it as `.git-exclude/review-request/036c-setext-collision/README.md`**, after `036b` lands.
