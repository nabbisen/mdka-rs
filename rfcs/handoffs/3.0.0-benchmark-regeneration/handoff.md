# Developer Handoff — regenerate the performance page for `3.0.0`

**Source.** Owner, 2026-09-26: *"We had better get new benchmarkings. v3 may be slower than v2. We had better put description around why and the counter profits, too."*
**Method.** RFC 012 — `rfcs/done/012-benchmark-hardening.md`. That RFC made this page reproducible; this is the first use of it since.
**Milestone.** Unassigned; documentation and measurement. No release depends on it.
**Prepared.** 2026-09-26
**Baseline.** `eff0e52`, `3.0.0` published — 629 Rust (`cargo test --workspace`), 59 Node, 25 loader, 91 Python.
**Scope.** `docs/src/design/performance-characteristics.md` and whatever benchmark code the measurement needs. **No library change. If a measurement suggests one, report it — do not make it.**

---

## 1. Do not start from the conclusion 🛑

The request came with a hypothesis — *"v3 may be slower than v2"* — and asks for the explanation and the
counter-benefits. **Measure first, then write whatever the numbers say.**

**A first look does not support it.** Interleaved on one harness (the published CLIs, `--mode minimal`,
medians):

| Input | `2.5.1` | `2.9.0` | `3.0.0` |
|---|---|---|---|
| `scale_5m.html` (5 MB), 7 rounds | 69 ms | 71 ms | **72 ms** |
| `large.html` (1 MB), 11 rounds | 14 ms | 14 ms | **14 ms** |
| `deep_nest.html`, 11 rounds | 29 ms | 30 ms | **30 ms** |

That is coarse — process startup included, few rounds, one machine — so it is a **signal, not a result**.
But it is enough that **you must not write a regression narrative before you have a regression.** Publishing
an explanation for a slowdown that does not exist would be a false claim in the documentation, which is the
defect class this project keeps finding.

**If `3.0.0` is not slower, say that, with the numbers.** That is a better page than an apology.

## 2. The existing benchmarks cannot see the two places a regression could hide

This is the part worth getting right. The suite measures **single-document string conversion** — and that
path is exactly what `3.0` did not change. Two changes it did make are invisible to it:

1. **`FileOutcome` owns its `src`.** Bulk file conversion returned `Vec<(&P, Result<PathBuf, _>)>` — the key
   was **borrowed**. It now returns `Vec<FileOutcome>` with an owned `PathBuf`. **That is one allocation per
   file**, and no current benchmark converts files at all.
2. **Node rejects unknown option keys.** Measured during slice 2 at ~55 ns for one key and ~150 ns for
   three, nothing for calls without options, against ±100 ns noise. **Re-measure it here** on the published
   `3.0.0` rather than a local build, so the number in the page is the shipped one.

**Add a bulk-file-conversion benchmark** for (1), across a realistic file count (say 10, 100, 1000), and
compare `2.9.0` against `3.0.0`. If one allocation per file is invisible at 1000 files, say so — that is a
result, and it retires the question.

## 3. Method — RFC 012's, unchanged

- **One sitting, one quiet machine, interleaved.** Never compare a number taken today against one taken on a
  different day, machine or version. The page's existing text says this; keep it true.
- **Record the environment** and the exact commit, as the page does now.
- **The peer libraries are pinned `[dev-dependencies]`** and the page says *"regenerating this page never
  bumps them, so a stale claim never re-stales itself silently."* **Keep them pinned at the versions they
  are.** If you believe a bump is warranted, that is a separate decision — report it, do not take it.
- Every figure on the page is regenerated in the same sitting, or the page says which were not.

## 4. What the page must end up saying

1. **Which version it measures** — the note added in slice 3 says the figures are from `2.3.0` and have not
   been re-measured. That note is replaced by the new measurement, not kept beside it.
2. **`2.9.0` vs `3.0.0`**, as its own comparison, on the same harness in the same sitting. This is the
   question that was asked and the page has never answered it.
3. **What changed between them and why**, in one short section — but **written from the numbers.** If the
   difference is within noise, the honest sentence is that the surface changed and the conversion path did
   not, so nothing moved. If something did move, name the mechanism and what it bought:
   - the result model exists so a caller can tell *cannot fail* from *can*, and so Node's type stops
     declaring fields it never sets;
   - the removals took away eight names that could not affect output.
4. **The peer comparison**, regenerated, with its pinned versions restated.

## 5. Criteria

1. Every table regenerated in one sitting, environment and commit recorded.
2. A **`2.9.0` vs `3.0.0`** comparison, interleaved on one harness.
3. A **bulk file conversion** benchmark exists and is measured at both versions (§2.1).
4. Node's option-key check re-measured on the **published** `3.0.0` (§2.2).
5. The staleness note from slice 3 is gone, replaced by the real measurement.
6. **The narrative matches the numbers.** If there is no regression, the page says so; if there is, it names
   the mechanism and the trade. **No pre-written explanation survives contact with a null result.**
7. Peer versions unchanged, and stated.
8. No change to `src/`, `cli/`, `node/`, `python/` except benchmark code; test counts unchanged.

## 6. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. **Name who approved
anything you push.** `git commit -F <file> -- <explicit paths>`.

**`docs example gate` compiles the page's runnable blocks**, so a broken sample fails CI rather than
shipping.

Report to `.git-exclude/review-request/3.0.0-benchmark-regeneration/README.md`, leading with §5.2 and §5.6 —
the `2.9.0` vs `3.0.0` numbers and the sentence you wrote from them.
