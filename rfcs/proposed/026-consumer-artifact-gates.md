# RFC 026 — Consumer-artifact verification gates

**Status.** Proposed
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P0 — this is the control, not the symptom
**Touches.** `.github/workflows/ci.yaml`, a new release-time job.
**Source.** Architect analysis of the 2026-08-31 audit — the root cause behind `S-01` and `D-08`.
**Prepared.** 2026-08-31

## Summary

Every CI job verifies the **source tree**. None verifies the **artifact a user
installs**. Add gates that consume each published artifact the way a consumer
does.

## The gap, measured

`ci.yaml` has four jobs. Here is what each actually exercises:

| Job | What it runs | What it proves |
|---|---|---|
| `rust` | `cargo fmt/clippy/test/build` | The workspace compiles and its own tests pass |
| `msrv` | `cargo check` at 1.88 | It compiles on the MSRV |
| `node` | `npm ci`, `npm run build`, `node test.js` | The **locally built** native module works |
| `python` | `maturin develop`, `pytest` | The **locally built** extension works |

Every row operates on a tree where the artefact was produced in place. Not one
installs a packaged artifact from outside the workspace.

**This is exactly why `S-01` survived twelve releases.** `npm run build` puts the
native binding in `node/` by construction, so `node test.js` can never observe
that a *published* package fails to resolve it. The suite was green throughout,
and it was green about a different question than the one that mattered.

The same hole produced `D-08`. `maturin develop` installs from source, so a
`py.typed` marker missing from the built **wheel** is invisible to `pytest`. The
audit found it by checking the published sdist. We could not have.

## Principle

> **A gate must consume the artifact the way a consumer does: built, packaged,
> installed from outside the workspace, then exercised.**

Anything short of that tests our intent rather than our output. This is the same
lesson as `verify-ci` in M1b — a control validated in an environment that does
not resemble the one it protects — generalised from the pipeline to the product.

## Required gates

### 1 · npm — owned by [RFC 020](./020-npm-distribution-repair.md)

`npm pack` → install the tarball in a clean directory outside the workspace →
`require()` → convert one string. Specified there because it blocks that fix;
listed here so the set is complete.

### 2 · PyPI — build a wheel, install it, import it

`maturin build` (**not** `develop`) → install the resulting wheel into a fresh
virtualenv → `import mdka` → convert one string.

Additionally assert **inside the installed package**:

- `py.typed` is present, if RFC 023 decides to ship it
- no Japanese text in any published surface, once RFC 007 lands

Both are claims that are currently checkable only in a wheel, and neither is
checkable by `maturin develop`.

### 3 · crates.io — verify the packaged crate, not the workspace

`cargo package` for each of the four crates, then build **from the packaged
source**. A crate can compile in-workspace and fail standalone through a missing
`include`, a path dependency, or a file absent from the package. Cheap, and the
only registry currently with no artifact-level check at all.

`cargo publish --dry-run` is a reasonable implementation and does most of this;
whichever is used, the build must come from the packaged output.

### 4 · Documentation examples — run them

`D-12` and `D-13` shipped examples that fail on their first line: a duplicate
`const`, an undeclared import, an unexported type. Nothing executes them.

Extract every runnable example from `docs/src/` and execute it. Examples that are
deliberately fragments must be marked so, and the marker is what excludes them —
not their absence from a list somewhere.

This is lower priority than 1–3 and may land with RFC 023, which is correcting
those examples anyway. **The correction without the gate is a one-time fix; they
will drift again.**

## Where each gate runs

| Gate | Trigger | Why |
|---|---|---|
| npm pack-install-require | every push | Cheap, and the defect it catches is total |
| PyPI wheel install | every push | Same |
| `cargo package` build | every push | Fast |
| Docs examples | every push | Fast |
| Per-platform resolution | release | Needs published packages; cannot run pre-publication |

The last one is a real residual gap and is stated rather than hidden: a gate can
prove the Linux package installs, but only publication can prove the macOS and
Windows packages resolve. Release-time verification of each published platform
package is required, and until it exists the risk is that **`S-01` could recur on
a platform CI does not run on**.

## What this does not do

It does not test conversion correctness — that is RFC 025. These gates ask only
"does the thing we shipped load and run at all". A gate that tried to do both
would be slower and would fail for reasons that obscure each other.

## Compatibility

CI only. No product change.

## Risks

| Risk | Mitigation |
|---|---|
| CI time grows | The gates are small. If any becomes slow, move it to release-time and say so. |
| A gate is green because it silently skipped | Every gate must **fail** at least once before being trusted. RFC 020 requires this for npm; the same applies here. Record the observed failure. |
| Clean-environment isolation is imperfect | Install outside the workspace, into a fresh temp dir or venv. Never rely on the workspace being absent from the path. |
| Platform coverage remains partial | Stated above rather than papered over. |

## Acceptance criteria

1. A wheel-install gate exists, imports `mdka` from an installed wheel in a fresh
   virtualenv, and asserts `py.typed` if RFC 023 ships it.
2. A `cargo package` gate builds each crate from packaged output.
3. A docs-example gate executes every runnable example.
4. **Each new gate has been observed failing** against a deliberately broken
   input, and that observation is recorded in the review request.
5. No gate reads from the workspace tree for the artifact it is verifying.
6. The residual platform gap is documented in `ROADMAP.md`.
