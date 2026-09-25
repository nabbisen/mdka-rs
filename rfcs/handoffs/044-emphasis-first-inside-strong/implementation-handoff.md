# Developer Handoff — RFC 044 · Emphasis first inside `<strong>` is lost

**RFC.** `rfcs/accepted/044-emphasis-first-inside-strong.md` — accepted by the owner, 2026-09-25
**Milestone.** Unassigned. An output-correctness fix; a patch or a minor
**Priority.** **P2 until §1 says otherwise.** Measuring is the first task, not the last
**Prepared.** 2026-09-25, after the `2.6.0` tag
**Baseline.** `2.6.0`, tag `da687c4` — 605 Rust (`cargo test --workspace`), 42 Node, 25 loader, 91 Python; seven workflows green
**Scope.** The inner emphasis delimiter choice and the harness. **No other element, no option, no binding signature.**
**Credit.** You found this, fuzzing your own subscript marker during RFC 043, and reproduced it on the published binary rather than working around it.

---

## 1. Measure it first 🛑

**RFC 044 §5 says plainly that prevalence is unmeasured** — zero occurrences in the four-page corpus used
for RFC 043, out of 42 `<b>`/`<strong>` openings. Those pages are reference and mathematical prose. The
shape this defect needs — bold text beginning with an italic run, followed by more bold text — is an
*editorial* construct.

**Build a prose-heavy corpus and count before you write the fix.** Editorial writing, not documentation:
news articles, essays, blog posts, encyclopaedia prose sections. Ten or more pages. **Fetch them yourself
and record the URLs**, so the count is reproducible — the same discipline RFC 043's corpus table used, and
the reason my own first `<sup>` figure (1%) was wrong by a factor of twenty-five.

Report **two numbers**: `<b>`/`<strong>` openings total, and how many match the trigger shape. If the answer
is zero again over real editorial prose, say so — **that is a finding, not a failure**, and it means this
stays P2 and probably waits for a quiet release.

**This measurement was going to come from bekoedit**, whose paste corpus is the most representative HTML
anyone involved has. Correspondence with them is paused until `3.0.0` by owner decision, so it falls to us.

## 2. The defect

```
<b><em>q</em>a</b>   ->   **_q_a**   ->   parses as  strong("_" "q_a")
```

The emphasis is lost and a literal underscore lands in the text. RFC 044 §2 has the bisection: seven shapes,
three broken, four fine. **The trigger is narrow** — emphasis as the *first* child of a strong element,
closed immediately against a word character, where CommonMark will not let an intraword `_` close.

## 3. The fix, and the part I could not settle

RFC 044 §4 verified the rule before the RFC was written: **keep the `_` swap, and prefer `*` when the
closing `_` would land against a word character.** `***q*a**` parses correctly as `strong(em("q") "a")`.

**A hypothesis about the site, for you to verify rather than accept.** The swap looks like it lives in
`src/renderer/sink.rs` around lines 870–887, in the block that rewrites a just-closed `*…*` run to `_…_`
when a strong span opens against it — RFC 037's addendum. `src/renderer.rs:936` computes `underscore_safe`
for RFC 043's subscript marker using `sink.glued_to_word()` and `sink.underscore_may_pair()`, which is the
same *kind* of question this needs to ask.

**The part I could not settle, and it is the real design problem:** that rewrite appears to happen when the
span opens — **before the following character exists in the buffer.** The rule in §4 depends on what comes
*after* the closing delimiter. So either the decision is deferred until the next character is known, or the
swap is applied and then reverted when the next character turns out to be a word character. **Pick
deliberately and say which in the report**, with the reason. Do not take my reading of the code as the
diagnosis — see how the last two npm diagnoses went.

## 4. Criteria

1. All three broken shapes in RFC 044 §2 parse as `strong(em(…) …)`; the four working shapes are
   **byte-identical to `2.6.0`**. Compare bytes, not eyes.
2. `***q***`-style inversion does not return: `<b><em>q</em></b>` still emits a form parsing as
   `strong(em("q"))`, not `em(strong("q"))`. That inversion is why the `_` swap exists.
3. Harness cells for **every row** of §2's table — the four working rows as guards, not only the broken
   ones — in all five modes and both readings.
4. **The new cells fail on the unfixed source**, demonstrated with output, as `2.4.2` and RFC 043 both did.
5. No literal `_` or `*` in any output that was not in the source.
6. §1's corpus table, with URLs and both counts.
7. 605 Rust (`cargo test --workspace` — `cargo test` alone gives 578 and is the wrong command), 42 Node,
   25 loader, 91 Python, plus the new cells; seven workflows green.

## 5. Not in this slice

- Any other emphasis case, any other element, any option.
- Removing or weakening the `_` swap. It solves a real problem; §3 narrows it.
- Telling bekoedit. Correspondence is paused until `3.0.0` and is the owner's decision regardless.

## 6. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.

**Report §1 before implementing §3.** If the corpus says zero, stop there and report — do not write the fix
speculatively. Report to `.git-exclude/review-request/044-emphasis-first-inside-strong/README.md`.
