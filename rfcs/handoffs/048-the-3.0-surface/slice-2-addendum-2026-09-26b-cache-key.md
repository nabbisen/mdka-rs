# Addendum 2 — RFC 048 slice 2, 2026-09-26 · your patch is right; it needs a cache namespace bump

**Amends.** `slice-2-addendum-2026-09-26-cache.md`, whose §3 prescribed the wrong exclusion. Both addenda are frozen; this one supersedes §3 and §4 of the first.
**Answers.** Part 2 of `.git-exclude/review-request/048-slice-2-result-model/README.md` — **option 1**, with one addition.
**Baseline.** `3ad59c3`.
**Scope.** `.github/workflows/crates-package-gate.yaml`. Nothing else.

---

## 1. You were right and my prescription was wrong

`!target/package` does not fix it — you measured variant B and it fails with the same three errors. The
stale thing is the **compiled** `target/debug/**/*mdka*`, not the extraction. `target/package/` is rewritten
every run; my own clean-worktree check had already shown that and I did not follow the implication.

**Take option 1: your patch as written, the single exclusion.** Do **not** add `!target/package` — E passes
without it, and an exclusion with no demonstrated reason is the thing this whole addendum exists to remove.
You quoted that argument back at me correctly.

**Not option 3.** Keying on a source hash either keeps `restore-keys`, which restores the stale entry anyway,
or drops it and makes every source change a cold build — 78–113 s by your own numbers, against ~2 s for the
exclusion.

## 2. One thing the patch cannot work without, which a local emulation cannot show 🛑

```
run 36198816984:  Cache hit for: Linux-cargo-package-gate-0658c3…
run 36195410652:  Cache hit for: Linux-cargo-package-gate-0658c3…
```

**Exact key hits, and neither run has a `Cache saved with key` line.** `actions/cache` does not save when the
primary key hit exactly. The key is `hashFiles('**/Cargo.lock')`, which a source-only change cannot move, so
**the stale entry is frozen**: changing `path:` changes only what a *future save* would contain, and no save
will happen until a dependency changes.

And `restore-keys: ${{ runner.os }}-cargo-package-gate-` would pull that same entry back even if the key did
move.

**So bump the namespace on both:**

```yaml
key:          ${{ runner.os }}-cargo-package-gate-v2-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-cargo-package-gate-v2-
```

**Expect, and say in the report:** the first run after this is a cache **miss** and cold (~78–113 s); the
second should be a **hit** and green. A cold first run is the fix working, not failing.

## 3. Criteria

1. The exclusion of your patch, unchanged, **plus** the `-v2-` namespace on `key` and `restore-keys`.
2. **Two consecutive runs on `main`:** the first a miss, the second a hit — both green, both quoted with
   their cache lines. That is the runner demonstration the first addendum asked for, and the hit run is the
   one that proves a restored cache no longer carries the defect.
3. **The stale-source control.** Your local variants A/B/E already are it; put the table in the report and
   say it was local. **If the two runs in (2) are green, the runner form is satisfied** — a warm run that
   passes across the API change is the control. No scratch branches needed.
4. The cost as a number: the cold first run and the warm second, from the runs themselves. Your local ~2 s
   estimate stands as an estimate until then.
5. The comment block covers **both** exclusions — RFC 030's `registry/src` and this one — dated 2026-09-26,
   naming this slice, and **saying that `target/package` was measured and is not the cause**. That sentence
   is worth keeping: it stops the next reader re-deriving my mistake.
6. No change outside that workflow file.

## 4. On the process here

You were asked to bring this back rather than run the demonstrations, and that was right: the prescription
was wrong, and running three dispatches to demonstrate a fix that does not fix it would have cost time and
produced a confusing record.

**Nothing in Part 2 needed my permission to be correct** — the measurement settled it. What needed me was
the choice between your three options, and §2, which is visible only from the run logs.

## 5. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. **Name who approved
anything you push.** `git commit -F <file> -- <explicit paths>`.

Append to `.git-exclude/review-request/048-slice-2-result-model/README.md` as Part 3. **Slice 3 unblocks the
moment the gate is green on a cache hit.**
