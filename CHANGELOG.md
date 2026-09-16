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

- **Project links on PyPI and crates.io.** The PyPI project page had no
  sidebar links at all. The Python package now declares Homepage,
  Documentation, Source and Changelog URLs. The `mdka` and `mdka-cli` crates
  now set `homepage` to the user guide at
  <https://nabbisen.github.io/mdka-rs/>; previously crates.io linked only the
  API reference on docs.rs.

### Changed

- **The Python package now requires CPython 3.10 or later, and ships one wheel
  per platform.** Until now, which Python versions got a wheel depended on the
  interpreters each build machine happened to have: 2.2.3 promised Python 3.8
  but had no 3.8 wheel for x86_64 Linux, and nothing below 3.11 for macOS. Each
  platform now gets a single wheel built on Python's stable ABI, which works on
  every CPython from 3.10 upward. That covers glibc and musl Linux on x86_64
  and aarch64, Windows x64 and macOS on Apple silicon. New CPython releases get
  a wheel from day one, instead of waiting for a release built on a machine
  that has them.

  - **On CPython 3.8 or 3.9**, pip does not offer this release and keeps
    installing 2.2.x. Nothing already installed stops working; those
    interpreters no longer receive new releases.
  - **Free-threaded CPython and PyPy wheels are no longer published.** On
    those interpreters `pip install mdka` builds from the source distribution,
    which needs a Rust toolchain.
  - **Source builds on free-threaded CPython now declare that they need the
    GIL**, so Python re-enables it on import, with a warning. The 2.2.3
    free-threaded wheels ran without the GIL, although the module had never
    been reviewed for that.

  The supported set is declared in `python/wheel-matrix.toml`. The release
  checks the built files against it before uploading, and a scheduled check
  compares it with what PyPI serves.

- **Deprecation messages now point to the documentation instead of an
  internal design record.** Setting one of the no-op attribute options
  (`preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`, and in
  Rust also `preserve_unknown_attrs` and `drop_presentation_attrs`) produced a
  warning ending *"see RFC 005"* — a reference a user could not follow. The
  Rust `#[deprecated]` note and the Node and Python `DeprecationWarning` text
  now link to <https://nabbisen.github.io/mdka-rs/api/options.html>.

  The start of the Node and Python message is unchanged — it still begins
  `` mdka: `<option>` `` — so a filter written against that prefix, including
  the narrow suppressions shown in the Python and Node guides, keeps working.
  The prefix is now covered by a test in each binding.

### Fixed

Conversion output — inline elements inside links, code and blockquotes. Output
changes for any document containing these shapes; each change is from Markdown
that did not say what the HTML said to Markdown that does.

- **An image, bold or italic text, or a code span inside a link now stays
  inside the link.** Previously it was written before the link, leaving an
  empty link or stray delimiters:

  | HTML | Before | Now |
  |---|---|---|
  | `<a href="/p"><img src="i.png" alt="pic"></a>` | `![pic](i.png)[](/p)` | `[![pic](i.png)](/p)` |
  | `<a href="/x"><strong>b</strong></a>` | `****[b](/x)` | `[**b**](/x)` |
  | `<a href="/x"><code>c</code></a>` | ` ``[c](/x) ` | `` [`c`](/x) `` |

- **Spaces inside a link are kept.** `<a href="/x">Read <strong>more</strong>
  now</a>` produced `****[Readmore now](/x)`, silently joining two words; it
  now produces `[Read **more** now](/x)`, the same text as outside a link. A
  space at the end of link text is kept after the link: `<a>x </a>y` →
  `[x](…) y`, previously `[x](…)y`.
- **A link with no text and no image is no longer emitted.** `[](/x)` renders
  as nothing. This includes the empty outer link left when HTML nests one
  `<a>` inside another.
