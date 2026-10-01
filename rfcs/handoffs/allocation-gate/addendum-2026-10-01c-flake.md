# Addendum 3 — allocation gate, 2026-10-01 (c)

Amends `handoff.md` and the two prior addenda, all of which stay as issued.

**The gate is flaky on CI.** It failed on `35d26b9`, a **documentation-only** commit:

```
string conversion allocation moved: measured 11219 B, baseline 10319 B, diff 900 B (tolerance 0 B)
```

**A re-run of that same job, same commit, same code, passed.** That is the decisive fact: this is
non-determinism on the runner, not a property of any commit. The failing workload is the **string**
one — no paths, no rayon, the one that was supposed to be the robust half.

Locally it does not reproduce: five runs of CI's exact invocation
(`cargo test --workspace --all-features --locked --no-fail-fast`) all pass, as did 80 runs across
eight thread counts in the previous round.

## Why this matters more than the previous two

A zero-tolerance gate that flakes **teaches people to re-run until green** — which is precisely how a
real regression gets waved through. A flaky gate is worse than no gate. The previous two defects were
wrong baselines; this one attacks the gate's credibility.

## The likely mechanism, and the property that gives a clean fix

`CountingAllocator`'s counters are **process-global atomics** (`benches/alloc_counter.rs` — note its
own header calls it a *thread-local* counter, which it is not; worth correcting while you are there).
The previous round removed the second `#[test]` that was racing them, but **the libtest harness's own
thread is still in the process**, printing and bookkeeping around the test. On a 4-vCPU runner its
work can land inside the measurement window; on a 32-thread machine it apparently does not.

**The useful property: foreign allocation is one-sided.** Both counters increase monotonically and
the delta is a saturating subtraction, so a stray allocation inside the window can only make the
measurement **too high**, never too low.

**So measure each workload `k` times in-process and assert on the minimum.** The minimum over a
handful of repetitions is the true value unless *every* repetition was contaminated. Tolerance stays
**0** — this does not loosen the gate, it removes a known one-sided error term. `k = 5` is a starting
point; justify whatever you choose.

A thread-local counter is the other candidate. It is a bigger change and it is **wrong for the bulk
workload**, whose rayon workers' allocations must count. Prefer the minimum unless you find evidence
the minimum does not hold.

## Criteria

1. Both workloads measured as a minimum over `k` in-process repetitions; tolerance stays 0 for both.
2. **Prove the estimator, do not just assert it.** Deliberately allocate from a second thread during
   the window, show the single-shot measurement inflates and the minimum does not.
3. **Re-derive both baselines** from the new estimator.
4. Keep the existing evidence in the module doc — thread counts, toolchains, path lengths — and add
   this round's.
5. **Correct `benches/alloc_counter.rs`'s header**, which describes a thread-local counter that does
   not exist. Comment only; no behaviour change, and nothing else in `benches/` touched.

## The bar, stated plainly

This is the gate's **fourth** environment problem — pool size, path length, toolchain (checked, held),
and now harness interference. It has caught **zero** real regressions so far. It is still worth
having: it is the one performance property this project can measure, and the +165 B/file change it
exists to catch is real and did happen.

**But if it fails on CI once more in a new way after this round, it comes out** — the gate is removed
and the property goes back to being measured by hand at releases. I would rather have no gate than
one the team has learned to re-run. Say in your report whether you think this fix clears that bar,
and say so honestly if you do not.

## Scope

`tests/allocation_gate.rs`, plus the comment-only correction in `benches/alloc_counter.rs`. No
`src/`, no workflow. **`main` is green right now** — keep it that way; push only when the fix is
proven locally. No release.
