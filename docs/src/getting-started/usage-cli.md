# Usage — CLI

The `mdka` command-line tool is provided by the `mdka-cli` crate.

## Quick Reference

```
mdka [OPTIONS] [FILE...]
```

Run `mdka --help` to see the full option list with descriptions.

## Common Patterns

**Convert from stdin:**
```bash
echo '<h1>Hello</h1>' | mdka
curl https://example.com | mdka
```

**Convert a single file** (output goes to the same directory):
```bash
mdka page.html          # → page.md
```

**Convert to a specific directory:**
```bash
mdka -o out/ page.html  # → out/page.md
```

**Bulk conversion** (`-o` is required for multiple files):
```bash
mdka -o out/ docs/*.html
```

**Choose a conversion mode:**
```bash
mdka --mode minimal --drop-shell page.html   # extract body text
mdka -o out/ *.html                          # balanced, the default
```

## All Options

This table mirrors `mdka --help`. If the two ever disagree, `--help` is the
truth — it is generated from the binary you are running.

| Flag | Description |
|---|---|
| `-o, --output <DIR>` | Output directory (defaults to the input's directory) |
| `-m, --mode <MODE>` | Conversion mode: `balanced` (default) · `minimal`. `strict`, `semantic` and `preserve` were removed in 3.0; asking for one is an error that says so |
| `--preserve-ids` | Emit `<a id="…"></a>` anchors for elements with an `id`. On by default in every mode except `minimal` |
| `--no-preserve-ids` | Turn anchor emission off, in any mode |
| `--drop-shell` | Drop `nav`, `header`, `footer`, `aside` |
| `-h, --help` | Show this help |
| `-V, --version` | Show the version |
| `--` | End of options; everything after is a path |

An unrecognised `-`-prefixed argument is rejected rather than treated as a
filename. If you genuinely have a file whose name begins with `-`, put `--`
before it: `mdka -- -weird.html`.

**Four flags were removed in 3.0:** `--preserve-classes`, `--preserve-data`,
`--preserve-aria` and `--unwrap-wrappers`. None of them ever changed the output.
Passing one now fails with exit status 1 and a single line on stderr that says so
— for example ``error: `--preserve-classes` was removed in 3.0; Markdown has no
attribute syntax, so it never changed the output. Remove it from the command
line: the output is the same without it.`` — so a script that still passes one
fails loudly instead of appearing to work. Remove the flag; nothing else changes.
Likewise `--mode strict`, `--mode semantic` and `--mode preserve` fail with
``error: conversion mode 'strict' was removed in 3.0; it was an alias of
'balanced'. Use 'balanced'.``

**Errors and per-file progress go to stderr, not stdout.** The
`in.html -> in.md` progress line printed for each converted file, and every error,
never mix into stdout. That is what makes the stdin form safe to redirect:
`echo '<h1>Hi</h1>' | mdka > out.md` leaves `out.md` holding only the converted
Markdown.

A file argument is a different mode: `mdka page.html` writes `page.md` beside
its input and prints only the progress line (to stderr), so stdout is empty.
`mdka page.html > out.md` therefore creates an **empty** `out.md` and the
conversion lands in `page.md`. To choose where the file goes, use `-o`.

For full mode descriptions see [Conversion Modes](../api/modes.md).