- **`<pre>` without a `<code>` child produces a balanced code block.** Only the
  closing fence was written, so everything after it became code:
  `<pre>plain</pre><p>After</p>` gave `` plain\n```\n\nAfter ``; it now gives
  `` ```\nplain\n```\n\nAfter ``.
- **Code holds text only.** Inside any code — `<pre>`, `<pre><code>` or an
  inline `<code>` — bold, italic and links contribute their text, with no
  Markdown syntax, and an image contributes nothing:

  | HTML | Before | Now |
  |---|---|---|
  | `<code><strong>b</strong></code>` | `` `**b**` `` | `` `b` `` |
  | `<pre><code><b>kw</b> fn</code></pre>` | `**kw** fn` in the code block | `kw fn` |
  | `<pre><code><a href="/x">t</a></code></pre>` | `t[](/x)` in the code block | `t` |
  | `<pre><code><img src="i.png" alt="pic"></code></pre>` | `![pic](i.png)` in the code block | an empty code block |

  An inline code span with no text is no longer emitted as a stray ` `` `.
- **A `<pre>` is one code block.** Several `<code>` elements, or text beside
  them, used to write extra fences into the content:
  `<pre><code>a</code><code>b</code></pre>` gave a block containing
  `` a```\nb ``; it now gives one block containing `ab`. The language comes from
  the first `<code>` when only whitespace comes before it.
- **Indented `<code>` inside `<pre>` no longer breaks the code block.**
  Whitespace between `<pre>` and `<code>` was written in front of the fence. At
  four or more spaces the fence was not a fence, and the closing fence opened a
  code block that swallowed the rest of the document:

  | HTML | Before | Now |
  |---|---|---|
  | `<pre>\n    <code class="language-js">x</code>\n</pre><p>after</p>` | `` ␣␣␣␣```js\nx\n```\n\nafter `` — `after` ends up inside code | `` ```js\n␣␣␣␣x\n```\n\nafter `` |
  | `<pre>\n  <code class="language-js">x</code>\n</pre>` | `` ␣␣```js\nx\n``` `` | `` ```js\n␣␣x\n``` `` |

  The whitespace is now part of the code, as a browser shows it, so the
  two-space shape, which parsed correctly before, also changes: its spaces move
  inside the block. Other `<pre><code>` output is unchanged.
- **A blockquote that begins with bold, italic, code, a link or an image keeps
  its `>`.** `<blockquote><strong>b</strong> rest</blockquote>` gave
  `**b** rest`, with no quote at all; it now gives `> **b** rest`.
- With `preserve_ids`, the anchor for an inline `<code id="…">` is now placed
  before the code span rather than inside it, where it was literal text.

These documentation fixes are already live on the user guide, which publishes
from `main`; they are listed here so the release records them.

