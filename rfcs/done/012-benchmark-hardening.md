# RFC 012 — Benchmark hardening and regenerating the published claims

**Status.** Implemented (2.4.0) — closed 2026-09-23 at `d567509`
**Author.** Architect
**Created.** 2026-09-23
**Milestone.** M4 · Coverage and durability → `2.4.0`
**Source.** Roadmap P1. The performance page carries numbers measured on **mdka 2.0.0** in April and already admits two of its own statements no longer hold.
**Depends on.** RFC 008 — done. **Touches** RFC 022's second half (removing `alloc_counter`), which owns `benches/memory.rs`.
**Touches.** `benches/`, `docs/src/design/performance-characteristics.md`, `README.md`.

---

## 0. Closed, 2026-09-23 at `d567509`

Every table on the performance page regenerated in one sitting on one machine, with the environment, the
commit and the exact commands recorded beside them. The `2.3.0` retraction note is gone, superseded. All
eight competitors measured at their unchanged pinned versions, in the same process as mdka's own row.

**The page carries numbers a reader can reproduce, for the first time.**

Also landed: the README tagline (*"at competitive speed and near-flat memory"*) and the depth-bound sentence
separating crash-safety from speed, linking to a new scaling section — the two owner items this RFC absorbed.

### Two things worth keeping

**`mdka_v1`'s memory figures are cumulative allocation, not peak.** It reports 933 MB on `flat` where the
old page said 7.90 MB. The implementer suspected the harness first, reproduced it outside Criterion, then
cross-checked `/proc/self/status`'s `VmHWM`: real peak resident memory **~8.9 MB**. `ALLOCATED_TOTAL` is
monotonic and never netted against frees, so the metric was never peak for any library. The page now states
what it measures **before** the table rather than after.

This does **not** invalidate the `2.3.0` CHANGELOG's *"peak memory … within about 3%"* or the same claim in
the letter sent to bekoedit — both came from child-RSS measurement, recorded in
`.git-exclude/reviewed/2.3.0-prep/README.md`, with the peak-versus-allocation distinction drawn at the time.

**A "parser's share" column was removed in review** because it read 100.5% and 100.2% on two rows: the total
and the parser are timed in two independent loops, so noise can make a part exceed its whole. The fix states
the independence and lets the two times stand side by side — the conclusion (almost all the cost is in the
parser) survives without a number that cannot be true as labelled.

## 1. The problem is not that the numbers are old

`docs/src/design/performance-characteristics.md` publishes a conversion-speed and memory table across
**eight libraries**, measured 2026-04-15 on **mdka 2.0.0**, on a machine that is nowhere recorded. A note
added at `2.3.0` prep already retracts two claims drawn from it and promises regeneration.

**The deeper problem is that nothing can be compared to it — including by us.** I demonstrated this on
myself before writing this RFC.

## 2. The measurement, and the trap

Running our own `convert` benchmark today and reading it against the published column suggests mdka has got
**faster** on three of six datasets:

| | published 2.0.0 (April, unknown machine) | criterion today | naive reading |
|---|---|---|---|
| large | 12.336 ms | 12.015 ms | 2.6% *faster* |
| deep_nest | 32.620 ms | 25.885 ms | 21% *faster* |
| flat | 5.6253 ms | 5.4899 ms | 2.4% *faster* |

**That is wrong.** Different machine, different day. Measured properly — mdka `2.0.0` and current `main`
built from the same source tree, run **interleaved on one machine**, best-of-N, median of five rounds:

| Dataset | 2.0.0 | `main` | delta |
|---|---|---|---|
| small (50k) | 0.556 ms | 0.655 ms | **+17.8%** |
| medium | 1.147 ms | 1.335 ms | **+16.4%** |
| large | 10.650 ms | 12.295 ms | **+15.4%** |
| deep_nest | 23.546 ms | 25.281 ms | +7.4% |
| flat | 4.938 ms | 5.604 ms | **+13.5%** |
| malformed | 0.030 ms | 0.035 ms | +16.7% |

**`main` is 7–18% slower than `2.0.0`, on every dataset.** Consistent with the recorded 9–14% against
`2.2.3` plus earlier drift.

