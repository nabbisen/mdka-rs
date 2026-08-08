# Developer Handoff — RFC 015 amendment · Revert Slice 1, restore manual crates.io publishing

**Governing RFC.** [RFC 015](../../proposed/015-release-tooling-completion.md), post-decision revision (2026-08-08)
**Supersedes.** Parts of [`implementation-handoff.md`](./implementation-handoff.md) — Slices 1 and 2 only. Slices 3 and 4 stand.
**Prepared.** 2026-08-08
**Do this before the `2.1.8` release**, so `cargo-publish.sh` is correct when it is used.

---

## 1. Purpose

The project owner has decided crates.io publishing stays manual. Remove the
workflow that would otherwise sit in the repository unable to authenticate, and
restore `cargo-publish.sh` as the documented primary path.

## 2. Why this reverses work you just did

You implemented Slice 1 correctly and it was approved. Nothing about the
implementation was wrong.

OIDC trusted publishing requires a one-time Trusted Publisher registration per
crate on crates.io, which only the account owner can perform. Rather than do
that, the owner chose to keep publishing from the desktop.

**The decisive point was the middle state.** Keeping `release-crates.yaml`
without registering the publishers would leave a workflow in the repository that
cannot authenticate — something that reads as protection and is not. That is
exactly the defect class this milestone exists to eliminate: the stale MSRV
claim, the six inert `ConversionOptions` fields, the never-compiled preprocessor.

So the choice was framed as two coherent states rather than three: register and
use it, or publish manually and delete it. The owner chose the second.

Automating crates.io publishing is recorded in `ROADMAP.md` as a **future
candidate**, not abandoned.

## 3. Change scope

| Path | Change |
|---|---|
| `.github/workflows/release-crates.yaml` | **Delete** |
| `cargo-publish.sh` | Rewrite — primary path, `cargo publish --workspace` |
| `crates-io` GitHub Environment | **Delete** (repository settings, not a file) |

## 4. Non-change scope — do not touch

- **`release-executable.yaml`, `release-npm.yaml`, `release-pypi.yaml`.** These
  publish three registries under the `verify-ci` guard and are unaffected.
- **`ci.yaml`.**
- **`version.sh`** — Slice 3, working, keep exactly as is.
- `src/`, `tests/`, `docs/`, any manifest.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml` — still do not commit it

Same standing hazard. It remains untracked and is **not gitignored**. Stage by
explicit path:

```
git add .github/workflows/release-crates.yaml cargo-publish.sh
git status        # confirm create-release.yaml is still untracked
```

Note the first path is a **deletion** — `git add` stages it correctly, but
confirm with `git status` that it shows as deleted rather than silently skipped.

## 5. Required implementation

### 5.1 Delete the workflow

```
git rm .github/workflows/release-crates.yaml
```

### 5.2 Rewrite `cargo-publish.sh`

It currently carries a break-glass banner saying the guarded workflow is the
normal path. That is now false and must go — a script whose own comment
misdescribes the process is the same problem in miniature.

The new script should:

- State plainly that this **is** the crates.io release path, run locally by the
  maintainer, and that crates.io is deliberately not covered by the `verify-ci`
  guard that covers the other three registries.
- **Tell the operator to confirm CI is green on the released commit first**,
  with the command to do it. That check is what the guard would have done; with
  the guard gone it becomes a documented human step rather than an unstated
  assumption.
- Use **`cargo publish --workspace`**, replacing the hand-rolled four-step loop.
  Verified available on cargo 1.97.1; it handles publish ordering and index
  propagation across all four crates.

Keep it short. The point is that someone reading it in a year knows what it is
for and what to check before running it.

### 5.3 Delete the `crates-io` environment

Repository settings, via `gh api` as you created it. It existed only to scope the
OIDC claim, which is no longer used.

**This is a repository-settings change invisible to `git diff`** — report it
explicitly with a read-back, as you did when creating it.

## 6. Required verification

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked          # expect 113, unchanged
```

No code changes here, so 113 must hold exactly. If it moves, something outside
scope was touched.

Then confirm:

- `.github/workflows/` contains exactly: `ci.yaml`, `docs.yaml`,
  `release-executable.yaml`, `release-npm.yaml`, `release-pypi.yaml`, and
  `scripts/`. No `release-crates.yaml`, no `create-release.yaml`.
- `gh api repos/nabbisen/mdka-rs/environments` no longer lists `crates-io`.
- `sh -n cargo-publish.sh` parses.

**Do not run `cargo-publish.sh`.** Publishing is irreversible and `2.1.8` has not
been cut. Verifying it parses is the limit of what is appropriate here.

## 7. Prohibited shortcuts

- Do not leave `release-crates.yaml` in place "in case". That is the middle state
  the decision exists to avoid.
- Do not leave the break-glass banner on `cargo-publish.sh`.
- Do not run the publish script.
- Do not touch the three working release workflows.
- Do not commit `create-release.yaml`.

## 8. Required evidence

1. `git status` after staging — deletion shown, `create-release.yaml` untracked.
2. `ls .github/workflows/` — the five expected files plus `scripts/`.
3. `gh api …/environments` read-back showing `crates-io` gone.
4. The new `cargo-publish.sh` in full.
5. `cargo test --workspace --locked` — 113, unchanged.

## 9. Acceptance checklist

- [ ] `release-crates.yaml` deleted
- [ ] `cargo-publish.sh` rewritten: primary path, CI-check instruction, `--workspace`
- [ ] Break-glass banner removed
- [ ] `crates-io` environment deleted, with read-back evidence
- [ ] Three working release workflows untouched
- [ ] Test count 113, unchanged
- [ ] `create-release.yaml` still untracked
- [ ] Script not executed
- [ ] No file outside §3 modified

## 10. Required review-request format

1. Implementation summary
2. Changed files, plus the repository-settings change
3. **The new `cargo-publish.sh` in full**
4. Environment-deletion read-back
5. Differences from this Handoff, if any
6. Executed verification and results
7. Evidence per §8
8. Unresolved issues
9. Known limitations
10. Requested review focus

## 11. Escalate rather than decide

Stop and raise it if: the environment cannot be deleted; `cargo publish
--workspace` turns out not to behave as expected for this workspace shape (report
it — do not fall back to the loop silently); or the test count moves.

## 12. After this lands

The `2.1.8` release handoff follows. That release publishes crates.io through
`cargo-publish.sh` — which is why this lands first.
