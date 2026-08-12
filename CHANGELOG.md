# Changelog

All notable changes to this project are documented in this file. The format
loosely follows [Keep a Changelog](https://keepachangelog.com/), grouped by
release with newest first. Sections use `Added` / `Changed` / `Fixed` /
`Removed`; entries describe user-visible effect, not commit subjects.

This file was reconstructed on 2026-08-02 from git tags and commit history
(RFC 002). Where a version's intent could not be established from history with
confidence, that is stated explicitly rather than guessed.

## [Unreleased]

### Added

- **`preserve_ids` now emits anchors.** An element carrying a non-empty `id`
  produces an escaped `<a id="…"></a>` anchor, making `#fragment` links usable
  in the converted Markdown. Enabled by default in every mode except `Minimal`.
  The anchor is emitted as the element's leading content — `<h2 id="install">`
  becomes `## <a id="install"></a>Install`, a list item becomes
  `- <a id="b"></a>two`, and a paragraph inside a blockquote becomes
  `> <a id="p"></a>Q`. On `<a>` and `<pre>` the anchor precedes the element
  instead, so neither link text nor code content is disturbed. An `id` on a
  descendant of a link or a code block is deliberately not emitted.

  The `id` value is escaped for HTML attribute context (`&` → `&amp;`,
  `"` → `&quot;`). This is the engine's first construction of raw HTML from an
  input-derived value; previously input reached output only through Markdown
  link and image syntax.

### Changed

- **Heading text may now contain inline HTML.** A consequence of the above: with
  `preserve_ids` on and an `id` present, a heading renders as
  `# <a id="t"></a>Title`. Tools that read raw Markdown heading text — table of
  contents generators, for instance — will see the anchor markup. Tools that
  read rendered output are unaffected. Set `preserve_ids = false`, or use
  `Minimal` mode, to restore the previous output.

### Deprecated

- **`preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`,
  `preserve_unknown_attrs` and `drop_presentation_attrs` have no effect and are
  deprecated.** They never had an effect in any 2.x release. Markdown has no
  attribute syntax, so the behaviour they described was not expressible; the
  options were documented but never implemented. Nothing is removed and no
  output changes — this records what was already true. See RFC 005.

  Attribute preservation remains a legitimate feature. Some Markdown flavours
  (Pandoc, kramdown) do have attribute syntax. If it is wanted it will be
  designed deliberately rather than folded into repairing this.

## [2.1.8] - 2026-08-12

### Fixed

- **`<hr>` no longer swallows the newline before following content.**
  `<p>A</p><hr><p>B</p>` produced `"A\n\n---B\n"` instead of
  `"A\n\n---\n\nB\n"` whenever `<hr>` was not the first element in the
  document — `---B` is not a thematic break in CommonMark, so the rule was
  destroyed and the following text corrupted into it. Present in every 2.x
  release since `2.0.0`.
- **A `<pre>` code block's closing fence no longer swallows the newline
  before following content.** When the block's content ended in two or more
  trailing newlines, e.g. `<pre><code>x\n\n</code></pre><p>B</p>`, the
  output was `` "```\nx\n\n```B\n" `` — `` ```B `` is not a valid closing
  fence, so the code block never closed and the rest of the document was
  rendered as code. Present in every 2.x release since `2.0.0`.

### Changed

- crates.io publishing now runs in a CI workflow
  (`.github/workflows/release-crates.yaml`), gated on the released commit's
  CI having concluded `success` and authenticated via OIDC trusted
  publishing, matching how GitHub releases, npm, and PyPI already publish.
- `version.sh` now updates `[workspace.dependencies]` self-references
  automatically and refuses to complete a version bump that leaves any
  touched manifest carrying the previous version string, rather than
  silently succeeding on a half-applied update.

## [2.1.7] - 2026-08-02

M1 · Trustworthy baseline. No executable code changed in this release beyond
comments and lint-suppression attributes — see `Removed` and `Fixed` below for
the two internal-truth items with no user-visible behavior change, included
for the same reason the MSRV correction is: an honest record beats a silent
gap, even when nothing a consumer does differently as a result.

### Changed

- **MSRV corrected from 1.85 to 1.88.** The published minimum supported Rust
  version was never actually true for the 2.x line — `scraper` (both `0.26.0`
  and `0.27.0`) uses let-chains, stabilized in Rust 1.88, not 1.85. This is a
  documentation correction, not a behavior change: no user loses a capability
  they previously had, since 2.x never built on 1.85 in the first place.
  `Cargo.toml` now declares `rust-version = "1.88"`.

### Added

- Blocking CI workflow (`.github/workflows/ci.yaml`) running format, lint
  (`-D warnings`), build, and test on every push and pull request to `main`,
  plus a dedicated MSRV-verification job and Node.js/Python binding test jobs.
- Release-time CI verification: all three release workflows
  (`release-executable.yaml`, `release-npm.yaml`, `release-pypi.yaml`) now
  require the released commit's CI run to have concluded `success` before
  publishing, failing closed if no run is found.
- `NOTICE` file (Apache-2.0 attribution).
- This changelog.

### Fixed

- Pre-existing formatting and lint failures in `examples/quick_bench.rs`,
  `examples/quick_compare.rs`, and `benches/parallel.rs` that predated CI and
  had gone unnoticed.
- Corrected several documentation statements that no longer matched observed
  engine behavior: the architecture page described a five-step pipeline with
  an intermediate HTML serialization and a second parse, neither of which
  exists (the engine parses once and traverses once); HTML comments were
  documented as retained in `Preserve` mode, but are dropped in every mode;
  the always-removed element list omitted `<svg>` and `<head>`; and
  `drop_interactive_shell`'s prose contradicted its own options table and
  `src/options.rs`. No behavior changed — only the documentation was wrong.

### Removed

- A standalone, mode-aware DOM preprocessor (`tests/utils/preprocessor.rs`
  and its 115-line test file) that had never been compiled or reachable from
  any published artifact since 2.0.0 shipped — dead code the compiler could
  never warn about, because nothing ever built it. Its test assertions were
  transcribed into RFC 004's design record before removal, to inform future
  attribute-handling work. No behavior changed; nothing using `mdka` could
  ever have exercised this code.

## [2.1.6] - 2026-06-05

### Changed

- Updated dependencies across the workspace (Rust, Node.js, and Python
  manifests), including `scraper` `0.26` → `0.27`. No known behavior change;
  this bump was later confirmed (RFC 001) not to be the source of any MSRV
  regression — `scraper 0.26.0` already required the same Rust version.

## [2.1.5] - 2026-05-01

### Added

- Project logo, referenced from `README.md` and the documentation
  introduction.

### Changed

- Package metadata updates; copyright notice format standardized to
  start-year-only.

## [2.1.4] - 2026-04-24

### Fixed

- Python CI: the Windows job used a shell script incorrectly invoked; switched
  to `bash`.

## [2.1.3] - 2026-04-24

### Changed

- Python packaging migrated to `uv`.

### Fixed

- The `README.md` copy used by the PyPI package build was missing.

## [2.1.2] - 2026-04-23

### Changed

- PyPI publishing migrated to trusted publishing, with the upload step moved
  to `uv`.
- The `cargo publish` helper script was relocated.

## [2.1.1] - 2026-04-23

### Fixed

- The `README.md` copy used by the npm and PyPI package builds was missing.

### Changed

- Adjusted workspace package version handling in the version-bump tooling.

## [2.1.0] - 2026-04-22

### Changed

- Packaging, benchmarking configuration, the issue template, and version
  tooling only. **No library behavior change was identified in this release's
  commit range** (`2.0.3..2.1.0`) — the minor version increment reflects
  process changes, not a new feature. Recorded here explicitly rather than
  rationalized into a feature, per RFC 002.

## [2.0.3] - 2026-04-17

### Changed

- Documentation and README updates; core-library dependency housekeeping.

## [2.0.2] - 2026-04-16

### Fixed

- npm CI crashing ([#77](https://github.com/nabbisen/mdka-rs/issues/77)).
- CLI CI crashing ([#78](https://github.com/nabbisen/mdka-rs/issues/78)).
- PyPI publish step failing with "No files given, exiting."
  ([#79](https://github.com/nabbisen/mdka-rs/issues/79)).
- Windows CI misinterpreting a path beginning with `.` as a command.

### Changed

- Various release-workflow adjustments (working directories, `napi
  create-npm-dirs`, an `attest` option); documentation fixes.

## [2.0.1] - 2026-04-15

### Fixed

- `thiserror` dependency version.

### Changed

- Documentation and README updates; PyO3 binding version bump.

## [2.0.0] - 2026-04-15

### Added

- **Complete rewrite of the conversion engine.** Reconstructed from the diff
  of a single squashed commit
  ([#76](https://github.com/nabbisen/mdka-rs/pull/76), ~46,900 insertions
  across 122 files) rather than from release notes, since the commit's own
  message ("2.0.0 dev") conveys nothing on its own. At a theme level, this
  release introduced:
  - A new non-recursive (stack-based) DOM traversal engine.
  - The `ConversionMode` / `ConversionOptions` system (five modes).
  - The Cargo workspace split into a lean library crate (`mdka`) plus separate
    `cli/`, `node/` (napi-rs), and `python/` (PyO3) crates.
  - `criterion` benchmarks and example programs.
  - mdBook-based documentation under `docs/`.
- The remaining commits in this release's range are documentation and CI/
  release-workflow setup for the rewrite (GitHub Pages docs deploy, napi
  command fixes, cache-directory fixes), not additional library changes.

This is recorded as the 1.x → 2.0.0 transition. It is a major rewrite; no
attempt is made here to map individual 1.x behaviors to their 2.0.0
equivalents.

## [1.x] - 2024-01-06 to 2026-04-01 (1.0.0 through 1.6.9)

Consolidated per RFC 002 — per-patch reconstruction of the 1.x line's 60
tagged releases was not undertaken; RFC 002 requires only a line-level summary
here. Themes present in the commit history across this line, in rough
chronological order:

- Initial HTML-element and text-processing support, including semantic
  elements, `<audio>`, and inline formatting.
- Several conversion-output fixes: preserving trailing whitespace/newlines in
  text nodes, switching italic emphasis markers from `*` to `_`, supporting
  the language class on `<code>` blocks.
- A CLI executable added, with its own CI pipeline.
- Python bindings added via PyO3; Node.js bindings added via napi-rs
  (`1.5.0`).
- Ongoing MSRV and dependency churn: the minimum Rust version was raised then
  reverted at least once during this line, and the napi-rs major version was
  bumped then temporarily reverted for a publish issue.
- A security advisory fix ([RUSTSEC-2025-0020](https://rustsec.org/advisories/RUSTSEC-2025-0020)).
- Frequent packaging, README, and CI maintenance commits, especially around
  crates.io/npm/PyPI publishing setup.

**Undetermined:** which specific 1.x patch releases shipped which of the above
changes is not reconstructed here — the line moved through 60 tags, many same
day, and RFC 002 does not require per-patch precision for this line.

## [0.x and earlier] - 2024-01-04 to 2024-01-06

Pre-1.0 history (`v0.0.6` through `0.5.1`). Per RFC 002, pre-1.0 history needs
no reconstruction beyond this line: this was the project's initial
implementation period, with all tags landing within a three-day span.
