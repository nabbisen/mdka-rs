# Handoff — `version.sh`'s stale-version check cries wolf

**Priority:** low, but before the next release. **Size:** small.
**Found by:** the dev team, during `3.1.0` prep; reported rather than worked around, which was right.

---

## The defect

`version.sh`'s post-update assertion (step 5, ~line 250) greps each touched file for
`"$OLD_VERSION"` and fails the bump if it finds one. At `3.1.0` it failed on a **correct** bump:

```
STALE VERSION: node/package-lock.json still contains "3.0.0"
Error: version bump to "3.1.0" is incomplete …
```

mdka's own `"version"` was correctly `3.1.0`. The three survivors, at lines 184, 1715 and 1803, were
`@inquirer/external-editor`, `fast-content-type-parse` and `mute-stream` — **third-party dependencies
that happened to be locked at `3.0.0`.** A lockfile records everyone's versions, so this recurs
whenever any transitive dependency sits at the version being left behind.

**Why it matters more than the noise:** this check exists to stop a half-applied bump shipping
silently. A guard that fails on correct input gets routed around, and then it is not a guard. This
one has now fired falsely at a release where a human had to inspect nine manifests by hand to
overrule it.

## The constraint — do not simply narrow it

The script's own comment states the intent plainly:

> this assertion catches *any* touched manifest that still carries the previous version string,
> **including locations added later that nobody remembered to teach this script about.**

That breadth **is** the feature. Scoping the whole check to a known list of fields would discard
exactly what it was built for. **Keep whole-file grep for every file where it works** — only
`*package-lock.json` has the ambiguity.

## What makes a lockfile exception provably complete

`version.sh` writes **exactly two** version fields in `node/package-lock.json` — the `3.1.0` diff
shows `node/package-lock.json | 2 +-`. The root `"version"` and the `"packages": {"": {…}}` entry.
So asserting **those two fields** for that one file is not a narrowing of coverage: it is the
complete set of what the script touches there.

How you read them is yours. `jq` is already used in this repo's workflows; `python3` is already used
by `.github/workflows/scripts/`. A line-number or first-N-lines heuristic is not acceptable — a
lockfile's shape is not ours to depend on.

## Criteria

1. A bump where a third-party dependency sits at the outgoing version **passes**. `3.0.0 → 3.1.0` is
   a ready-made case; reproduce it on a throwaway branch.
2. **The check still catches a real half-applied bump.** Deliberately revert one manifest's version
   after the script writes it, re-run the assertion, and show it **fails and names that file**. This
   is the criterion that matters — the fix is worthless if it only silences.
3. Whole-file grep unchanged for every file other than `*package-lock.json`; say so explicitly.
4. `sh version.sh --list` and `--dry-run` still behave. No release performed, no version left bumped
   in the tree.

## Report

Both outcomes from criteria 1 and 2, pasted. Say which mechanism you used to read the two fields and
why.

## Scope

`version.sh` only. No `src/`, no workflow, no manifest left modified. Push when green; no release.
