# Developer Handoff — calibrate the measurement method before building any gate

**Source.** Owner, 2026-09-30: *"did we get a good method or confirm it for stable measurement?"* — **No. We have a stated method that has never been validated.**
**Milestone.** Unassigned. **Precedes** the CI regression gate and the concurrency gate.
**Priority.** P1 — a gate's threshold cannot be chosen without the number this produces.
**Prepared.** 2026-09-30
**Baseline.** `aba2e50`.
**Scope.** Two experiments and a number. **No gate, no fix, no page change, no `src/` change.**

---

## 1. Why this comes first

Three sittings have produced three answers:

| Sitting | String conversion | Bulk at 16–32 threads |
|---|---|---|
| 1 | +2–5% | **−37%** |
| 2 (load logged) | noise | **+50–80%** |
| 3 (six measurements) | noise | noise |

**We believe the third only because it repeated.** We have no evidence about what the method can and cannot
detect — no measurement of its noise floor, and no demonstration that it finds an effect of known size.

**The single cheapest missing experiment has never been run: the harness has never compared a binary
against itself.** Every sitting compared `2.9.0` against `3.0.0` and attributed whatever it saw to the
difference between them. Nobody has asked what the harness reports when there **is** no difference.

## 2. Experiment 1 — the null control 🛑

**Build the same tag twice into two separate probe binaries. Run the full sweep. The answer should be zero.**

- Use `3.0.0` for both sides. **Two independent builds from identical source**, in two worktrees — not one
  binary invoked twice. Two builds of the same source can differ by a few percent from code layout alone,
  and a version comparison contains that noise too, so the control must contain it.
- Run **exactly** what Part 2 ran: 1/2/4/8/16/32 threads, with and without writes, 7 rounds × 5 reps,
  alternating order, medians. Change nothing about the harness.
- Repeat in **two sittings**, per the rule this project has now learned twice.

**What it produces is the number everything else needs: the harness's noise floor, per thread count.**

**And it may settle the open question outright.** If A-vs-A shows ±50% swings at 16 threads, Part 2's
finding was the harness, not the library, and the anomaly is explained. If A-vs-A is tight — say ±5% — then
Part 2 measured something real that has since stopped happening, and that is a different and more
uncomfortable conclusion. **Either way we stop guessing.**

## 3. Experiment 2 — the detection floor

**Inject a slowdown of known size and see whether the method finds it.**

Patch one side of an A-vs-A pair with a deliberate, tunable cost — a `black_box`ed busy loop, or an extra
allocation per file — sized to produce roughly **+5%, +20% and +50%** at 16 threads. Verify the injected
size independently (single-threaded, where noise is small) before running the sweep.

Report, per injected size and per thread count: **did the method report it, and how close was the reported
figure to the injected one?**

That gives the detection floor — the smallest effect this method can find reliably — which is the other
half of a gate threshold.

## 4. What to report

1. **The noise floor**, per thread count, from experiment 1, across two sittings.
2. **The detection floor**, from experiment 2.
3. **A recommended gate threshold** derived from those two numbers rather than from caution. The
   self-review guessed 2×; replace the guess.
4. **Whether experiment 1 explains Part 2.** State it plainly either way.
5. **If the noise floor at 16–32 threads turns out to be large**, say what that means for the concurrency
   gate — it may mean the gate must measure something more stable than wall time at high thread counts,
   and that is a finding worth more than the gate itself.

## 5. What not to do

- **Do not build either gate yet.** Their thresholds depend on this.
- **Do not tune the machine** — RFC 012 §9.3. The noise floor of an *ordinary* machine is the number we
  need, because that is where the gate will run.
- **Do not fix anything.** If experiment 2 makes a real cost visible, report it.
- **Do not skip the second sitting** for experiment 1. A noise floor measured once is a noise floor
  measured under one set of conditions, which is the error this whole thread has been about.

## 6. On cost

This is two sweeps plus a few patched variants, not a page regeneration — and per RFC 012 §9.1 the peer
benchmark is not part of it. **Timestamp each phase**, so if it turns out expensive we know where, rather
than discovering a three-hour job again.

**If experiment 1's first sitting shows a large noise floor, stop and report before running experiment 2** —
a detection floor measured against a noisy harness would tell us little, and the finding itself would be the
deliverable.

## 7. Committing and pushing

Only work that is yours and approved. Report first. **Name who approved anything you push.** Expect to push
nothing: probes and patches are throwaway, as in the last round.

Report to `.git-exclude/review-request/perf-method-calibration/README.md`, leading with §4.1 and §4.4.
