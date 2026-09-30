# Developer Handoff — find the cause of `3.0.0`'s bulk-conversion slowdown

**Authorised.** Owner, 2026-09-30, accepting the self-review at `.git-exclude/reviewed/benchmark-environment-architecture/README.md` §6 — *profile first, decide after.*
**Milestone.** Unassigned. A shipped regression in `3.0.0`.
**Priority.** P1 to **diagnose**. Not yet P1 to fix — that decision follows the cause.
**Prepared.** 2026-09-30
**Baseline.** `442fb65`; `3.0.0` published.
**Scope.** Investigation. **Change nothing in `src/` except throwaway experiments you revert.** The output is an explanation, not a patch.

---

## 1. What is known, and it is a lot

`3.0.0`'s bulk **file** conversion is slower than `2.9.0` and **stops scaling**. Measured twice, with and
without filesystem writes:

```
1t  +0.5% / −0.1%     2t  +2.4% / +1.8%     4t  +12.5% / −2.0%
8t  +11.5% / +32.2%   16t +70.2% / +57.9%   32t +52.5% / +64.8%
```

- **Noise below 8 threads**, not even consistent in sign at 4.
- **Large and reproducible from 8 up**, in both runs and both I/O conditions — **so it is the library, not
  the disk.**
- **`3.0.0` barely improves from 16 to 32 threads** (23.0→19.5 ms) where `2.9.0` still does (13.5→12.8 ms).
  A scaling ceiling, not a constant cost.

**The only code difference** in `html_files_to_markdown_with`, verified by diffing the function body
between worktrees at the two tags: the return type. `Vec<(&'a P, Result<PathBuf, MdkaError>)>` became
`Vec<FileOutcome>`, where `FileOutcome` owns its `src: PathBuf`. Element size **32 B → 48 B**; one extra
`PathBuf` clone per file, measured at **+165 B/file**.

**The standing candidate is allocator contention** from that clone. It is a candidate because it is the
only change on the path and because contention plausibly grows with thread count. **Nobody has confirmed
it, and the effect is larger than one allocation per file ought to buy.**

## 2. The question

**Why does `3.0.0` stop scaling at 8+ threads when `2.9.0` does not?**

Not *"is it slower"* — that is settled. Not *"should we fix it"* — that follows.

## 3. How to go at it

**Cheap discriminating experiments before any profiler.** Each is a throwaway patch on a worktree at the
`3.0.0` tag, reverted after; each rules something in or out:

1. **Remove the clone, keep the type.** Have `FileOutcome.src` be populated by `std::mem::take` from an
   owned input, or build the `Vec` without cloning. **If the ceiling disappears, it is the allocation.**
2. **Keep the clone, shrink the element.** `Box<Path>` instead of `PathBuf` (16 B vs 24 B), or reorder
   fields. **If the ceiling moves with element size but not with the clone, it is memory traffic or
   layout, not the allocator.**
3. **Swap the allocator.** Build both versions with `mimalloc` or `jemalloc`. **If `3.0.0`'s ceiling
   vanishes under a different allocator, the candidate is confirmed** and the fix may be as small as a
   documentation note about allocators, or as large as avoiding the allocation.
4. **Take the filesystem out entirely** — already done, and it survived; do not repeat it.

**Then profile only if 1–3 do not settle it.** `perf stat` first (cycles, cache-misses, page-faults for
both versions at 16 threads) before `perf record`; the counters often name the class without a flamegraph.

**Measure the way the last round measured** — paired, interleaved, same sitting, medians, load logged, on
an ordinary machine. RFC 012 §9.3: **no core pinning, no governor changes, no turbo fiddling.** The
regression is being characterised, not minimised.

## 4. What to report

1. **The cause**, or a precise statement of what has been excluded and what remains. *"Not identified"*
   is an acceptable outcome; *"probably X"* without an experiment that would have shown not-X is not.
2. **Whether a fix is available and what it costs** — but **do not write it.** If experiment 1 shows the
   clone is the cause, say what removing it would require of the public type, since `FileOutcome.src` is
   now part of the `3.0.0` API and cannot change shape in a patch release.
3. **Whether `2.9.0`'s scaling was itself unusual.** `2.9.0` reaches 4.38× from 2 to 32 threads (27% of
   ideal); `3.0.0` reaches 6.37×… on the earlier, now-superseded numbers. **Recompute both from the clean
   run** — that comparison was quoted from data we have since discarded, and I do not know that it holds.
4. **What a gate would have to measure to catch this**, in one paragraph. The next two pieces of work are a
   CI regression gate and a concurrency gate, and the second exists because this defect is invisible below
   8 threads. Your experiments will show which signal is cheapest to watch — wall time, allocation count,
   or scaling ratio.

## 5. Not in this slice

- **Any fix.** Diagnosis only.
- The CI regression gate and the concurrency gate. They follow, and §4.4 informs them.
- Regenerating the performance page. RFC 012 §9.1: it is no longer a release obligation.
- A `3.0.1`. That is the owner's call once the cause is known.

## 6. Committing and pushing

Only work that is yours and approved. Report first. **Name who approved anything you push.**
`git commit -F <file> -- <explicit paths>`.

**Expect to push nothing.** This slice's output is a report; the experiments are throwaway and reverted.
If you find you want to keep a benchmark harness, propose it rather than committing it — the concurrency
gate is the right home for it and it is the next piece of work.

Report to `.git-exclude/review-request/3.0.0-bulk-regression-profiling/README.md`, leading with §4.1.
