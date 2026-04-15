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

| Flag | Description |
|---|---|
| `-o, --output <DIR>` | Output directory (default: same as input) |
| `-m, --mode <MODE>` | `balanced` · `strict` · `minimal` · `semantic` · `preserve` |
| `--preserve-ids` | Keep `id` attributes |
| `--preserve-classes` | Keep `class` attributes |
| `--preserve-data` | Keep `data-*` attributes |
| `--preserve-aria` | Keep `aria-*` attributes |
| `--drop-shell` | Remove `nav`, `header`, `footer`, `aside` |
| `-h, --help` | Show help |

For full mode descriptions see [Conversion Modes](../api/modes.md).
