# RFC 040 — npm platform coverage, and an error message that contradicts the documentation

**Status.** Proposed
**Author.** Architect
**Created.** 2026-09-24
**Milestone.** M4 · Coverage and durability → a patch or `2.5.0`
**Source.** Owner selected the npm platform gap as the next M4 item, 2026-09-24. Request: `.git-exclude/review-request/post-2.4.1/README.md` §4.
**Touches.** `.github/workflows/release-npm.yaml`, `node/package.json`, `node/` (one new file), `docs/src/getting-started/installation.md`, `README.md`.

---

## 0. A correction to the request that produced this RFC

The request said the gap was **undocumented**. **That is wrong, and I did not check before writing it.**
`docs/src/getting-started/installation.md` names all three published platforms in a table, says plainly
*"On any other platform — musl, Linux arm64, macOS Intel, Windows ARM — there is no fallback inside the
package"*, explains why `npm run build` cannot work from an installed copy, and lists three things that do
work. `README.md` carries the same for the CLI archives.

The documentation is the best-covered part of this. **The defect is that the software contradicts it.**

## 1. Summary

Two independent problems, of which the second is the serious one.

1. **npm ships three platform families; the same release builds five for the CLI and six for PyPI.** musl
   and Linux aarch64 are already built, from recipes already in this repository, in the same tag run.
2. **On an unsupported platform, `npm install` exits 0 in silence, and `require('mdka')` then blames npm for
   a bug that is not there** and prescribes a reinstall loop that cannot terminate.

## 2. Findings

### 2.1 npm is the outlier among the three channels

Measured against the `2.4.1` release:

| | Linux x64 gnu | Linux x64 musl | Linux arm64 gnu | Linux arm64 musl | macOS arm64 | macOS x64 | Windows x64 |
|---|---|---|---|---|---|---|---|
| **CLI archives** | yes | **yes** | no | **yes** | yes | no | yes |
| **PyPI wheels** | yes | **yes** | **yes** | **yes** | yes | no | yes |
| **npm** | yes | **no** | **no** | **no** | yes | no | yes |

Confirmed by querying the registry — every absent `@mdka/lib-*` package returns 404:

```
linux-x64-gnu  PUBLISHED   darwin-arm64  PUBLISHED   win32-x64-msvc  PUBLISHED
linux-x64-musl 404         darwin-x64    404         linux-arm64-gnu 404
linux-arm64-musl 404       win32-arm64-msvc 404
```

**This is not a capability gap.** `release-pypi.yaml` builds manylinux and musllinux for both x86_64 and
aarch64; `release-executable.yaml` builds both musl targets. Only `release-npm.yaml` was left at the
original three. All of them run on the same `release: [created]` trigger.

**macOS Intel is absent from all three channels**, consistently. §5 keeps it out of scope: it is one
decision across the whole project, not an npm question.

### 2.2 Install succeeds, silently, with nothing installed

Reproduced against the published `2.4.1`, resolving as Alpine:

```
$ npm i mdka@2.4.1 --os=linux --cpu=x64 --libc=musl
added 1 package in 423ms
$ ls node_modules/@mdka
(nothing — no native package at all)
```

Exit 0. No warning. This is correct npm behaviour: per-platform packages are `optionalDependencies`, and a
non-matching optional dependency is skipped by design. It also means **CI is green and the deploy fails.**

### 2.3 The error message is wrong, and its advice cannot work

```
$ node -e "require('mdka')"
Error: Cannot find native binding. npm has a bug related to optional dependencies
(https://github.com/npm/cli/issues/4828). Please try `npm i` again after removing
both package-lock.json and node_modules directory.
```

This is napi's generated boilerplate in `node/index.js:563`, emitted whenever no binding loads. For a
genuine npm resolution bug the advice is right. **Here there is no bug and nothing to re-resolve** — no
binary for this platform was ever built.

Three things follow, and they are why this outranks §2.1:

- It **names the wrong culprit.** A reader concludes npm is broken and mdka is fine.
- Its remedy is a **loop with no exit.** Delete `node_modules`, reinstall, fail identically, forever.
- It **contradicts our own documentation**, which explains this situation accurately. A user who hits the
  error follows the error, not the docs — and the error tells them to keep going.

