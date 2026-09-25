# API Reference

mdka has one core and two bindings, and they are not identical. The tables below
list **every exported function and type in all three** — Rust, Node.js and
Python — one row per *operation*, with each language's own name and an em dash
(—) where a language does not have it. The gaps are real, and they are the
reason for the layout.

This page describes the current surface. The Rust functions have reference pages
([Core Functions](./core.md), [Error Handling](./errors.md)); Node.js and Python
are documented in their [usage pages](../getting-started/usage-nodejs.md), and
their error behaviour is in [Error Handling](./errors.md#python-errors)
([Node.js](./errors.md#nodejs-errors)).

## Functions

| Operation | Rust | Node.js | Python |
|---|---|---|---|
| Convert one string (default mode) | [`html_to_markdown`](./core.md#html_to_markdown) | `htmlToMarkdown` | `html_to_markdown` |
| … with options | [`html_to_markdown_with`](./core.md#html_to_markdown_with) | `htmlToMarkdownWith` | `html_to_markdown_with` |
| … asynchronously | — | `htmlToMarkdownAsync` | — |
| … asynchronously, with options | — | `htmlToMarkdownWithAsync` | — |
| Convert many strings (default mode) | [`html_to_markdown_many`](./core.md#html_to_markdown_many) | `htmlToMarkdownMany` (options optional) | `html_to_markdown_many` |
| … with options | [`html_to_markdown_many_with`](./core.md#html_to_markdown_many_with) | — (`htmlToMarkdownMany` takes them) | `html_to_markdown_many_with` |
| Convert one file | [`html_file_to_markdown`](./core.md#html_file_to_markdown) | `htmlFileToMarkdown` (async) | `html_file_to_markdown` |
| … with options | [`html_file_to_markdown_with`](./core.md#html_file_to_markdown_with) | `htmlFileToMarkdownWith` (async) | `html_file_to_markdown_with` |
| Convert many files (parallel) | [`html_files_to_markdown`](./core.md#html_files_to_markdown) | `htmlFilesToMarkdown` (async) | `html_files_to_markdown` |
| … with options | [`html_files_to_markdown_with`](./core.md#html_files_to_markdown_with) | `htmlFilesToMarkdownWith` (async) | `html_files_to_markdown_with` |
| The library's version | [`version`](./core.md#version) | `version` | `version` |

That is **9** functions in Rust, **10** in Node.js and **9** in Python. What the
table shows:

- **Node.js has no `htmlToMarkdownManyWith`.** `htmlToMarkdownMany` takes an
  optional options argument instead of a second function.
- **Only Node.js has the `…Async` string functions.** Neither Rust nor Python
  has them.
- **Node.js's file functions are async-only** — they return a `Promise`. Rust's
  and Python's block until done.

## Types

| Type | Language | Description |
|---|---|---|
| [`ConversionMode`](./modes.md) | Rust, Python | Enum (Rust) / class (Python): `Balanced` · `Minimal`. Node.js has no such type: it passes the mode as a string in `mode` |
| [`ConversionOptions`](./options.md) | Rust | Controls pre-processing per call; built via `for_mode()` |
| `JsConversionOptions` | Node.js | The options object taken by every Node.js `…With` function and by `htmlToMarkdownMany`. Every field is optional: `mode` (a string), `preserveIds` and `dropInteractiveShell`. Python takes the same fields as keyword arguments instead |
| [`ConvertResult`](./core.md#convertresult) | Rust, Node.js, Python | **Rust and Python:** returned by the single-file functions, `src` + `dest`. **Node.js:** `src`, optional `dest`, optional `error`, and it is returned by the single-file function *and* by the bulk ones — for a bulk entry, `error` is set and `dest` absent when that file failed |
| `BulkConvertResult` | Python | One entry per file from `html_files_to_markdown[_with]`: `src`, `dest` (or `None`), `error` (or `None`), and `ok` |
| [`MdkaError`](./errors.md) | Rust, Python | Rust: the only error type, an enum whose one variant wraps `std::io::Error`. Python: the exception the file functions raise. Node.js has no error class: it rejects with a plain `Error` |

## Guarantees

**Everywhere — Rust, Node.js and Python:**

- **The string conversions accept any string** — empty, binary-looking, or deeply
  nested — **and never panic.** (In Node.js and Python a *bad option* is
  different: an unknown `mode` string throws or rejects in Node.js, and a `str`
  where a `ConversionMode` belongs is a `TypeError` in Python. That is a bad
  argument, not bad HTML.)
- **Output is always valid UTF-8.**
- **Output always ends with a single newline** when the input produces any content.

**Rust only:**

- **File functions report IO errors as a `Result<_, MdkaError>`.** The bindings do
  not: Python **raises** `MdkaError`, and Node.js **rejects** with an `Error` (or,
  for a failing file inside a bulk call, returns it in that entry's `error`). See
  [Error Handling](./errors.md).
