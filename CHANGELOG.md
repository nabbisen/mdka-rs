# Changelog

All notable changes to this project are documented in this file. The format
loosely follows [Keep a Changelog](https://keepachangelog.com/), grouped by
release with newest first. Sections use `Added` / `Changed` / `Fixed` /
`Removed`; entries describe user-visible effect, not commit subjects.

This file was reconstructed on 2026-08-02 from git tags and commit history
(RFC 002). Where a version's intent could not be established from history with
confidence, that is stated explicitly rather than guessed.

## [2.7.0] - 2026-09-25

**`2.7.0` — one conversion fix, and a README that tells Windows and macOS users
what to type.** No API change: every export, signature and option is the same as
`2.6.0`. One conversion behaviour changes, described first.

### Fixed

- **Emphasis that opens a bold is no longer lost, and no longer leaves a
  literal `_`.** `<b><em>q</em>a</b>` — also `<b><i>q</i>a</b>` and
  `<strong><em>q</em>1</strong>` — converted to `**_q_a**`. CommonMark does not
  let an intraword `_` close, so that parses as `strong("_" "q_a")`: the italic
  is gone and an underscore is printed in the text. It is now `***q*a**`, which
  parses as `strong(em("q") "a")`, the structure the HTML had. Output for every
  shape that already worked is byte-identical to `2.6.0`, and the four
  conversion modes still produce identical output.
  The trigger is narrow: emphasis as the *first* child of a bold, closed
  immediately against a letter or digit. Measured over 424 bold openings on 42
  pages of published prose it was **zero** occurrences, so this is a real and
  silent defect that is rare in that kind of text; editor-generated HTML has
  not been measured.
  Two neighbouring shapes are **unchanged and still wrong**, and are recorded in
  the roadmap rather than hidden: `<b><em>q.</em>a</b>` cannot be fixed by
  choosing a delimiter, and `<b><em>q</em>a<em>r</em></b>` is fixable but is
  not fixed by this release.

### Changed

- **The README's prebuilt-binary Quick Start now has a block for each
  platform.** It had one Linux-only block ending in a POSIX pipeline into
  `./mdka`, which a Windows user, whose archive holds `mdka.exe`, could not
  follow. There are now Linux (`tar`), macOS (`unzip`) and Windows
  (PowerShell, `Expand-Archive`, `.\mdka.exe`) blocks, each showing the output
  to expect. Each is executed verbatim, on its own operating system, against
  the published archives (see below), so the instructions cannot drift from
  what the archives contain.

### Added

- **The release now has a gate on the GitHub Release archives.** Every other
  channel — npm, PyPI, crates.io — already had a gate that fetched what the
  registry actually serves; GitHub Releases had none. `release artifact gate`
  downloads the latest published release and, for each of the five archives,
  follows the README's own steps and runs the binary on a runner for its
  platform (Linux natively, `aarch64` musl under QEMU, macOS, Windows), then
  applies the artifact contract from `2.6.0`. It replaces a manual step that
  depended on who performed it. This changes nothing in what you install.

## [2.6.0] - 2026-09-25

**`2.6.0` — superscripts that no longer change the meaning, a CLI that starts on
older Linux, and Python types that are actually checked.** No API change: every
export, signature and option is the same as `2.5.1`. One conversion behaviour
changes, described first.

### Changed

- **`<sup>` and `<sub>` no longer silently convert to a different statement.**
  `10<sup>-9</sup>` produced `10−9`, which reads as a subtraction, and
  `2<sup>n - 1</sup>` produced `2n - 1`, which reads as multiplication. Measured
  over 417 occurrences on four real pages, **104 (25%) converted to something
  that meant something else**. Now:
  - more content reaches a real Unicode superscript or subscript — U+2212 MINUS
    SIGN, the full lowercase modifier-letter sets, and content wrapped only in
    `<i>`/`<em>`/`<b>`/`<strong>`/`<var>`, so `10<sup>-9</sup>` → `10⁻⁹` and
    `x<sup><i>n</i></sup>` → `xⁿ`;
  - what Unicode cannot express gets a visible marker instead of nothing:
    `2<sup>n - 1</sup>` → `2^(n - 1)`, `x<sub>y</sub>` → `x_(y)`;
  - **text that already delimits itself is untouched**, so citation markers
    (`<sup><a href="#c1">[1]</a></sup>`) convert exactly as they did — that is
    186 of the 417 measured occurrences.

  `docs/src/api/elements.md` states the three rules and lists which letters
  Unicode actually has: 25 of 26 for superscript, only 17 for subscript.

- **`--help` and the `ConversionMode` rustdoc say what the modes do.** They
  described fidelity the modes cannot deliver — `Preserve` claimed to keep "as
  much of the original as possible" and named a feature that does not exist.
  Wording only; no mode behaves differently.

### Fixed

- **The prebuilt Linux x64 CLI now starts on Ubuntu 20.04, Debian 11 and
  RHEL 8.** It was built against glibc 2.34 and failed to start on anything
  older, with nothing in the documentation saying so. It is now built against
  **glibc 2.17**, the same floor the npm bindings and Python wheels promise.
  The musl archives are statically linked and were never affected.

- **The Python package no longer makes a type checker certify wrong code.** It
  shipped a `py.typed` marker with no stub files, so `mypy --strict` reported
  *"Success: no issues found"* on code assigning a `str` to an `int`, passing an
  `int` where a `str` was required, and omitting a required argument. Without
  the marker mypy would have said *"missing library stubs"*, which was true — so
  a correct warning had been replaced with a false clearance. The wheel now
  ships `__init__.pyi` and `mdka_python.pyi` covering all thirteen public names,
  and CI runs `stubtest` against the built extension so the stubs cannot drift.

### Added

- **The release now asserts what is inside each artifact before publishing it.**
  A declared per-target contract — glibc floor, architecture, linkage, expected
  files — is checked against every built binary, and the publish stops if an
  artifact does not match. Nothing in this project inspected artifact *contents*
  before; that is how the glibc defect above stayed live through a dozen
  releases with every gate green.

### Note on an earlier entry

**The `2.5.1` entry below gives the wrong cause, and is corrected here rather
than rewritten.** It says the `2.5.0` npm failure was restricted default access
for new scoped packages. The `publishConfig.access: public` fix it describes was
real and is still correct to have, but it was **not** what fixed the failure.
The actual cause: npm trusted publishing is configured **per package**, and a
package that does not exist cannot hold that configuration, so CI could not
create one — npm answers `404` rather than `403` for a resource a credential
cannot reach. The three new packages had to be created by hand once. This is now
a named prerequisite in the release checklist.

## [2.5.1] - 2026-09-24

**`2.5.1` — the release `2.5.0` was supposed to be on npm.** `2.5.0` published
correctly to crates.io and PyPI; its npm publish failed partway and `mdka` on
npm stayed at `2.4.2`. Identical library code to `2.5.0`; the only change is one
field that lets the three new platform packages be created.

### Fixed

- **`2.5.0` could not create its new npm packages.** `napi pre-publish` runs a
  plain `npm publish` for each per-platform package, and a *new* scoped package
  defaults to restricted access, which this account cannot create — so the first
  brand-new package (`@mdka/lib-linux-x64-musl`) failed with a 404 on PUT and the
  publish stopped there. Existing packages were unaffected, which is why
  `@mdka/lib-linux-x64-gnu@2.5.0` went out and nothing else did. `node/package.json`
  now sets `publishConfig.access` to `public`, which napi copies into every
  generated per-platform manifest.

### Note for npm users

`2.5.0` does not exist on npm; take `2.5.1`, which is the same code. On
crates.io and PyPI, `2.5.0` and `2.5.1` are identical and either is fine.
`@mdka/lib-linux-x64-gnu@2.5.0` was published before the failure and is
referenced by nothing; it can be ignored.

## [2.5.0] - 2026-09-24

**`2.5.0` — npm reaches six platforms instead of three, and stops blaming npm
when it cannot load.** No API change: every export, signature and option is
unchanged, and conversion output is byte-identical to `2.4.2`.

### Added

- **Prebuilt npm bindings for Linux x64 musl, Linux arm64 glibc and Linux arm64
  musl**, joining the existing Linux x64 glibc, macOS Apple Silicon and Windows
  x64. Alpine and arm64 hosts no longer fall off the end of `npm install`. The
  CLI archives and the PyPI wheels already covered musl and arm64; npm was the
  channel left behind.

### Fixed

- **`require('mdka')` blamed npm for a bug that was not there.** On a platform
  with no prebuilt binary, `npm install` exits 0 — a native package that does
  not match the platform is skipped by design — and the load then failed with
  *"npm has a bug related to optional dependencies … try `npm i` again after
  removing package-lock.json and node_modules"*. There was no npm bug, the
  advice was a loop that could not terminate, and it contradicted the
  installation page, which described the situation correctly. The package entry
  point is now `loader.js`, which names the platform, lists the six published
  ones, links the installation page, and attaches the real per-path load errors
  as `error.cause`. A platform that *is* published but whose binary will not
  load gets a different message rather than being called unsupported
  (`MDKA_UNSUPPORTED_PLATFORM` / `MDKA_BINARY_LOAD_FAILED`).

- **The Linux x64 glibc binding required glibc 2.34**, because it was built
  natively on the CI runner, so it would not load on Ubuntu 20.04, Debian 11 or
  RHEL 8 — while the PyPI wheels for the same platform reach glibc 2.17. It is
  now cross-built against **glibc 2.17**, like the new arm64 glibc binding.
  Conversion output is byte-identical across 360 conversions; the only changes
  to the binary are the ones that follow from the older baseline.

### Documentation

- The installation page and `README.md` list six npm platforms, state the glibc
  floor, and describe what happens on an unsupported one. `macOS Intel` and
  `Windows ARM` remain unsupported and are still named as such.

## [2.4.2] - 2026-09-24

**`2.4.2` — a patch release that fixes an option documented as having no
effect, and corrects what `2.4.1` said about the four modes.** No public
signature, option default or `--help` output changes.

### Correction

`2.4.1` said of the `id`-anchor fix: *"An unwrapped wrapper now keeps the
anchor, so the four modes are identical as documented."* **They were not
identical.** `Balanced`, `Strict`, `Semantic` and `Preserve` still differed
whenever an unwrapped wrapper sat inside a table cell (below), and the
`2.4.1` release page, which links to this file frozen at that tag, cannot be
edited to say so. As of `2.4.2` the four agree everywhere we check, and what we
check is now a corpus — every input in it, in every mode — rather than
hand-picked fixtures. That is deliberately a statement about the evidence:
`2.4.1` claimed identity outright, and this entry does not repeat the form of
the claim it corrects.

### Fixed

- **An unwrapped wrapper in a table cell broke the row.** With
  `unwrap_unknown_wrappers` on — its default in `Semantic` and `Minimal`, and
  what `--unwrap-wrappers` turns on in any mode — two siblings in a cell, such
  as `<td><div>a</div><div>b</div></td>` or `<td><div>a</div><p>b</p></td>`,
  wrote a real blank line into the row instead of the cell's `<br>`. The table
  ended after `a`, and `b |` came out as body text with a literal pipe.
  The result is now `| a<br>b |` in all five modes. The option is documented as
  having no effect on output; it now has none here either. Inside a cell's own
  `<pre>` a wrapper still contributes nothing. Input without a wrapper in a
  table cell is byte-identical to `2.4.1`.

### Tests

- Two properties over a corpus that includes wrappers in `td`/`th`, beside a
  `<p>`, in `<li>`, in `<blockquote>` and in a cell's `<pre>`: flipping any one
  option documented as having no effect changes nothing in any mode, and
  `Balanced`, `Strict`, `Semantic` and `Preserve` agree byte for byte. Both
  fail on `2.4.1` and pass now. The `2.4.1` fixtures put one child in a cell,
  and this defect needs two.

### Documentation

- `node/README.md` pointed at `release-executable.yaml`, but the workflow that
  overwrites that file is `release-npm.yaml`; the link now points there.
- `docs/src/design/architecture.md` called `examples/` an "Allocation
  measurement tool"; it holds four helpers, two for allocation and two for
  speed.

## [2.4.1] - 2026-09-23

**`2.4.1` — a patch release, mostly to correct what `2.4.0` said about
itself.** The conversion engine changes only in the two output fixes below (an `id`
anchor `Semantic` dropped, and blank lines in a table cell's code block); the
release exists because statements published in `2.4.0` were false — its
README denied the release's headline feature and the four-modes-identical
claim did not hold — and a registry page can only be corrected by publishing.

### Fixed

- **`2.4.0`'s README said tables were not converted.** It read *"Tables are not
  yet converted … a table becomes a run of joined text"* — precisely the defect
  `2.4.0` fixed, live on crates.io, npm and PyPI. The README now says what
  happens: tables convert to GFM tables, and four shapes (more than one header
  row, `<th>` as each row's first cell, a nested `<table>`, a `<caption>`) fall
  back to one paragraph per cell. The documentation site was already correct.

- **`Semantic` did not match the other three modes on `id` anchors.** `Balanced`, `Strict`, `Semantic` and `Preserve` are
  documented as producing identical output, but `<main id="x"><p>hi</p></main>`
  gave `<a id="x"></a>` + `hi` in three of them and just `hi` in `Semantic`:
  a wrapper (`div`, `span`, `section`, `article`, `main`) that a mode unwraps
  lost its `id` anchor along with its tag, although `preserve_ids` asks for an
  anchor on every element that has an `id`. An unwrapped wrapper now keeps the
  anchor, so the four modes are identical as documented. *(Correction: this
  sentence was wrong — the four modes were still not identical. See
  [2.4.2](#242---2026-09-24).)* `Minimal` still emits
  none, because `preserve_ids` is off there. Wrappers without an `id`, and
  everything that is not a wrapper, are byte-identical to `2.4.0`. The
  characterisation tests that were cited as proving the modes identical put no
  `id` on a wrapper, which is why this shipped; they now do, and cover every
  wrapper tag, nesting, containers, table cells and shell elements.

- **A blank line inside a code block in a table cell corrupted the cell.** The
  blank line was written as ` `` `, which is a stray delimiter run rather than
  an empty code span; it paired with the next run, so the following line landed
  inside a code span it was never in and the `<br>` separators became literal
  text. A blank line is now a bare `<br>` between the lines' own code spans.
  Cells with no blank line are byte-identical. This affected real pages: the
  Wikipedia article for Markdown, whose syntax-reference table is what makes
  the page worth converting, was mangled in 24 cells.

- **`mdka page.html > out.md` leaves `out.md` empty, not holding the
  conversion.** `--help`, the CLI docs and the source claimed the opposite. A
  file argument writes `page.md` beside its input and prints only a progress
  line to stderr; only converting stdin (`… | mdka > out.md`) puts the Markdown
  on stdout. All three places now say what actually happens, and use `-o` to
  choose where the file goes.

## [2.4.0] - 2026-09-23

**If you convert documents containing tables, your output changes in this
release.** Tables that previously came out as a run of cell text now become
real Markdown tables. Nine of ten tables measured across five real pages
convert; the tenth, and three other shapes, fall back to one paragraph per
cell — readable and complete, but not a table. The **Added** and **Changed**
entries below say which. Peak memory is unchanged; conversion of table-heavy
documents does more work than before.

### Added

- **Tables convert to real Markdown tables.** `<table>` was the largest known
  gap: until now every cell's text ran together with no separator at all, so
  `<th>A</th><th>B</th><td>1</td><td>2</td>` became `AB12`. Tables now produce
  GFM tables, with column alignment carried over from `align` and
  `text-align`, and `|` inside a cell escaped so it cannot split the row.
  Measured across five real pages, 10 of 11 tables now convert, including all
  seven on the Wikipedia article for Markdown, which previously produced none.

  Four shapes have no Markdown expression and fall back to one paragraph per
  cell rather than producing a broken table: a table with more than one header
  row, one using `<th>` as each row's first cell instead of a header row, one
  containing a nested `<table>`, and one with a `<caption>`. **No table welds
  its cells any more, on either path.**

  Where a table does convert but a cell holds block content, the cell is
  flattened: paragraphs join with `<br>`, a list becomes `- one<br>- two`, a
  code block becomes one code span per line. The grid is kept; structure
  inside a cell is not. `colspan` and `rowspan` are expanded by repeating the
  spanned content across the cells it covered.

- **Strikethrough.** `<del>` and `<s>` become `~~text~~`. `<ins>` and `<u>`
  remain plain text — Markdown has no syntax for either, and `~~` would say
  the opposite of what `<ins>` means.

- **Task-list checkboxes.** A list item beginning with
  `<input type="checkbox">` becomes `- [x]` or `- [ ]`.

- **Superscript and subscript, where the characters exist.** `2<sup>7</sup>`
  now yields `2⁷`, `H<sub>2</sub>O` yields `H₂O`, `x<sup>n</sup>` yields `xⁿ`.
  Previously these lost the distinction entirely and produced `27`, which is a
  different number rather than merely unstyled text. Content that has no
  Unicode form — a citation marker such as `<sup><a>[1]</a></sup>`, or any
  letter outside the small mappable set — is left exactly as before.

- **`html_to_markdown_many` in Rust and Node**, converting a slice of HTML
  strings in one call, parallel when the `parallel` feature is on. Python had
  this already and now has `html_to_markdown_many_with` to pass options to it.

- **Python gains `html_file_to_markdown_with` and
  `html_files_to_markdown_with`.** The `_with` convention already existed for
  string conversion, so a reader could reasonably conclude that file
  conversion could not take options. It always could, as keyword arguments on
  the plain functions, which still work.

- **`mdka::version()` in Rust**, which Node and Python already had.

- **The Python wheel ships `py.typed`.** mypy no longer reports the package as
  untyped. There are still no `.pyi` stubs, so symbols resolve as `Any`.

- **`mdka --no-preserve-ids`.** `id` anchors are on by default in every mode
  except `minimal`, and there was previously no way to turn them off without
  changing mode — which also drops nav/header/footer and unwraps wrappers.
  `--help`'s description of `--preserve-ids` was also wrong: it emits
  `<a id="…"></a>` anchors rather than keeping attributes.

### Changed

- **Definition lists no longer run together.** `<dl><dt>Term</dt><dd>Desc</dd>`
  produced `TermDesc`; each `<dt>` and `<dd>` is now its own paragraph. Note
  what this does not do: Markdown has no definition-list syntax, so a long
  glossary becomes a run of paragraphs with nothing marking which term belongs
  to which description. We chose that over bolding the term or joining with a
  dash, either of which would invent structure the source did not have.

- **`Semantic` now produces the same output as `Balanced`.** Unwrapping a
  `<div>`, `<section>`, `<article>` or `<main>` used to remove the paragraph
  break along with the tag, so `<div>First.</div><div>Second.</div>` became
  `First.Second.` in `Minimal` and `Semantic`. Unwrapping now removes the tag
  and keeps the break — which is what the documentation always promised, and
  which leaves `unwrap_unknown_wrappers` with no observable effect today.
  **Four of the five modes — `Balanced`, `Strict`, `Semantic`, `Preserve` —
  now produce byte-identical output.** `Minimal` is the only one that still
  differs. The option and the modes are kept: a future mode that preserves raw
  HTML would make them differ again.

- **The `parallel` feature no longer changes which functions exist.** Building
  with `default-features = false` used to remove `html_files_to_markdown` and
  `html_files_to_markdown_with` entirely. The feature now only decides whether
  bulk conversion runs in parallel.

- **CLI: deprecation notices and per-file conversion progress now go to
  stderr, not stdout.** Until now, a deprecated flag's warning and the
  `in.html -> in.md` progress line both wrote to stdout alongside single-file
  stdin-to-stdout conversion. `mdka page.html > out.md` was fine, but `echo
  '<h1>Hi</h1>' | mdka --preserve-classes > out.md` silently mixed the
  deprecation notice into `out.md`, and any script piping `mdka`'s stdout
  further downstream could pick up progress noise it never asked for. Both
  now go to stderr; stdout carries only the converted Markdown (RFC 039 §3
  A7).

- **The performance page is measured again.** Its tables had been carried
  forward from `2.0.0`, measured in April on a machine that was never
  recorded, which made them impossible for anyone to reproduce or compare
  against. Every figure is now from one sitting on one machine, with the
  hardware, versions, commit and commands printed beside them. The page also
  states what its memory column measures — cumulative bytes allocated, not
  peak resident memory — and publishes depth-scaling data showing that the
  cost of deeply nested input is quadratic and lies almost entirely inside the
  HTML parser, before mdka's own traversal begins.

### Fixed

- **A list item whose content is indented no longer splits the list.** Any
  pretty-printed HTML — `<li>` followed by a newline and indented content, as
  every formatter and CMS produces — could end the list early: the following
  paragraph escaped the item, nesting flattened, and a code block inside the
  item was ejected from it.

- **An empty inline element no longer emits literal asterisks.**
  `a<b></b>b` produced `a****b`, and a paragraph containing only an empty
  `<b>` produced `****`, which reads as a horizontal rule.

- **Nested identical emphasis no longer inverts meaning.**
  `<em><em>x</em></em>` produced bold where the source said italic twice. The
  nesting order of mixed emphasis is now preserved as well, wherever Markdown
  can express it.

- **Degenerate nested list markers no longer become a horizontal rule.** Three
  or more empty nested list items emitted `- - -`, which reads as a thematic
  break; with a non-empty sibling, the content was destroyed outright. An
  `<hr>` as a list item's first content had the same effect.

- **An empty nested list no longer turns its parent's text into a heading.**
  `<li>y<ul><li></li></ul></li>` emitted `- y` followed by `  -`, which reads
  as a setext heading underline: the text became an `<h2>` and the sub-list
  disappeared.

### Removed

- **Breaking: `mdka::alloc_counter` is gone.** Deprecated in `2.2.2`, removed
  here as scheduled. It was never part of the conversion API — a
  benchmark-only counting allocator this project's own `benches/memory` and
  `examples/{quick_mem,measure_mem}` used to measure heap allocation — so
  removing it changes nothing for any code that calls `html_to_markdown` or
  any other conversion function. If your code did `use
  mdka::alloc_counter::...` directly, that import now fails to resolve; the
  same counting allocator lives on as an internal `benches/alloc_counter.rs`
  for this project's own use, not published as part of the crate's API. A
  crates.io reverse-dependency check (re-run the day of removal, not cited
  from the original two-releases-ago ruling) confirmed no dependent
  references it.

## [2.3.0] - 2026-09-22

### Added

- **Project links on PyPI and crates.io.** The PyPI project page had no
  sidebar links at all. The Python package now declares Homepage,
  Documentation, Source and Changelog URLs. The `mdka` and `mdka-cli` crates
  now set `homepage` to the user guide at
  <https://nabbisen.github.io/mdka-rs/>; previously crates.io showed the
  repository and, for `mdka`, the API reference on docs.rs, but no link to the
  user guide.

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

- **The `mdka::alloc_counter` deprecation warning now points to this
  changelog instead of an internal design record.** It ended *"See RFC 022"*,
  a reference a user could not follow; the 2.2.2 **Deprecated** entry below is
  the explanation.

- **Converting text-heavy HTML is about 9% to 14% slower than in 2.2.3.** On
  the project's benchmark documents, from a 12 KB page to a 1 MB one, a
  conversion takes 9% to 14% longer than it did in 2.2.3. Documents made
  almost entirely of list items or quotes can be a little slower still, up to
  about 15%. Deeply nested documents are slightly faster, and peak memory use
  is essentially unchanged: within about 3% on every input measured.

  The time buys output that is valid Markdown. In 2.2.3, code containing a
  fence line, a wrapper `<b>` around several paragraphs, a list item holding a
  paragraph, or a line of `~~~` could destroy content or leave it outside the
  list or quote it belongs to; the entries under **Fixed** list the cases.
  Doing that right means tracking where every line sits in a list or quote,
  and deciding character by character what would otherwise be read as
  Markdown.

  Recovering the speed is planned and is on the
  [roadmap](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md).

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
- **Escaping now depends on where text is written, so text reads back as it
  was.** The output bytes of most documents change: backslashes are removed
  where nothing needed them, moved to the character that needed them, and
  added where text was being read as Markdown. Several of the old outputs
  destroyed content:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<ul><li>1986. A great year</li></ul>` | `- 1986. A great year` — a nested list; the number is lost | `- 1986\. A great year` |
  | `<p>1986. A great year</p>` | `\1986. A great year` — the backslash shows | `1986\. A great year` |
  | `<p>~~~</p><p>after</p>` | `~~~` — opens a code block that swallows the rest | `\~\~\~` |
  | `<p>&lt;div&gt; hidden</p>` | `<div> hidden` — an HTML block; the text disappears | `\<div> hidden` |
  | `<p>&amp;copy; 2024</p>` | `&copy; 2024` — renders as `© 2024` | `\&copy; 2024` |
  | `<p>~~not struck~~</p>` | `~~not struck~~` — struck through on GitHub | `\~\~not struck\~\~` |
  | `<a href="/a b.html">x</a>` | `[x](/a b.html)` — not a link | `[x](</a b.html>)` |
  | `<a href="/x" title='say "hi"'>x</a>` | `[x](/x "say "hi"")` — not a link | `[x](/x 'say "hi"')` |
  | `<img src="i.png" alt="a]b">` | `![a]b](i.png)` — not an image | `![a\]b](i.png)` |

  Code is no longer escaped at all, and its delimiters grow instead:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `` <p><code>snake_case</code> and <code>a`b</code></p> `` | `` `snake\_case` and `a\`b` `` — backslashes in the code, and the second span broken | ```` `snake_case` and ``a`b`` ```` |
  | ```` <pre><code>a\n```\nb</code></pre> ```` | the ```` ``` ```` line closes the block early | a four-backtick fence |

  Plain text keeps fewer backslashes: `<p>snake_case and 2 * 3!</p>` gave
  `snake\_case and 2 \* 3\!` and now gives `snake_case and 2 * 3!`.
  Two touching bold or italic runs, `<strong>a</strong><strong>b</strong>`,
  gave `**a****b**`, which renders as literal asterisks; one of them is now
  written with underscores: `__a__**b**`. The rules are on the
  [Text Processing](https://nabbisen.github.io/mdka-rs/api/text-processing.html#markdown-escaping)
  page.
- **A line break (`<br>`) inside code no longer adds spaces to the code.** It
  was written as a Markdown hard break, two spaces and a newline, which inside
  code are literal characters: code copied from the output gained trailing
  spaces that the HTML never had. Inside a code block `<br>` is now a plain line
  break, and inside inline code a single space:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<pre><code>x<br>y</code></pre>` | `` ```\nx␣␣\ny\n``` `` — the code line is `x␣␣` | `` ```\nx\ny\n``` `` |
  | `<p><code>a<br>b</code></p>` | `` `a␣␣\nb` `` — the code reads `a␣␣␣b` | `` `a b` `` |

  `<br>` outside code is unchanged.
- **Paragraphs, quotes, lists and other blocks inside `<pre>` are code
  text.** `<pre>` is meant to hold text, but pages put blocks in it. Their
  markup was written into the code block, with only a closing fence, so the
  rest of the document became code. Now each block contributes its text, and a
  line break separates one block from the next:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<pre><p>one</p><p>two</p></pre><p>after</p>` | ``one\n\ntwo\n\n```\n\nafter`` — `after` is inside code | `` ```\none\ntwo\n```\n\nafter `` |
  | `<pre><blockquote><p>q</p></blockquote><ul><li>a</li></ul></pre><p>after</p>` | ``q\n\n- a\n\n```\n\nafter`` — `after` is inside code | `` ```\nq\na\n```\n\nafter `` |
  | `<ul><li><pre><p>a</p><p>b</p></pre></li></ul>` | ``- \n\na\n\nb\n\n``` `` | `` - ```\n␣␣a\n␣␣b\n␣␣``` `` |

- **A blockquote that begins with bold, italic, code, a link or an image keeps
  its `>`.** `<blockquote><strong>b</strong> rest</blockquote>` gave
  `**b** rest`, with no quote at all; it now gives `> **b** rest`.
- With `preserve_ids`, the anchor for an inline `<code id="…">` is now placed
  before the code span rather than inside it, where it was literal text.
- **Bold, italic and code around block content no longer write stray
  delimiters.** When `<strong>`/`<b>`, `<em>`/`<i>` or `<code>` contains a
  paragraph, heading, list, quote or code block, the delimiters were written
  around the blocks, on lines of their own. A lone `*` line even reads as an
  empty list item. The delimiters are now left out and the blocks kept:

  | HTML | Before | Now |
  |---|---|---|
  | `<strong><p>x</p><p>y</p></strong>` | `**\n\nx\n\ny\n\n**` | `x\n\ny` |
  | `<em><p>x</p><p>y</p></em>` | `*\n\nx\n\ny\n\n*` — two empty list items | `x\n\ny` |
  | `<b>text<p>para</p></b>` | `**text\n\npara\n\n**` | `text\n\npara` |
  | `<code><p>x</p><p>y</p></code>` | `` `\n\nx\n\ny\n\n` `` | `x\n\ny` |

  This is what every Google Docs paste of more than one paragraph produces:
  Google Docs wraps the copied content in `<b style="font-weight:normal">`.
- **A link around block content links each block.** `<a href="/x"><h2>Title</h2></a>`
  gave `## \n\n[Title](/x)` — an empty heading, then the text as a link after
  it. It now gives `## [Title](/x)`. Paragraphs, list
  items and quotes inside a link are each linked the same way; a code block
  inside a link is not linked, since code holds text only.
- **Bold or italic whose own style says it is not bold or italic is no longer
  emphasised.** Google Docs wraps a single-paragraph copy in a `<b>` whose own
  `style` is `font-weight:normal`, so the whole paste came out bold:
  `<b style="font-weight:normal"><span>Hello world</span></b>` gave
  `**Hello world**` and now gives `Hello world`.
  - This applies to `<b>`/`<strong>` whose own `style` sets `font-weight` to
    `normal` or a number up to 500, and `<i>`/`<em>` whose own `style` sets
    `font-style: normal`.
  - Only the element's own `style` attribute is read, and only those two
    properties. `lighter`, `bolder`, stylesheets and inherited styles do not
    count, and a `style` never adds emphasis: `<span style="font-weight:700">`
    stays plain.
  - Bold and italic without such a style are unchanged.
- **Paragraphs, code blocks, quotes and nested lists inside a list item now
  stay in the item.** Content wrapped in `<p>`, which WordPress and most CMSs
  produce for every list item, was written after an empty bullet, outside the
  list:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<ul><li><p>a</p></li><li><p>b</p></li></ul>` | `- \n\na\n\n- \n\nb` — two empty items | `- a\n- b` |
  | `<ul><li><p>a</p><p>b</p></li></ul>` | `- \n\na\n\nb` | `- a\n\n  b` |
  | `<ul><li>see<pre><code>x</code></pre></li></ul>` | ``- see\n\n```\nx\n``` `` — the code is outside the list | ``- see\n\n  ```\n  x\n  ``` `` |
  | `<ul><li><blockquote><p>q</p></blockquote></li></ul>` | `- \n\n> q` | `- > q` |

  Lines inside an item are indented to the item's content column. A list is
  loose (blank lines between items) only when some item holds two or more
  blocks; one `<p>` per item stays tight. The rule is on the
  [Supported HTML Elements](https://nabbisen.github.io/mdka-rs/api/elements.html#lists-tight-and-loose)
  page.
- **A list nested in a numbered item is nested.** It was indented by two
  spaces, which is less than `1. ` needs, so it read as a continuation of the
  parent's text:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<ol><li>one<ol><li>inner</li></ol></li></ol>` | `1. one\n  1. inner` — no sublist | `1. one\n   1. inner` |
  | `<ol><li>one<ul><li>inner</li></ul></li></ol>` | `1. one\n  - inner` — no sublist | `1. one\n   - inner` |

  Lists nested under `- ` items are unchanged.
- **A blockquote with several paragraphs, or with a multi-line code block, is
  one quote.** Blank lines between its blocks had no `>`, which ends a quote,
  and code lines after the first had none either, so the code left the quote:

  | HTML | 2.2.3 | Now |
  |---|---|---|
  | `<blockquote><p>one</p><p>two</p></blockquote>` | `> one\n\n> two` — two quotes | `> one\n>\n> two` |
  | `<blockquote><pre><code>l1\nl2</code></pre></blockquote>` | ``> ```\nl1\nl2\n``` `` | ``> ```\n> l1\n> l2\n> ``` `` |

  The same holds at any depth: quotes in list items, lists in quotes, and
  quotes in quotes. A line break inside a list item is now followed by the
  item's indent too (`- a  \n  b`, was `- a  \nb`); both read the same.

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
  provide `mdka`, so on the examples that use it, which is most of them, Run
  could only fail with an unresolved import, even on correct code. The button
  is now removed from every Rust example.
- **The Node.js "Async Conversion" example did not run.** It mixed `require()`
  with top-level `await` and used undefined variables. It now runs as written.
- **The Python "Conversion with Options" example did not run.** It used an
  undefined `html` and raised `NameError`. It now runs as written.
- **The TypeScript example did not compile in a project created with
  `tsc --init`.** It imported `JsConversionOptions` and `ConvertResult` as
  values, which `verbatimModuleSyntax` (on by default there) rejects with
  error TS1484. They are now type-only imports, which also compile under
  CommonJS and older ESM settings.
- **The deprecation warning is now documented.** The options page said the
  no-op options "simply do nothing", and the Python guide said existing calls
  "keep working". In fact setting one emits a deprecation warning, and in a
  strict setup that fails: a Rust build under `-D warnings` stops, and the
  Python call raises under `python -W error` or pytest
  `filterwarnings = error`. Under Node's `--throw-deprecation` the warning is
  thrown as an uncaught exception, which ends the process; a `try`/`catch`
  around the call does not see it. Node's `Async` functions cannot emit the
  warning at all, so silence from them does not mean that no deprecated option
  is in use. The options page (for Rust) and the Python and Node guides now say
  so, and show a narrow suppression for use while migrating.
- **The HTML-table example stated the wrong output.** It printed the markup
  across two lines but gave the result for one line: written that way it
  converts to `H1H2 ab`, not `H1H2ab`. The example now prints the markup on
  one line, which does convert to `H1H2ab`, and says what the two-line form
  gives.
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
