# Developer Handoff — RFC 019 · Release creation via explicit dispatch

**Governing RFC.** [RFC 019](../../proposed/019-release-creation-via-dispatch.md)
**Supersedes.** [RFC 015](../../done/015-release-tooling-completion.md) Slice 2, withdrawn
**Milestone.** M4 · Release automation
**Prepared.** 2026-08-12

---

## 0. ⚠ Two inversions of standing instructions — read first

**1 · `create-release.yaml` now gets committed.**

Every handoff since RFC 015 has warned you never to stage
`.github/workflows/create-release.yaml`. **That warning is lifted for this slice
only.** Landing that file is the point of the work.

Stage it **by explicit path**. Do not `git add -A`, and do not assume it is the
only untracked file — check `git status` first and stage nothing else.

**2 · This is blocked until `main` is green.**

`main` is currently red on the `node` job (RFC 005 Slice B1). Do not start the
dry run in §4 until that is fixed and CI is green, because the dry run reads CI
status on the tagged commit.

## 1. Purpose

Make `git push` of a release tag cut the release: create the GitHub Release, then
start all four publishing workflows.

Today only a **human-created** Release starts them. A Release created by a
workflow using `GITHUB_TOKEN` starts nothing — GitHub suppresses those events to
prevent recursion. `workflow_dispatch` is an explicit exception to that
suppression, so we dispatch the four deliberately instead of relying on fan-out.

## 2. Change scope

| Path | Change |
|---|---|
| `.github/workflows/create-release.yaml` | Add `actions: write`; add a dispatch step; **commit the file** |
| `.github/workflows/release-crates.yaml` | Add `workflow_dispatch:` to `on:` |
| `.github/workflows/release-npm.yaml` | Same |
| `.github/workflows/release-pypi.yaml` | Same |
| `.github/workflows/release-executable.yaml` | Same |

Nothing else. No source, no tests, no docs.

## 3. The changes

### 3.1 Each publisher — add the trigger, keep the existing one

```yaml
on:
  release:
    types: [created]
  workflow_dispatch:
```

**Keep `release: created`.** The manual path must continue to work unchanged —
that is the fallback if anything here misbehaves, and it is how every release to
date has been cut.

`release-pypi.yaml:256` already has
`if: … || github.event_name == 'workflow_dispatch'`. Adding the trigger makes
that reachable; leave the condition alone.

### 3.2 `create-release.yaml` — permission

The `create-release` job currently has:

```yaml
permissions:
  contents: write
```

`gh workflow run` calls the Actions API and needs `actions: write`:

```yaml
permissions:
  contents: write
  actions: write
```

**Without this the dispatch step fails with a 403** — and since it runs after the
release is created, you would get exactly the silent half-release this RFC exists
to prevent. Do not skip it.

### 3.3 `create-release.yaml` — the dispatch step

After the existing `gh release create` step:

```yaml
      - name: Start the publishing workflows
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          for wf in release-crates release-npm release-pypi release-executable; do
            echo "Dispatching ${wf}.yaml at ${{ github.ref_name }}"
            gh workflow run "${wf}.yaml" --ref "${{ github.ref_name }}"
          done
```

Echo each dispatch. When this fails in a year's time, the log naming which of the
four was reached is the whole diagnosis.

**Do not** make the loop tolerate failures (`|| true`, `continue`). If a dispatch
fails, the step must go red. A partial release that reports success is the
failure mode being designed out.

## 4. Required verification — the dry run

**Do not skip this and do not substitute reasoning for it.** Three times this
milestone, release automation that looked correct was not: the `verify-ci`
missing-checkout bug, the OIDC claim keyed to filename, and the `node_modules`
version bug. The rule earned from those: **prove the automation before a release
depends on it.**

### 4.1 Preconditions — check all three

1. `main` is green.
2. Manifest versions are **2.1.8** in all four `Cargo.toml` files, and 2.1.8 is
   already published on crates.io, npm and PyPI. Verify, do not assume.
3. No version bump has been started.

**Why precondition 2 is load-bearing:** the dry run really does dispatch the real
publishers. They are safe *only* because every version they would publish already
exists in its registry, so each fails harmlessly with "already exists." **If you
run this after a version bump, it will publish for real.**

If you cannot satisfy precondition 2, stop and raise it.

### 4.2 Temporarily widen the tag filter

`create-release.yaml` matches `'[0-9]+.[0-9]+.[0-9]+'`, which deliberately
excludes pre-release tags. Add a second pattern **for the dry run only**:

```yaml
on:
  push:
    tags:
      - '[0-9]+.[0-9]+.[0-9]+'
      - '[0-9]+.[0-9]+.[0-9]+-test.[0-9]+'   # TEMPORARY — remove after dry run
```

### 4.3 Run it

