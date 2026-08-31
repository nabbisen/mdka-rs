# Developer Handoff — RFC 020 · npm distribution repair

**Governing RFC.** [RFC 020](../../proposed/020-npm-distribution-repair.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P0 — start here, ahead of everything else in M2b
**Prepared.** 2026-08-31
**Baseline.** `main` @ `2e04a89`, 136 tests, CI green.

---

## 1. Purpose

`npm install mdka` has produced an unusable package for the entire 2.x line —
twelve releases, roughly four months. Fix it, and build the gate that would have
caught it.

## 2. Reproduce it first

Do this before reading further. It takes a minute and it calibrates everything
else.

```
mkdir /tmp/x && cd /tmp/x && npm init -y
npm install mdka@2.2.0          # succeeds
node -e "require('mdka')"       # Error: Cannot find module 'mdka-linux-x64-gnu'
```

## 3. Scope

| Path | Change |
|---|---|
| `.github/workflows/ci.yaml` | New pack-install-require job |
| `.github/workflows/release-npm.yaml` | Add the missing publish step |
| `node/package.json` | Reconcile `napi.package.name` with what `index.js` requires |

**Scope boundary, per RFC 027 Rule 2.** This handoff covers **npm only**. PyPI,
crates.io and documentation examples have the same class of gap and are **not**
covered here — they belong to RFC 026, which is a separate handoff with a named
owner. The boundary is at npm because npm is the one that is currently broken for
users; the others are unverified, not known-broken.

## 4. Order of work — the gate lands first

**This ordering is not negotiable.**

1. Write the gate. Commit and push it **alone**.
2. **Observe it fail** against the current broken package. Capture that output.
3. Fix the publish path.
4. Observe the same gate pass.

A gate that has only ever been seen green proves nothing — that is the whole
lesson of this RFC, and of `verify-ci` before it. Step 2's captured failure is a
**required deliverable**, not a nicety.

If the gate passes at step 2, stop and raise it. Either the gate is not testing
what we think, or the defect is not what we think.

## 5. The gate

A CI job that, in a directory **outside the workspace**:

```
cd node && npm pack
mkdir -p "$(mktemp -d)" && cd it
npm init -y && npm install /path/to/mdka-<version>.tgz
node -e "const m=require('mdka'); if(!m.htmlToMarkdown('<h1>x</h1>').includes('# x')) process.exit(1)"
```

Requirements:

- It **must** install the packed tarball, not the source tree. A test that runs
  inside `node/` passes today and proves nothing.
- It runs on **every push**, not only at release.
- It fails loudly. No `|| true`, no `continue-on-error`.

## 6. The fix — two independent faults

Both must be fixed. Either alone leaves the package broken.

### 6.1 `napi prepublish` is never run

`release-npm.yaml` runs `napi create-npm-dirs`, `napi artifacts`, then
`npm publish`. Missing is the step that publishes the per-platform packages and
injects `optionalDependencies` into the main manifest.

Evidence: published `mdka@2.2.0` has no `optionalDependencies`, and
`@mdka/lib-linux-x64-gnu` has no version above `1.6.9`.

### 6.2 The names disagree

`node/index.js` requires **unscoped** `mdka-linux-x64-gnu` — the file contains
zero occurrences of `@mdka`. `napi.package.name` is `@mdka/lib`, so
`create-npm-dirs` produces **scoped** `@mdka/lib-*` directories.

**Determine what napi-rs 3.8.6 actually generates by running it.** Do not reason
from its documentation. This mismatch exists because a config value and a
generated file disagreed silently, and reading either one alone would have looked
correct.

### ⚠ 6.3 Settle name ownership before choosing

| Name | Registry state |
|---|---|
| `@mdka/lib-linux-x64-gnu` | exists, up to `1.6.9`, publication history |
| `mdka-linux-x64-gnu` | **resolves, zero versions published** |

The unscoped name resolving with nothing published is **unexplained**. Run
`npm owner ls` on both before choosing.

**Default to scoped `@mdka/lib-*`** — proven ownership and publication history.
Adopt unscoped only on positive proof we own it. Depending on a name a third
party could claim is worse than the current breakage: today it fails loudly;
that would fail silently and maliciously.

**If ownership is unclear, stop and raise it.** Do not publish to find out.

## 7. After the fix ships

Deprecate the broken range so the registry explains itself:

```
npm deprecate mdka@">=2.0.2 <2.2.1" "Native binding fails to load; upgrade to 2.2.1+"
```

Owner-run if you lack registry rights — say so rather than skipping it.

## 8. Required verification

Per RFC 027 Rule 3, state for **each** item whether it ran against the workspace
tree or an installed artifact.

1. The reproduction in §2, before any change.
2. The gate failing at step 4.2 — captured output.
3. The gate passing after the fix.
4. `npm owner ls` for whichever name is chosen.
5. After publishing 2.2.1: `npm install mdka@2.2.1` in a clean directory on a
   machine that has never built this project, then `require()`.
6. Each published platform package resolves at 2.2.1 — check the registry for
   each, since CI only exercises Linux.
7. `cargo test --workspace --locked`, `cargo fmt --check`, clippy — unchanged.

## 9. Prohibited shortcuts

- Do not fix the publish path before the gate exists and has been seen failing.
- Do not test against `node/` and call it verification.
- Do not choose the unscoped name without ownership proof.
- Do not `git add -A`.
- Do not report "the workflow looks right" as evidence. This workflow has looked
  right for twelve releases.

## 10. Known risks

| Risk | If it happens |
|---|---|
| `napi prepublish` needs a token scope we lack | It publishes to npm, so `NPM_TOKEN` should suffice. If it 403s, report the exact scope needed. |
| Ownership of `mdka-*` is someone else's | **Stop and raise.** Do not publish. |
| macOS/Windows still broken after Linux is fixed | Likely if the cause is per-platform. §8.6 is how you find out; do not infer from Linux. |
| The gate is slow | It is one pack and one install. If it is slow, say so with numbers. |

## 11. Acceptance checklist

- [ ] Gate exists, runs on every push, installs the packed tarball from outside the workspace
- [ ] **Gate observed failing pre-fix, output captured in the review request**
- [ ] `napi prepublish` (or equivalent) in the publish path
- [ ] `index.js`, generated dirs and registry all use one naming scheme
- [ ] Ownership of the chosen name verified and recorded
- [ ] `optionalDependencies` present in the published manifest
- [ ] Clean-machine install + `require()` succeeds at 2.2.1
- [ ] Every published platform package resolves at 2.2.1
- [ ] Broken range deprecated, or the owner asked to
- [ ] `create-release.yaml` untouched

## 12. Escalate rather than decide

Stop and raise if: the gate passes before the fix; name ownership is unclear; the
fix needs a credential we do not have; or a platform other than Linux is still
broken after the change.

## 13. Required review-request format

Standard eleven parts. The substance:

4. **The gate's pre-fix failure and post-fix pass**, both as captured output
5. **The naming decision**, with ownership evidence
6. **Per-platform registry resolution** at 2.2.1
7. **Anything that behaved differently from this handoff's prediction** — this
   document predicts napi-rs behaviour, and every prediction about release
   machinery in this project has been wrong at least once
