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
    Ok(result)            => println!("→ {}", result.dest.display()),
    Err(MdkaError::Io(e)) => eprintln!("IO error: {e}"),
}
```

Because there is only one variant today, you can also use `?` directly:

```rust,no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = mdka::html_file_to_markdown("page.html", None::<&str>)?;
    Ok(())
}
```

## Bulk Conversion Errors

In `html_files_to_markdown`, each file fails independently.
A failed file does not abort the rest of the batch:

```rust,fragment
for (src, result) in mdka::html_files_to_markdown(&files, Path::new("out/")) {
    if let Err(e) = result {
        eprintln!("skipped {}: {e}", src);
    }
}
```

---

## Python errors

Python raises. `mdka.MdkaError` is an ordinary subclass of `Exception`.

| Call | On failure |
|---|---|
| `html_file_to_markdown`, `html_file_to_markdown_with` | **Raises `MdkaError`** when the file cannot be read or the output cannot be written. The message is `IO error: …` |
| `html_files_to_markdown`, `html_files_to_markdown_with` | **Raises `MdkaError`** only if the output directory cannot be created (`cannot create out_dir: …`). A **failing input file does not raise**: its entry comes back with `ok == False`, `dest is None` and `error` set, and the other files are unaffected |
| the string functions | No `MdkaError`. Passing a `str` where a `ConversionMode` belongs (`mode="balanced"`) is a **`TypeError`**: Python does not accept a mode as a string |

Each element of a bulk result is a `BulkConvertResult`:

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
| `htmlFileToMarkdown`, `htmlFileToMarkdownWith` | The promise **rejects** with an `Error` (`IO error: …`) when the file cannot be read or the output cannot be written |
| `htmlFilesToMarkdown`, `htmlFilesToMarkdownWith` | The promise **rejects** only if the output directory cannot be created (`cannot create out_dir: …`). A **failing input file does not reject**: its entry in the resolved array has `error` set and no `dest`, and the other files are unaffected |
| `htmlToMarkdownWith`, `htmlToMarkdownMany` | **Throw** an `Error` for an unknown `mode` string (`unknown conversion mode: …`) |
| `htmlToMarkdownWithAsync` | **Rejects** with the same message for an unknown `mode` |

```js,fragment
for (const r of await htmlFilesToMarkdown(files, 'out/')) {
  if (r.error) console.error(`${r.src}: ${r.error}`)
  else console.log(`${r.src} -> ${r.dest}`)
}
```

`ConvertResult` is the same shape for the single-file and the bulk functions: in
Node.js, `dest` and `error` are both optional, and exactly one of them is set.
