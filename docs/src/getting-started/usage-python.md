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

# Balanced is the default: naming it is optional
md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.Balanced,
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

Available modes: `ConversionMode.Balanced` (default) and `Minimal`, which are the
two that convert differently. `Strict`, `Semantic` and `Preserve` are aliases of
`Balanced`, **deprecated since 2.8.0 and removed in 3.0**: naming one emits a
`DeprecationWarning` and the output is unchanged — see
[Conversion Modes](../api/modes.md).

## Parallel Batch Conversion (GIL released)

`html_to_markdown_many` releases the GIL and uses rayon for parallel conversion:

```python
import mdka

pages = ["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"]
results = mdka.html_to_markdown_many(pages)
# ['# A\n', 'B\n', '- C\n']
```

This is faster than calling `html_to_markdown` in a Python loop for large batches.

Use `html_to_markdown_many_with` to pass `mode` and the other conversion
options, the same keyword arguments `html_to_markdown_with` accepts:

```python
import mdka

pages = ["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"]
results = mdka.html_to_markdown_many_with(pages, mode=mdka.ConversionMode.Minimal)
```

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

`html_file_to_markdown_with` is the same function under the name the
`_with` convention would predict, accepting the same keyword arguments.

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

`html_files_to_markdown_with` is the same function under the `_with` name,
accepting `mode` and the other conversion options.

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

mdka ships type stubs (`mdka/__init__.pyi` and `mdka/mdka_python.pyi`) next to a
`py.typed` marker, so a type checker (checked with mypy) reports calls against
the real signatures:

```python,fragment
import mdka

n: int = mdka.html_to_markdown("<p>hi</p>")   # error: "str" is not assignable to "int"
mdka.html_to_markdown(42)                      # error: argument 1 must be "str"
mdka.html_file_to_markdown()                   # error: missing argument "path"
```

The stubs are the reference for every signature; read them, or ask your editor,
rather than a copy in prose. Three things worth knowing about them:

- **They describe what the extension accepts, which is narrower than "anything
  path-like".** File paths are `str`, not `os.PathLike`: pass `str(path)`. The
  list arguments (`html_list`, `paths`) take any sequence of `str` — but a bare
  `str` is rejected at runtime, which a type checker cannot express.
- **The three deprecated keywords are in the signature.** `preserve_classes`,
  `preserve_data_attrs` and `preserve_aria_attrs` are still accepted, have no
  effect and emit a `DeprecationWarning`; see [Conversion Options](../api/options.md).
  `preserve_unknown_attrs` is not accepted and is not in the stubs.
- **They are checked against the built extension in CI** (`python -m
  mypy.stubtest mdka`), so a signature that changes in Rust without the stub
  changing fails the build rather than misleading a type checker.
