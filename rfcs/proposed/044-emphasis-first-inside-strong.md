# RFC 044 — Emphasis first inside `<strong>` is lost, and leaves a literal `_`

**Status.** Proposed
**Author.** Architect
**Created.** 2026-09-25
**Milestone.** Unassigned. An output-correctness fix; a patch or a minor.
**Source.** Found by the dev team while fuzzing their own subscript marker during RFC 043 — 28 000 generated cases across six contexts, each converted **and parsed**. Not related to `<sup>`/`<sub>`; they reproduced it on the published binary and did not touch it.
**Touches.** wherever RFC 037's addendum chooses the inner emphasis delimiter, plus the output-validity harness.
**Relates to.** RFC 037's addendum, which introduced the `_` swap this narrows. Same class as RFC 036/037/038.

---

## 1. The defect

```
<b><em>q</em>a</b>   ->   **_q_a**   ->   parses as  strong("_" "q_a")
```

**The emphasis is lost and a literal underscore appears in the text.** Verified against the published
`2.5.1` binary, not a build.

Expected: `strong(em("q") "a")`.

## 2. Exactly when it happens

Bisected on the published binary:

| Input | Output | Parse | |
|---|---|---|---|
| `<b><em>q</em>a</b>` | `**_q_a**` | `strong("_" "q_a")` | **broken** |
| `<b><i>q</i>a</b>` | `**_q_a**` | `strong("_" "q_a")` | **broken** |
| `<strong><em>q</em>1</strong>` | `**_q_1**` | `strong("_" "q_1")` | **broken** |
| `<b><em>q</em> a</b>` — space after | `**_q_ a**` | `strong(em("q") " a")` | fine |
| `<b><em>q</em></b>` — nothing after | `**_q_**` | `strong(em("q"))` | fine |
| `<b>a<em>q</em>b</b>` — not first | `**a*q*b**` | `strong("a" em("q") "b")` | fine |
| `<em><b>q</b>a</em>` — the other nesting | `***q**a*` | `em(strong("q") "a")` | fine |

**The trigger is narrow: emphasis as the first child of a strong element, closed immediately against a word
character.** CommonMark will not let an intraword `_` close, so the delimiter is left as literal text and the
emphasis never forms.

## 3. Why the `_` is there, and why it must stay for the other cases

`_` is not arbitrary. With `*`, the all-emphasis case inverts the nesting:

```
***q***   ->   em(strong("q"))      wrong: the source is strong(em)
**_q_**   ->   strong(em("q"))      correct
```

So the swap solves a real problem, and mdka already varies by position — `<b>a<em>q</em></b>` emits
`**a*q***`, using `*`, and parses correctly. **The gap is the one position in between**: first child, with
trailing content, where `_` is chosen and cannot close.

## 4. The fix, verified before proposing

In that position, `*` works and `_` does not:

```
***q*a**   ->   strong(em("q") "a")     correct
__*q*a__   ->   strong(em("q") "a")     also correct
```

The inverted-nesting problem of §3 only arises when the emphasis spans the **whole** strong content. With
trailing content present, `***q*a**` is unambiguous.

**Proposal: keep the `_` swap, and prefer `*` when the closing `_` would land against a word character.**
That is the same shape of rule the renderer already applies by position; this extends it to look at what
follows the closing delimiter, which is the thing it currently does not do.

## 5. What I could not establish

**Prevalence. I measured and found nothing.** The four-page corpus used for RFC 043 contains **zero**
occurrences of the shape, out of 42 `<b>`/`<strong>` openings — too small a sample to conclude anything, and
those pages are reference and mathematical prose rather than editorial text.

**So this RFC does not claim the defect is common.** It claims it is real, reproducible, silent, and cheap to
fix. Bold text containing an italic run followed by more bold text is an ordinary editorial construct and
plausible in editor-generated HTML — which is the paste case our one known consumer cares about — but I have
not shown that, and a prose-heavy corpus should be measured before anyone calls this urgent.

**Priority: P2**, on that basis. If the implementer's own corpus work turns up occurrences, that changes.

## 6. Acceptance criteria

1. All three broken shapes in §2 parse as `strong(em(…) …)`; the four working shapes are **byte-identical** to
   `2.5.1`.
2. `***q***`-style inversion does not return: `<b><em>q</em></b>` still emits a form parsing as
   `strong(em("q"))`.
3. Harness cells for every row of §2's table, in all five modes and both readings — the working rows as
   guards, not only the broken ones.
4. **The new cells fail on the unfixed source**, demonstrated, as with `2.4.2` and RFC 043.
5. No literal `_` or `*` left in any output that was not in the source.
6. Existing counts pass; no change outside the delimiter choice and the harness.

## 7. A note on how this was found

Nobody was looking for it. It surfaced because a fuzzer built for an unrelated marker **parsed** its outputs
instead of eyeballing them, and because the dev team chased an anomaly in their own guard back to a
reproduction on the published binary rather than working around it.

That is now the fourth defect found this way, and the second found by machinery built for something else.
It is the argument for the harness parsing rather than comparing strings.
