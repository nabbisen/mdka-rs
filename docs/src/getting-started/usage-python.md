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

html = "<nav>menu</nav><h1>Title</h1><p>Body</p>"

# Strip nav/header/footer — useful for LLM pre-processing
md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.Minimal,
    drop_interactive_shell=True,
)

# Favour semantic structure — for SPAs and accessibility-aware output
md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.Semantic,
)
```

**Three** of the deprecated attribute options are accepted here and have **no
effect**: `preserve_classes`, `preserve_data_attrs` and `preserve_aria_attrs`.
Markdown has no attribute syntax to carry them into.

Passing any of the three emits a `DeprecationWarning`. By default that is only a
warning and the call succeeds — but **under warnings-as-errors it fails**:
`python -W error`, or pytest with `filterwarnings = error`, turns the call into
a raised `DeprecationWarning`. Remove the argument; it changes nothing. While
migrating, suppress it narrowly — for mdka's notices only, for one block:

```python
import warnings
import mdka

with warnings.catch_warnings():
    warnings.filterwarnings(
        "ignore",
        category=DeprecationWarning,
        message=r"mdka: `preserve_",
    )
    md = mdka.html_to_markdown_with("<p>x</p>", preserve_classes=True)
```

This still works under `python -W error`, and leaves every other library's
deprecation warnings raising as before.

The other two — `preserve_unknown_attrs` and `drop_presentation_attrs` — exist
on the Rust `ConversionOptions` but are **not exposed by this binding at all**.
Passing either raises:

```
TypeError: html_to_markdown_with() got an unexpected keyword argument
'preserve_unknown_attrs'
```

Use `mode` to influence the output.

Available modes: `ConversionMode.Balanced` (default), `Strict`, `Minimal`,
`Semantic`, `Preserve`.

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
    mode=mdka.ConversionMode.Minimal,
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

**mdka does not ship type information.** There is no `py.typed` marker and no
`.pyi` stubs, so a type checker treats every symbol below as `Any`. mypy will
say so directly:

```
error: Skipping analyzing "mdka": module is installed, but missing library
stubs or py.typed marker  [import-untyped]
```

That message is accurate, and it is better than the alternative. Every public
symbol is implemented in Rust and exposed through a compiled extension module,
which a type checker cannot read signatures from. Shipping a bare `py.typed`
would silence the warning without providing anything to check — the symbols
would still resolve as `Any`, and a genuinely wrong annotation would then pass
silently. Typed stubs are the real fix and are not written yet.

Until then, the signatures are:

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