**A reader doing what I first did would conclude the opposite of the truth.** That is the case for this RFC:
not that the table is stale, but that it is *unfalsifiable and currently misleading*.

## 3. What "hardening" has to mean

The published note promises regeneration *"on a quiet machine"*. That is necessary and not sufficient — a
number with no recorded environment cannot be checked by anyone, including us next time.

**Every published figure must carry the method that produced it**: machine, OS, CPU, Rust version, date,
competitor versions, and the command. Reproducibility is the deliverable; the numbers are its output.

## 4. Decisions

### 4.1 Keep the eight-library comparison, or drop it?

The page says it is *"not intended to rank libraries or declare a winner"* and then prints a table with the
winner in bold. mdka is **not fastest in any row** — `fast_html2md` leads five of six.

| Option | For | Against |
|---|---|---|
| **A — keep and regenerate** ⭐ | It is honest, it is the only reason to trust our own numbers, and **all eight competitors are pinned dev-dependencies**, so it is mechanical, not research | Goes stale every release unless regeneration is part of release prep |
| B — drop it, publish mdka-only | Nothing to maintain | Removing a table in which we do not win, right after measuring that we got slower, is the wrong look and the wrong instinct |

**Recommendation: A.** The pinning is the project's own earlier good decision and it makes this cheap.

### 4.2 Regenerate at the pinned versions, or bump the competitors?

**Recommendation: regenerate at the pinned versions, and say so.** Bumping seven crates invites bench
breakage and re-stales immediately. A table that says *"measured against htmd 0.5.4"* is honest; a table
that silently compares against whatever resolved last is not. Bumping is its own maintenance decision, not
part of this.

### 4.3 A CI regression gate — **no**

**Recommendation: do not add one.** CI runners are too noisy for absolute timings, and a gate that flaps
gets disabled — this project has already learned what a gate nobody trusts is worth. The honest control is a
documented procedure run deliberately at release prep, not a red X of unknown meaning.

### 4.4 The two owner items this absorbs

**The README tagline.** *"…without sacrificing speed or memory"* is live on three registry pages. Peak memory
is genuinely near-flat; speed is now measurably **7–18% behind 2.0.0**. Recommendation: qualify the speed
half, keep the memory half — *"correct, readable Markdown from real-world HTML, at competitive speed and
near-flat memory."*

**The depth bound.** *"No stack overflow, no matter the nesting depth"* is true and should stay — it is a
claim about **crashing**. A reader with untrusted input will hear it as a claim about **safety**, and 860 kB
of nested `<div>` costs 6.7 seconds, quadratic in depth, **almost entirely inside the HTML parser** (measured:
6.60 s of 6.65 s in `scraper::Html::parse_document`). One sentence separating the two, next to the existing
claim, plus the scaling data on this page.

## 5. Acceptance criteria

1. Every table on the performance page regenerated on one machine in one sitting, with the **environment and
   command recorded on the page**.
2. Competitor versions named in the table, matching the pinned dev-dependencies.
3. The `2.3.0` retraction note removed — superseded by numbers that are actually current.
4. mdka's own row measured **interleaved** against whatever it is compared to, never across sittings.
5. The depth/width scaling data published, with the parser attribution stated.
6. README tagline updated per §4.4, and the crash-vs-speed sentence added.
7. A short "how these were produced" section sufficient for a reader to re-run them.
8. No claim on the page that cannot be reproduced from §7.

## 6. Size

**M.** No source changes: this is measurement, documentation, and one README line. The cost is care, not
code — every number on the page has to come from one sitting, and the temptation to reuse a figure measured
a week earlier is exactly what produced the current state.

**Sequencing:** RFC 022's second half removes `alloc_counter`, which `benches/memory.rs` uses. **Do that
first or this RFC's memory table is measured against code that is about to change.**

## 7. Not in scope

Making mdka faster. This RFC measures and publishes honestly; recovering the 7–18% is separate work and
should be scoped from these numbers once they exist.

## 8. Quietness needed evidence too, 2026-09-30

