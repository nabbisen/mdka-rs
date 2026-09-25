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

**Four keyword arguments were removed in 3.0:** `preserve_classes`,
`preserve_data_attrs`, `preserve_aria_attrs` and `unwrap_unknown_wrappers`. None of
them ever changed the output. Passing one is now a `TypeError`, like any unknown
keyword:

```
TypeError: html_to_markdown_with() got an unexpected keyword argument
'preserve_classes'
```

Remove it. (`preserve_unknown_attrs` and `drop_presentation_attrs` were never
exposed by this binding.) The `_with` functions now take `mode`, `preserve_ids`
and `drop_interactive_shell`.

Available modes: `ConversionMode.Balanced` (default) and `ConversionMode.Minimal`.
`Strict`, `Semantic` and `Preserve` were aliases of `Balanced` and were removed in
3.0: `ConversionMode.Strict` is now an `AttributeError`. A mode is never a string
in Python — `mode="balanced"` is a `TypeError` — so there is no string form to
migrate. See [Conversion Modes](../api/modes.md).

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
# Returns the path that was written, as a str.
dest = mdka.html_file_to_markdown("page.html")
print(f"page.html → {dest}")

# Output to a specific directory
dest = mdka.html_file_to_markdown("page.html", "out/")

# With options
dest = mdka.html_file_to_markdown(
    "page.html",
    "out/",
    mode=mdka.ConversionMode.Minimal,
    drop_interactive_shell=True,
)
```

A single file **fails the call**: if it cannot be read or written it raises
`MdkaError` (`IO error: …`). There is no result object to inspect.

`html_file_to_markdown_with` is the same function under the name the
`_with` convention would predict, accepting the same keyword arguments.

## Bulk File Conversion

```python
import mdka

files = ["a.html", "b.html", "c.html"]
results = mdka.html_files_to_markdown(files, "out/")

# One FileOutcome per input, in input order: src, dest, error and ok — exactly
# one of dest and error is set, and ok says which.
for r in results:
    if r.ok:
        print(f"{r.src} → {r.dest}")
    else:
        print(f"Error: {r.src}: {r.error}")
```

**A failing file does not raise and does not stop the others**; its entry has
`ok == False`. `MdkaError` is raised only if the call as a whole cannot proceed —
the output directory cannot be created. That is the difference from
`html_file_to_markdown`, on purpose: with one file, failing the call is the answer;
with many, one bad file must not hide the rest. (`ConvertResult` and
`BulkConvertResult` were removed in 3.0; `FileOutcome` is the one result type.)

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

## Package Version

```python
import mdka

print(mdka.version())  # the installed version, e.g. "3.0.0"
print(mdka.__version__)  # the same string
```

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
rather than a copy in prose. Two things worth knowing about them:

- **They describe what the extension accepts, which is narrower than "anything
  path-like".** File paths are `str`, not `os.PathLike`: pass `str(path)`. The
  list arguments (`html_list`, `paths`) take any sequence of `str` — but a bare
  `str` is rejected at runtime, which a type checker cannot express.
- **They are checked against the built extension in CI** (`python -m
  mypy.stubtest mdka`), so a signature that changes in Rust without the stub
  changing fails the build rather than misleading a type checker.
