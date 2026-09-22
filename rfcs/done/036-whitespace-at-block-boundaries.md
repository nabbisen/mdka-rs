# RFC 036 — Whitespace and separators at block boundaries

**Status.** Implemented (2.4.0) — closed 2026-09-22 at `559e8ff`
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0` (owner, 2026-09-22)
**Sequencing.** First slice of M4. It completes RFC 035 and fixes a documented-behaviour contradiction; RFC 008's tables sit inside list items often enough that this should land first.
**Source.** `2.3.0` consumer pass, findings §5.1, §5.2, §5.5, §5.6 — `.git-exclude/reviewed/2.3.0-consumer-pass/README.md`.
**Touches.** `src/renderer.rs`, `src/renderer/sink.rs`, `src/traversal.rs`, `docs/src/api/elements.md`, `tests/output_validity/`.

---

## 0. Closed, 2026-09-22

All four defects fixed across four slices, every one verified by parsing the output, not comparing strings:

| Slice | Commit | Covers |
|---|---|---|
| 1 / 2 / 4 | `c6b2ea7` | leading whitespace in `<li>` and in a heading; empty nested markers vs. a thematic break |
| `036b` | `4ee6aaa` | the leading-strip clear moved to the sink's write choke point; sibling bullet propagation; the `<li><hr>` collision |
| `036d` (slice 3) | `559e8ff` | unwrapped wrappers keep their block separation, and the option is documented as inert — owner's **option C** |

Final state at `559e8ff`: **443 Rust, 39 Node, 80 Python tests passing**, all seven CI workflows green, and
**no `known_defect` marker on any cell** for these shapes.

Three defects found *during* the work and fixed with it, none of them in this RFC's original list: a
leading-space leak through `<br>`, empty emphasis and the fence path; welding inside `<pre>`; and RFC 028's
pre-pass disagreeing with the traversal once a `<div>` became a block again.

**Not closed by this RFC:** the setext collision found in `036b`'s review — promoted to its own proposal,
**RFC 038**, because it is not one of the four defects scoped here.

## 1. Summary

Four defects, all at the seam where a block's own text begins or ends. None is a `2.3.0` regression — every
one reproduces identically in `2.2.3`, and §5.1 is *less* damaging in `2.3.0` than it was before. They are
here because the `2.3.0` consumer pass found them on real pages and because RFC 035 got close enough to
§5.1 that finishing it is cheap.

| # | HTML | Output at `bec40bf` | Reader gets |
|---|---|---|---|
| §5.1 | `<ul><li> <p>a</p><p>b</p></li><li>c</li></ul>` | `-  a\n\n  b\n\n- c\n` | **two lists**, paragraph `b` outside both |
| §5.1 | `<ul><li> a<ul><li>b</li></ul></li></ul>` | `-  a\n  - b\n` | **nesting lost** — a flat two-item list |
| §5.1 | `<ul><li> <p>a</p><pre><code>x</code></pre></li>…` | — | **the code block is ejected from the item** |
| §5.2 | `<div>First.</div><div>Second.</div>` (Minimal/Semantic) | `First.Second.` | words welded together |
| §5.5 | `<ul><li><ul><li><ul><li></li></ul></li></ul></li></ul>` | `- - -\n` | a **thematic break** |
| §5.6 | `<h1>\nQuarterly Report</h1>` | `#  Quarterly Report\n` | cosmetic double space |

## 2. Why

### 2.1 §5.1 and §5.6 — leading whitespace is not stripped

RFC 035 set the rule and the sink obeys it: an item's continuation prefix is *"spaces equal to that item's
content column — marker width plus one space: `- ` → 2, `1. ` → 3"*. The marker writer does not obey it.
It emits `- ` followed by the item's text **including its leading whitespace**, so the real content column
becomes 3 while the sink keeps prefixing 2. CommonMark then ends the item at the first continuation line.

The same unstripped whitespace produces `#  Heading`, which is harmless but is the same bug.

**This contradicts the project's own written invariant twice** — `docs/src/api/elements.md:17`
(*"Everything inside it stays in the item"*) and `docs/src/design/architecture.md:76`.

**The trigger is pretty-printed HTML.** `<li>\n  <p>…</p>\n</li>` reproduces it, which is what editors,
CMSs and formatters emit. Counted on real pages: MDN 109 occurrences, docs.python.org 14.

### 2.2 §5.2 — an unwrapped wrapper stops separating

`docs/src/api/elements.md:19` lists `<div>`, `<article>`, `<section>`, `<main>` as **"Block separator — Act
as paragraph breaks; unwrapped (tag removed, children kept) when `unwrap_unknown_wrappers` is on — Minimal
and Semantic by default."** In exactly the two modes where they are unwrapped they stop acting as paragraph
breaks. Unwrapping currently removes the tag *and* its separation; it should remove only the tag.

A `<p>` inside the `<div>` still separates, which is why this was never noticed: hand-written fixtures put
`<p>` inside. Div-per-line markup without `<p>` is ordinary CMS and SPA output, and Minimal is the mode the
documentation recommends for **"LLM input, text extraction"** — the use that suffers most from welded words.

### 2.3 §5.5 — degenerate nesting collides with `---`

Three empty nested items emit `- - -`, which is a thematic break. The content type changes. Low frequency,
but it is a validity defect and it falls out of the same marker-emission code.

## 3. Design

