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
mdka --mode preserve -o archive/ *.html      # maximum fidelity
```

## All Options

This table mirrors `mdka --help`. If the two ever disagree, `--help` is the
truth — it is generated from the binary you are running.

| Flag | Description |
|---|---|
| `-o, --output <DIR>` | Output directory (defaults to the input's directory) |
| `-m, --mode <MODE>` | Conversion mode: `balanced` (default) · `strict` · `minimal` · `semantic` · `preserve` |
| `--preserve-ids` | Keep `id` attributes |
| `--preserve-classes` | **Deprecated, no effect.** Markdown has no attribute syntax |
| `--preserve-data` | **Deprecated, no effect.** Same reason |
| `--preserve-aria` | **Deprecated, no effect.** Same reason |
| `--drop-shell` | Drop `nav`, `header`, `footer`, `aside` |
| `--unwrap-wrappers` | **No effect today.** Unwraps `div`, `span`, `section`, `article`, `main` tags, keeping their content and separation — see [Conversion Options](../api/options.md#unwrap_unknown_wrappers) |
| `-h, --help` | Show this help |
| `-V, --version` | Show the version |
| `--` | End of options; everything after is a path |

An unrecognised `-`-prefixed argument is rejected rather than treated as a
filename. If you genuinely have a file whose name begins with `-`, put `--`
before it: `mdka -- -weird.html`.

The three deprecated flags are still accepted, so existing command lines keep
working, but they change nothing about the output. They are documented here
only so that you can recognise them; do not reach for them expecting an
effect. See [`ConversionOptions`](../api/options.md).

For full mode descriptions see [Conversion Modes](../api/modes.md).
