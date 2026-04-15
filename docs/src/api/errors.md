# Error Handling

## MdkaError

```rust
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

`html_to_markdown` and `html_to_markdown_with` **never fail**. They accept
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

```rust
let result = mdka::html_file_to_markdown("page.html", None::<&str>)?;
```

## Bulk Conversion Errors

In `html_files_to_markdown`, each file fails independently.
A failed file does not abort the rest of the batch:

```rust
for (src, result) in mdka::html_files_to_markdown(&files, Path::new("out/")) {
    if let Err(e) = result {
        eprintln!("skipped {}: {e}", src);
    }
}
```
