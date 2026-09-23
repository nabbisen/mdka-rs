# Developer Handoff — RFC 012 · Benchmark hardening and regenerating the published claims

**Governing RFC.** [RFC 012](../../done/012-benchmark-hardening.md) — §2 the measurement and the trap, §3 what hardening means, §4 the decisions, §5 criteria
**Milestone.** M4 · `2.4.0`
**Priority.** P1
**Prepared.** 2026-09-23
**Baseline.** whatever the `alloc_counter` removal lands as — **not** `d2c5448`
**🛑 Blocked on.** RFC 022's second half: `rfcs/handoffs/022-cli-allocator-and-jemalloc/second-half-handoff-2026-09-23.md`. **Do not start until it lands.**

---

## 0. This slice is discipline, not code

There are no source changes. Every number on the performance page must come from **one sitting, one
machine, against code that is not about to change** — which is why the `alloc_counter` removal goes first,
and why reusing a figure measured last week is the exact habit that produced the page's current state.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Read §2 of the RFC before you measure anything

The published table was measured on **mdka 2.0.0** in April, on a machine recorded nowhere. Reading today's
criterion output against it suggests mdka got **faster** on three of six datasets. **It did not.** I made
that mistake before writing the RFC.

Measured properly — `2.0.0` and `main` from the same tree, **interleaved on one machine**, best-of-N, median
of five rounds — `main` is **7–18% slower on every dataset**. The table in RFC 012 §2 has the figures.

**So: never compare across sittings or machines, including against the page's own history.** If a number
cannot be produced beside the thing it is compared to, it does not go on the page.

## 2. What to produce

1. **Regenerate every table** — conversion speed and memory, all eight libraries — in one sitting on one
   quiet machine.
2. **Record the environment on the page**: machine, CPU, OS, Rust version, date, and the exact command.
3. **Name the competitor versions in the table**, matching the pinned dev-dependencies. They are pinned
   (`mdka-v1 =1.6.9`, `html2md =0.2.15`, `fast-html2md =0.0.61`, `htmd =0.5.4`,
   `html-to-markdown-rs =3.1.0`, `html2text =0.16.7`, `dom_smoothie =0.17.0`) — regenerate **at those
   versions**, do not bump. RFC 012 §4.2 says why.
4. **Delete the `2.3.0` retraction note** — it is superseded by numbers that are actually current.
5. **Publish the depth/width scaling data**, with the attribution: the cost is quadratic in nesting depth and
   **almost entirely inside the parser** — 6.60 s of a 6.65 s run at depth 80 000 is
   `scraper::Html::parse_document`. `benches/scaling.rs` exists for this.
6. **A "how these were produced" section** a reader can re-run from.

## 3. Two documentation changes the RFC absorbed, both owner-approved

**README tagline** — *"…readable output from real-world HTML, without sacrificing speed or memory."* Memory
is genuinely near-flat; speed is not. Qualify the speed half, keep the memory half. Suggested, not mandated:
*"…correct, readable Markdown from real-world HTML, at competitive speed and near-flat memory."* **It ships
to crates.io, npm and PyPI**, so whatever you write is what three registry pages say.

**The depth bound** — *"no stack overflow, no matter the nesting depth"* **stays**. It is true and it is a
claim about **crashing**. Add one sentence beside it separating that from **speed**, pointing at §2.5's
scaling data. A reader with untrusted input currently reads a crash-safety claim as a performance one.

## 4. Criteria

RFC 012 §5, all eight. The two that are easiest to fudge:

- **§5.4** — mdka's own row measured **interleaved** against whatever it is compared to. Not "we ran ours,
  then theirs".
- **§5.8** — no claim on the page that cannot be reproduced from §7's method. If you cannot re-run it, it
  does not go on.

## 5. Report back

`.git-exclude/review-request/012-benchmark-regeneration/README.md`: the regenerated tables, the recorded
environment, the command, the README diff, and — the part I care most about — **anything on the old page you
could not reproduce and therefore removed**. That list is the real output of this slice.

**And say plainly if the numbers are worse than you expected.** They already are: we are 7–18% behind
`2.0.0`. Publishing that accurately is the job; making it look better is not.
