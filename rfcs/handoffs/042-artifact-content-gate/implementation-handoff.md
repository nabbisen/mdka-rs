# Developer Handoff — RFC 042 · Assert what is inside a published artifact

**RFC.** `rfcs/accepted/042-artifact-content-gate.md` — accepted by the owner, 2026-09-24
**Milestone.** Unassigned; release vehicle is the owner's call at prep time
**Priority.** P1 — §4 is a live defect in every release we have shipped
**Prepared.** 2026-09-24
**Baseline.** `cf590fb` — 583 Rust (`cargo test --workspace`), 42 Node, 25 loader, 90 Python; seven workflows green
**Order.** §2 and §3 first, **demonstrated failing**, then §4. Do not fix the binary before the gate can see it break.

---

## 1. What this is

**Nothing in this project inspects the contents of a built artifact.** Every control asks *"does it work
here?"* and none asks *"what is it?"* — so a binary that runs on the machine testing it looks perfect, and
the population it excludes is invisible by construction.

That is how the npm `linux-x64-gnu` binding required **glibc 2.34** for a dozen releases, excluding Ubuntu
20.04, Debian 11 and RHEL 8, while our own PyPI wheels reached 2.17.

## 2. The contract — as data, not as conditionals 🛑

Create a checked-in declaration of what each published target must satisfy. **A reader must be able to
answer "what do we promise?" from this file alone, without reading shell.** JSON or TOML beside the script,
your choice; the structure matters more than the format.

| Target | Must satisfy |
|---|---|
| `x86_64-unknown-linux-gnu` | ELF, x86-64; max `GLIBC_` **≤ 2.17**; `NEEDED` ⊆ {`libc.so.6`, `libgcc_s.so.1`, `libm.so.6`, `libdl.so.2`, `librt.so.1`, `libpthread.so.0`, `ld-linux-x86-64.so.2`} |
| `aarch64-unknown-linux-gnu` | ELF, aarch64; max `GLIBC_` **≤ 2.17**; same `NEEDED` allowlist with the aarch64 loader |
| `x86_64-unknown-linux-musl` | ELF, x86-64; **no `GLIBC_` symbol versions at all** |
| `aarch64-unknown-linux-musl` | ELF, aarch64; **no `GLIBC_` symbol versions at all** |
| `aarch64-apple-darwin` | Mach-O, arm64 |
| `x86_64-pc-windows-msvc` | PE, x86-64 |
| napi bindings, additionally | dynamic exports are **exactly** `{napi_register_module_v1}` |

The `NEEDED` allowlist above is the union of what the current npm and CLI Linux binaries actually link —
`libdl`/`librt`/`libpthread` appear on anything built against a pre-2.34 glibc, because 2.34 merged them into
`libc`. Verify it against the artifacts rather than trusting my list; if a target legitimately needs
something else, add it to the declaration with a comment saying why.

## 3. The gate

One script under `.github/workflows/scripts/`, invoked from **both** `release-executable.yaml` and
`release-npm.yaml`, **after build and before publish**, failing the job on any violation.

- CLI: run it against the binary that is about to be archived.
- npm: run it against each `node/npm/<platform>/*.node` before `napi pre-publish`.
- Release-triggered only. **Nothing goes into the seven per-push workflows.**
- `objdump -T`, `readelf -d`, `file` are enough on Linux runners. For the macOS and Windows entries, assert
  what you can cheaply from the runner you are on and say in the script what is not checked — an honest
  partial check beats a fake complete one.

**Report every violation, not the first.** A release that breaks three targets should say so in one run.

## 4. The live defect 🛑

**The published CLI `Linux-x64-gnu` binary at `2.5.1` requires `GLIBC_2.34`.** `README.md` offers it as the
prebuilt download for "Linux x64 (glibc)" and states no floor, so a user on Ubuntu 20.04, Debian 11 or RHEL 8
who follows our Quick Start gets a binary that will not start.

Cause: `release-executable.yaml` builds `x86_64-unknown-linux-gnu` with a plain
`cargo build --release --target …` natively on `ubuntu-latest` — exactly what produced the npm floor before
RFC 040.

**The npm fix does not transfer.** That used `--use-napi-cross`, which is a napi concept; the CLI has no napi
in its build. The mechanism to use here is **`cargo-zigbuild` with a glibc-suffixed target**:

```
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.17 --bin mdka --locked
```

`cargo-zigbuild` is already in this repository's release path — RFC 040 added it for the npm musl builds — so
this is an existing dependency, not a new one.

**If the glibc-suffixed target does not work, stop and report.** Do not substitute `cross`, a container
build, or anything else: that is a different change with a different risk profile and it is mine to decide.

This should be straightforward — `mdka-cli` depends only on `mdka`, and the published binary links just
`libgcc_s.so.1` and `libc.so.6`.

## 5. Sequence, and the part I care most about 🛑

1. Land §2 and §3.
2. **Run the gate against `2.5.1`'s CLI `Linux-x64-gnu` binary and show it fail.** Paste the output.
3. Then land §4.
4. Re-run: the gate passes on the rebuilt binary.

**A gate that has only ever passed has not been shown to work.** This is the same discipline the `2.4.2`
properties were held to, and it is the acceptance criterion I will check first. If the gate passes on the
2.34 binary, the gate is wrong — stop and tell me rather than adjusting the threshold until it goes green.

## 6. Documentation

`README.md` and `docs/src/getting-started/installation.md` must state the glibc floor for the **CLI
archives**, as `installation.md` already does for npm and PyPI. Do not write the number until §4 is done and
§5 step 4 has confirmed it from the rebuilt binary — the documentation states what the artifact guarantees,
not what we intend.

## 7. Explicitly not in this slice

- macOS/Windows minimum-version policy; reproducible builds; supply-chain attestation.
- Changing which platforms we ship — RFC 040 settled that.
- **`install-rust.sh` carries a workaround for `libdbus-sys` via `opener`.** Neither is in `Cargo.lock`, and
  `mdka-cli` depends only on `mdka`, so it appears vestigial — the script's own comment says *"I don't know
  if this is really the right thing to do"*. **Do not remove it here.** Recorded for its own slice; touching
  the cross toolchain and the gate in one change would confound them.

## 8. Criteria

1. The contract of §2 is data, and reading that file alone answers what each target must satisfy.
2. The gate runs in both release workflows, after build, before publish; not in any per-push workflow.
3. **The gate is demonstrated failing on the unfixed `2.5.1` CLI binary**, with output pasted.
4. After §4, every target passes — verified by running the gate against the **downloaded published**
   artifacts at the next release, not only against build outputs.
5. The rebuilt CLI binary: max `GLIBC_` ≤ 2.17, same `NEEDED` set or a justified change, and it still runs —
   `mdka --version` plus a conversion.
6. `README.md` and `installation.md` state the CLI glibc floor, matching the measured artifact.
7. 583 Rust / 42 Node / 25 loader / 90 Python still pass; seven workflows green.

## 9. Report back

`.git-exclude/review-request/042-artifact-content-gate/README.md`. Lead with §5 step 2 — the failing output.
Then the rebuilt binary's `objdump`/`readelf` beside the published `2.5.1` one, so the change is visible in
the artifact and not only in the workflow diff.

## 10. Committing and pushing

**Only work that is yours and has been approved.** Only the files this handoff asks you to change; the tree
holds RFCs, handoffs and review records that are not yours. Report back first — approval comes through that
review, and a green run is not approval.

**Tagging, releasing and triggering release workflows are not yours.** Note this slice touches two release
workflows: change them, do not run them. Ask before, not after.
