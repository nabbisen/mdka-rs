# RFC 015 — Release tooling completion

**Status.** Proposed
**Tracks.** M1b · Release tooling. Closes the four gaps the `2.1.7` release exposed.
**Touches.** A new crates.io publish workflow; a new tag-push workflow that
creates the GitHub release; `version.sh`; `cargo-publish.sh`; possibly
`publish` flags in `node/Cargo.toml` and `python/Cargo.toml`.
**Depends on.** RFC 014 (the `verify-ci` guard this extends).

## Summary

Four defects in release tooling surfaced during `2.1.7`. None blocked the
release in the end; none is fixed. They interact, so they are addressed
together: bring crates.io under the guard, automate GitHub release creation,
fix `version.sh`'s silent blind spot, and decide deliberately whether the
binding crates belong on crates.io.

## Motivation

### 1. crates.io is published manually, outside the guard

RFC 014 added `verify-ci` to the three GitHub release workflows. Verified at the
`2.1.7` release: **none of them publishes to crates.io.** That happens by running
`./cargo-publish.sh` on a maintainer's machine.

So the guard protects the tarballs, npm, and PyPI — and not the registry where a
bad version can be **yanked but never replaced**, and which is the primary
distribution channel for a Rust library.

RFC 014 was framed as closing "the release-path exposure." It closes three of
four paths. This is an architect error carried forward from that RFC.

### 2. GitHub release creation is manual

The `2.1.7` release was created with `gh release create` from a laptop. The
project owner has asked for this to become a CI job.

### 3. `version.sh` silently misses `[workspace.dependencies]`

`version.sh`'s awk rewrites only the first line matching
`^[[:space:]]*version[[:space:]]*=`. The root manifest's
`mdka = { version = "…", path = "." }` begins `mdka =`, so it is never updated.

It drifted undetected across three consecutive releases:

| Tag | `[package] version` | `[workspace.dependencies] mdka` |
|---|---|---|
| 2.1.3 | 2.1.3 | **2.1.0** |
| 2.1.4 | 2.1.4 | **2.1.0** |
| 2.1.5 | 2.1.5 | **2.1.0** |
| 2.1.6 | 2.1.6 | 2.1.6 — hand-corrected |
| 2.1.7 | 2.1.7 | 2.1.7 — hand-corrected again |

Twice caught by hand, only because a release handoff explicitly flagged it. The
next release without that reminder will drift again.

Consequence when it drifts: `mdka-cli` publishes depending on `mdka ^<old>`, so a
fresh `cargo install mdka-cli` can resolve an older `mdka` than the one released
alongside it.

### 4. The binding crates are published to crates.io by accident of scripting

`cargo-publish.sh` publishes `mdka`, `mdka-cli`, `mdka-node`, and `mdka-python`.
The latter two are napi/PyO3 `cdylib` binding crates that reach users through npm
and PyPI. They are on crates.io because the script iterates a hard-coded list,
not because anyone decided they should be.

## Goals

- No artifact reaches **any** registry — crates.io included — from a commit whose
  CI did not pass.
- Cutting a release is: push an annotated tag, then watch. No manual release
  creation, no manual publish script.
- A version bump cannot silently leave an old version string in any manifest.
- The binding crates' presence on crates.io is a recorded decision.

## Non-goals

- Changing what CI checks, or `ci.yaml`.
- Changing packaging, archive layout, or the artifacts produced. The macOS
  BSD-tar mitigation, the `cp -f README.md python/` staging, and the
  `napi create-npm-dirs` sequence all stay untouched.
- Removing the human from the crates.io decision. See the design below —
  approval moves from "remember to run a script" to an explicit approval gate.
- Signing or provenance attestation.
- Restructuring the workspace to `[workspace.package] version`.

## Proposed design

Four slices, independently reviewable. Slice 3 has no dependency on 1 or 2 and
can land first.

### Slice 1 — crates.io publishing as a guarded workflow

A new `.github/workflows/release-crates.yaml`, triggered on `release: created`
like its three siblings, with the same `verify-ci` job as its gate.

Two properties it must have:

**Trusted Publishing rather than a long-lived token.** crates.io supports OIDC
trusted publishing, the same mechanism `release-pypi.yaml` already uses for PyPI
(`id-token: write`, `environment: pypi`, established in commit `627bf93`). The
implementer must confirm the current crates.io mechanism and action against
crates.io's own documentation rather than assuming — this RFC specifies the
intent, not a version-pinned recipe.

**An approval gate, not just automation.** Publishing to crates.io is
irreversible. Today the safeguard is that a human runs the script. That
safeguard must not simply disappear. Use a GitHub Environment
(`environment: crates-io`) with a required reviewer, exactly as `release-pypi.yaml`
already does with its `pypi` environment. The result: execution is automated and
guarded, and a human still approves the irreversible step — but by an explicit
click rather than by remembering a command.

Publish order is `mdka` → `mdka-cli` → (binding crates, per Slice 4). `mdka-cli`
depends on `mdka`, so ordering is load-bearing. Modern cargo waits for index
propagation itself — observed working at `2.1.7` — so no manual sleep is needed,
but the job must not run the four publishes in parallel.

`cargo-publish.sh` is then either deleted or reduced to a documented
break-glass path. Do not leave two live publish routes.

### Slice 2 — automated GitHub release creation

A new workflow triggered on pushing a release tag:

```yaml
on:
  push:
    tags: ['[0-9]+.[0-9]+.[0-9]+']
```

It verifies CI is green on the tagged commit — the same guard logic, including
`GH_REPO` — then creates the GitHub release, which in turn fires the four
`release: created` workflows.

