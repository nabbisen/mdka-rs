# Developer Handoff — RFC 015 · Release tooling completion

**Governing RFC.** [RFC 015](../../done/015-release-tooling-completion.md) — Proposed
**Milestone.** M1b · Release tooling. No release of its own.
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 015. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

---

## 1. Purpose

Close the four release-tooling defects the `2.1.7` release exposed, so the next
release is "push an annotated tag, then watch" and nothing publishes anywhere
from a commit whose CI did not pass.

## 2. Background

You lived through all four of these. Briefly, for the record:

1. **crates.io publishes outside the guard.** RFC 014's `verify-ci` protects the
   three GitHub workflows. crates.io publishing is `./cargo-publish.sh` on a
   laptop. It is the registry where a version can be yanked but never replaced.
2. **GitHub release creation is manual.** The owner has asked for a CI job.
3. **`version.sh` silently misses `[workspace.dependencies]`.** It drifted across
   2.1.3–2.1.5 undetected, and was caught by hand at 2.1.6 and again at 2.1.7 —
   both times only because a handoff explicitly flagged it.
4. **The binding crates reach crates.io by accident of scripting**, not decision.

## 3. Change scope

| Path | Change |
|---|---|
| `.github/workflows/release-crates.yaml` | New — Slice 1 |
| `.github/workflows/` (tag-push release creator) | New — Slice 2 |
| `version.sh` | Slice 3 |
| `cargo-publish.sh` | Removed or reduced to break-glass — Slice 1 |
| `node/Cargo.toml`, `python/Cargo.toml` | Only if Slice 4 decides to stop publishing |
| `rfcs/done/015-release-tooling-completion.md` | Record the Slice 4 decision |

## 4. Non-change scope — do not touch

- **`ci.yaml`.** Untouched, entirely.
- **Any packaging, build, or archive step** in the existing three release
  workflows. Specifically leave: the macOS BSD-tar mitigation; the
  `cp -f README.md python/` staging; the `napi create-npm-dirs` / `napi artifacts`
  sequence; the archive-root layout. Each was a fix for a real shipped bug.
- **The existing `verify-ci` jobs.** They work now. Copy the pattern; do not
  refactor them, and do not "extract shared logic" across workflows.
- `src/`, `cli/`, `node/src`, `python/src`, `tests/`, `docs/`, any manifest
  version.
- Japanese comments — RFC 007 and RFC 013 own them.
- Release notes sourced from `CHANGELOG.md`. Explicitly out of scope; keep
  `--generate-notes`.

## 5. Slice order

**Slice 3 first.** It is independent, immediately testable, and fixes the defect
with a proven history of silent damage. Land it before the workflow work.

Then Slices 1 and 2 — they are one design and should be reviewed together, but
may land as separate commits. Slice 4 is a decision to record, not code.

## 6. Slice 3 — `version.sh`

Two changes.

**(a)** Update `[workspace.dependencies] mdka`'s version alongside the package
versions. The current awk matches only `^[[:space:]]*version[[:space:]]*=`; that
line begins `mdka =`.

**(b) Add a post-update assertion — this is the more important half.** After a
bump, no manifest may still contain the previous version string. Fail loudly.

The assertion is what makes this durable. Fixing (a) alone fixes the one line
that has bitten so far; the assertion catches every location the script does not
know about, including ones added later. **A version bump that half-applies must
fail, not succeed quietly.**

Verify by testing the assertion's failure case, not only its success case:
deliberately leave a stale version string in a scratch copy and confirm the
script fails. A guard verified only by its success path proves nothing — that is
precisely how the `verify-ci` bug reached production.

## 7. Slice 1 — crates.io as a guarded workflow

New `.github/workflows/release-crates.yaml`, triggered on `release: created`,
gated by the same `verify-ci` job as its three siblings — **including
`GH_REPO: ${{ github.repository }}`**, without which it fails exactly as `2.1.7`
did.

Three requirements:

**Trusted Publishing, not a long-lived token.** `release-pypi.yaml` already does
this for PyPI (`id-token: write`, `environment: pypi`, commit `627bf93`) — follow
that shape. **Confirm the current crates.io mechanism against crates.io's own
documentation.** This handoff deliberately does not name an action or version;
verify rather than assume. If Trusted Publishing turns out not to be usable here,
**stop and report** — do not silently fall back to a token.

**An approval gate.** Publishing to crates.io is irreversible. Today the
safeguard is that a human runs the script; that safeguard must not simply
vanish. Use `environment: crates-io` with a required reviewer, mirroring the
existing `pypi` environment. Automated execution, human approval, guarded by CI —
all three.

**Sequential publish order:** `mdka` → `mdka-cli` → binding crates per Slice 4.
`mdka-cli` depends on `mdka`; ordering is load-bearing. Do **not** parallelise.
Modern cargo waits for index propagation itself — observed working at `2.1.7` —
so no manual sleep is needed.

Then remove `cargo-publish.sh`, or reduce it to a clearly-labelled break-glass
path. **Do not leave two live publish routes** — that is how the guarded one gets
bypassed under time pressure.

## 8. Slice 2 — automated GitHub release creation

New workflow on:

```yaml
on:
  push:
    tags: ['[0-9]+.[0-9]+.[0-9]+']
```

It verifies CI is green on the tagged commit — same guard logic, same `GH_REPO` —
then creates the GitHub release, which fires the four `release: created`
workflows.

The tag pattern must not match pre-release tags such as `2.0.0-rc.1`, which exist
in this repository's history. Verify your pattern against the real tag list.

This workflow creates a release; it publishes nothing itself. Keep it that way —
one job, one responsibility, and the publishing guards stay where they are.

## 9. Slice 4 — record the binding-crate decision

**Recommendation: keep publishing `mdka-node` and `mdka-python` to crates.io.**

They are already there through `2.1.7`; stopping now leaves an inconsistent
history for no benefit, publishing costs nothing, and holding the names prevents
someone else taking them.

If you disagree after looking, say so with reasoning — this is a genuine
judgement call, not a formality. Either way, **record the outcome and the reason
in RFC 015**, so nobody re-derives it from a shell script's loop variable.

## 10. Required verification

**Apply the lesson from `2.1.7` — verification must run in an environment
resembling the target, not merely execute the same commands.** The `verify-ci`
bug reached production because its logic was verified from inside a clone, which
silently supplied a precondition production lacks.

For this RFC specifically:

- Any `gh`-based logic: exercise **from outside a git repository**. Reproduce the
  failure with `GH_REPO` unset, then show it fixed with `GH_REPO` set.
- `version.sh`: test both the success path and the assertion's failure path.
- **The tag-push and crates.io workflows cannot be fully proven without cutting a
  release.** Verify what can be verified in isolation, then state plainly what
  remains unproven until the next real release. Do not let "it should work" read
  as "it was tested" — that framing is what made `2.1.7`'s failure a surprise.

## 11. Security constraints

- No new secret if Trusted Publishing works. If it does not, stop and report.
- No third-party action beyond what the repository already trusts.
- `actions: read` at job level; do not widen any top-level `permissions`.
- No `pull_request_target`.
- The `crates-io` environment must require a reviewer, not merely exist.

## 12. Prohibited shortcuts

- No `continue-on-error` on any guard.
- No treating a missing CI run as success.
- No publishing route that bypasses `verify-ci`.
- No token fallback for crates.io without reporting first.
- No refactoring the existing `verify-ci` jobs.
- No touching packaging or `ci.yaml`.

## 13. Known risks

| Risk | If it happens |
|---|---|
| Trusted Publishing not configurable on crates.io | Stop and report. Do not fall back to a token silently. |
| Tag pattern matches `2.0.0-rc.1`-style tags | Would create releases for pre-release tags. Verify against the real tag list. |
| Two publish routes left live | Defeats the RFC. Removing `cargo-publish.sh` is part of Slice 1, not optional cleanup. |
| The full path stays unproven until the next release | Expected. Report it as unproven rather than implying coverage. |

## 14. Required evidence

1. `version.sh`: success-path transcript **and** failure-path transcript showing
   the assertion firing on a deliberately stale version string.
2. Outside-a-git-repo transcripts for any `gh` logic, failure case first.
3. `git diff` — showing the existing three release workflows' packaging steps
   untouched.
4. Confirmation the `crates-io` environment exists **and requires a reviewer**.
5. Tag-pattern check against the real tag list, showing pre-release tags excluded.
6. A plain statement of what remains unproven until the next real release.

## 15. Acceptance checklist

- [ ] `version.sh` updates `[workspace.dependencies]`
- [ ] `version.sh` fails when any manifest retains the previous version string — demonstrated
- [ ] crates.io publishing runs in a workflow gated by `verify-ci`, with `GH_REPO` set
- [ ] Trusted Publishing used, or a report explaining why not
- [ ] `crates-io` environment requires a reviewer
- [ ] Publish order sequential: `mdka` → `mdka-cli` → bindings
- [ ] Only one live publish route per registry; `cargo-publish.sh` removed or clearly break-glass
- [ ] Tag push creates the GitHub release, CI-gated
- [ ] Tag pattern excludes pre-release tags
- [ ] Slice 4 decision recorded in RFC 015 with reasoning
- [ ] `ci.yaml` untouched; packaging steps untouched
- [ ] No file outside §3 modified

## 16. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 015 acceptance criteria, by number)
3. Changed files — complete list
4. **`version.sh` assertion failure-path demonstration**
5. **What remains unproven until the next real release** — stated plainly
6. Differences from RFC 015, if any, and why
7. Executed verification and results
8. Evidence per §14
9. Unresolved issues
10. Known limitations
11. Requested review focus

Items 4 and 5 are the substance. Every guard in this RFC is easy to write and
easy to write wrongly; what matters is demonstrating each one actually blocks.

## 17. Evidence standard

Standing: if a captured transcript or count does not reconcile, say so
explicitly, even when it does not change the conclusion you were asked to reach.

## 18. Escalate rather than decide

Stop and raise it if: Trusted Publishing is unavailable or requires a
configuration you cannot make; the tag pattern cannot exclude pre-release tags
cleanly; removing `cargo-publish.sh` would break something you did not expect; or
any guard cannot be made to fail closed.
