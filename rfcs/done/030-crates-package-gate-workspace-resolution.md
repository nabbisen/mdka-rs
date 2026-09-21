# RFC 030 — Crates package gate: verify the workspace, not the registry

**Status.** Implemented (2.3.0)
**Author.** Architect
**Created.** 2026-09-16
**Milestone.** M3 (control repair; should land before `2.3.0`)
**Supersedes.** Nothing. Repairs the gate introduced by RFC 026 §4.2.
**Related.** RFC 026 (consumer-artifact gates), RFC 027 (verification discipline)

---

## 1. Summary

The `crates package gate` has failed on **every release since it was
introduced**, for a reason that has nothing to do with the code it inspects, and
in failing it **skips the two crates nobody else checks**. Replace its four
sequential per-crate `cargo package` steps with a single
`cargo package --workspace`, which resolves workspace members against each other
instead of against crates.io.

## 2. The problem

The gate runs, in order:

```
cargo package -p mdka          # passes
cargo package -p mdka-cli      # FAILS
cargo package -p mdka-node     # skipped
cargo package -p mdka-python   # skipped
```

The failure is always the same:

```
failed to select a version for the requirement `mdka = "^2.2.3"`
candidate versions found which didn't match: 2.2.2, 2.2.1, 2.2.0, ...
location searched: crates.io index
```

`cargo package` verifies a crate by building the **extracted tarball**, where
path dependencies have been stripped and `mdka` must come from the registry. On
`main`, the workspace version is by definition the *next* version — the one this
release will publish. So it cannot be on crates.io yet, and the three dependent
crates cannot resolve it.

**This is not a flaky failure — but it is not constant either.** It fails
**exactly at the release commit**, where the workspace version has just been
bumped and not yet published, and passes on either side of it.

> **Corrected during review.** The first draft said this failure was
> "guaranteed, at every commit, forever". That was wrong, and contradicted §2.2
> below. The implementer checked CI history before changing anything:
>
> | Commit | Version on `main` | Published then? | Gate |
> |---|---|---|---|
> | `64ae672` | 2.2.2 | yes | success |
> | **`98dcf1f`** — Release 2.2.3 | **2.2.3** | **not yet** | **failure** |
> | `15d5b11`, `52c1935` | 2.2.3 | yes | success |
>
> **That pattern is why it survived two releases.** The gate was green whenever
> anyone happened to look, and red only at the one moment the release checklist
> consults it — when a red with a ready explanation is easiest to accept.

### 2.1 Why it matters more than "a red we explain"

Two compounding harms:

1. **It skips `mdka-node` and `mdka-python` entirely.** GitHub Actions stops the
   job at the first failed step. Across `2.2.2` and `2.2.3`, *nothing* has
   verified that those two crates package and build standalone — which is the
   single thing the gate exists to check. The gate's red is not merely noise; it
   is **masking its own coverage**.
2. **A permanently-red gate stops being read.** The release checklist §1 asks
   for "green, or each red one is explained", and the explanation is now
   boilerplate written twice. A check that always fails for a known reason
   trains its readers to skip it, and the next failure — a real one — arrives
   wearing the same colour.

Harm 2 is the one RFC 027 exists to prevent, and we have walked into it.

### 2.2 The tell: its colour tracks the calendar, not the code

Immediately after `2.2.3` published, the workspace version `2.2.3` *became*
resolvable on crates.io. So the gate will now go **green on `main`** — with no
code change whatsoever — and stay green until the next version bump flips it
red again, also with no code change.

**A check whose result is determined by what we published this morning rather
than by what is in the tree is not measuring the tree.** Anyone reading a green
crates package gate between now and the next bump is reading an artifact of
release timing. That is worse than the red, because red at least announced
itself.

## 3. The fix

```yaml
- name: cargo package --workspace
  run: cargo package --workspace --locked
```

Cargo builds a **temporary local registry** from the packaged members and
resolves the dependents against it, so the unpublished version is found.

### 3.1 Verified, not assumed

Run at `98dcf1f` in a throwaway worktree, with all four manifests and the
`[workspace.dependencies]` entry bumped to **2.2.4 — a version that does not
exist on crates.io**, so registry resolution could not possibly succeed:

| Command | Result |
|---|---|
| `cargo package -p mdka` | success |
| `cargo package -p mdka-cli` | **fails** — `failed to select a version for the requirement mdka = "^2.2.4"` |
| **`cargo package --workspace`** | **exit 0** |

Artifacts produced by the `--workspace` run:

```
target/package/mdka-2.2.4.crate          target/package/mdka-2.2.4/
target/package/mdka-cli-2.2.4.crate      target/package/mdka-cli-2.2.4/
target/package/mdka-node-2.2.4.crate     target/package/mdka-node-2.2.4/
target/package/mdka-python-2.2.4.crate   target/package/mdka-python-2.2.4/
target/package/tmp-registry/
```

The **extracted directories** are the point: each crate was unpacked and
compiled, not merely tarred. `tmp-registry/` is cargo's temporary registry, the
mechanism that makes it work.

`--no-verify` appears nowhere, preserving RFC 026's requirement that the gate
build what it packs.

### 3.2 Independent corroboration from this release

`2.2.3` published `mdka-cli`, `mdka-node` and `mdka-python` to crates.io without
incident, immediately after the gate said they could not be packaged. That is
direct evidence the red was an artifact of resolution order and not a defect —
and equally, evidence that the gate told us nothing either way.

## 4. What this does not fix

**The gate still cannot catch a dependent that breaks against the *published*
`mdka`**, because it now builds against the locally packaged one. In a
single-version-lockstep workspace like ours — all four crates share a version
and release together — that distinction is theoretical. It would stop being
theoretical if the crates ever version independently. Recorded here so the next
reader does not have to rediscover the limit.

## 5. Acceptance criteria

- [ ] `crates-package-gate.yaml` uses a single `cargo package --workspace --locked`.
- [ ] No `--no-verify`.
- [ ] The "show what each package contains" step still runs and still lists all
      four `.crate` files.
- [ ] The workflow comment explains *why* `--workspace` rather than four steps,
      naming the unpublished-version problem, so nobody "simplifies" it back.
- [ ] **The gate is observed green against an _unpublished_ workspace version.**

      ⚠ **Read this carefully — the obvious reading is wrong.** It is tempting
      to say "green on `main` proves it", and I wrote exactly that in the first
      draft of this RFC. It is false, and §2.2 explains why: `2.2.3` is now
      published, so `main`'s current version *is* resolvable from crates.io and
      the **old** gate would also pass right now. A green on today's `main`
      distinguishes nothing.

      So prove it deliberately instead of waiting for the calendar:

      1. `git worktree add` a throwaway at the commit under test.
      2. Bump all four `version = ` fields **and the
         `[workspace.dependencies] mdka = { version = ..., path = "." }` entry**
         in the root `Cargo.toml` to a version that is not on crates.io.
         *The workspace-dependency line is easy to miss — a `^version = ` sed
         does not match it, and missing it makes the experiment silently
         meaningless.*
      3. Confirm `cargo package -p mdka-cli` **fails** there — this establishes
         the experiment is actually testing the unpublished case.
      4. Confirm `cargo package --workspace` **succeeds**, and that
         `target/package/` holds four `.crate` files *and* four extracted
         directories. The extracted directories are the proof each crate was
         built, not merely tarred.
      5. Remove the worktree. Record the commands and output.

      Step 3 is not optional: without it, a pass in step 4 could simply mean the
      version was published after all.
- [ ] **The gate is observed red for a real defect.** Break one crate
      deliberately in a scratch branch or worktree — e.g. remove a file its
      `include` needs — confirm the gate fails on *that*, and record the output.
      This is the step that distinguishes a working gate from a green light.
- [ ] Release checklist §1 no longer carries a standing explanation for this
      gate.

## 6. Risks

| Risk | Assessment |
|---|---|
| `--workspace` masks a per-crate failure | No — it fails the job on any member, and now reports *all four* rather than stopping at the second |
| Longer runtime | Slightly; one dependency graph is built once rather than four times, which may be faster in practice |
| Cargo version dependence | The temporary-registry behaviour is long-standing and present well before our MSRV 1.88; the gate runs on stable |
| We stop noticing the gate | §5's two observation requirements exist for exactly this |
