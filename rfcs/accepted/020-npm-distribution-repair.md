# RFC 020 — npm distribution repair and a published-artifact install gate

**Status.** Accepted 2026-08-31 — implementer may start
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P0 — the highest-impact item on the roadmap
**Touches.** `.github/workflows/release-npm.yaml`, `node/package.json`, a new CI job.
**Source.** External audit 2026-08-31, `S-01` (Critical). Reproduced independently.
**Prepared.** 2026-08-31

## Summary

`npm install mdka` has produced an unusable package for the entire 2.x line —
**twelve releases across roughly four months**. Repair the publish path, and add
a gate that consumes the published artifact the way a user does.

## The defect, reproduced

```
$ npm install mdka@2.2.0        # succeeds
$ node -e "require('mdka')"
Error: Cannot find native binding.
  [cause]: Error: Cannot find module 'mdka-linux-x64-gnu'
```

`mdka@1.6.9` was the last working release.

## Root cause — two independent faults

Either alone breaks the package. Both must be fixed.

**1 · `napi prepublish` is never run.** `release-npm.yaml` runs
`napi create-npm-dirs`, then `napi artifacts`, then `npm publish`. The missing
step is the one that publishes the per-platform packages **and** injects
`optionalDependencies` into the main manifest. Verified: published `mdka@2.2.0`
carries no `optionalDependencies` at all, and `@mdka/lib-linux-x64-gnu` has no
version above `1.6.9`.

**2 · The platform package names disagree with the loader.** The generated
`node/index.js` requires **unscoped** `mdka-linux-x64-gnu` — the file contains
zero occurrences of `@mdka`. But `napi.package.name` is `@mdka/lib`, so
`create-npm-dirs` produces **scoped** `@mdka/lib-*` directories. Even with fault
1 fixed, the loader would look for names that were never published.

## The naming decision — resolve empirically, do not assume

Registry state, checked 2026-08-31:

| Name | Status |
|---|---|
| `@mdka/lib-linux-x64-gnu` | exists, versions up to `1.6.9`, none in 2.x |
| `mdka-linux-x64-gnu` | **name resolves, zero versions published** |

The unscoped name existing with no versions is unexplained and **must be
investigated before it is chosen**. If we do not own it, publishing under it is
impossible and depending on it is a supply-chain hazard — an unowned name a
third party could later claim is strictly worse than a broken install.

**Default to the scoped `@mdka/lib-*` names**, which have publication history and
demonstrable ownership. Adopt unscoped only on positive proof of ownership.

Whichever is chosen, **three things must agree**: what `index.js` requires, what
`create-npm-dirs` generates, and what is actually published. Determine what
napi-rs 3.8.6 generates by running it, not by reading its documentation — the
current mismatch exists because a config value and a generated file disagreed
silently.

## Design

1. **The gate lands first, as its own commit**, and is observed **failing**
   against the current broken package.
2. Repair the publish path.
3. Observe the same gate pass.
4. Ship `2.2.1`.
5. `npm deprecate mdka@">=2.0.2 <2.2.1"` with a message naming the fix, so the
   registry explains the breakage to anyone pinned to a broken version.

### The gate

A CI job that, in a clean directory outside the workspace:

```
npm pack           →  install the resulting tarball  →  require() it  →  convert one string
```

**It must consume the packed tarball, not the source tree.** A test that runs
against `node/` passes today and proves nothing; that is precisely how this
shipped twelve times. The job runs on every push, not only at release.

## Why the existing checks did not catch this

`release-npm.yaml` has a `verify-ci` guard, and CI was green for all twelve
releases. Every check validated **what we wrote**. None validated **what a user
receives**. `node test.js` runs against the local build, where the native module
is present by construction.

This is the same failure mode as `verify-ci` in M1b — a control tested in an
environment that does not resemble the one it protects — one layer further out.
There it was the pipeline; here it is the artifact.

## Scope boundary

Not in scope: the `engines` floor (`S-09`, unconfirmed), the PyPI wheel matrix
(`S-06`), or the `index.js` drift check (`R-02`, now largely subsumed since
`version.sh` maintains that file). Note only that R-02's own recommended fix —
extending the drift check to `index.js` — **would also have caught this**.

## Compatibility

`2.2.1` is a patch. No API changes. For anyone on 2.x the package goes from
non-functional to functional, which cannot break a working consumer because
there are none.

## Risks

| Risk | Mitigation |
|---|---|
| The chosen name is not ours | Verify ownership with `npm owner ls` before publishing. Stop and raise if unclear. |
| `napi prepublish` needs a token scope the workflow lacks | It publishes to npm, so it needs `NPM_TOKEN` with publish rights — already present. Confirm before assuming. |
| Fixing publication but not naming | The gate catches it: `require()` fails. This is why the gate lands first. |
| macOS/Windows still broken while Linux works | The gate runs on one platform only. Publication is per-platform, so verify each published package resolves, even if `require()` is exercised on Linux alone. |

## Acceptance criteria

1. The install gate exists, and its failure against the pre-fix package is
   **recorded in the review request** — not merely asserted.
2. `npm install mdka@<version> && node -e "require('mdka')"` succeeds in a clean
   directory.
3. `optionalDependencies` is present in the published manifest.
4. A platform package for each published target exists at the release version.
5. `index.js`, the generated dirs, and the registry all use the same naming.
6. The broken range is deprecated on npm.
7. Ownership of the chosen name is verified and recorded.