1. **Strip leading and trailing whitespace from a block's own text before emitting its marker or prefix.**
   Applies to list items and headings alike. The content column then matches the sink's prefix by
   construction, which is what RFC 035 assumed. Whitespace inside the text, and whitespace anywhere inside
   `<pre>`, is untouched.
2. **An unwrapped wrapper keeps its block separation.** Unwrapping removes the element, not the paragraph
   break it stood for. Adjacent unwrapped wrappers separate like adjacent `<p>`.
3. **An item whose content is empty emits its marker and nothing else**, and a list of such items must not
   produce a line that parses as a thematic break. Simplest sufficient rule: never emit a bare marker run
   that matches a thematic break; a single empty item already emits `-` safely.

## 4. Acceptance criteria

1. Each of the six rows in §1 produces the structure the source describes, verified by **parsing the
   output**, not by string comparison — the nesting case and the code-block case are invisible to a string
   check that only looks at the marker.
2. The same six shapes with the whitespace removed produce **byte-identical output to `2.3.0`**. This is a
   no-change-where-it-was-right criterion, and it is the narrow form: it names the six shapes, it does not
   protect the corpus wholesale. (Blanket byte-identity locked in a defect at `024c`.)
3. `docs/src/api/elements.md:19` and the behaviour agree — whichever way the owner rules in §6, one of the
   two changes.
4. Harness cells added for every shape in §1, in all five modes.

### 4.1 Re-derivation, 2026-09-22, release build of `bec40bf`

Two shapes in §1 were checked more closely than the consumer pass reported, and both narrow the work:

- **Only *leading* whitespace is at fault.** `<li>a </li>` already emits `- a` — trailing whitespace is
  stripped. So is trailing whitespace in a heading (`<h2>T </h2>` → `## T`). The fix is one-sided.
- **`<blockquote>` inside a whitespace-led item survives structurally** — `<ul><li> <blockquote>q…` gives
  `- >  q`, which parses correctly as a quote inside the item. It carries the same stray space and should
  move with the rest, but it is cosmetic, not a validity defect.
- **An ordered list keeps its numbering across the split** — `ol_ws_two_p` yields a second list with
  `start=2`, so the damage is structural only, not a renumbering bug on top.

## 5. Harness gap this closes — and one it does not

**§5.1 was catchable and was missed for want of an input shape.** It is a structure-preservation violation;
the property exists, no cell feeds leading whitespace into `<li>`. Adding cells closes it.

**§5.2 was not catchable by the harness as designed.** The agreement proof at
`tests/output_validity/proofs.rs:430` probes `a<div>b</div>` across all five modes and asserts the harness's
block model agrees with what mdka renders. Under Minimal, mdka renders `ab` — one block — so the proof
concludes `div` is not a block in that mode and **passes**. The harness ratified the defect.

That is correct behaviour for an intent-free property, and it is the design's boundary:

> **Intent-free properties cannot catch a violation of documented intent.**

Where the documentation makes a promise, the promise needs an assertion that states it, kept separate from
the validity properties so the two cannot quietly merge. This RFC adds that category with `<div>` as its
first member; populating it from the rest of `api/elements.md` is its own slice, not this one.

## 6. Resolved by the owner, 2026-09-22 — **option C**

> **Decision: option C.** Unwrapping keeps the block separation, **and** `unwrap_unknown_wrappers` is marked
> *no effect today*, extending `api/modes.md`'s existing alias warning from three modes to four. Slice 3 is
> unblocked; handoff at `rfcs/handoffs/036-whitespace-at-block-boundaries/slice-d-handoff.md` (named `036d`; `036c` was already assigned to the setext-collision slice).
>
> **Neither A nor B as originally framed.** Re-examination before the decision found that
> `unwrap_unknown_wrappers`'s **only** observable effect is deleting the paragraph separator — twelve probe
> shapes, four differ between Balanced and Semantic, every difference just the lost break. So option A alone
> would have made Semantic byte-identical to Balanced and turned the option into a sixth do-nothing field,
> silently, while the documentation still described both as meaningful. Option C fixes the output **and**
> says so. Full reasoning: `.git-exclude/review-request/m4-open-decisions-v2/README.md` §2.
>
> **Not a deprecation.** `unwrap_unknown_wrappers` is inert *today* because `<div>` has no Markdown form, not
> because it is unimplementable — unlike the five attribute fields, a future mode that preserves raw HTML
> would revive it at once. `api/modes.md` already draws exactly this line (*"a statement about today's
> behaviour, not a deprecation"*). **Do not add `#[deprecated]` and do not emit a warning for it.**

**The two directions as originally framed, kept for the record:**

- **Make the behaviour match the documentation** — unwrapped wrappers separate. Minimal and Semantic output
  changes for div-per-line input: words stop being welded. This is what I recommend; it is what the
  documentation has promised since it was written, and the failure mode it fixes is silent.
- **Make the documentation match the behaviour** — say that unwrapping in Minimal and Semantic removes the
  separation too. Cheaper, changes no output, and leaves `Minimal` weld-prone in the mode we recommend for
  LLM input.

## 7. Not in scope

The emphasis defects (§5.3, §5.4) — RFC 037. The parser's O(depth²) cost (§6.1) — documentation, and not
ours to fix. The CLI `--preserve-ids` surface (§6.4) and the documentation nits (§6.5) — their own slices.
