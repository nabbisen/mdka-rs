# Error Handling

## MdkaError

```rust,fragment
#[derive(Error, Debug)]
pub enum MdkaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

`MdkaError` is the only error type in mdka. It has one variant, `Io`,
which wraps a `std::io::Error`.

IO errors arise from the file-based functions when:
- the input file does not exist or is not readable
- the output directory cannot be created
- the output file cannot be written

## Infallible Functions

`html_to_markdown`, `html_to_markdown_with`, `html_to_markdown_many` and
`html_to_markdown_many_with` **never fail**. They accept
any string and return a `String`. Malformed HTML, empty input, binary-looking
content, deeply nested structures — none of these cause a panic or an error.

## Pattern Matching

```rust
use mdka::{html_file_to_markdown, MdkaError};

match html_file_to_markdown("page.html", None::<&str>) {
    Ok(dest)              => println!("→ {}", dest.display()),
    Err(MdkaError::Io(e)) => eprintln!("IO error: {e}"),
}
```

Because there is only one variant today, you can also use `?` directly:

```rust,no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dest = mdka::html_file_to_markdown("page.html", None::<&str>)?;
    Ok(())
}
```

## Bulk Conversion Errors

In `html_files_to_markdown`, each file fails independently.
A failed file does not abort the rest of the batch, and each [`FileOutcome`](./core.md#fileoutcome)
names the file it belongs to:

```rust,fragment
for outcome in mdka::html_files_to_markdown(&files, Path::new("out/")) {
    if let Err(e) = outcome.result {
        eprintln!("skipped {}: {e}", outcome.src.display());
    }
}
```

That is the difference by arity, on purpose: **one file fails the call** (the `Err`
of `html_file_to_markdown`), **many report per file**. If the output directory cannot
be created, Rust reports it on **every** outcome, since creating it is part of
converting each file; the CLI and the bindings check it once, up front, and fail the
call instead (below).

---

## Python errors

Python raises. `mdka.MdkaError` is an ordinary subclass of `Exception`.

| Call | On failure |
|---|---|
| `html_file_to_markdown`, `html_file_to_markdown_with` | **Raises `MdkaError`** when the file cannot be read or the output cannot be written. The message is `IO error: …`. On success they return the destination path, as a `str` |
| `html_files_to_markdown`, `html_files_to_markdown_with` | **Raises `MdkaError`** only if the output directory cannot be created (`cannot create out_dir: …`). A **failing input file does not raise**: its entry comes back with `ok == False`, `dest is None` and `error` set, and the other files are unaffected |
| the string functions | No `MdkaError`. Passing a `str` where a `ConversionMode` belongs (`mode="balanced"`) is a **`TypeError`**: Python does not accept a mode as a string |

Each element of a bulk result is a `FileOutcome` — `src`, `dest`, `error` and `ok`,
and exactly one of `dest` and `error` is set:

```python,fragment
for r in mdka.html_files_to_markdown(files, "out/"):
    if r.ok:
        print(r.src, "->", r.dest)
    else:
        print("skipped", r.src, r.error)
```

## Node.js errors

Node.js has no error class: every failure is a plain `Error`, and a file
problem's message begins `IO error: `. Every file function is async, so a failure
is a **promise rejection**.

| Call | On failure |
|---|---|
| `htmlFileToMarkdown` | The promise **rejects** with an `Error` (`IO error: …`) when the file cannot be read or the output cannot be written. On success it resolves to the destination path, a `string` |
| `htmlFilesToMarkdown` | The promise **rejects** only if the output directory cannot be created (`cannot create out_dir: …`). A **failing input file does not reject**: its entry in the resolved array has `ok: false` and `error` set, and the other files are unaffected |
| `htmlToMarkdown`, `htmlToMarkdownMany` | **Throw** an `Error` for a bad option: an unknown `mode` string (`unknown conversion mode: …`), a mode removed in 3.0, or an unrecognised or removed option key (which the message names) |
| `htmlToMarkdownAsync` and the file functions | **Reject** with the same message for a bad option |

```js,fragment
for (const r of await htmlFilesToMarkdown(files, 'out/')) {
  if (r.ok) console.log(`${r.src} -> ${r.dest}`)
  else console.error(`${r.src}: ${r.error}`)
}
```

A bulk entry is a `FileOutcome`: `src`, `ok`, and exactly one of `dest` and `error`.
A single-file call has no such object — it resolves to the destination path or rejects
— which is the arity difference described above.
