# Developer Handoff — RFC 036 slice 3 (`036d`) · Unwrapped wrappers keep their separation

**Governing RFC.** [RFC 036](../../done/036-whitespace-at-block-boundaries.md) §5.2, §6 — **owner decision: option C, 2026-09-22**
**Read first.** RFC 036 §6 (the decision and why it is neither A nor B), then `.git-exclude/review-request/m4-open-decisions-v2/README.md` §2
**Milestone.** M4 · `2.4.0`
**Priority.** P1 — this is the last slice of RFC 036
**Prepared.** 2026-09-22
**Sequencing.** **After `036b` lands** (it touches the same renderer paths). Independent of `036c`, the setext-collision slice.
**Naming.** This is `036d`. `036c` was already given to the setext collision in `addendum-b-2026-09-22.md`; that name stands, and this slice took the next letter rather than churn one already sent.

---

## 0. What the owner decided, and what it is not

**Option C: fix the output *and* say what that makes the option.** Two halves, both required — half one alone
would leave the documentation quietly wrong, which is the thing this RFC exists to stop.

**It is not a deprecation.** `unwrap_unknown_wrappers` is inert *today* because `<div>` has no Markdown form,
not because it is unimplementable. A future mode that preserves raw HTML revives it at once. `api/modes.md`
already draws this line for the other fields — *"a statement about today's behaviour, not a deprecation"*.
**Do not add `#[deprecated]`. Do not emit a warning. Do not remove the field or the CLI flag.**

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Half one — unwrapping removes the tag, not the paragraph break

```
<div>First.</div><div>Second.</div><div>Third.</div>
  Balanced/Strict/Preserve  ->  "First.\n\nSecond.\n\nThird.\n"   correct today
  Minimal/Semantic          ->  "First.Second.Third.\n"           the bug
```

Same for `<section>`, `<article>`, `<main>`. Not even a space survives. A `<p>` inside the wrapper still
separates, which is why hand-written fixtures never caught it: they put `<p>` inside. Div-per-line markup
without `<p>` is ordinary CMS and SPA output, and **Minimal is the mode the documentation recommends for
"LLM input, text extraction"** — the use that suffers most from welded words.

`docs/src/api/elements.md:19` already promises the right behaviour: *"Block separator — Act as paragraph
breaks; unwrapped (tag removed, children kept) when `unwrap_unknown_wrappers` is on."* Make the code keep
that promise.

Not a regression: identical in `2.2.3`.

## 2. Half two — prove inertness, then write it down. **Prove first.**

The decision rests on a measurement of mine: across twelve probe shapes, every observable difference between
Balanced and Semantic was the lost separator and nothing else. Twelve shapes is evidence, **not proof**, and
the whole of half two is built on it.

**So establish it properly before changing any documentation.** Run every harness cell and every accumulated
battery through Balanced and Semantic after half one and confirm the outputs are byte-identical.

> 🛑 **If you find even one shape where they still differ, stop and report it.** Do not "fix" it, do not
> document around it, do not proceed to half two. A counter-example means option C's premise is wrong and the
> owner's decision has to be revisited — that is an escalation, not an obstacle. Half one still stands on its
> own in that case.

Once proven, add a characterisation test in the shape the project already uses —
`tests/characterisation_structural.rs` has `balanced_strict_preserve_are_identical_on_the_wrapper_fixture`
and `..._on_an_attribute_rich_element`, both built on fixtures **chosen to discriminate a difference if one
existed** rather than fixtures that happen not to. Match that standard: your fixture must contain
div-per-line markup *without* `<p>`, which is precisely what used to distinguish them.

## 3. Half two — the documentation changes

Only after §2 is proven.

| File | Change |
|---|---|
| `docs/src/api/options.md:72` | `unwrap_unknown_wrappers` Effect column → **no effect today** (the wording is yours, but it must **not** read "deprecated" like the five above it) |
| `docs/src/api/options.md:74–77` | The note says "the five deprecated fields" and lists three identical modes. It now has to carry two distinct reasons — five fields that are deprecated, and one that is inert today — and four identical modes |
| `docs/src/api/modes.md:17` | Heading: Balanced, Strict, **Semantic** and Preserve produce identical output |
| `docs/src/api/modes.md:21–31` | The body names the five fields and "all three modes". Extend, and cite the new test beside the two existing ones |
| `docs/src/api/modes.md:40–43` | *"`Minimal` and `Semantic` are genuinely distinct from the other three and from each other"* — **false for Semantic** after this change. Rewrite: Minimal is the only mode that still differs, and it differs by `drop_interactive_shell` |
| `docs/src/api/modes.md`, Semantic's own section | *"What it does today"* must stop claiming a distinguishing effect |
| `docs/src/api/elements.md:19` | The row is already correct — **verify it, do not edit it** |
| `cli/` `--help` | `--unwrap-wrappers` reads *"Unwrap div/span/section/article/main that carry no meaning"*. After this change it changes nothing. Mark it the same way as the field — **not** `[deprecated, no effect]`, which is the other five's label |

**Check for other places that claim the effect.** The two pages above are the ones I found; grep the book and
the READMEs for `unwrap`, `Semantic` and "three modes" and fix what you find. Report what you changed beyond
this table.

## 4. Acceptance criteria

1. The four shapes in §1 separate correctly in **all five modes**, verified by parsing.
2. **Balanced and Semantic byte-identical** across every harness cell and every battery — or an escalation
   per §2.
3. A discriminating characterisation test, matching the two existing ones' standard.
4. **Minimal still differs** from the other four: `drop_interactive_shell` is untouched by this slice.
5. Byte-identical to `036b` for: all of Balanced, Strict and Preserve; `<div>` **with** `<p>` inside in every
   mode; `<figure>`/`<figcaption>`, which are never unwrapped in any mode.
6. No `#[deprecated]`, no new warning, nothing removed from the public surface.
7. Harness cells for div-per-line, section/article/main-per-line, and nested wrappers, in all five modes.
8. `cargo test` / `fmt` / `clippy` clean; report the count against `036b`'s baseline.

## 5. Things to watch

- **`<figure>` and `<figcaption>` are never unwrapped in any mode** (`api/elements.md:20`, deliberate — they
  are excluded from the wrapper-candidate set, not merely blocked by a check). Do not let this slice reach
  them.
- **Do not chase the mode count.** It is not a goal that four modes become identical; it is a consequence,
  and half two's job is only to state it accurately.
- The separator you are adding is the same block boundary `<p>` already produces — reuse that path rather
  than inventing a second notion of "block break" beside it.

## 6. Report back

`.git-exclude/review-request/036d-div-separator/README.md`: what moved, the §1 shapes' before/after parses,
the §4.2 identity evidence in full, the criterion-5 byte-identical set, the docs diff, and anything here I
got wrong. The last two packages each corrected something of mine; that is the standard, not the exception.
