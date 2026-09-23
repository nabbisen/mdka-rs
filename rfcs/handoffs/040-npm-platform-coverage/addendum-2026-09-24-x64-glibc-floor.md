# Addendum to the RFC 040 handoff — 2026-09-24 · the x64 glibc floor

**Addendum to.** `rfcs/handoffs/040-npm-platform-coverage/implementation-handoff.md`, which is frozen and
**not** edited. This adds one matrix entry change to the slice you already pushed at `688f99d`.
**Authorised.** Owner, 2026-09-24, on `.git-exclude/review-request/rfc040-release-scope/README.md` §3.
**Why it exists.** Your flag 5. It was correct, it was out of scope, and inspecting the rehearsal artifacts
turned it into a release blocker.
**Baseline.** `688f99d` — 583 Rust / 42 Node / 25 loader / 90 Python, seven workflows green.
**Priority.** P1 — the release is held for this.

---

## 1. What the rehearsal showed

I dispatched `release-npm.yaml` on `main` (run `35933063222`), downloaded the six binding artifacts and ran
`objdump -T` on them:

| Artifact | Minimum glibc |
|---|---|
| `mdka.linux-arm64-gnu.node` — **your** new entry, `--use-napi-cross` | **2.17** |
| `mdka.linux-x64-gnu.node` — the existing entry, native on `ubuntu-latest` | **2.34** |
| both musl bindings | static, none |

**In one release, the new arm64 glibc binding reaches glibc 2.17 and the existing x64 one demands 2.34.** An
arm64 user on Debian 11 is served; an x86-64 user on the same Debian 11 is not. Ubuntu 20.04 (2.31) and
RHEL/CentOS 8 (2.28) are excluded on x64 as well, and our PyPI wheels already reach 2.17 there.

**This is why it cannot wait.** Your `loader.js` will tell that user:

> *"a prebuilt binary **is published** for this platform, so the platform is supported and something else
> stopped it from loading."*

True by platform name, wrong in effect. Shipping RFC 040 unchanged means shipping a better error message that
lies to a real population of users — the exact defect class this RFC exists to close.

## 2. The change

In `.github/workflows/release-npm.yaml`, give the **`Linux-x64-gnu`** matrix entry the same treatment your
`Linux-arm64-gnu` entry has:

```yaml
- name: Linux-x64-gnu
  target: x86_64-unknown-linux-gnu
  os: ubuntu-latest
  archive_ext: .tar.gz
  napiplatform: linux-x64-gnu
  cross: true
  build_args: --target x86_64-unknown-linux-gnu --use-napi-cross
```

Nothing else. **Do not** switch it to `zigbuild`: `--use-napi-cross` is the mechanism already proven at 2.17
in that same run, and using the same one for both glibc targets is the point.

**If `--use-napi-cross` does not support `x86_64-unknown-linux-gnu`, stop and report.** Do not substitute
another mechanism — that would be a different change with a different risk, and it is mine to re-decide.

## 3. The gate — inspect the output, do not trust the build passing

This alters how the **most-used artifact in the project** is produced; `linux-x64-gnu` has shipped correctly
for a dozen releases. A green build is not sufficient evidence.

Before you report back, establish all four:

1. **`objdump -T` on the rebuilt `mdka.linux-x64-gnu.node` shows a maximum of `GLIBC_2.17`.** Paste the
   sorted list of `GLIBC_*` versions, not a summary.
2. **It still loads and works on this machine**: `node test.js` and `node test-loader.js` pass against it —
   42 and 25.
3. **Conversion output is unchanged.** Convert a corpus with the old binding and the new one and diff:
   byte-identical, or the change stops here. `tests/output_validity/mode_corpus/` plus
   `benches/benchdata/` is a reasonable set; say which you used and how many files.
4. **Nothing else about the binary moved** that you can see — `file`, `readelf -d` `NEEDED` list, exported
   symbol count. Report them side by side with the current published `2.4.2` binding.

**If anything beyond the glibc floor differs, stop and tell me.** The agreed fallback is to ship RFC 040
without this change and fix the floor in its own RFC; that decision is already made and needs no new
discussion.

## 4. Not in this addendum

- `release-executable.yaml` (CLI archives) and `release-pypi.yaml`. The CLI ships musl builds and PyPI
  already reaches `manylinux_2_17`; neither is affected. **Do not touch them.**
- macOS Intel, Windows ARM, the WASI fallback.
- Documenting the floor. `installation.md` and `README.md` are still **held out of your commits** — I apply
  them at release prep, per the review's §3.
- Version and CHANGELOG. The owner has set **`2.5.0`**; the bump is mine.

## 5. Report back

`.git-exclude/review-request/rfc040-x64-glibc-floor/README.md`, with §3's four results pasted in full.

Then commit and push the single workflow file, under the same rule as before: only work that is yours and
approved, no tag, no release, and **do not dispatch the workflow** — I run the second rehearsal.
