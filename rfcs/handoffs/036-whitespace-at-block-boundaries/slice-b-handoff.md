# Developer Handoff — RFC 036 slice `036b` · Three follow-ups from the slice 1/2/4 review

**Governing RFC.** [RFC 036](../../done/036-whitespace-at-block-boundaries.md)
**Follows.** slices 1/2/4, approved 2026-09-22 — `.git-exclude/reviewed/036-whitespace-at-block-boundaries/README.md`
**Read first.** `addendum-2026-09-22.md` in this folder (baseline is **407**, not 405; §6's sink caution is narrower than it reads)
**Milestone.** M4 · `2.4.0`
**Priority.** P2 — nothing here changes a parse except item 3
**Prepared.** 2026-09-22
**Baseline.** the slice 1/2/4 commit, **424 passing tests**

---

## 0. Scope

Three items. Slice 3 (`<div>`) is **still blocked** on RFC 036 §6 and is still not yours to pick.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Move the leading-strip clear to the sink's write choke point

You asked whether `consumed_leading()` covers every content producer that bypasses `process_text`. It does
not. Three leaks, all reproduced against your tree and a clean `bec40bf`:

| Input | `bec40bf` | your tree | Leak |
|---|---|---|---|
| `<ul><li><br> x</li></ul>` | `-   \n   x` | `-   \n  x` | `hard_break()` |
| `<ul><li><b></b> x</li></ul>` | `- **** x` | `- ****x` | empty emphasis — `emphasis_open`/`close` write `****`, no text reaches `process_text` |
| `<ul><li> <pre><code>x</code></pre> t</li></ul>` | ``…```\n\n   t`` | ``…```\n\n  t`` | the fence path |

**None changes a parse** — I checked all three; the `<br>` and `<pre>` cases render identically and the
emphasis case is junk either way. **But the first two have no leading whitespace in the source at all**, so
they are outside RFC 036's scope and my acceptance criteria could not have caught them. That is a fault in
my criteria as much as in the patch.

**The fix is structural, not another call site.** `pending_leading_strip` is currently cleared by whoever
remembers; every future non-text writer inherits an invisible obligation. There is already one choke point —
**the sink is the only writer** (RFC 024/028). Clear the flag there, on the first visible byte, and
`consumed_leading()` and both of its call sites delete themselves.

This is a sanctioned sink change; see the addendum §3.

**Criteria.** The three rows above byte-identical to `bec40bf`. `consumed_leading()` gone. Every slice 1/2
cell still passing, unchanged. Cells for all three shapes. If the choke point turns out to be the wrong
place, say why and propose the alternative before building it.

## 2. An empty innermost item with a non-empty sibling must stay one list

Slice 4's bullet swap ends the list at the bullet change:

```
<ul><li><ul><li><ul><li></li><li>x</li></ul></li></ul></li></ul>
  bec40bf  : "- - - \n    - x"  ->  hr  code("- x\n")    content destroyed
  your fix : "- - * \n    - x"  ->  ul(li(ul(li( ul(li()) ul(li("x")) ))))   two lists
```

**Your fix is a large improvement** — the content survives instead of being swallowed into an indented code
block — and this shape was not in the handoff, so it is not a missed requirement. But "same list" is the
right answer, and a mechanism that ends a list to avoid a thematic break has a sharp edge worth revisiting.

Either make the sibling match, or find a way to break the dash run that does not change the bullet
character — breaking the run's *homogeneity* was your insight and it is a good one; the bullet is only one
of the things on that line that could carry it. **Say which you chose and why.**

**Criteria.** The shape above → `ul(li(ul(li(ul(li(), li("x"))))))`. `empty_li_d1`–`d5` still correct. The
all-empty siblings case (`- - * \n    *`, currently correct) must not regress. The trailing space in
`- - * ` is pre-existing at `bec40bf` — out of scope, do not chase it unless your fix removes it for free.

## 3. `<li><hr>` destroys the item and splits the list

Your finding 4, which you correctly left alone as out of scope. It is worse than you described and it
belongs to this family — the same marker-plus-dashes collision as slice 4:

```
<ul><li> <hr> text</li></ul>      ->  "- ---\n\n   text"  ->  hr  para("text")
<ul><li><hr></li><li>b</li></ul>  ->  "- ---\n- b"        ->  hr  ul(li("b"))
```

**The second has no leading whitespace and loses the entire first item.** The ordered variant is correct, as
you said. Pre-existing at `bec40bf`; at `2.2.3` it was broken differently (the `<hr>` escaped the item — the
A-06 class RFC 035 fixed), so there is no clean earlier behaviour to restore. Decide what the item *should*
emit and argue for it.

**Criteria.** Both shapes above keep the item and the list. The ordered variants stay byte-identical. Cells
for both, plus the no-leading-whitespace form.

## 4. Report back

`.git-exclude/review-request/036b-sink-choke-point-and-marker-collisions/README.md`, same shape as last
time: what moved, before/after parses, controls shown byte-identical, test count against the **424**
baseline, and anything here I got wrong. The last package corrected two of my numbers and one of my
predictions; that is the standard.
