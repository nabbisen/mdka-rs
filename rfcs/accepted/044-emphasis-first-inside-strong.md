# RFC 044 — Emphasis first inside `<strong>` is lost, and leaves a literal `_`

**Status.** Accepted — owner, 2026-09-25
**Author.** Architect
**Created.** 2026-09-25
**Milestone.** Unassigned. An output-correctness fix; a patch or a minor. **Scheduled after `2.6.0`** — see §8.
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

---

## 8. Owner decision, 2026-09-25

**Accepted**, with the sequencing recommended in §5 and in the release-scope package: **this does not ride
`2.6.0`**. That release was authorised the same day and carries a live glibc exclusion, a live false type
all-clear and RFC 043's output fix; holding it for a P2 whose prevalence is unmeasured would have been the
wrong trade.

**So the handoff is issued after `2.6.0` is tagged**, not before. Two reasons, and the second is the real one:

1. A conversion change landing on `main` during release prep would move the tip I have to tag.
2. **`2.6.0` gives the corpus this RFC could not have.** §5 says plainly that the four-page corpus found zero
   occurrences and that a prose-heavy corpus should be measured before anyone calls this urgent. The
   implementing handoff will carry that measurement as its first task, so the priority is set by evidence
   rather than by the plausibility argument in §5.

Nothing in §1–§7 changes. The fix in §4 stands as proposed and was verified before the RFC was written.

---

## 9. Residuals, recorded 2026-09-25 after implementation

The fix covers §2's three broken shapes. Three shapes remain wrong and are **not** covered. They are split
here because "cannot be fixed" and "not fixed yet" are different claims and the difference decides whether
anyone looks again.

### 9.1 Unfixable by delimiter choice — `<b><em>q.</em>a</b>`

```
**_q._a**   ->   strong("_" "q." "_" "a")
```

An italic run ending in punctuation, closed against a letter. **No delimiter works.** A closing `*` here is
preceded by punctuation (`.`) and followed by an alphanumeric (`a`), which under CommonMark is **not
right-flanking**, so `*` cannot close either — verified: `***q.*a**` → `"*" "*" em("q." em("a"))`, and
`__*q.*a__` → `strong("*" "q." "*" "a")`.

Closing this needs something other than a delimiter choice — raw HTML passthrough, which RFC 008 §4 and
RFC 009 §4.3 both declined. **Nothing to schedule.**

### 9.2 Fixable, unscheduled — `<b><em>q</em>a<em>r</em></b>`

```
**_q_a*r***   ->   strong("_" "q_a" em("r"))
```

**A working encoding exists:** `__*q*a*r*__` parses as `strong(em("q") "a" em("r"))`, correctly. It is not
applied because it swaps the **outer** delimiter rather than the inner one, which is a larger change than
this RFC's rule and needs its own design — the outer `__` has its own flanking constraints, and RFC 037's
addendum owns that choice.

**This is work, not an impossibility.** It belongs to whoever next touches the delimiter machinery.

### 9.3 Pre-existing, in the other direction — `<b><em>q</em></b>x`

```
***q***x   ->   em(strong("q")) "x"
```

**The inverted nesting the `_` swap exists to prevent**, surviving where the swap does not reach: the outer
flank guard (RFC 037's addendum, which looks at what follows the whole bold) reverts to `*`. The source is
`strong(em(…))`; the output reads `em(strong(…))`. Bold-italic either way, so visually identical and
structurally wrong.

Byte-identical to `2.6.0` and untouched by this RFC. Found by the dev team while implementing it.

---

## 10. Prevalence, measured 2026-09-25 — the answer is zero

**0 of 424** `<b>`/`<strong>` openings match the trigger, over **42 pages** of editorial prose, blogs,
essays, news and how-to. Confirmed two independent ways: a spec-compliant DOM parse, and the output the
published `2.6.0` binary actually writes, parsed. A positive control shows the detector fires.

Four openings had emphasis first; three were *whole-bold-is-italic* (which parses correctly) and one was
followed by a space (also correct). **Rule of three: a 95 % upper bound of about 0.7 %** of bold openings.

**What it does not measure.** These are published web pages, which have been through a CMS that normalises
markup. §5's plausibility argument was about **editor-generated paste HTML**, and that remains unmeasured —
bekoedit's corpus is the representative sample and correspondence with them is paused until `3.0.0`.

**So the priority stands at P2** and this ships in whatever release is next, not one of its own. The defect
is real, silent and reproduced on the published binary; it is simply not common in published prose.

