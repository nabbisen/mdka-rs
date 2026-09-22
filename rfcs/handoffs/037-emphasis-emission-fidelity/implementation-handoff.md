# Developer Handoff — RFC 037 · Emphasis emission fidelity

**Governing RFC.** [RFC 037](../../accepted/037-emphasis-emission-fidelity.md) — §1 the shapes, §1.1 what re-derivation added, §3 design, §4 criteria
**Milestone.** M4 · `2.4.0`
**Priority.** P2
**Prepared.** 2026-09-22
**Baseline.** `2e9af49` — **455 Rust, 39 Node, 80 Python** tests, seven CI workflows green
**Sequencing.** Independent of everything now landed. RFC 036 and RFC 038 are closed.

---

## 0. Read §1.1 first

I re-derived this at `2e9af49` before handing it over and the scope grew in three places. Two of them change
what you build; one changes a criterion you would otherwise have failed against at the baseline.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. The shapes, re-derived at `2e9af49`

**Empty emphasis — delimiters written around nothing:**

| HTML | Output | Parses as |
|---|---|---|
| `<p>a<b></b>b</p>` | `a****b` | `para("a","*","*","*","*","b")` |
| `<p>a<em></em>b</p>` | `a**b` | literal `a**b` |
| `<p>Hello <b></b>world</p>` | `Hello ****world` | literal asterisks mid-sentence |
| `<p>a<b> </b>b</p>` | `a**** b` | whitespace-only; the space escapes the delimiters |
| **`<p><b></b></p>`** | `****` | **`hr`** — content type changed |
| **`<p><em><em></em></em></p>`** | `****` | **`hr`** |
| `<li><b></b>x</li>` | `- ****x` | the same inside a list item |

**Nested emphasis — delimiters concatenate:**

| HTML | Output | Parses as | Wrong how |
|---|---|---|---|
| `<p><em><em>x</em></em></p>` | `**x**` | `strong("x")` | italic became **bold** |
| `<p><i><i>x</i></i></p>` | `**x**` | `strong("x")` | same |
| `<p><em><i>x</i></em></p>` | `**x**` | `strong("x")` | mixed tags too — the rule is about the *delimiter* |
| `<p><b><b>x</b></b></p>` | `****x****` | `strong(strong("x"))` | ugly, not a meaning change |
| `<p><em><strong>x</strong></em></p>` | `***x***` | `em(strong("x"))` | correct |
| `<p><strong><em>x</em></strong></p>` | `***x***` | `em(strong("x"))` | **order inverted** |
| `<p><b><i><b>x</b></i></b></p>` | `*****x*****` | `em(strong(strong("x")))` | double strong |

**All pre-existing in `2.2.3`. No regressions.**

## 2. Design — RFC 037 §3, unchanged

1. **Emit no delimiters for emphasis whose rendered content is empty.** Whitespace-only counts as empty, and
   so does emphasis containing only a dropped element (`<b><span></span></b>` → nothing). This is the same
   shape as the settled rule for a link with no text and no image (RFC 024 criterion 7) — make it read like
   that one.
2. **Collapse nested emphasis of the same delimiter class to one level.** Italic in italic is italic; bold in
   bold is bold. Mixed keeps both.

Fixing (1) removes the two `hr` shapes as a side effect — an empty paragraph that emits nothing is not a
thematic break. **Confirm that rather than assuming it**, and if it does not fall out, say so.

## 3. Criterion 2 — order, and the mechanism that makes it reachable

RFC 037 §4 criterion 2 requires `<em><strong>x</strong></em>` and `<strong><em>x</em></strong>` to parse back
to both, nested, **in the source's order**. §1.1 B shows that **fails at the baseline** — both produce
`***x***`. So this is real work, not a regression guard.

It is reachable with the mechanism already in the file:

```
**_x_**  ->  strong(em("x"))        _**x**_  ->  em(strong("x"))
```

and `src/renderer/sink.rs:634–640` **already alternates to `_`** when adjacent emphasis would merge
(`<p><b>a</b><b>b</b></p>` → `__a__**b**`). Same idea, different position.

**If you conclude order cannot be preserved without an unacceptable cost, stop and say so** with the
evidence — I will relax the criterion rather than have you force it. That is a legitimate outcome; silently
dropping the criterion is not.

## 4. Acceptance criteria — RFC 037 §4, with §1.1 folded in

1. Every empty-emphasis row in §1 produces what the source means, verified by **parsing**: `a b` / `ab` for
   the inline cases, and **nothing at all** for `<p><b></b></p>` — no `hr`, no empty paragraph.
2. Nested same-class emphasis collapses: `para(em("x"))` for nested italic, `para(strong("x"))` for nested
   bold, and no `strong(strong(…))`.
3. `<em><strong>x</strong></em>` and `<strong><em>x</em></strong>` each parse back to both, nested, **in the
   source's order** — or §3's escalation.
4. Emphasis containing only whitespace, and emphasis containing only a dropped element, are treated as empty
   and tested as such. `<p>a<b> </b>b</p>` → `a b`.
5. Non-empty, non-nested emphasis is **byte-identical to `2e9af49`** for the shapes named here — the narrow
   form, enumerated, not the corpus. In particular `<p><b>a</b><b>b</b></p>` → `__a__**b**` must not move.
6. Cells for every row in §1, all five modes.
7. **All three suites** — `cargo test`, `node test.js`, `pytest` — run and reported separately before commit.
8. `fmt`/`clippy` clean; counts against 455 / 39 / 80.

## 5. Things to watch

- **`<p><b></b></p>` is a validity defect, not cosmetics.** It is RFC 038's family wearing different clothes:
  a line that is only delimiters reads as a thematic break. Treat it with the same seriousness.
- **Do not disturb the `_` alternation.** It exists so adjacent emphasis does not merge; criterion 5 pins it.
- **Parse, don't diff.** `****` looks like output; it is a horizontal rule.
- **`<del>`/`<s>` are out of scope** — strikethrough is dropped entirely today, and that is RFC 009.
- **Check `<pre>`.** Every slice this milestone has had to prove it does not write into a code fence.

## 6. Report back

`.git-exclude/review-request/037-emphasis-emission-fidelity/README.md`: what moved, before/after parses for
every row in §1, the criterion-5 byte-identical set, your finding on criterion 3, the three test counts, and
anything here I got wrong. Every package this milestone has corrected something of mine.