§3 says *"every published figure must carry the method that produced it"* and lists six things to record:
machine, OS, CPU, Rust version, date, competitor versions, command. **A seventh claim was made without
evidence: "on a quiet machine."** Every other environmental fact on the page is a number or a version
string a reader can check; quietness was an assertion nobody could check, including us.

This gap was not theoretical. The `3.0.0` benchmark regeneration (`rfcs/handoffs/3.0.0-benchmark-regeneration/`)
measured a "+2% to +5%, repeatable" cost for one code path and, separately, a "faster by up to 37%, growing
with thread count" result for another, on a machine asserted quiet but not logged. The owner later reported
background processes running during that sitting. A re-run on a machine with `/proc/loadavg` sampled and
recorded before, during, and after each `cargo bench` invocation found the first result was **not there** —
it collapsed to within noise — and the second **reversed direction** once measured cleanly. Neither of those
two outcomes could have been told apart from "the numbers moved because the machine got quieter" without
a logged number to compare against. The internal control (`deep_nest`, a dataset RFC 012's own change never
touches) passed on the noisy run too — quietness has to be checked directly, because a control checks
whether *the change itself* leaked into an unrelated path, not whether *the room* is quiet.

**Every regeneration from now on records `/proc/loadavg` (or the equivalent) immediately before, at least
once during, and immediately after each benchmark invocation, in the Conditions table.** A number without
a load figure beside it is the same unfalsifiable claim §3 was written to end, just for one more variable.

---

## 9. What the page is for, and what it costs — amendment, 2026-09-30

Four corrections, from the `3.0.0` regeneration. The original text stands; this replaces the assumptions
behind §4.1 and §3.

### 9.1 §4.1's decision rested on a cost claim that was wrong

Its reasoning for keeping the eight-library table: *"the pinning … makes this cheap."* **Measured:** the
three `cargo bench` invocations took **33.5 minutes** (from the load log's own timestamps), inside a
regeneration the owner reports as **over three hours**. The remaining ~2.5 hours has never been broken down.

**The table stays** — §4.1's argument against dropping it is still right, and more so after finding a
regression. **What changes is that it is not a release obligation.** It never appeared in
`RELEASE-CHECKLIST.md` and must not be added: the peers are pinned, so only `mdka`'s column can move, and
that column has been measured as near-static across six months. Regenerate when someone has a reason.

**Any future regeneration timestamps each phase**, so the next person optimising it is aiming at the right
two hours rather than the visible half-hour.

### 9.2 Only numbers from the same sitting may be compared

The `3.0.0` page compared its peer ratios with the `2.3.0`-era page's and called them essentially
unchanged. **Five of six had moved 28–48%** — because `mdka_v1`, pinned at `1.6.9` with unchanged code,
measured 30–48% faster in the new sitting while `mdka` measured flat.

**A pinned library's own number moved by half, on the same machine, six months apart.** So: within a table,
columns measured together are comparable. Across pages, nothing is — not absolutes, and not ratios derived
from them. **When a trend matters, re-measure every endpoint again.**

### 9.3 The objective is a representative average, not a fastest lap

The page exists to tell a reader what `mdka` costs **on an ordinary machine**. That rules out the
laboratory practices that would otherwise be the obvious way to reduce variance: **no core isolation, no
fixed governor, no disabling turbo.** Each of them buys precision by measuring something no user
experiences.

Variance is handled by **sampling across it** — repetitions, and a reported spread rather than a bare point
— not by removing it.

**§8's load log keeps its place with a corrected purpose:** it shows the machine was **ordinary**, not that
it was quiet. A runaway background process is not a general environment; it is a broken one, and that is
what the log is there to catch.

### 9.4 Comparison and measurement are different jobs

*"Did we get slower?"* does not need this page, this machine, or a quiet room. It needs **paired A/B,
interleaved, in one sitting** — which cancels the environment, and which is why the `2.9.0`/`3.0.0` deltas
survived a contended sitting while the absolutes did not.

That is a **gate's** design, not a page's, and its absence is why `3.0.0` shipped a 50–80% bulk-conversion
regression with every other gate green. **Performance is the last property this project asserts by hand.**

