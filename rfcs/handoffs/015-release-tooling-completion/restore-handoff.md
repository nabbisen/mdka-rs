# Developer Handoff — RFC 015 · Restore Slice 1 (crates.io CI publishing)

**Governing RFC.** [RFC 015](../../done/015-release-tooling-completion.md), second revision (2026-08-08)
**Supersedes.** [`amendment-handoff.md`](./amendment-handoff.md) — that reverted Slice 1; this restores it.
**Prepared.** 2026-08-08
**Blocked on** the owner completing the Trusted Publisher registrations. Do not start until confirmed.

---

## 1. Purpose

Restore crates.io publishing as a guarded workflow. This reverses the revert you
just did.

## 2. Why, and why the churn was not waste

You built Slice 1, then deleted it, and are now restoring it. That is real time
spent twice, and it deserves a straight explanation.

The deletion happened because OIDC trusted publishing needs a per-crate Trusted
Publisher registration that only the account owner can perform, and it was not
clear how. The choice was framed as two coherent states — register and use it,
or publish manually and delete it — because the third state, a workflow in the
repository unable to authenticate, is the one thing worse than either.

The owner has since worked out how to configure it. **The only constraint
driving the deletion is gone.**

The goal never changed: all four registries guarded. What moved was the
estimated cost of getting there, and it is now grounded rather than assumed.

Your work is recoverable rather than rewritten — see §5.1.

## 3. Precondition — ✅ SATISFIED 2026-08-08

**The project owner has configured Trusted Publishers on all four crates:**
`mdka`, `mdka-cli`, `mdka-node`, `mdka-python` — each naming workflow filename
`release-crates.yaml`.

You are clear to proceed.

The ordering mattered: the previous attempt failed because the workflow existed
before the registration did, leaving a workflow in the repository that could not
authenticate. That is why this precondition came first.

**One honest caveat.** Neither the owner nor the architect can verify from here
that the *token exchange* works — only that the registrations were made. The
configuration is not visible through any public API, so the first real proof is
either §6's optional dry-run or the `2.1.8` release itself.

That makes §6 more attractive than when it was written. It is now the only way
to know before a release depends on it.

## 4. Change scope

| Path | Change |
|---|---|
| `.github/workflows/release-crates.yaml` | Restore from `f2e803b^` |
| `cargo-publish.sh` | Return to break-glass — **keeping** both improvements, see §5.3 |
| `crates-io` GitHub Environment | Recreate, **no protection rules** |

## 5. Required implementation

### 5.1 Restore the workflow

It was reviewed and approved before deletion. Recover it rather than rewriting:

```
git show f2e803b^:.github/workflows/release-crates.yaml > .github/workflows/release-crates.yaml
```

Then read it and confirm it still matches its surroundings — in particular that
its `verify-ci` job carries `GH_REPO: ${{ github.repository }}`, without which it
fails exactly as `2.1.7` did.

### 5.2 Recreate the `crates-io` environment — **without a reviewer**

```
gh api --method PUT repos/nabbisen/mdka-rs/environments/crates-io
```

**No protection rules.** It must match `pypi`'s shape: `{"protection_rules":[]}`.

This matters. D-2 decided against a required reviewer and that decision is *not*
reversed here — only the deletion of the environment is. The environment exists
solely to scope the OIDC claim, which is what `pypi` has always used its
environment for. Restoring the environment is not restoring the approval gate.

Read it back and report, as you did when creating and deleting it. Repository
settings are invisible to `git diff`.

### 5.3 `cargo-publish.sh` — break-glass, but keep what it gained

Re-add a break-glass banner making clear that `release-crates.yaml` is the normal
path and this script bypasses it.

**Do not strip either improvement the script gained while it was primary:**

- `cargo publish --workspace`
- The enforced, fail-closed CI-green check on the exact commit

A break-glass path that still verifies CI is strictly better than the one that
existed before this RFC. The banner changes; the logic stays.

