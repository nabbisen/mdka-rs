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

---

## Correction record — the root cause, 2026-09-01

Implementation traced the cause into `@napi-rs/cli`'s source rather than
reasoning from the config's shape, and found it narrower than § Root cause
above describes. **That section's fault 2 is superseded by this one.**

### What § Root cause got wrong

It said `index.js` (unscoped) and `create-npm-dirs` (scoped) were two mechanisms
disagreeing, and that the fix was to reconcile them.

**They were not disagreeing.** `napi-rs` 3.x reads a **flat** `napi.packageName`
string; `node/package.json` carried it nested as `napi.package.name`, a shape
`readNapiConfig` never reads. It therefore fell back to the root `name` field —
`"mdka"`, unscoped — and **both** mechanisms produced unscoped names, in
agreement. The scoped `@mdka/lib-*` packages on the registry were residue from an
older, correct configuration, which is what made it look like a disagreement.

One silently-ignored config key, plus a registry preserving evidence of a
configuration that no longer existed.

The fix is the flat key:

```json
"napi": { "binaryName": "mdka", "packageName": "@mdka/lib", ... }
```

After which `napi build` and `napi create-npm-dirs` both emit `@mdka/lib-*` with
no CLI override.

**Why the distinction matters:** the original framing points an implementer at
reconciling two mechanisms — a fix for a problem that was not there, which would
have left the ignored key in place to resurface on the next regeneration.

### `napi pre-publish --no-gh-release` is required, not cosmetic

`napi pre-publish` defaults `--gh-release` to **true** and will try to create a
second GitHub release for a tag `create-release.yaml` has already released. It is
non-fatal — caught and logged — but there is no reason to invite it, and a future
maintainer seeing an unexplained flag will remove it. Confirmed against the CLI's
source that the flag suppresses the code path, not just the symptom.

### The unscoped name — resolved

§ 6.3 flagged `mdka-linux-x64-gnu` resolving with zero versions as unexplained.
It was **unpublished by someone on 2026-02-06**, and `npm owner ls` finds no
admin. Not merely unproven: active evidence against depending on it. Scoped
`@mdka/lib-*` was adopted on that evidence rather than on the default.

### Release scope

`2.2.1` carries this RFC alone. The fix is unprovable except by releasing, so it
ships without other changes travelling with it. The rest of M2b moves to `2.2.2`.

---

## Correction — where an artifact gate may live, 2026-09-01

Found at the `2.2.1` tag checkpoint, before any damage.

The gate landed in `.github/workflows/ci.yaml`, as this RFC's handoff §5 said
("runs on every push"). **`ci.yaml` is the workflow `verify-ci` keys on**, in
`create-release.yaml` and in all four publishers:

```sh
run_id=$(gh run list --commit "$SHA" --workflow ci.yaml --limit 1 ...)
gh run watch "$run_id" --exit-status
```

A failing gate makes the whole `ci.yaml` run conclude `failure`, so `verify-ci`
fails, so no release is created and nothing is dispatched. That produces a
circular dependency:

```
the gate cannot pass     until the platform packages are published
publishing requires      ci.yaml green
ci.yaml cannot be green  while the gate lives inside it
```

**The rule:**

> An artifact gate must not live in the workflow that `verify-ci` keys on.

Artifact gates verify what a release *produced*. `verify-ci` decides whether a
release may *proceed*. Nesting the first inside the second means a broken
artifact can never be repaired by a release — the gate blocks its own fix.

Each artifact gate gets its own workflow file. It still runs on every push and
still goes red when broken; only its file changes.

**Promotion is a separate, later decision.** Once a gate has been observed green
against a real release, it may deliberately be made release-blocking. That can
only happen after the observation, never before — a gate that has never passed
cannot be a precondition for the release that would first make it pass.

This was an architect error: the handoff specified the placement and the review
approved it. It would have recurred three more times through RFC 026.

## Correction — the gate was testing the wrong artifact, 2026-09-01

Found after `2.2.1` published successfully: the gate stayed **red** while
`npm install mdka@2.2.1` in a clean directory installed and converted
correctly.

**The gate could never pass.** It packed the local tarball and installed that,
per this RFC's handoff §5, which I wrote. But `optionalDependencies` are
injected by `napi pre-publish` **at release time** — a locally packed tarball
carries none, resolves no per-platform package, and always fails with
`Cannot find native binding`. It reported on an artifact that is not the one
published.

The instruction to install **outside the workspace** was right. The artifact
chosen was wrong, and the two are easy to conflate: installing a local tarball
outside the workspace *looks* like consuming a published package and is not.

**Corrected:** the gate installs the published package from the registry and
requires it. No build needed, so it is also far faster.

**The lag this introduces, stated rather than hidden:** the gate now reports on
the *last published* release, not the working tree. That is inherent — a
published artifact cannot be verified before it is published. What it buys is an
alarm that would have been red for all twelve broken 2.x releases, plus a weekly
schedule so a registry-side regression surfaces without waiting for a push.

**Pre-publication verification of the artifact is therefore still absent**, and
no gate can close it. The control that covers that gap is RFC 027's consumer
pass, performed after a release.
