# RFC 014 — Release-time CI verification

**Status.** Implemented (2.1.7)
**Tracks.** M1 · Trustworthy baseline. Closes the release-path exposure left
open by the project's push-to-`main` merge policy.
**Touches.** `.github/workflows/release-executable.yaml`,
`.github/workflows/release-npm.yaml`, `.github/workflows/release-pypi.yaml`.
**Depends on.** RFC 001 (the `ci.yaml` workflow this guard queries).

## Summary

Add a `verify-ci` gate job to each of the three release workflows, so no
artifact publishes from a commit whose CI run did not conclude `success`. Day-to-day
pushes to `main` are unaffected.

## Motivation

**Merge policy, decided by the project owner on 2026-08-02: keep pushing
directly to `main`. No pull-request requirement, no branch protection.**

That is a reasonable call for a one-committer repository. Pull requests mostly
exist to coordinate multiple humans; with a single committer the recurring
friction (branch, PR, wait, merge, delete, pull — per RFC) buys enforcement that
is not needed. Detection is unaffected either way: CI runs on push and reports
within minutes. Recovery is a `git revert`, blocking nobody. The project has
shipped 86 tags this way without incident.

The consequence, accepted deliberately, is that **CI on `main` is advisory** — it
reports, it cannot block.

For `main` that is fine. For releases it is not:

- All three release workflows trigger on `release: created` and, verified
  2026-08-02, **none has any dependency on CI status**.
- Publication reaches users on crates.io, npm, and PyPI.
- crates.io versions can be **yanked but never replaced**. The project's own
  incident note (handoff bundle, Part G) already documents that cleanup path.

So the risk is asymmetric: low probability, high and partly irreversible cost,
and it is the only exposure in this area that reaches users rather than just the
maintainer.

Note this is **independent of pull requests**. Branch protection would not have
fixed it either — an admin bypass, or a tag placed on an older commit, sidesteps
it entirely. It is a release-path problem and wants a release-path fix.

## Goals

- No artifact publishes from a commit whose CI run is not `success`.
- Applies uniformly to all three release workflows.
- Zero added friction to normal pushes.
- Fails **closed**: absence of a CI run is a failure, never a pass.

## Non-goals

- Branch protection or a pull-request requirement. Explicitly considered and
  rejected by the project owner; see Alternatives.
- Changing what CI checks, or `ci.yaml` itself.
- Changing release triggers, packaging, archive layout, or publish steps.
- Artifact signing, provenance, or attestation. Different problem, later
  conversation if wanted.
- Protecting against a determined maintainer. This guards against accident, not
  against someone with admin rights who edits the workflow.

## Proposed design

A `verify-ci` job at the head of each release workflow. Every existing top-level
job gains `needs: [verify-ci]`.

The job resolves the CI run for the released commit and requires it to conclude
`success`, waiting if it is still in progress:

```yaml
  verify-ci:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    permissions:
      actions: read
      contents: read
    steps:
      - name: Require green CI on the released commit
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          run_id=$(gh run list --commit "${{ github.sha }}" \
                     --workflow ci.yaml --limit 1 --json databaseId \
                     --jq '.[0].databaseId // empty')
          if [ -z "$run_id" ]; then
            echo "::error::No CI run found for ${{ github.sha }}. Refusing to publish."
            exit 1
          fi
          gh run watch "$run_id" --exit-status
```

`gh run watch --exit-status` blocks until the run completes and exits non-zero
if it did not succeed, so a release created immediately after a push waits for
CI rather than failing spuriously. The explicit empty check handles the case
that matters most for safety — a tag on a commit CI never saw.

### Failure behaviour

| Situation | Outcome |
|---|---|
| CI concluded `success` | Publish proceeds |
| CI concluded `failure` / `cancelled` | Blocked |
| CI still running | Waits, then follows its result |
| No CI run for the commit | **Blocked** — fails closed |
| CI run exists but for a different workflow | Not matched; treated as no run → blocked |

### Permissions

Each workflow needs `actions: read` to query runs. Current top-level
permissions differ per file (`contents: write`, `contents: read`,
`id-token: write`), so scope the addition at **job level** as shown rather than
widening any workflow's top-level grant.

## Compatibility

None for library consumers. No API, output, packaging, or artifact-layout
change. The only behavioural change is that a release whose commit lacks green
CI now fails instead of publishing.

## Security considerations

Positive. The guard's one critical property is **failing closed** — a missing or
unmatched CI run must never be read as success. That is the difference between a
guard and decoration.

`secrets.GITHUB_TOKEN` with job-scoped `actions: read` is sufficient; no new
secret, no third-party action, and no widening of existing top-level
permissions.

## Testing and verification

The guard cannot be exercised end-to-end without cutting a release, and cutting
one to test it is not acceptable. Verify the query logic locally with `gh`
instead, against real commits:

| Commit | Expectation |
|---|---|
| `0c26bef` (CI green — RFC 001 Slice 3) | Resolves a run id; `--exit-status` succeeds |
| `a4e9009` (predates `ci.yaml`; no run exists) | Resolves empty → **must fail closed** |

The second case is the one that matters. A guard that passes when it finds
nothing is worse than no guard, because it looks like protection.

**Use the full 40-character SHA when reproducing this locally.**
`gh run list --commit` matches the head SHA exactly and does **not** do prefix
matching, so an abbreviated 7-character SHA returns an empty result —
indistinguishable in shape from the genuine fail-closed case, and easily
misread as a bug in the guard. The guard itself is unaffected, because
`${{ github.sha }}` is always the full 40-character form at runtime. This note
exists for whoever re-verifies the guard later.

*(Surfaced by the implementer during RFC 014 verification, 2026-08-02.)*

## Acceptance criteria

1. All three release workflows contain a `verify-ci` job.
2. Every existing top-level job in each workflow declares `needs: [verify-ci]`.
3. The guard fails when no CI run exists for the commit.
4. The guard waits for an in-progress run rather than failing immediately.
5. `actions: read` is granted at job level; no top-level permission widened.
6. Logic demonstrated locally against both commits in the table above.
7. No change to triggers, packaging, archive layout, or publish steps.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Release created before CI starts; `gh run list` finds nothing and blocks | A legitimate release fails and must be re-created | Accepted — failing closed is the point. Push, let CI start, then create the release. |
| `gh run watch` times out on a slow run | Release job fails | `timeout-minutes: 30` against CI's 20-minute per-job ceiling |
| Someone edits the guard away | Protection lost | Out of scope; guards accident, not intent |
| `ci.yaml` renamed later | `--workflow ci.yaml` silently matches nothing → blocks | Fails closed, so safe. Note the coupling in the workflow comment. |
| **CI run cancelled by a later push, making a good commit unreleasable** | Release blocked on a commit whose code is fine | Fails closed, so safe — but see the operational note below. Remedy: re-run CI for that commit (`gh run rerun <id>`), which reuses the run id, after which the guard passes. |
| `github.sha` on a `release` event does not resolve as expected for annotated tags | Guard blocks every release | Watched at first live release; see below. |

### Operational note — interaction with `ci.yaml`'s concurrency group

Identified during RFC 014 review, 2026-08-02. Neither RFC anticipated it.

`ci.yaml` (RFC 001) sets:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

Every push to `main` shares the group `CI-refs/heads/main`. So pushing commit B
while commit A's CI is still running **cancels A's run**. A cancelled run is not
`success`, so this guard will block a release cut from commit A — even though
A's code may be perfectly good.

The failure direction is safe (blocked, not published) and the likelihood is
low, because releases are normally cut from the tip of `main`, whose run is not
cancelled by anything after it. But the diagnosis is non-obvious at release
time: the commit looks fine and CI "ran."

**If a release blocks unexpectedly, check whether that commit's CI run was
cancelled rather than failed.** Re-running it resolves the block.

This is deliberately *not* fixed by weakening the guard. Treating `cancelled`
as acceptable would reintroduce exactly the hole this RFC closes.

### Untested assumption — first live release

The specific thing to watch the first time this runs for real: that
`${{ github.sha }}` on a `release: created` event resolves to the **commit** CI
ran on, and not to the annotated tag object. This project uses annotated tags
(`git cat-file -t 2.1.6` → `tag`). GitHub documents `GITHUB_SHA` for release
events as the last commit in the tagged release, so this is expected to work,
but it cannot be proven without cutting a release — and cutting one to test the
guard is explicitly out of scope.

If the first guarded release blocks with "No CI run found" on a commit whose CI
is known green, this assumption is the first thing to check.

## Alternatives considered

| Option | Assessment |
|---|---|
| **Branch protection + required PR** | Rejected by the project owner. Recurring friction per RFC; in a one-committer repo the benefit is mostly enforcement that is not needed. Would also not fix this exposure — admin bypass and tags on older commits both sidestep it. |
| **Manual discipline: check CI before tagging** | Status quo. No mechanism, and the failure is silent and irreversible when it happens. |
| **`workflow_run` trigger chaining** | Would restructure how releases trigger. Larger change to working, load-bearing workflows for no additional safety. |
| **Signing / provenance attestation** | Solves a different problem (artifact integrity, not artifact correctness). Worth a separate conversation; not a substitute. |

## Consequence for RFC 001

RFC 001 should be recorded as delivering an **advisory** gate on `main` — which
is what it delivers under the approved merge policy — with release-time
enforcement provided here. RFC 001's acceptance criteria are unchanged and
remain met; only the characterisation changes, so that the record does not claim
enforcement the project does not have.
