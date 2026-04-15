# Core Functions

## `html_to_markdown`

```rust
pub fn html_to_markdown(html: &str) -> String
```

Converts an HTML string to Markdown using the default **`Balanced`** mode.

**Input:** Any valid or malformed HTML string. Empty strings are accepted.  
**Output:** A Markdown string. Always ends with `\n` if the input produced any content.  
**Errors:** None — this function is infallible.

```rust
let md = mdka::html_to_markdown("<h1>Hello</h1>");
assert_eq!(md, "# Hello\n");
```

---

## `html_to_markdown_with`

```rust
pub fn html_to_markdown_with(html: &str, opts: &ConversionOptions) -> String
```

Same as `html_to_markdown`, but accepts a [`ConversionOptions`](./options.md)
value that controls pre-processing and conversion behaviour.

**Input:** Any HTML string + a `ConversionOptions` value.  
**Output:** Markdown string.  
**Errors:** None.

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Minimal)
    .drop_interactive_shell(true);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## `html_file_to_markdown`

```rust
pub fn html_file_to_markdown(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
) -> Result<ConvertResult, MdkaError>
```

Reads one HTML file, converts it, and writes a `.md` file.

**`path`:** Path to the input `.html` file.  
**`out_dir`:**
- `None` → the `.md` file is written alongside the input (same directory, stem unchanged).
- `Some(dir)` → the `.md` file is written into `dir`. The directory is created automatically if it does not exist.

**Returns:** [`ConvertResult`](#convertresult) with the resolved `src` and `dest` paths.  
**Errors:** `MdkaError::Io` if the file cannot be read or the output cannot be written.

```rust
// page.html → page.md in the same folder
let r = mdka::html_file_to_markdown("page.html", None::<&str>)?;

// page.html → out/page.md
let r = mdka::html_file_to_markdown("page.html", Some("out/"))?;
println!("{} → {}", r.src.display(), r.dest.display());
```

---

## `html_file_to_markdown_with`

```rust
pub fn html_file_to_markdown_with(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
    opts: &ConversionOptions,
) -> Result<ConvertResult, MdkaError>
```

Same as `html_file_to_markdown`, but applies the given `ConversionOptions`.

---

## `html_files_to_markdown`

```rust
pub fn html_files_to_markdown<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
```

Converts multiple HTML files in parallel using [rayon](https://crates.io/crates/rayon).

**`paths`:** Slice of paths to input HTML files.  
**`out_dir`:** Directory for all output `.md` files. Must exist before calling (unlike single-file variants which create it automatically).  
**Returns:** A `Vec` of `(input_path, Result<output_path, error>)` pairs in the **same order** as `paths`. Each element represents the outcome for one file independently.

```rust
use std::path::Path;

let files = vec!["a.html", "b.html", "c.html"];
std::fs::create_dir_all("out/")?;

for (src, result) in mdka::html_files_to_markdown(&files, Path::new("out/")) {
    match result {
        Ok(dest) => println!("{} → {}", src, dest.display()),
        Err(e)   => eprintln!("{src}: {e}"),
    }
}
```

---

## `html_files_to_markdown_with`

```rust
pub fn html_files_to_markdown_with<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
```

Same as `html_files_to_markdown`, but applies the given `ConversionOptions` to every file.

---

## `ConvertResult`

```rust
pub struct ConvertResult {
    pub src:  PathBuf,
    pub dest: PathBuf,
}
```

Returned by the single-file functions. Both fields are absolute or relative paths
depending on how `path` was passed in.

> **Note:** The bulk functions (`html_files_to_markdown*`) return
> `(&P, Result<PathBuf, MdkaError>)` tuples rather than `ConvertResult`,
> because individual files within a batch may fail independently.