- **Five Rust examples in the guide did not compile** — two on *Usage — Rust*,
  two on *Core Functions*, one on *Error Handling*. Each used `?` without a
  fallible `main`. They now show a complete program, including
  `fn main() -> Result<…>`, so the code on the page and the code the **Copy**
  button gives you are the same, and both compile. (An intermediate fix hid
  that `main` with mdBook's hidden lines, which the Copy button leaves out.)
- **The README's conversion-modes links no longer depend on the site showing
  the README.** They pointed at an in-page `#conversion-modes` anchor, and
  GitHub, crates.io, npm and PyPI each generate heading ids their own way. They
  now link to the Conversion Modes page of the user guide.
- **Rust examples no longer show a Run button.** The Rust Playground does not
  provide `mdka`, so Run could only fail with an unresolved import, including
  on correct examples.
- **The Node.js "Async Conversion" example did not run.** It mixed `require()`
  with top-level `await` and used undefined variables. It now runs as written.
- **The Python "Conversion with Options" example did not run.** It used an
  undefined `html` and raised `NameError`. It now runs as written.
- **The TypeScript example did not compile in a project created with
  `tsc --init`.** It imported `JsConversionOptions` and `ConvertResult` as
  values, which `verbatimModuleSyntax` (on by default there) rejects with
  error TS1484. They are now type-only imports, which also compile under
  CommonJS and older ESM settings.
- **The deprecation warning is now documented.** The options pages said the
  no-op options "simply do nothing" and that existing calls "keep working". In
  fact each emits a deprecation warning, which fails the build or the call
  under `-D warnings`, `python -W error`, pytest `filterwarnings = error`, or
  Node `--throw-deprecation`. The Rust, Python and Node guides now say so and
  show a narrow suppression for use while migrating.
- **The HTML-table example stated the wrong output.** The markup as printed
  converts to `H1H2 ab`, not `H1H2ab`.
- **The "Not Yet Supported" table cited trackers a reader could not follow** —
  an internal finding ID and two RFC numbers with no corresponding document.
  It now links the roadmap.
- **The Installation page omitted the prebuilt binaries** that the README
  leads with. It now links them.

## [2.2.3] - 2026-09-16

### Added

- **`mdka --version`.** It previously failed with
  `error: IO error: No such file or directory`, because an unrecognised
  `-`-prefixed argument was treated as a filename. `-V` works too.

### Changed

- **The CLI now rejects unknown options instead of treating them as
  filenames.** `mdka --drop-shel page.html` used to exit 0 having converted
  nothing; it now prints `error: unknown option '--drop-shel'` with the usage
  text and exits 1.

  **This can break an existing invocation.** A file whose name begins with `-`
  was previously converted and is now rejected. Put `--` before it:

  ```
  mdka -- -weird.html
  ```

  A bare `-` is still passed through as a path, unchanged.

### Fixed

- **The README's Node.js Quick Start did not parse.** It declared `const md`
  twice, called `htmlToMarkdownWithAsync` without importing it, referenced an
  undefined `html`, and used top-level `await` in a CommonJS example. Its Rust
  and Python examples also used an undefined `html`. All of them now run as
  written. This is the file rendered on GitHub, crates.io, npmjs.com and PyPI.
- **README links and the logo now resolve outside GitHub.** The logo path was
  root-absolute and `./docs/`, `./CHANGELOG.md` and `./ROADMAP.md` are not in
  the published npm tarball, so all four were broken on npmjs.com and the PyPI
  project page. They are absolute URLs now, and CI fails if a relative link is
  reintroduced.
- **The Python and Node.js guides documented two options that do not exist.**
  `preserve_unknown_attrs` and `drop_presentation_attrs` are fields of the Rust
  `ConversionOptions` but are not exposed by either binding: Python raises
  `TypeError`, TypeScript reports `TS2353`. Three of the five deprecated
  attribute options are reachable from the bindings, and the pages now say
  which.
- **`mdka --help` no longer shows a multi-file example that fails.**
  `mdka --mode minimal --drop-shell *.html` needs `-o` for more than one input,
  two lines below the rule saying so. Fixed in `--help` and in the README.
- **`installation.md` no longer tells you to run `npm run build` on
  unsupported platforms.** The published npm package contains four files and no
  Rust source, so that could never work. It now names the three platforms with
  prebuilt bindings and what is actually possible elsewhere.
- **The README now carries two caveats it previously omitted**: that
  `Balanced`, `Strict` and `Preserve` currently produce identical output, and
  that tables are not yet converted.

## [2.2.2] - 2026-09-16

### Fixed

- **Bulk conversion no longer silently destroys files whose output names
  collide.** Converting two inputs that map to the same output — two
  `index.html` files from different directories, the most common case on the
  web — wrote one file and reported success for both, so one document's
  content was lost with no indication. Which one survived was a race between
  worker threads. Now the **first input in order wins**, deterministically, and
  every later collider returns an error naming both source paths and the
  contested destination. Nothing is written for a rejected input, and the
  process exits non-zero.

  **Not detected:** two paths differing only in case (`Index.html` vs
  `index.html`) on a case-insensitive filesystem. They are still treated as
  distinct.

### Changed

- **The CLI's `--help` output, the Rust API documentation, the TypeScript type
  definitions and the Python package docstring are now in English.** They were
  previously Japanese, which made the published reference unreadable for most
  users of a library whose documentation is otherwise English.

- **`mdka` no longer claims to ship type information for Python.** The
  documentation previously stated that a `py.typed` marker (PEP 561) was
  included. It was not, and adding a bare marker would have been worse than
  the gap: every symbol comes from a compiled extension module with no stubs,
  so a type checker would report *"Success: no issues found"* while treating
  every value as `Any` — turning an honest "I cannot check this" into a false
  "everything is fine". The page now states plainly that no type information
  ships, quotes the warning a user will see, and names typed stubs as the real
  fix.

- **The CLI no longer installs a counting allocator as its global allocator.**
  It existed to measure heap allocation for benchmarking, but every allocation
  on the hottest path — including the flagship parallel bulk-conversion path
  — paid for a counter nothing read at runtime. Removing it measured roughly
  8% faster on a 400-file, 1 MB-each bulk conversion (400 MB total, 32 cores,
  release build): ~356ms before, ~327ms after (medians of 5 runs). Output is
  byte-identical.

### Deprecated

- **`mdka::alloc_counter` is deprecated and will be removed in `2.4.0`.** It
  was never part of the conversion API: it exists only to let this project's
  own benchmarks (`benches/memory`) and measurement examples
  (`examples/quick_mem`, `examples/measure_mem`) count heap allocations, so
  there is no reason for a downstream crate to depend on it. The module
  remains public and functional in `2.2.2` — nothing breaks now — and the
  removal in `2.4.0` gives two releases and a full milestone of notice.

### Removed

- **The `jemalloc` Cargo feature.** It gated `tikv-jemallocator` and
  `tikv-jemalloc-ctl`, but no code in the crate ever referenced either
  dependency — enabling it changed nothing except build time and the
  dependency graph. A consumer building with `--features jemalloc` was
  already getting the system allocator; nothing they observe changes.

## [2.2.1] - 2026-09-01

### Fixed

- **`npm install mdka` now installs a working package.** Every 2.x release
  (`2.0.2` through `2.2.0`) published a main package whose manifest never
  carried `optionalDependencies`, and whose bundled `index.js` required
  unscoped per-platform packages (`mdka-<platform>`) that were never
  published past `1.6.9` — only the scoped `@mdka/lib-<platform>` packages
  were current. The result: `require('mdka')` threw `Cannot find module` for
  every npm consumer, on every platform, for the whole 2.x line. Root cause:
  `node/package.json` declared the scoped name under a nested
  `napi.package.name` key that `@napi-rs/cli` never reads, silently falling
  back to the unscoped root `name`. Fixed by using the flat `napi.packageName`
  key napi-rs actually reads, and by adding the missing `napi pre-publish`
  step to the npm release workflow, which publishes the per-platform packages
  and injects `optionalDependencies` into the main package's manifest.
- CI now packs and installs the actual npm tarball outside the checked-out
  workspace on every push, asserting the installed package converts HTML
  correctly. The previous `node` CI job tested the source tree directly,
  which cannot detect this class of defect: the compiled native binding is
  never in the published tarball's `files` list, so it passes regardless of
  whether the published package is installable.

## [2.2.0] - 2026-08-12

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

- **`unwrap_unknown_wrappers` is now reachable from every binding** — CLI
  `--unwrap-wrappers`, Node `unwrapUnknownWrappers`, Python
  `unwrap_unknown_wrappers`. It is one of only two options that has ever
  affected output, and it was exposed by no binding for the whole 2.x line,
  while three options that do nothing were exposed by all of them.

- **Node and Python emit a deprecation warning** when one of the no-op options
  is explicitly passed. Passing nothing stays silent. Known gap: the
  asynchronous Node entry points cannot emit the warning — `Env` is unavailable
  inside a `#[napi] async fn` — so check with the synchronous API; silence there
  is not proof that no deprecated option is in use.

### Fixed

- **`mdka-node`'s generated binding loader no longer carries a stale version.**
  `node/index.js` hardcoded `2.0.2` in its native-binding version checks while
  the package was at `2.1.8` — six releases out of step, shipped to npm that
  way. Any consumer setting `NAPI_RS_ENFORCE_VERSION_CHECK` got
  *"Native binding package version mismatch, expected 2.0.2"* on load, with a
  reinstall suggestion that could not help. `version.sh` now updates and
  asserts this file, so it cannot drift again.

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
