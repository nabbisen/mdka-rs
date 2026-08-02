# Developer Handoff — RFC 014 · Release-time CI verification

**Governing RFC.** [RFC 014](../../done/014-release-time-ci-verification.md) — Implemented (2.1.7)
**Milestone.** M1 · Trustworthy baseline → `2.1.7`
**Position in M1.** After RFC 001 (which provides the `ci.yaml` this queries). Independent of 002/003/004 — can run in parallel with any of them.
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 014. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

---

## 1. Purpose

Make it impossible to publish an artifact from a commit whose CI did not pass.
Leave day-to-day pushes to `main` exactly as they are.

## 2. Background — read this before you start

**The project owner has decided to keep pushing directly to `main`.** No pull
requests, no branch protection. That is settled and is not up for
reinterpretation while implementing this.

The accepted consequence is that CI on `main` is **advisory** — it reports, it
cannot block. For a one-committer repository that is fine: recovery is a
`git revert`, and nobody else is blocked.

What is not fine is the release path. Verified 2026-08-02:

```
release-executable.yaml   on: release: [created]   — no CI dependency
release-npm.yaml          on: release: [created]   — no CI dependency
release-pypi.yaml         on: release: [created]   — no CI dependency
```

Nothing mechanically prevents tagging a commit with red or still-running CI and
publishing it to crates.io, npm, and PyPI. crates.io versions can be yanked but
never replaced.

This RFC closes that, and only that.

## 3. Change scope

| Path | Change |
|---|---|
| `.github/workflows/release-executable.yaml` | Add `verify-ci` job; add `needs:` to existing top-level jobs |
| `.github/workflows/release-npm.yaml` | Same |
| `.github/workflows/release-pypi.yaml` | Same |

## 4. Non-change scope — do not touch

- **`.github/workflows/ci.yaml`.** RFC 001 is complete and approved. Do not
  modify it, including "small improvements."
- **Any packaging, build, archive, or publish step** in the three release
  workflows. In particular, leave alone: the macOS BSD-tar mitigation in
  `release-executable.yaml`; the `cp -f README.md python/` staging steps; the
  `napi create-npm-dirs` / `napi artifacts` sequence; the archive-root layout.
  These are load-bearing and were each fixed in response to a real shipped bug.
- **Release triggers.** Do not add `workflow_dispatch`, do not change
  `release: [created]`.
- Top-level `permissions:` blocks. Add `actions: read` at **job level** only.
- `src/`, `cli/`, `node/`, `python/`, `tests/`, any manifest.
- `docs/` — RFC 003 owns it. `tests/utils/` — RFC 004 owns it.
- Japanese comments — RFC 007 and RFC 013 own them.
- Do **not** add branch protection, a PR requirement, or a `CODEOWNERS` file.
  That was explicitly rejected.

## 5. Required implementation

Add to each of the three release workflows:

```yaml
  verify-ci:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    permissions:
      actions: read
      contents: read
    steps:
      # Releases are cut from commits pushed straight to main, where CI is
      # advisory. This is the enforcement point: nothing publishes unless
      # ci.yaml concluded success on this exact commit. Coupled by name to
      # ci.yaml — if that file is renamed, this fails closed (blocks), which
      # is the safe direction.
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

Then add `needs: [verify-ci]` to **every existing top-level job** in each file.
Do not miss any — a job without `needs:` runs unguarded and defeats the whole
change. Per file, at time of writing:

| File | Jobs requiring `needs: [verify-ci]` |
|---|---|
| `release-executable.yaml` | all existing top-level jobs |
| `release-npm.yaml` | `build`, `publish` |
| `release-pypi.yaml` | `linux`, `musllinux`, `windows`, `macos`, `sdist`, `release` |

**Re-read each file and enumerate its jobs yourself** rather than trusting this
table — it is a convenience, not a specification. Report any discrepancy.

Where a job already has `needs:` (e.g. `release-npm.yaml`'s `publish` needs
`build`, and `release-pypi.yaml`'s `release` needs five jobs), **append** to the
existing list; do not replace it.

## 6. The one property that actually matters

**Fail closed.** If no CI run is found for the commit, the job must fail.

A guard that passes when it finds nothing is worse than no guard, because it
looks like protection while providing none. The explicit empty-string check
exists for exactly this. Do not "simplify" it away, and do not let a missing run
fall through to success.

## 7. Required verification

The guard cannot be exercised end-to-end without cutting a release, and cutting
one to test it is **not acceptable**. Verify the query logic locally with `gh`
against real commits:

| Commit | Expected | Why |
|---|---|---|
| `0c26bef` | Resolves a run id; `gh run watch --exit-status` succeeds | CI green — RFC 001 Slice 3 |
| `a4e9009` | Resolves empty → guard exits 1 | Predates `ci.yaml`; no run exists. **This is the important case.** |

Run the exact shell body from §5, substituting the SHA, and capture both
transcripts.

Also confirm by inspection, per file:

```
# every top-level job has needs: [verify-ci] (or verify-ci appended)
grep -n 'needs:' .github/workflows/release-*.yaml
# no top-level permissions block was widened
git diff .github/workflows/
```

## 8. Compatibility constraints

None for consumers. No API, output, packaging, or artifact-layout change.

`git diff` must show **only** the added `verify-ci` jobs and the added `needs:`
entries. Any other line in those three files changing is out of scope — if you
believe one is needed, stop and report.

## 9. Security constraints

- No new secret. `secrets.GITHUB_TOKEN` is sufficient.
- No third-party action. Use `gh`, which is preinstalled on GitHub runners.
- `actions: read` at job level only. Do not widen any top-level `permissions:`.
- Do not add `pull_request_target` or any trigger that runs untrusted code with
  repository credentials.

## 10. Prohibited shortcuts

- No `continue-on-error` on `verify-ci`.
- No treating a missing CI run as success.
- No skipping the guard for any job, any workflow, or any tag pattern.
- No modifying `ci.yaml`.
- No touching packaging or publish logic.
- No branch protection, PR requirement, or `CODEOWNERS`.

## 11. Known risks

| Risk | If it happens |
|---|---|
| Release created before CI starts → no run found → blocked | Working as designed. Push, let CI start, then create the release. Do not weaken the guard to avoid this. |
| `gh run watch` times out | `timeout-minutes: 30` against CI's 20-minute per-job ceiling. If it proves too tight, report — do not silently raise it. |
| A workflow has a job you did not notice | The whole change is defeated. Enumerate jobs per file explicitly; this is checked at review. |
| `gh run list` returns a run for a different branch or event | Report before proceeding. `--commit` should scope it correctly, but confirm against `0c26bef`. |

## 12. Required evidence

1. Local transcript for `0c26bef` — run id resolved, exit 0.
2. Local transcript for `a4e9009` — empty result, exit 1, error message shown.
3. `git diff .github/workflows/` — full diff, showing only additions.
4. Per-file job enumeration, showing every top-level job now has
   `needs: [verify-ci]`.
5. Confirmation that no top-level `permissions:` block changed.

## 13. Acceptance checklist

- [ ] `verify-ci` job present in all three release workflows
- [ ] Every top-level job in each declares `needs: [verify-ci]`
- [ ] Existing `needs:` lists appended to, not replaced
- [ ] Guard fails closed on a commit with no CI run — demonstrated
- [ ] Guard succeeds on a commit with green CI — demonstrated
- [ ] `actions: read` at job level; no top-level permission widened
- [ ] No change to triggers, packaging, archive layout, or publish steps
- [ ] `ci.yaml` untouched
- [ ] No file outside §3 modified

## 14. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 014 goals, by number)
3. Changed files — complete list
4. **Per-file job enumeration, showing none was missed**
5. **Fail-closed demonstration** (the `a4e9009` transcript)
6. Differences from RFC 014, if any, and why
7. Executed verification and results
8. Evidence per §12
9. Unresolved issues
10. Known limitations
11. Requested review focus

Items 4 and 5 are the substance. The YAML is trivial; whether it actually
blocks is the deliverable.

## 15. Evidence standard

Per the standing standard: if a captured transcript contains a count or total
that does not reconcile, say so explicitly — even when it does not change the
conclusion you were asked to reach.

## 16. Escalate rather than decide

Stop and raise it if you find: a release workflow job that cannot take `needs:`;
`gh run list --commit` returning runs you did not expect; any need to modify
`ci.yaml`; or any reason the guard cannot fail closed.