**Consequence to design around:** this changes the release trigger from a human
action to a tag push. The tag becomes the single point of intent. That is the
point of the change, and it is why Slices 1 and 2 belong in one RFC: the guard's
placement and the trigger's shape are one design, not two.

Release notes: `--generate-notes` matches current behaviour and the `2.1.6`/`2.1.7`
release bodies. Sourcing notes from `CHANGELOG.md`'s matching section is a
plausible improvement and is **out of scope** — mentioned so it is not invented
mid-implementation.

### Slice 3 — `version.sh` correctness

Two changes:

1. Update `[workspace.dependencies] mdka`'s version alongside the package
   versions.
2. **Add a post-update assertion**: after a bump, no manifest may still contain
   the previous version string. Fail loudly if one does.

The assertion matters more than the specific fix. It catches this entire class of
defect — any manifest location the script does not know about — rather than only
the one line that has bitten so far. A version bump that half-applies must fail,
not succeed quietly.

### Slice 4 — decide the binding crates

**Recommendation: keep publishing them, and record that as a decision.**

They are already on crates.io through `2.1.7`; stopping now leaves an
inconsistent history for no benefit. Publishing costs nothing, and holding the
names prevents another crate from taking `mdka-node` / `mdka-python`.

The alternative — `publish = false` in both manifests — is defensible for a
greenfield project but not worth the discontinuity here. Existing published
versions cannot be withdrawn in any case.

Whichever way it goes, record it in the RFC so the next person does not have to
re-derive it from a shell script's loop variable.

### Slice 4 — Decision (recorded 2026-08-03)

**Decision: keep publishing `mdka-node` and `mdka-python` to crates.io,
endorsing this RFC's own recommendation above.**

Reasoning, beyond what's already stated above: both crates have now been
published to crates.io twice with no incident — once manually at `2.1.7`
(`cargo-publish.sh`) and, implicitly, every prior release since the binding
crates were added. Nothing about the manual publish path surfaced a reason to
stop; the only real defect was that the *decision* to publish them had never
been made explicitly, not that publishing them was itself a problem. `Cargo.toml`
in `node/` and `python/` carries no `publish = false`, and this decision leaves
that unchanged: both remain published, now through the guarded
`release-crates.yaml` workflow (Slice 1) in the same run and the same order
(`mdka` → `mdka-cli` → `mdka-node` → `mdka-python`) as the manual script used.

No manifest change results from this decision — `publish = false` was the only
alternative action available, and it was not taken.

## Compatibility

None for library consumers. No API, output, or artifact-content change. The
change is to how artifacts are produced and published.

## Security considerations

Net improvement:

- Trusted Publishing removes a long-lived crates.io token from the release path.
- crates.io comes under the same fail-closed guard as the other registries.
- The environment approval gate preserves human authorisation for the
  irreversible step while removing the "did I remember to check CI?" failure
  mode.

Constraints carried from RFC 014: no third-party action beyond what the
repository already trusts; `actions: read` at job level; no `pull_request_target`;
no widening of top-level `permissions`.

## Testing and verification

**Apply RFC 014's hard-won lesson: verification must run in an environment
resembling the target, not merely execute the same commands.** The `2.1.7`
failure happened because the guard's logic was verified from inside a clone,
which silently supplied a precondition production lacks.

Concretely for this RFC:

- Any `gh`-based logic must be exercised from **outside** a git repository, with
  the failure case reproduced first and then shown to be fixed.
- The tag-push workflow cannot be fully proven without pushing a tag. Verify what
  can be verified in isolation, and state plainly what remains unproven until the
  next real release — do not let "it should work" read as "it was tested."
- `version.sh` changes are testable directly: run `--dry-run` against a scratch
  copy, confirm every manifest updates, and confirm the new assertion **fails**
  when a version string is deliberately left stale.

## Acceptance criteria

1. crates.io publishing runs in a workflow gated by `verify-ci`.
2. That workflow uses Trusted Publishing, or documents why a token was necessary.
3. An environment approval gate protects the crates.io publish step.
4. Pushing a release tag creates the GitHub release automatically, CI-gated.
5. Only one live publish route exists per registry.
6. `version.sh` updates `[workspace.dependencies]`.
7. `version.sh` fails when any manifest retains the previous version string.
8. The binding-crate decision is recorded in this RFC.
9. No change to packaging, archive layout, or artifact contents.
10. `ci.yaml` untouched.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Automated crates.io publish fires on an unintended tag | Irreversible publish | Environment approval gate (Slice 1); tag pattern restricted to `N.N.N` |
| Trusted Publishing not configured on crates.io before the workflow lands | Release fails at publish | Configure crates.io side first; keep `cargo-publish.sh` until the workflow is proven once |
| Tag-push workflow and `release: created` workflows race | Duplicate or missed runs | Release creation is the only trigger for the other four; the tag-push workflow does not publish anything itself |
| The full path cannot be proven before the next real release | An untested release path | Accepted and stated. Same class as RFC 014's first exercise — the mitigation is honest reporting, not false confidence |
| `version.sh` assertion produces false failures | Blocked releases | Assert against the previous version string only, in manifests only |

## Alternatives considered

| Option | Assessment |
|---|---|
| **Add the CI check to `cargo-publish.sh`** | Cheaper, but leaves publishing dependent on someone running a local script, and leaves the token in place. Solves the check, not the exposure. |
| **Publish crates.io from an existing workflow** | Conflates concerns; the pypi and npm workflows are already long. A separate workflow matches the existing one-registry-per-file structure. |
| **Fix only `version.sh`** | Considered and rejected by the project owner: it leaves the guard gap and the manual release creation open, and both would be exercised at the next release. |
| **Restructure to `[workspace.package] version`** | Would remove one drift source, but is a larger manifest change touching every crate, for a problem the Slice 3 assertion already catches generally. |
