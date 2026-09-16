# Implementation handoff — RFC 030 · Crates package gate: workspace resolution

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/accepted/030-crates-package-gate-workspace-resolution.md`](../../accepted/030-crates-package-gate-workspace-resolution.md) — **Accepted 2026-09-16 by the owner**
**Milestone.** M3, and **first in the M3 order** — 030, then 025, then 024, then 028.
**Target release.** `2.3.0`
**Size.** Small. One workflow file, one checklist line. The verification is larger than the change, deliberately.

---

## 0. Preconditions

None. This is independent of RFC 024/025/028 and does not touch `src/`.

**This handoff is an instruction to start.**

## 1. Why you are doing this

`.github/workflows/crates-package-gate.yaml` runs four sequential steps:

```
cargo package -p mdka          # passes
cargo package -p mdka-cli      # FAILS -- cannot resolve mdka ^X.Y.Z from crates.io
cargo package -p mdka-node     # skipped
cargo package -p mdka-python   # skipped
```

`cargo package` verifies by building the **extracted tarball**, where path
dependencies are stripped and `mdka` must come from the registry. On `main` the
workspace version is the *next* version — not yet published — so the three
dependents cannot resolve it.

Two harms, and the second is the one that matters:

1. It has been red at every release since RFC 026 created it.
2. **The failure skips `mdka-node` and `mdka-python`.** Across `2.2.2` and
   `2.2.3`, nothing has verified that those two package and build standalone —
   which is the only reason the gate exists. The red masks its own coverage.

## 2. The change

Replace the four steps with one:

```yaml
- name: cargo package --workspace
  run: cargo package --workspace --locked
```

Cargo builds a temporary local registry from the packaged members
(`target/package/tmp-registry/`) and resolves the dependents against it.

**Constraints:**

- **No `--no-verify`**, anywhere. RFC 026's whole point is that the gate builds
  what it packs.
- Keep the final "show what each package actually contains" step. It must still
  list all four `.crate` files.
- **Rewrite the leading comment** to explain *why* `--workspace` rather than four
  steps, naming the unpublished-version problem explicitly. Someone will
  eventually look at one line and think it lost per-crate granularity; the
  comment is what stops them "simplifying" it back.

## 3. ⚠ How to verify — the obvious method is wrong

**Do not verify this by pushing and observing the gate green on `main`.**

`2.2.3` is now published and `main` sits at `2.2.3`, so the workspace version
*is* currently resolvable from crates.io — which means **the old, broken gate
would also pass right now.** A green on today's `main` distinguishes the fix
from the bug not at all.

I wrote exactly that wrong criterion into the RFC's first draft. It is corrected
in §5 there; this is the same instruction.

### 3.1 Prove it green against an unpublished version

```
git worktree add .git-exclude/tmp/gate-probe <commit> --detach
```

In the worktree, bump to a version that is **not** on crates.io — e.g. `2.2.4`:

- the `version = ` field in `Cargo.toml`, `cli/Cargo.toml`, `node/Cargo.toml`,
  `python/Cargo.toml`, **and**
- the `[workspace.dependencies]` line in the root `Cargo.toml`:
  `mdka = { version = "...", path = "." }`

⚠ **That last one is the trap.** A `sed 's/^version = ...'` does not match it,
because it does not start the line. Miss it and the dependents still request the
old version, which *is* published — so the experiment passes while testing
nothing. I hit this on the first attempt.

Then, in order:

1. `cargo package -p mdka --allow-dirty` → expect **success**
2. `cargo package -p mdka-cli --allow-dirty` → expect **failure**, with
   `failed to select a version for the requirement mdka = "^2.2.4"`
3. `cargo package --workspace --allow-dirty` → expect **exit 0**
4. Confirm `target/package/` holds **four `.crate` files and four extracted
   directories**. The extracted directories are the proof each crate was built,
   not merely tarred.
5. `git worktree remove --force .git-exclude/tmp/gate-probe`, and confirm the
   main tree is clean afterwards.

**Step 2 is not optional.** Without it, a pass at step 3 might just mean the
version was published after all. Step 2 is what establishes the experiment is
live.

This is reproducible — I ran it at `98dcf1f` and it behaved exactly as above.

### 3.2 Prove it red for a real defect

A gate nobody has seen fail is a green light, per RFC 026.

In a throwaway worktree, break one crate in a way that only shows up when
packaged — e.g. remove a file the crate's `include`/`exclude` needs, or add a
`path` dependency with no `version`. Confirm the gate fails **on that**, and
record the output.

Say which crate you broke and how. If you cannot make it fail, that is a finding
and I want to hear it before you close this out.

## 4. Also update

`.git-exclude/release/RELEASE-CHECKLIST.md` §1: the four-gate line currently
reads "green, **or each red one is explained**". Once this lands, the crates
package gate should have no standing explanation. Remove any wording that
normalises its red.

Do **not** weaken "or each red one is explained" itself — that clause is doing
real work for the other gates.

## 5. Out of scope

- The other three gates. Untouched.
- Anything in `src/`, `cli/`, `node/`, `python/` **source**. Manifest edits
  happen only inside throwaway worktrees and are never committed.
- Release automation (`create-release.yaml`, the publishers). This gate does not
  publish.

## 6. Acceptance checklist

- [ ] Single `cargo package --workspace --locked`; no `--no-verify`
- [ ] Contents-listing step retained, lists four `.crate` files
- [ ] Leading comment explains why `--workspace`, naming the unpublished-version problem
- [ ] §3.1 performed, with the step-2 failure recorded as well as the step-3 pass
- [ ] §3.2 performed — gate seen failing for a real, deliberate defect, with output
- [ ] Release checklist §1 no longer normalises this gate's red
- [ ] Main tree clean; no worktree left behind; no manifest bump committed

## 7. Report back

A review request under `.git-exclude/review-request/030-crates-package-gate/`,
entry point `README.md`.

Include the **raw output** of §3.1 steps 2 and 3, and of §3.2. Those three
captures are the deliverable — the workflow diff is trivial by comparison, and
per RFC 027 Rule 3 label each capture with what it ran against.
