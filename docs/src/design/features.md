# Features

## Crash Resistance

mdka uses non-recursive DFS traversal throughout. An explicit `Vec` stack
replaces the call stack, so documents with arbitrarily deep nesting will
not cause a stack overflow. This has been tested with 10,000 levels of
nested `<div>` elements.

Some fast converters use recursive tree traversal and will crash on
deeply nested input. If your input source is not fully controlled,
crash resistance matters.

## Five Conversion Modes

Rather than a single fixed conversion strategy, mdka offers five
named modes that tune the pre-processing pipeline:

- **Balanced** — readable output for general use
- **Strict** — maximum attribute retention for debugging
- **Minimal** — body text only; good for LLM input preparation
- **Semantic** — preserves ARIA and document structure
- **Preserve** — maximum fidelity for archiving

Each mode can be further customised with per-call option flags.
See [Conversion Modes](../api/modes.md) and [ConversionOptions](../api/options.md).

## Parallel File Conversion

`html_files_to_markdown` and `html_files_to_markdown_with` use
[rayon](https://crates.io/crates/rayon) to convert multiple files
in parallel. Each file's result is independent — one failed file does
not stop the batch.

The Node.js and Python bindings expose this as an async function
(`htmlFilesToMarkdown`, `html_files_to_markdown`) so the thread pool
work does not block the caller's event loop or hold the GIL.

## Multi-Language API

The same Rust implementation is accessible from three languages:

| Language | Package | Mechanism |
|---|---|---|
| Rust | `mdka` on crates.io | native library |
| Node.js | `mdka` on npm | napi-rs v3 native module |
| Python | `mdka` on PyPI | PyO3 v0 extension module |

All three call the same underlying conversion code and produce
identical output for identical input.

## html5ever Parser Foundation

The HTML parser is [scraper](https://crates.io/crates/scraper),
which is built on [html5ever](https://github.com/servo/html5ever).
html5ever implements the HTML5 parsing algorithm, the same one that
web browsers use.

This means:
- Missing closing tags are inferred correctly
- Unknown elements are preserved (not silently dropped)
- Malformed attribute syntax is normalised
- The result is always a valid DOM tree, no matter the input

## Predictable, Deterministic Output

For a given HTML input and `ConversionOptions`, mdka always produces
the same Markdown string. There is no randomisation, no date-stamping,
and no version-dependent output variation within a semver major version.

## Minimal Dependencies

The runtime dependencies of the `mdka` library crate are:

| Crate | Purpose |
|---|---|
| `scraper` | HTML parsing (html5ever wrapper) |
| `ego-tree` | DOM tree traversal |
| `rayon` | Parallel file conversion |
| `thiserror` | `MdkaError` derive macro |

Benchmark and comparison dependencies (`fast_html2md`, `criterion`) are
`[dev-dependencies]` and do not affect library consumers.
