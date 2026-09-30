# Developer Handoff — an allocation gate

**Authorised.** Owner, 2026-09-30, accepting `.git-exclude/review-request/perf-scope-correction/README.md` §6.
**Milestone.** Unassigned. Small.
**Prepared.** 2026-09-30
**Baseline.** `abbf19b`.
**Scope.** A test. **No benchmark, no timing, no workflow, no `src/` change.**

---

## 1. What this is, and what it deliberately is not

The scope correction cancelled every wall-time gate: zero performance defects in this project's history, a
fat-tailed noise floor, and a machine that cannot be kept quiet.

**Allocation is the exception.** It does not vary with load, scheduling or machine — `+165 B/file` came out
byte-identical across three sittings that disagreed about everything else. It is the one performance
property this project has ever measured reliably, and it is the one thing `3.0.0` actually changed.

**So: a unit test, running in milliseconds, in `cargo test`. Not a benchmark.** If this grows past a
single small file, it has gone wrong.

## 2. What to assert

Two workloads, both tiny:

1. **String conversion** — one fixed document, `html_to_markdown_with`, `Balanced`.
2. **Bulk file conversion** — a handful of files through `html_files_to_markdown`. This is the path
   `3.0.0` changed and the one no test covers.

For each: total bytes allocated, against a **committed baseline number**, with a tolerance.

**Golden-file discipline, as `mode_identity`'s P3 already uses:** changing a baseline is a decision made in
the same commit as the change that moves it, with a stated reason — never a way to make the test pass.

## 3. The tolerance is the whole design problem 🛑

**Measure it; do not take my number.** From the `3.0.0` round, run-to-run variation looked like **±0.026%**
and the change worth catching was **+0.12%** — a margin of under 5×. A tolerance chosen carelessly either
fires constantly or catches nothing.

- **Measure the actual run-to-run spread** for each workload, over enough runs to see the tail, and set the
  tolerance from it. Report both numbers.
- **Check whether the string workload is exactly deterministic.** It has no file pre-pass and no `HashMap`
  over paths, so it may allocate identically every time — in which case its tolerance is **zero**, which is
  a far better gate than any percentage.
- If the bulk workload's variance cannot be separated from the signal, **say so and gate only the string
  one.** A gate that cannot distinguish is worse than no gate, because it launders inattention as coverage.

## 4. Prove it would have caught the real change 🛑

The only validation that matters: **run the finished gate against `2.9.0` and against `3.0.0`.**

- Against `3.0.0`'s baseline it passes.
- Against `2.9.0`'s code it must **fail** on the bulk workload — that is the `+165 B/file` this gate exists
  for.

If it cannot tell them apart, the tolerance is too loose and §3 needs redoing. **Paste both outcomes.**

## 5. Mechanics

- `#[global_allocator]` affects a whole test binary, so this needs **its own file in `tests/`**.
- The counting allocator lives in `benches/alloc_counter.rs`. Reaching it from `tests/` means a
  `#[path = "../benches/alloc_counter.rs"]` include — the same `examples/`→`benches/` coupling this project
  already flagged and did not fix. **Use the include and note it**; moving the allocator is a separate
  decision and not worth making here.
- Keep the fixtures inline or reuse an existing corpus file. Do not add new test data.

## 6. Criteria

1. One test file, running in `cargo test`, in milliseconds.
2. Baselines committed, with the measured spread and the chosen tolerance recorded **in the file's header**,
   so the next reader knows why the number is what it is.
3. §4's two runs, pasted.
4. If the bulk workload is ungateable, that conclusion with its evidence — an acceptable outcome.
5. Test counts reported with their commands; workflows green.
6. Nothing added to `src/`, no workflow change, no benchmark.

## 7. Committing and pushing

Only work that is yours and approved. Report first. **Name who approved anything you push.**
`git commit -F <file> -- <explicit paths>`.

**If this turns out to want more than one small file, stop and say so** rather than growing it. The point of
this slice is that it is the cheap remainder of a large idea that was cut.

Report to `.git-exclude/review-request/allocation-gate/README.md`, leading with §4.
