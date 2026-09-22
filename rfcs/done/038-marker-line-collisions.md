# RFC 038 — Marker lines colliding with other CommonMark constructs

**Status.** Implemented (2.4.0) — closed 2026-09-22 at `190e266`
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0` (owner, 2026-09-22)
**Source.** Found during RFC 036 slice `036b`'s review; **promoted out of RFC 036**, whose four defects are closed at `559e8ff`. It was scoped as a slice (`036c`) in `rfcs/handoffs/036-whitespace-at-block-boundaries/addendum-b-2026-09-22.md`; it is not one of RFC 036's four defects, so it gets its own number rather than hanging off a closed RFC. **Nothing has been implemented under the `036c` name.**
**Touches.** `src/renderer/sink.rs`, `tests/output_validity/`.

---

## 0. Closed, 2026-09-22 at `190e266`

All ten shapes fixed by **one mechanism**, in one commit: a pre-pass hint in `structure_hints` recording
whether a nested list's own first item is empty, checked only on nested entry, forcing a blank line before
that list. Seven CI workflows green; **455 Rust, 39 Node, 80 Python** tests; battery re-diff 0 of 7,070.

**The answer to §3 was decided by the implementer and is better than the recommendation it followed:**
CommonMark's tight/loose determination runs on the **output bytes**, not on mdka's internal flag, so
looseness and output-safety cannot be merged even in principle — one describes the source, the other is
decided by a reparser that never sees mdka's state. RFC 035 §3.1 is unchanged; `api/elements.md` carries the
exception.

**Known, accepted consequence.** The blank line makes the *enclosing* list loose, so every sibling in it is
reparsed with its text in a paragraph — including siblings that were entirely correct. Spec-mandated, and no
previously-correct document is affected, because the trigger only exists in documents that were already
broken (verified: 20 protected shapes × 5 modes unchanged, and 0 of 7,070 battery outputs moved).

**Coverage note for whoever plans the `025b` corpus slice.** Zero battery diffs also means **no fixture in
fifteen battery directories contains this family's trigger**. The twelve cells added here are the only guard
on ten content-destroying shapes, and the family was found by hand-probing during a review, not by any gate.
Degenerate-but-legal list nesting is a blind spot, not merely uncovered.

## 1. Summary

A list item's marker line can satisfy the grammar of a **different** CommonMark construct, and that other
construct wins. RFC 036 fixed two members of this family — three or more bullets on one line reading as a
thematic break, and `-` plus `---` reading as one. **A third is open, and it is the most reachable of the
three.**

```
<ul><li>y<ul><li></li></ul></li></ul>   ->  "- y\n  -"  ->  ul(li(h2("y")))
```

**Depth two. One empty sub-item. The parent's text becomes an H2 heading and the sub-list disappears.**
`  -` under `- y` is a setext heading underline, not a nested empty item.

The three-level form behaves the same way:

```
<ul><li>x<ul><li>y<ul><li></li></ul></li></ul></li></ul>   ->  ul(li("x", ul(li(h2("y")))))
```

## 2. Why this is not a degenerate shape

A list item containing an empty nested item is ordinary CMS and editor output — an outline with a blank
child, a template with an unfilled sub-bullet. It needs no unusual nesting: **depth two is enough**, and the
damage is not cosmetic. The text is relabelled as a heading and a list level vanishes.

**Pre-existing, not a regression.** Byte-and-parse identical in `2.2.3`, verified.

**Its severity was masked.** At `c6b2ea7` an over-triggering bullet swap happened to break the setext
underline by accident; `036b` fixed that over-trigger correctly and, in doing so, restored this collision in
the three-level shape. An accidental no-op is not a defence, but it is why the family looked smaller than it
is.

## 2.5 Re-derived at `559e8ff`, 2026-09-22 — the family is **ten shapes, not two**

Probed systematically after acceptance. Of twelve shapes, **ten are broken**:

| Parent | Empty child | Output | Parses as |
|---|---|---|---|
| `ul` text | `ul` | `- y\n  -` | `ul(li(h2("y")))` — setext |
| `ul` text | `ul` + sibling | `- y\n  - \n  - z` | `ul(li(h2("y"), ul(li("z"))))` |
| `ul` text | **`ol`** | `- y\n  1.` | `ul(li("y" SB "1."))` — **the child becomes literal text** |
| `ul` text | **`ol`** + sibling | `- y\n  1. \n  2. z` | `ul(li("y" SB "1." SB "2. z"))` — **the whole nested list, including `z`, becomes text** |
| `ol` text | `ul` | `1. y\n   -` | `ol(li(h2("y")))` — setext |
| `ol` text | `ol` | `1. y\n   1.` | `ol(li("y" SB "1."))` |
| …and the `+ sibling` and three-level variants | | | |

**Two distinct failure modes, not one.** An unordered empty child is read as a **setext underline**; an
ordered one is absorbed as **lazy continuation text** — and that second kind destroys more, taking the
non-empty siblings with it.

**Two shapes are correct, and one of them shows the way.** A blockquote parent emits a blank line
(`> y\n>\n> -`) and parses correctly; a nested list whose first item has content is fine.

## 2.6 The remedy, measured

**A blank line before the nested list fixes all ten**, both failure modes, including the three-level shape
`036b`'s finding 2 deferred here:

```
- y\n\n  -              -> ul(li(para("y"), ul(li())))
- y\n\n  1.\n  2. z     -> ul(li(para("y"), ol(li(), li("z"))))
- x\n  - y\n\n    -     -> ul(li("x", ul(li(para("y"), ul(li())))))
```

**The bullet swap does not work and is ruled out.** `- y\n  *` gives `ul(li("y" SB "*"))` — the `*` becomes
literal text by lazy continuation. It never fixed the unordered case either; `c6b2ea7`'s accidental swap
turned one wrong tree into a different wrong tree. And it cannot apply to an ordered child at all, since
`1.` is not a setext underline.

**So the answer to §3's question is yes — one remedy covers the family.**

## 3. Design

**The remedy is settled by §2.6: a blank line.** What is *not* settled, and is the real work, is where that
rule lives — see §3.1.

What the answer must respect:

1. **An empty nested item is a list item.** Whatever is emitted, it must parse back as one.
2. **The parent's text stays text.** No heading appears where the source had none.
3. **`---` stays `---`.** RFC 036 settled that the documented thematic break character is not traded away to
   dodge a collision; the same reasoning applies to any new character.
4. Prefer a remedy that removes the collision's *cause* over one that dodges its *symptom* — the
   continuation-line fix for `<li><hr>` is the model, because it made the anomalous case behave like the
   ordinary one.

### 3.1 The open question: looseness, or disambiguation?

The blank line makes the outer list **loose** — `"y"` becomes `para("y")`. That is a real tree change, and
it **contradicts RFC 035 §3.1 as written**: *"A list is loose if and only if some item contains two or more
blocks … nested lists are not counted."* By that rule this item has one block and is tight.

Two ways to express it, same emitted bytes, different rule and different documentation:

- **Amend RFC 035 §3.1's looseness definition** to count an empty-leading nested list. Honest if you believe
  looseness is the right concept here; it edits a settled rule, and `api/elements.md` documents it.
- **Keep §3.1 and add a separate disambiguation step** ⭐ — the blank line is emitted for output safety, not
  because the source is semantically loose. *Recommended:* looseness describes what the **source** means;
  this is about what the **output** can be misread as, and conflating the two makes the next collision
  harder to reason about.

Either way `api/elements.md`'s loose/tight section needs a sentence, because the reader can otherwise derive
the wrong output from the documented rule. **No correct output changes under either choice** — only the ten
broken shapes move — so this is a naming and documentation decision, not a behavioural one.

## 4. Acceptance criteria

1. Both shapes in §1, and their sibling variants, parse to the structure the source describes — verified by
   **parsing**, never by string comparison.
2. `036b`'s and `036d`'s fixes are untouched: `empty_li_d1`–`d5`, the empty-item-with-non-empty-sibling
   shape, both `<li><hr>` shapes, and the ordered variants all stay byte-identical to `559e8ff`.
3. A `known_defect`-marked cell for the over-triggering-dash-run shape from `036b`'s finding 2, if the fix
   does not reach it — `036b`'s review deferred it here deliberately, because the correct tree is produced by
   neither the old nor the new output, so a plain cell fails either way.
4. Cells for every shape, all five modes.
5. Ordered-list variants stay byte-identical.

## 5. Not in scope

The `<hr>`-after-content and thematic-break members — fixed in RFC 036. The emphasis defects — RFC 037.
