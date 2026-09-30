# Addendum 2 — benchmark regeneration, 2026-09-30 · re-run on a quiet machine, and prove it was quiet

**Amends.** `handoff.md` and `addendum-2026-09-30-isolate-the-filesystem.md`, both frozen. **This supersedes addendum 1's §2 by folding its control into the re-run** — do not measure twice.
**Source.** Owner, 2026-09-30: background processes were running during the measurement.
**State.** `b641446` is unpushed. **Hold it.** The figures in it are superseded.
**Scope.** Re-measurement, one sitting; the page rewritten from the new numbers; a dated amendment to RFC 012.

---

## 1. Yes, re-run — but know what was and was not at risk

Background load does not invalidate everything equally, and the page should not be re-run in a panic.

| Figure | Robust to background load? |
|---|---|
| **The peer-comparison tables** — absolute ms and MB, published as facts about `mdka` | **No.** Load inflates them, and they are quoted as properties of the library |
| **`2.9.0` vs `3.0.0` string conversion**, +2–5% | **Fairly.** Interleaved and alternating, so a proportional slowdown cancels in the ratio — and your `deep_nest` control at ~0% is itself evidence against a load artifact, since load does not care which code path it slows. **I expect this to reproduce** |
| **The bulk 37%** | **Least of all.** It is I/O-bound (~12–20 µs per file *including a write*), and background disk activity is exactly what would fake it |

**So the re-run is not an admission that the work was wrong.** It is that the page publishes absolutes, and
the one anomaly nobody can explain sits in the measurement most sensitive to the thing that was running.

## 2. RFC 012 asks for evidence of everything except the thing that matters most 🛑

RFC 012 §3, its own words:

> The published note promises regeneration *"on a quiet machine"*. **That is necessary and not sufficient** —
> a number with no recorded environment cannot be checked by anyone, including us next time.

It then requires machine, OS, CPU, Rust version, date, competitor versions and the command. **It requires no
evidence that the machine was quiet.** It names the necessary condition and then hardens only the
sufficient ones — which is how a measurement can satisfy the method in full and still be wrong.

**Fix that here**, since we have just been bitten by it:

- **Record load before, during and after each `cargo bench` invocation** — one-minute load average is
  enough, plus a note of anything known to be running. Put it in the page's Conditions table.
- **Abort and restart a run whose load moved materially mid-measurement**, and say if that happened.
- **Add a dated amendment to `rfcs/done/012-benchmark-hardening.md`** recording this: *"quiet" is a claim
  about the machine and must carry evidence like every other environmental claim.* Style as RFC 043 §8 —
  a section at the end, the original text untouched.

**A quiet machine is something the owner provides; you can only record whether it was one.** If load cannot
be brought down, say so and report the numbers with that caveat rather than calling the machine quiet.

## 3. What to measure, in one sitting

1. **Everything the page publishes** — the three `cargo bench` invocations, all eight libraries together
   within each, as before.
2. **`2.9.0` vs `3.0.0` string conversion**, the four-run protocol that found the +2–5%. If it reproduces,
   it is real and load was never the explanation; if it collapses to ~0%, say that plainly — it would mean
   the finding you correctly refused to discard was a load artifact after all, which is worth knowing and
   worth writing.
3. **Bulk file conversion, both ways in the same sitting** (this is addendum 1's control, folded in):
   - **with** file writes, as before;
   - **without** them — convert to memory, discard — at **1, 8 and 32 threads**.

   The comparison between those two is what tells you whether the 37% is the library or the filesystem.
4. **The 4-thread point repeated** (addendum 1 §3.1): the sweep reads −8.65% at 2, **+2.16% at 4**, −21.99%
   at 8, on one measurement each, and the page claims the gap widens with threads.
5. **Node's option-key check** — already settled at noise-scale on published packages, twice. **Do not
   re-run it** unless it is free to include.

## 4. Writing the page afterwards

**Discard the current draft's bulk narrative entirely and write it from the new numbers.** No mechanism may
be asserted that the data does not carry; *"unidentified"* is an acceptable word, and addendum 1 §1.1
explains why the borrow story is not (the new type is 48 B against 32 B, allocates +165 B/file, and is
slower sequentially — none of which a removed lifetime explains).

Keep from the current draft: the string-conversion section's structure and its `deep_nest` control, the
allocation table, the Node result, and the honesty about run provenance.

**Run 1's lost JSON** (addendum 1 §3.2) is moot if the four-run protocol is repeated cleanly — the new runs
all have files. Say in the page that the figures were re-measured on a quiet machine after an earlier
sitting was found to have had background load. **That sentence is the reason anyone should trust the
second set**, so do not omit it out of embarrassment.

## 5. Criteria

1. Every published figure re-measured in one sitting on a machine whose load is **recorded**, not asserted.
2. The Conditions table carries the load evidence and anything known to have been running.
3. §3.3's two bulk measurements, with and without writes, and the page's bulk narrative written from their
   difference.
4. §3.4's 4-thread point repeated, or the non-monotonicity stated.
5. RFC 012 amended per §2, original text untouched.
6. The page says the figures were re-measured after background load was found in the first sitting.
7. Peer versions still unbumped; scope still one page plus the RFC amendment.

## 6. Committing and pushing

`b641446` is unpushed — amend or replace it; one commit is fine. Report first. **Name who approved anything
you push.**

Append to `.git-exclude/review-request/3.0.0-benchmark-regeneration/README.md` as Part 2.
