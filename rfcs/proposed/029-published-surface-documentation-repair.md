# RFC 029 — Published-surface documentation repair

**Status.** Proposed
**Tracks.** M2c → `2.2.3` (patch)
**Priority.** P0 for the README items; P1 for the rest
**Touches.** `README.md`, `docs/src/getting-started/{usage-python,usage-nodejs,installation}.md`, `cli/src/main.rs`, `.github/workflows/scripts/check-docs-examples.py`.
**Source.** `2.2.2` consumer pass (RFC 027 Rule 1, first run). Disposition at `.git-exclude/reviewed/2.2.2-consumer-pass/README.md`.
**Prepared.** 2026-09-16

## Summary

The first consumer pass found that **`README.md` has never been in scope for any
documentation RFC**, and it contains broken code. Two binding pages document
options that raise exceptions. Fix those, and close the two gate blind spots
that let them through.

## Why a patch of its own

`README.md` renders on GitHub, crates.io, npm and PyPI. Its Node Quick Start
**does not parse**. `usage-python.md` tells users to pass a keyword argument that
raises `TypeError`.

Both are live, both are cheap, and neither should wait behind M3's renderer
work. The `2.2.1` precedent applies: user-facing harm ships alone and fast.

**Documentation and CLI-argument handling only. No engine change.**

## The defects, all reproduced

### 1 · README Node Quick Start does not parse — P0

```js
const { htmlToMarkdown, htmlToMarkdownWith } = require('mdka')
const md = htmlToMarkdown('<h1>Hello</h1>')
const md = await htmlToMarkdownWithAsync(html, { … })
```

Four defects: `const md` declared twice (`SyntaxError`), `htmlToMarkdownWithAsync`
used but not imported, `html` undefined, and top-level `await` in CommonJS.

`docs/src/getting-started/usage-nodejs.md` gets all of this right. **Only the
README is wrong**, and it is the version most people see.

### 2 · Two binding pages document options that do not exist — P0

`usage-python.md:46` says `preserve_unknown_attrs` and `drop_presentation_attrs`
are *"accepted but have no effect"*. Verified: neither appears in
`python/src/lib.rs`; passing either raises `TypeError`. `JsConversionOptions` has
seven fields and neither is among them, so TypeScript gives `TS2353`.

Three of the five deprecated options are reachable from the bindings. Two are
not. **The pages must say which.**

### 3 · `installation.md`'s source fallback is impossible — P1

*"On other platforms, run `npm run build`"*. The npm tarball contains four files
and no Rust source; `napi` is a devDependency. Users on musl, linux-arm64, macOS
Intel and Windows ARM have neither a prebuilt package nor a working fallback.
Say what is actually possible.

### 4 · README CLI example fails with two or more files — P1

`mdka --mode minimal --drop-shell *.html` needs `-o` for multiple inputs.
`mdka --help` repeats the same example two lines below the rule that forbids it.

### 5 · README logo path is root-absolute — P1

`![logo](/docs/src/assets/logo.png)` resolves only on GitHub. On npmjs.com and
PyPI it 403s. The same applies to the README's `./docs/` and `./CHANGELOG.md`
links, which do not exist in the published tarballs.

### 6 · `--version` errors, and unknown flags are treated as filenames — P1

`mdka --version` → `error: IO error: No such file or directory (os error 2)`.
This is audit finding `A-17`, which **was never scheduled** — an architect
omission. Add `--version`, and reject unknown `-`-prefixed arguments rather than
treating them as input paths.

### 7 · The README never surfaces its own most important caveat — P1

`api/modes.md` opens with *"Balanced, Strict and Preserve currently produce
identical output"* and calls it the page's most important fact. The README sells
five modes and "lossless archiving" with no hint of it. Same for the table
limitation, documented only in `api/elements.md`.

**The honesty is already written. It is filed where new users do not look.**

## Gate corrections — so this class cannot recur

**The docs-example gate's root must include `README.md`.** It defaults to
`docs/src`, which is why defect 1 was invisible to a gate built in the same
milestone to catch exactly that.

**The gate must check keyword arguments, not only symbol resolution.** Defect 2
is a kwarg; the gate resolves the function name and stops.

## Not in scope

The engine. F-04 (link text losing spaces) belongs to RFC 024; F-01/F-05/F-06 to
RFC 024 and RFC 010; tables to RFC 008. F-23 (`preserve_ids` emitting 1,190
anchors on a real page) is a design question shared with bekoedit's item 8, not
a defect. F-24 (quadratic deep-nesting) is recorded for M4.

## Acceptance criteria

1. Every code block in `README.md` executes as written.
2. The README's CLI examples run as written, including with multiple files.
3. The binding pages name exactly the options that exist, and say what the other
   two do.
4. `installation.md` describes a fallback that works, or says plainly there is
   none.
5. README image and link paths resolve **in the published npm and PyPI
   renderings**, verified against a built artifact, not on GitHub.
6. `mdka --version` prints the version; an unknown `-`-prefixed argument is
   rejected with a usable message.
7. The README carries the modes-identical and table caveats, or a pointed link
   to them.
8. The docs gate covers `README.md` and checks keyword arguments — **and is
   observed failing against the pre-fix README** before the fix lands.
9. No engine change: conversion output byte-identical, test count unchanged.