## 6. Optional — prove the credential path first

**Recommended, not required.** The OIDC exchange can be verified without
publishing anything: a temporary `workflow_dispatch` job running only
`crates-io-auth-action` and reporting whether it obtained a token. Delete it once
proven.

This project has produced two automations that broke on first real exercise, and
neither was caught by review. Ten minutes to prove the credential path before
`2.1.8` depends on it is proportionate.

If skipped, `2.1.8` becomes the proving run and the fallback on failure is
`cargo-publish.sh`. That is acceptable — say which you chose.

## 7. Non-change scope — do not touch

- `release-executable.yaml`, `release-npm.yaml`, `release-pypi.yaml`, `ci.yaml`,
  `docs.yaml`.
- `version.sh`.
- `src/`, `tests/`, `docs/`, any manifest.
- **Do not add an approval gate** to `crates-io`. See §5.2.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml` — still parked

Untracked, not gitignored, Slice 2 withdrawn. Stage by explicit path:

```
git add .github/workflows/release-crates.yaml cargo-publish.sh
git status        # confirm create-release.yaml is still untracked
```

You have handled this correctly three times. Same again.

## 8. Required verification

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked      # expect 113, unchanged
sh -n cargo-publish.sh
```

No code changes, so 113 must hold exactly.

Then confirm:

- `.github/workflows/` contains `ci`, `docs`, `release-crates`,
  `release-executable`, `release-npm`, `release-pypi`, `scripts/`.
- `gh api repos/nabbisen/mdka-rs/environments` lists `crates-io` again, with
  **empty** `protection_rules`.
- The restored workflow's `verify-ci` step includes `GH_REPO`.

**Do not run `cargo-publish.sh`.** Publishing is irreversible and `2.1.8` is not
cut.

## 9. Prohibited shortcuts

- Do not restore before the registrations are confirmed.
- Do not add a required reviewer to `crates-io`.
- Do not strip `--workspace` or the CI check from `cargo-publish.sh`.
- Do not rewrite the workflow from scratch when git has the reviewed version.
- Do not run the publish script.
- Do not commit `create-release.yaml`.

## 10. Required evidence

1. Confirmation the owner completed all four registrations.
2. `git diff` of the restored workflow against `f2e803b^` — expect no difference,
   or an explained one.
3. Environment read-back showing `crates-io` present with empty
   `protection_rules`.
4. The new `cargo-publish.sh` in full.
5. `cargo test --workspace --locked` — 113.
6. Whether you took §6's optional dry-run, and its result if so.

## 11. Acceptance checklist

- [ ] Registrations confirmed before starting
- [ ] `release-crates.yaml` restored, `GH_REPO` present
- [ ] `crates-io` environment recreated with **no** protection rules
- [ ] `cargo-publish.sh` break-glass banner restored
- [ ] `--workspace` and the CI check retained
- [ ] Test count 113, unchanged
- [ ] `create-release.yaml` still untracked
- [ ] Script not executed
- [ ] No file outside §4 modified

## 12. Required review-request format

Standard eleven parts. Items worth particular care:

4. **The new `cargo-publish.sh` in full**
5. **Environment read-back proving no protection rules**
6. Whether §6's dry-run was taken, and what it showed

## 13. Escalate rather than decide

Stop and raise it if: the registrations are not confirmed; the restored workflow
differs from `f2e803b^` in any way you did not intend; the environment cannot be
created without protection rules; or the test count moves.

## 14. After this lands

The `2.1.8` release handoff follows, carrying both RFC 016 and RFC 017.

One rule discovered during your last submission goes into it, and is worth
knowing now: **tag the tip of a push, never an intermediate commit.** Being on
`origin/main` does not mean a commit was CI-verified — GitHub runs push-triggered
workflows only against the tip, so intermediate commits never get their own run.
`55914c8` demonstrates it: on `origin/main`, no CI run. Every guard would
correctly refuse to release it.
