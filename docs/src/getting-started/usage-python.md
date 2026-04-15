# Usage — Python

## Installation

```bash
pip install mdka
```

## Basic Conversion

```python
import mdka

html = """
<h1>Hello</h1>
<p>mdka converts <strong>HTML</strong> to <em>Markdown</em>.</p>
"""

md = mdka.html_to_markdown(html)
print(md)
# # Hello
#
# mdka converts **HTML** to *Markdown*.
```

## Conversion with Options

```python
import mdka

# Strip nav/header/footer — useful for LLM pre-processing
md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.MINIMAL,
    drop_interactive_shell=True,
)

# Preserve ARIA attributes for accessibility-aware output
md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.SEMANTIC,
    preserve_aria_attrs=True,
)
```

Available modes: `ConversionMode.BALANCED` (default), `STRICT`, `MINIMAL`,
`SEMANTIC`, `PRESERVE`.

## Parallel Batch Conversion (GIL released)

`html_to_markdown_many` releases the GIL and uses rayon for parallel conversion:

```python
import mdka

pages = ["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"]
results = mdka.html_to_markdown_many(pages)
# ['# A\n', 'B\n', '- C\n']
```

This is faster than calling `html_to_markdown` in a Python loop for large batches.

## Single File Conversion

```python
import mdka

# Output to same directory: page.html → page.md
result = mdka.html_file_to_markdown("page.html")
print(f"{result.src} → {result.dest}")

# Output to a specific directory
result = mdka.html_file_to_markdown("page.html", "out/")

# With options
result = mdka.html_file_to_markdown(
    "page.html",
    "out/",
    mode=mdka.ConversionMode.MINIMAL,
    drop_interactive_shell=True,
)
```

## Bulk File Conversion

```python
import mdka

files = ["a.html", "b.html", "c.html"]
results = mdka.html_files_to_markdown(files, "out/")

for r in results:
    if r.ok:
        print(f"{r.src} → {r.dest}")
    else:
        print(f"Error: {r.src}: {r.error}")
```

## Error Handling

```python
import mdka

try:
    result = mdka.html_file_to_markdown("missing.html")
except mdka.MdkaError as e:
    print(f"Conversion failed: {e}")
```

`MdkaError` is raised for IO errors (file not found, permission denied, etc.).
`html_to_markdown` and `html_to_markdown_with` are always safe to call — they
never raise exceptions regardless of input quality.

## Type Annotations

mdka ships with a `py.typed` marker (PEP 561). All public symbols are annotated:

```python
from mdka import (
    html_to_markdown,          # (html: str) -> str
    html_to_markdown_with,     # (html: str, mode=..., **flags) -> str
    html_to_markdown_many,     # (html_list: list[str]) -> list[str]
    html_file_to_markdown,     # (path, out_dir=None, ...) -> ConvertResult
    html_files_to_markdown,    # (paths, out_dir, ...) -> list[BulkConvertResult]
    ConversionMode,            # enum
    ConvertResult,             # dataclass: src, dest (str)
    BulkConvertResult,         # dataclass: src, dest?, error?, ok
    MdkaError,                 # exception
)
```