A user who is told "this platform is not supported, see <link>" is served. A user told "npm has a bug" is
sent away from the answer we already wrote.

## 3. Two axes, deliberately separable

**Widening the matrix** (§2.1) and **telling the truth** (§2.2, §2.3) are independent. Widening never
reaches every platform — someone is always outside it — so the honesty work is required whatever is built.
It is also the cheap half. **If only one half ships, it should be this one.**

## 4. Proposal

### Part A — bring npm up to what the project already builds

Add three targets to `release-npm.yaml`'s matrix and to `node/package.json`'s `napi.targets`:

| Target | Package | Precedent already in this repo |
|---|---|---|
| `x86_64-unknown-linux-musl` | `@mdka/lib-linux-x64-musl` | `release-executable.yaml`, `release-pypi.yaml` |
| `aarch64-unknown-linux-gnu` | `@mdka/lib-linux-arm64-gnu` | `release-pypi.yaml` (manylinux aarch64) |
| `aarch64-unknown-linux-musl` | `@mdka/lib-linux-arm64-musl` | `release-executable.yaml`, `release-pypi.yaml` |

Three → six. **Cost is three more jobs per release, in parallel, on the release trigger only.** Nothing is
added to the seven workflows that run on every push. Pricing this per push would be the mistake made over
the CI cache; it is not that shape.

### Part B — own the error message

`node/index.js` is regenerated by `napi build`, so **it must not be hand-edited** — a patch would be silently
reverted at the next build. Instead, add a small committed wrapper that becomes `main`:

- `node/loader.js` requires `./index.js` inside `try`/`catch`.
- On success it re-exports, unchanged, including every named export.
- On failure it throws an error naming `process.platform`/`process.arch`/libc, listing the published
  platforms, and linking the installation page — attaching the original error as `cause` so a real npm
  resolution bug is still diagnosable.
- `package.json`: `main` → `loader.js`, and `loader.js` added to `files`.

The wrapper is regeneration-proof, needs no `postinstall` script, and does not fight the toolchain.

**Rejected:** adding `os`/`cpu` to the main package, which makes `npm install` itself fail. It cannot express
libc, and it breaks the ordinary practice of installing on one platform and deploying to another.

### Part C — not proposed, recorded as the alternative

A single `wasm32-wasi` target would cover **every** platform at reduced speed, and `index.js` already has the
fallback path wired for `@mdka/lib-wasm32-wasi`. It is one build, not three, and it ends the category.
Untested here: the crate's parallel feature under WASI, and the performance cost, would both need measuring
before it could be promised. Raised so the owner knows the option exists.

## 5. Not in scope

- **macOS Intel, and Windows ARM.** Absent from all three channels. One project-wide decision, separately.
- **Part C**, until measured.
- Any change to the conversion engine. This RFC changes no output.

## 6. Acceptance criteria

1. `@mdka/lib-linux-x64-musl`, `@mdka/lib-linux-arm64-gnu` and `@mdka/lib-linux-arm64-musl` are **published
   and resolvable on the registry**, at the release version, and are listed in `mdka`'s
   `optionalDependencies` — verified by querying the registry, not by a green workflow.
2. On **Alpine x64 and Linux arm64**, in a container, `npm install mdka` followed by
   `require('mdka').htmlToMarkdown('<p>hi</p>')` returns `hi`. Install-then-require, not install alone: an
   install that exits 0 is what this RFC is about.
3. On a platform that is still unsupported, the thrown message **names that platform**, lists the published
   ones, links the installation page, and **does not mention the npm bug** — asserted by a test that matches
   on the message text.
4. The original load error survives as `cause`, asserted.
5. Every existing named export is still exported through the wrapper — asserted by comparing the export list
   of `loader.js` against `index.js`, so a future generated export cannot be dropped silently.
6. `docs/src/getting-started/installation.md` and `README.md` list six npm platforms, and the sentence
   naming musl and Linux arm64 as unsupported is gone. **This documentation is currently correct; the change
   must keep it correct** — the only false statement about platforms in this project is the runtime one.
