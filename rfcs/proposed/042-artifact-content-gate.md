# RFC 042 — Assert what is inside a published artifact

**Status.** Proposed
**Author.** Architect
**Created.** 2026-09-24
**Milestone.** Unassigned → proposed for the next minor. Part of it is a defect fix that could go sooner.
**Source.** The glibc floor found during RFC 040, and a survey of every published artifact afterwards. Every control this project has was green for a dozen releases while the npm Linux binary excluded three widely-used distributions.
**Touches.** `.github/workflows/release-executable.yaml`, `.github/workflows/release-npm.yaml`, a new script under `.github/workflows/scripts/`, `README.md`, `docs/src/getting-started/installation.md`.
**Relates to.** RFC 040, which fixed the npm half of the same defect without noticing the CLI half.

---

## 1. Summary

**Nothing in this project inspects the contents of a built artifact.** CI builds and tests the tree, four
consumer gates install and run the package, the release checks query the registries, and a whole-documentation
audit reads every claim. All of them were green throughout the period in which the npm `linux-x64-gnu` binding
silently required **glibc 2.34** — excluding Ubuntu 20.04, Debian 11 and RHEL 8 — while our own PyPI wheels for
the same platform reached **2.17**.

It surfaced only because RFC 040 added a cross-compiled arm64 binding, which put two differently-built
artifacts side by side, and someone ran `objdump`.

**This RFC proposes a contract and a gate:** declare what each published artifact must satisfy, and assert it
against the built bytes **before publishing**.

## 2. The survey — measured 2026-09-24 against published `2.5.1`

| Channel | Artifact | glibc floor | Content check today |
|---|---|---|---|
| **CLI archive** | `Linux-x64-gnu` | **`GLIBC_2.34`** 🛑 | **none** |
| CLI archive | `Linux-x64-musl`, `Linux-aarch64-musl` | static, none | none |
| npm | `@mdka/lib-linux-x64-gnu` | `GLIBC_2.14` ✅ *(fixed by RFC 040)* | none |
| npm | `@mdka/lib-linux-arm64-gnu` | `GLIBC_2.17` ✅ | none |
| PyPI | `manylinux_2_17_x86_64` wheel | `GLIBC_2.14` against a tag claiming 2.17 ✅ | **`auditwheel`, via maturin** |

**The one channel that has a content check is the only channel that never had this bug.** That is the whole
argument for this RFC, and it was not designed — it came free with maturin.

### 2.1 There is a live defect right now

**The published CLI binary for `Linux-x64-gnu` at `2.5.1` requires `GLIBC_2.34`.** `README.md` offers it as
the prebuilt download for "Linux x64 (glibc)" and states no floor, exactly the silence that hid the npm case.
A user on Ubuntu 20.04, Debian 11 or RHEL 8 who follows our Quick Start gets a binary that will not start.

RFC 040 fixed the npm binding and did not look at the CLI. I did not think to ask whether the same build
pattern existed elsewhere — `release-executable.yaml` builds `x86_64-unknown-linux-gnu` natively on
`ubuntu-latest`, which is precisely what produced the npm floor.

## 3. Why no existing gate can catch this

| Gate | What it proves | Why it misses this |
|---|---|---|
| `ci.yaml` | the tree compiles and tests pass | builds for the host; never looks at a release artifact |
| `npm install gate` | `npm install` then `require` works | runs on `ubuntu-latest`, whose glibc is newer than 2.34 — the defect is invisible on the machine that tests it |
| `pypi wheel gate`, `pypi published gate` | wheels exist and install | the real check here is `auditwheel`, inside maturin, not ours |
| `crates package gate` | the crate packages | source only |
| Release registry verification | the right versions are published | version numbers, not bytes |
| Documentation audit | every claim matches behaviour | reads documents and runs conversions; never opens a binary |

**The common shape:** every control asks *"does it work here?"* None asks *"what is it?"* A binary that runs
on the machine testing it looks perfect, and the excluded population is invisible by construction.

## 4. Proposal

### 4.1 Declare the contract

A single declared table, checked in, saying what each published artifact must satisfy. Not derived from the
build — stated, so a change to it is a visible decision:

| Target | Must satisfy |
|---|---|
| `x86_64-unknown-linux-gnu` | ELF x86-64; `NEEDED` ⊆ allowlist; **max `GLIBC_` ≤ 2.17** |
| `aarch64-unknown-linux-gnu` | ELF aarch64; **max `GLIBC_` ≤ 2.17** |
| `*-unknown-linux-musl` | correct arch; **no `GLIBC_` references at all** |
| `aarch64-apple-darwin` | Mach-O arm64 |
| `x86_64-pc-windows-msvc` | PE x86-64 |
| napi bindings additionally | exports exactly `napi_register_module_v1` |

2.17 is chosen because it is what our PyPI wheels already promise and what `--use-napi-cross` already
produces. **The point is not the number — it is that there is one, written down.**

### 4.2 Assert it before publishing

One script, invoked from `release-executable.yaml` and `release-npm.yaml` **after build and before publish**,
failing the job on violation. It runs at release only; nothing is added to the seven per-push workflows.

For the CLI the assertion runs on the archive that will be uploaded; for npm, on each per-platform package
directory before `napi pre-publish`.

### 4.3 Fix the CLI floor

Apply to `release-executable.yaml`'s `x86_64-unknown-linux-gnu` entry the same treatment RFC 040 applied to
npm. Without it §4.2 fails on its first run — **which is the correct order**: land the gate, watch it fail on
a real defect, then fix. A gate that has never failed has not been shown to work.

## 5. Not in scope

- **macOS and Windows minimum-version policy.** Worth a contract eventually; nothing indicates a defect now.
- **Reproducible builds.** A different and much larger goal.
- **Supply-chain attestation.** npm provenance already exists for CI-published packages.
- **Changing what platforms we ship** — RFC 040 settled that.

## 6. Acceptance criteria

1. The contract of §4.1 exists as **data**, not as conditionals inside a workflow, so reading it answers "what
   do we promise?" without reading shell.
2. The gate runs in both release workflows, after build, before publish.
3. **It fails on `2.5.1`'s CLI `Linux-x64-gnu` binary** — demonstrated before the fix lands. A gate that has
   only ever passed proves nothing; this is the same discipline the `2.4.2` properties were held to.
4. After §4.3, every published artifact satisfies the contract, verified by re-running the gate against the
   **downloaded published** artifacts, not the build outputs.
5. `README.md` and `installation.md` state the glibc floor for the CLI archives, as `installation.md` already
   does for npm and PyPI.
6. No change to the per-push workflows, and no change to conversion output.

## 7. Cost

A script of maybe a hundred lines using `objdump`/`readelf`/`file`, two workflow steps, and one matrix entry
change. It runs once per release, in parallel with work already happening.

**Against:** a dozen releases in which we shipped a binary that could not start on three of the most common
Linux distributions, and did not know.
