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
| Convert one string (default mode) | [`html_to_markdown`](./core.md#html_to_markdown) | `htmlToMarkdown` (options optional) | `html_to_markdown` |
| … with options | [`html_to_markdown_with`](./core.md#html_to_markdown_with) | — (`htmlToMarkdown` takes them) | `html_to_markdown_with` |
| … asynchronously | — | `htmlToMarkdownAsync` (options optional) | — |
| Convert many strings (default mode) | [`html_to_markdown_many`](./core.md#html_to_markdown_many) | `htmlToMarkdownMany` (options optional) | `html_to_markdown_many` |
| … with options | [`html_to_markdown_many_with`](./core.md#html_to_markdown_many_with) | — (`htmlToMarkdownMany` takes them) | `html_to_markdown_many_with` |
| Convert one file | [`html_file_to_markdown`](./core.md#html_file_to_markdown) | `htmlFileToMarkdown` (async, options optional) | `html_file_to_markdown` |
| … with options | [`html_file_to_markdown_with`](./core.md#html_file_to_markdown_with) | — (`htmlFileToMarkdown` takes them) | `html_file_to_markdown_with` |
| Convert many files (parallel) | [`html_files_to_markdown`](./core.md#html_files_to_markdown) | `htmlFilesToMarkdown` (async, options optional) | `html_files_to_markdown` |
| … with options | [`html_files_to_markdown_with`](./core.md#html_files_to_markdown_with) | — (`htmlFilesToMarkdown` takes them) | `html_files_to_markdown_with` |
| The library's version | [`version`](./core.md#version) | `version` | `version` |

That is **9** functions in Rust, **6** in Node.js and **9** in Python. What the
table shows:

- **Node.js has no `…With` functions.** JavaScript has optional arguments, so
  `htmlToMarkdown(html, options?)` is the idiom, and each Node.js function takes
  its options optionally. Rust has no optional parameters, so a `_with` pair *is*
  the idiom there, and Python keeps its pair for consistency with itself.
- **Only Node.js has an async string function**, and only one: `htmlToMarkdownAsync`.
  Neither Rust nor Python has one.
- **Node.js's file functions are async-only** — they return a `Promise`. Rust's
  and Python's block until done.
- **What a file function returns differs by arity, on purpose.** One file returns
  the destination path and fails the call through the language's own channel;
  many files return one [`FileOutcome`](#types) each, so one failure cannot hide
  the rest. See [Error Handling](./errors.md).

## Types

| Type | Language | Description |
|---|---|---|
| [`ConversionMode`](./modes.md) | Rust, Python | Enum (Rust) / class (Python): `Balanced` · `Minimal`. Node.js has no such type: it passes the mode as a string in `mode` |
| [`ConversionOptions`](./options.md) | Rust | Controls pre-processing per call; built via `for_mode()` |
| `JsConversionOptions` | Node.js | The options object taken by every Node.js function except `version`. Every field is optional: `mode` (a string), `preserveIds` and `dropInteractiveShell`. **An unrecognised key is an error** that names it, and a key removed in 3.0 says so. Python takes the same fields as keyword arguments instead |
| [`FileOutcome`](./core.md#fileoutcome) | Rust, Node.js, Python | What happened to one file in a bulk conversion. **Rust:** `{ src: PathBuf, result: Result<PathBuf, MdkaError> }`. **Node.js and Python** cannot export a `Result`, so they flatten it: `src`, `dest`, `error` and `ok`, where exactly one of `dest` and `error` is set and `ok` says which |
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
