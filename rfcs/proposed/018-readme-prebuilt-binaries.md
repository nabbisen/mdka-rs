# RFC 018 — README Quick Start: prebuilt binaries

**Status.** Proposed
**Tracks.** Documentation. Raised by the project owner, 2026-08-12.
**Touches.** `README.md` — the Quick Start section only.
**Depends on.** Nothing. Independent of M2.

## Summary

Quick Start offers only `cargo install mdka-cli`, and states that Rust is
required. Five prebuilt binaries are published with every release and go
unmentioned, so a visitor without a Rust toolchain has no way to try the tool.

Add a short download-and-run path alongside the existing one.

## Motivation

The README currently opens Quick Start with:

> ### Try it from the command line
>
> `cargo` (Rust language) installed is required.

That is the first actionable instruction a visitor meets, and for anyone without
Rust it is a dead end — despite the project already building, archiving, and
attaching binaries for five targets on every release.

Per the project's own README structure rules, Quick Start is the *"guide for
immediate setup and usage."* Requiring a language toolchain before a first run is
not immediate.

### What is actually published

Confirmed from `release-executable.yaml` and the `2.1.7` release assets:

| Target | Asset |
|---|---|
| Linux x64 (glibc) | `mdka@Linux-x64-gnu-<version>.tar.gz` |
| Linux x64 (musl) | `mdka@Linux-x64-musl-<version>.tar.gz` |
| Linux aarch64 (musl) | `mdka@Linux-aarch64-musl-<version>.tar.gz` |
| macOS Apple Silicon | `mdka@macOS-aarch64-<version>.zip` |
| Windows x64 | `mdka@Windows-x64-<version>.zip` |

Archives place the binary at the archive root with no wrapping directory, so
extraction yields a runnable `mdka` directly.

## Goals

- A visitor with no Rust toolchain can run `mdka` from the README's first
  section.
- The platform coverage stated is accurate, including where it does **not**
  reach.
- Quick Start stays short. The project's rules require a concise README.

## Non-goals

- Restructuring the README or any other section.
- An install script, package-manager recipes, or checksums. Worth considering
  later; not this.
- Adding build targets — see §Finding.
- Duplicating `--help` output or the CLI reference. Those live in `docs/`.

## Proposed design

A short subsection **before** the existing `cargo install` path, since it is the
lower-friction route and the section is meant to get someone running quickly.

It should cover, in about as few lines as possible:

1. Where to get it — link to `/releases/latest`, **not** a versioned asset URL,
   which would go stale every release.
2. Which asset to pick — the five targets above, compactly.
3. That the archive extracts to a bare binary with no wrapper directory.
4. A first command, matching the existing example so the two paths converge.

The existing `cargo install` path stays, reworded so "Rust is required" attaches
to *that* option rather than to trying the tool at all.

### Be explicit about what is not covered

**macOS on Intel has no prebuilt binary** — only Apple Silicon is built. Anyone
on an Intel Mac following a download link will find nothing that runs, and the
README must say so and point them at `cargo install`.

Same for Windows on ARM and Linux aarch64 with glibc, though those are rarer.
One honest sentence covering "not listed here → use `cargo install`" is enough.

## Compatibility

None. Documentation only. No code, no API, no release artifact change.

The README ships inside the crates.io, npm, and PyPI packages, so the change
reaches those on the next publish. On GitHub it is visible as soon as it lands.

## Testing and verification

- Every asset name stated matches what `release-executable.yaml` produces —
  check the workflow's matrix, not a previous release's asset list, since the
  matrix is the source of truth.
- The `/releases/latest` link resolves.
- Quick Start still reads as a quick start: if it no longer fits on a screen,
  it has grown too far.

## Acceptance criteria

1. A visitor without Rust can run `mdka` following Quick Start alone.
2. All five targets listed, names matching the workflow matrix.
3. macOS Intel is explicitly noted as not covered, with `cargo install` as the
   fallback.
4. The `cargo install` path is retained, with the Rust requirement attached to
   it specifically.
5. No section other than Quick Start is modified.
6. `mdbook build` still succeeds — the README is not part of the book, but the
   docs workflow runs on the same push.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Asset names drift from the workflow matrix | README sends people to files that do not exist | Names taken from the matrix; the release-executable matrix is the single source. A future target change must update both — noted here so it is findable. |
| Quick Start grows unwieldy | Violates the project's concise-README rule | Cap it. Detail belongs in `docs/getting-started/installation.md`. |
| Versioned asset URLs used | Broken on every release | Link `/releases/latest` only |

## Finding — not fixed here

**There is no `x86_64-apple-darwin` build.** The matrix covers Apple Silicon
only, so Intel Mac users have no prebuilt binary at all.

Adding it is a one-line matrix entry, but it is a release-matrix decision — more
build minutes, another artifact to support — and belongs to the project owner,
not to a README change. Recorded here; not actioned.

The same reasoning applies to Windows ARM and Linux aarch64-gnu, both currently
unbuilt.

## Alternatives considered

| Option | Assessment |
|---|---|
| **Link to the releases page without listing targets** | Shorter, but leaves the visitor to guess which of five assets they need, and hides the macOS Intel gap. |
| **Put this in `docs/` only** | The docs already cover installation. But Quick Start is where a visitor lands first, and sending them elsewhere to find a binary defeats the section's purpose. |
| **Add an install script** | Convenient, and a real maintenance and security surface. Out of scope; revisit only if asked for. |
