# RFC 019 — Automated release creation via explicit dispatch

**Status.** Proposed
**Tracks.** M4 · Release automation
**Supersedes.** [RFC 015](../done/015-release-tooling-completion.md) Slice 2, withdrawn 2026-08-11
**Touches.** `.github/workflows/create-release.yaml` (currently untracked), and the `on:` block of the four publishing workflows.
**Prepared.** 2026-08-12

## Summary

Cut a release by pushing a tag. `create-release.yaml` creates the GitHub Release
and then **explicitly dispatches** the four publishing workflows, rather than
relying on the `release: created` event to fan out.

## Why RFC 015 withdrew this, and why that reasoning was incomplete

RFC 015 Slice 2 was withdrawn for a correct reason:

> GitHub suppresses workflow-triggered events, so the release would be created
> and the four publishing workflows would never fire — a release that appears to
> succeed and publishes nothing.

That much still holds. Verified against the current documentation on 2026-08-12:

> "Events triggered by the `GITHUB_TOKEN` will not create a new workflow run,
> with the following exceptions:" — `workflow_dispatch` and `repository_dispatch`
> always create runs; `pull_request` with `opened`/`synchronize`/`reopened` runs
> in an approval-required state.

`release: created` is not on that list.

**But the withdrawal then enumerated the escape routes as: a PAT, a GitHub App,
or a restructure of four working publishing workflows** — and rejected the slice
on the cost of those three.

**That enumeration was wrong. It omitted the exception the same rule grants.**
`workflow_dispatch` fires from `GITHUB_TOKEN` by design. Dispatching the
publishers explicitly needs no new identity, no secret, no expiry to track, and
no restructure — only a `workflow_dispatch:` trigger added beside each existing
`release:` trigger, and one step in `create-release.yaml`.

The slice was rejected on a cost that was never necessary. This RFC corrects
that.

## Why not the alternatives

| Route | Assessment |
|---|---|
| **PAT** | Rejected in RFC 015 and still rejected. Expires within a year and fails silently when it does — the worst failure mode for something exercised a few times a year. |
| **GitHub App** | Works, no expiry, but is a standing credential and an installation to administer, for one saved command per release. |
| **Reusable workflows** (`workflow_call`) | **Actively harmful here.** The crates.io and PyPI Trusted Publisher claims are keyed to the *workflow filename*. Called as a reusable workflow, the OIDC subject reflects the calling workflow, and the claim fails. This would break publishing that currently works. |
| **Explicit dispatch** | **Chosen.** Additive, no credential, preserves every existing trigger path and every OIDC claim. |

## Design

Keep both triggers on each publisher:

```yaml
on:
  release:
    types: [created]
  workflow_dispatch:
```

`create-release.yaml`, after creating the release, dispatches each publisher at
the release tag.

**The manual path is unchanged.** Creating a Release by hand still fires all four
via `release: created`, exactly as it does today. This RFC adds a second entry
point; it removes none.

### Why the publishers are already compatible

Checked, not assumed. None of the four reads `github.event.release.*` — the
payload that would be empty under dispatch:

| Workflow | Derives the version from |
|---|---|
| `release-npm.yaml` | `TAG: ${{ github.ref_name }}` |
| `release-executable.yaml` | `TAG: ${{ github.ref_name }}`, and `gh release upload ${{ github.ref_name }}` |
| `release-crates.yaml` | the checked-out ref; `verify-ci` uses `git rev-parse HEAD` |
| `release-pypi.yaml` | the checked-out ref |

`github.ref_name` resolves to the tag when dispatched with `--ref <tag>`, so all
four behave identically under either trigger.

`release-pypi.yaml:256` already carries
`if: startsWith(github.ref, 'refs/tags/') || github.event_name == 'workflow_dispatch'`
— a condition whose dispatch branch is currently unreachable, because the
workflow has no `workflow_dispatch:` trigger. Adding the trigger activates
intent that is already written.

## What this does not fix

Dispatch is **fire-and-forget**. `create-release.yaml` will go green once the
four are dispatched, whether or not they succeed. A publish failure is still
visible only in the Actions tab.

That is no worse than today — a manually created release fans out the same way,
with no aggregate status — but it means "green create-release" must not be read
as "released successfully." Out of scope here; a follow-up could have
`create-release.yaml` wait on the four and report.

## Compatibility

Additive. No existing trigger, credential, or Trusted Publisher registration
changes. The only behavioural change is that pushing a release tag now does
something it previously did not.

## Risks

| Risk | Mitigation |
|---|---|
| A publisher is missed when adding `workflow_dispatch:` | It silently never runs. The dry run (below) must observe **all four** runs appear, not three. |
| A dispatched publisher behaves differently from the release-triggered path | Verified above that all four derive the tag from `github.ref_name` or the checked-out ref. The dry run confirms empirically. |
| Landing `create-release.yaml` sweeps in unrelated untracked files | It has been deliberately untracked for weeks. Stage by explicit path. |
| Double-firing publishes twice | Cannot occur on the automated path: the release is created by `GITHUB_TOKEN`, so no `release: created` event exists to race the dispatch. |
| A future maintainer "tidies" the publishers into reusable workflows | Recorded above and in the handoff: this breaks the OIDC claims. |

## Verification — required before a real release depends on it

**Standing rule, earned three times this milestone: prove the automation before a
release needs it.** The `verify-ci` bug and the OIDC dry run both cost a release
or nearly did.

A pre-release tag will not do — `create-release.yaml`'s filter is
`'[0-9]+.[0-9]+.[0-9]+'`, which deliberately excludes `-rc` and similar. The dry
run therefore needs a temporary filter extension, detailed in the handoff.

The observable that matters is **four runs appearing**. The publishers are
expected to *fail* at their publish step during the dry run, because the version
in the manifests is 2.1.8 and 2.1.8 is already published to all four registries.
That failure is the proof they were triggered.

## Acceptance criteria

1. Pushing a release tag creates the GitHub Release and starts all four publishers.
2. Creating a Release by hand still starts all four, unchanged.
3. No PAT, App, or new secret is introduced.
4. All four Trusted Publisher registrations continue to work.
5. The dry run observed four runs starting, evidenced.
6. `create-release.yaml` is tracked, and nothing else was swept in with it.