1. Commit and push §3 plus the temporary filter to `main`. Wait for CI green.
2. Tag the **pushed tip** `2.1.8-test.1` and push the tag.
   (Tag the tip, not an intermediate commit — GitHub only runs CI against a
   push's tip, so an intermediate commit has no run for `verify-ci` to find.)

### 4.4 What to observe

**The observable that matters is four runs appearing.**

```
gh run list --limit 12
```

Expected:

- `create-release.yaml` — runs, creates a pre-release for `2.1.8-test.1`,
  dispatch step green
- **Four new runs**, one per publisher, all started by `workflow_dispatch`

The publishers are **expected to fail** at their publish step, because 2.1.8 is
already published. **That failure is the proof they were triggered.** A run that
starts and fails on "already exists" is a pass for this test. A run that never
appears is the failure.

Count them. Four, not three — a missed `workflow_dispatch:` on one publisher
looks exactly like success if you only check that "the dispatch step went green."

### 4.5 Clean up — all four steps

1. Delete the GitHub release for `2.1.8-test.1`
2. Delete the tag, local and remote
3. Remove the temporary `-test` filter line
4. Commit and push the removal, and confirm `main` is green afterwards

Leaving the `-test` pattern in place is the kind of debris that looks harmless
and then matches something real. Confirm in your review request that it is gone.

## 5. Non-change scope

- **Do not** convert any publisher to a reusable workflow (`workflow_call`). The
  crates.io and PyPI Trusted Publisher claims are keyed to the workflow
  **filename**; called that way, the OIDC subject reflects the *caller* and the
  claim fails. This would break publishing that currently works.
- **Do not** introduce a PAT, GitHub App, or any new secret. Needing one means
  the design is wrong — stop and raise it.
- **Do not** remove `release: created` from any publisher.
- **Do not** touch publisher logic — only the `on:` block.
- **Do not** touch `version.sh`, `cargo-publish.sh`, source, tests, or docs.

## 6. Prohibited shortcuts

- Do not skip the dry run.
- Do not run the dry run after a version bump.
- Do not make the dispatch loop swallow errors.
- Do not `git add -A`.
- Do not report "dispatch step green" as evidence the publishers ran. Only four
  observed runs count.

## 7. Known risks

| Risk | If it happens |
|---|---|
| Dispatch 403s | You forgot `actions: write` (§3.2). |
| Only three runs appear | One publisher is missing `workflow_dispatch:`. Name which, then fix. |
| A publisher fails on something other than "already exists" | **Report it.** It means the dispatch path differs from the release-triggered path in a way I did not find. |
| A publisher actually publishes something | Stop immediately and raise it. Precondition 2 was violated. |
| `create-release.yaml` fails on `gh release create` because the release exists | Expected if you re-run the dry run without cleaning up. Do §4.5 first. |

## 8. Required evidence

1. `git status` before staging, showing what was and was not staged.
2. The `on:` block of all four publishers after the change.
3. The `permissions:` block and dispatch step of `create-release.yaml`.
4. `gh run list` output from the dry run showing **all five** runs
   (`create-release` plus four publishers) with their triggering event.
5. The failure reason from at least one dispatched publisher, showing it failed
   on an already-published version rather than on a configuration error.
6. Proof the temporary `-test` filter, the test tag, and the test release are all
   removed, and `main` is green.

## 9. Acceptance checklist

- [ ] `workflow_dispatch:` added to all four publishers, `release: created` kept
- [ ] `actions: write` added to the `create-release` job
- [ ] Dispatch step added, echoes each workflow, does not swallow errors
- [ ] `create-release.yaml` committed, staged by explicit path, nothing else swept in
- [ ] Dry run performed with manifests at an already-published version
- [ ] **Four** publisher runs observed
- [ ] Temporary filter, test tag and test release removed
- [ ] `main` green at the end

## 10. Required review-request format

Standard eleven parts. The substance:

4. **The dry-run evidence** — the run list, and which event triggered each
5. **Anything that behaved differently from this handoff's prediction**, however
   small — this document is a prediction about GitHub's behaviour, and predictions
   about release machinery have been wrong three times this milestone

## 11. Escalate rather than decide

Stop and raise it if: a dispatch needs a credential other than `GITHUB_TOKEN`; a
publisher behaves differently under dispatch than under `release: created`; any
publisher actually publishes during the dry run; or the dry run cannot be made
safe.

## 12. After this lands

RFC 019 moves to `rfcs/done/`, and RFC 015's Slice 2 withdrawal gets a forward
pointer noting it was superseded.

The remaining gap, out of scope here and noted in RFC 019: dispatch is
fire-and-forget, so a green `create-release.yaml` does not mean the release
published. Worth a follow-up that waits on the four and reports an aggregate
status.
