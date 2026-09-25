# Core Functions

This page is the **Rust** reference. Every Rust function below has a Node.js and
a Python counterpart, and some operations exist in only one binding — the table
on the [API Reference](./index.md) lists all three side by side.

## `html_to_markdown`

```rust,fragment
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

```rust,fragment
pub fn html_to_markdown_with(html: &str, opts: &ConversionOptions) -> String
```

Same as `html_to_markdown`, but accepts a [`ConversionOptions`](./options.md)
value that controls pre-processing and conversion behaviour.

**Input:** Any HTML string + a `ConversionOptions` value.  
**Output:** Markdown string.  
**Errors:** None.

```rust,fragment
use mdka::options::{ConversionMode, ConversionOptions};

let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
opts.drop_interactive_shell = true;
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## `html_to_markdown_many`

```rust,fragment
pub fn html_to_markdown_many<S>(htmls: &[S]) -> Vec<String>
where
    S: AsRef<str> + Sync,
```

Converts many HTML strings, each independently, with the default **`Balanced`**
mode. Equivalent to calling [`html_to_markdown`](#html_to_markdown) on each.

**Input:** A slice of anything that is `AsRef<str>` — `&[&str]`, `&[String]`.  
**Output:** A `Vec<String>` in the **same order** and of the **same length** as
the input. An empty slice gives an empty `Vec`.  
**Errors:** None — this function is infallible, so it returns plain strings and
not a `Result`.

The conversions run in parallel across CPU cores when the `parallel` feature is on
(the default), and sequentially otherwise. The function exists either way; only
*how* it runs changes.

```rust
let mds = mdka::html_to_markdown_many(&["<h1>A</h1>", "<h1>B</h1>"]);
assert_eq!(mds, vec!["# A\n", "# B\n"]);
```

---

## `html_to_markdown_many_with`

```rust,fragment
pub fn html_to_markdown_many_with<S>(htmls: &[S], opts: &ConversionOptions) -> Vec<String>
where
    S: AsRef<str> + Sync,
```

Same as `html_to_markdown_many`, but applies the given
[`ConversionOptions`](./options.md) to every string.

**Errors:** None.

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let mds = mdka::html_to_markdown_many_with(&["<p>a</p>", "<p>b</p>"], &opts);
assert_eq!(mds.len(), 2);
```

---

## `html_file_to_markdown`

```rust,fragment
pub fn html_file_to_markdown(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
) -> Result<PathBuf, MdkaError>
```

Reads one HTML file, converts it, and writes a `.md` file.

**`path`:** Path to the input `.html` file.  
**`out_dir`:**
- `None` → the `.md` file is written alongside the input (same directory, stem unchanged).
- `Some(dir)` → the `.md` file is written into `dir`. The directory is created automatically if it does not exist.

**Returns:** the path of the `.md` file that was written. The caller already has the input path, and a failure is the `Err`, so there is nothing else to return.  
**Errors:** `MdkaError::Io` if the file cannot be read or the output cannot be written. A single file fails *the call*; only [`html_files_to_markdown`](#html_files_to_markdown) reports per file.

```rust,no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // page.html → page.md in the same folder
    let dest = mdka::html_file_to_markdown("page.html", None::<&str>)?;

    // page.html → out/page.md
    let dest = mdka::html_file_to_markdown("page.html", Some("out/"))?;
    println!("wrote {}", dest.display());
    Ok(())
}
```

---

## `html_file_to_markdown_with`

```rust,fragment
pub fn html_file_to_markdown_with(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
    opts: &ConversionOptions,
) -> Result<PathBuf, MdkaError>
```

Same as `html_file_to_markdown`, but applies the given `ConversionOptions`.

---

## `html_files_to_markdown`

```rust,fragment
pub fn html_files_to_markdown<P>(paths: &[P], out_dir: &Path) -> Vec<FileOutcome>
where
    P: AsRef<Path> + Sync,
```

Converts multiple HTML files in parallel using [rayon](https://crates.io/crates/rayon).

**`paths`:** Slice of paths to input HTML files.  
**`out_dir`:** Directory for all output `.md` files. Created automatically if it does not exist, as with the single-file variants.  
**Returns:** a `Vec` of [`FileOutcome`](#fileoutcome) in the **same order** as `paths`. Each element is the outcome for one file independently: a failing file does not stop the others, and the outcome names the file it belongs to.

```rust,no_run
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec!["a.html", "b.html", "c.html"];
    std::fs::create_dir_all("out/")?;

    for outcome in mdka::html_files_to_markdown(&files, Path::new("out/")) {
        let src = outcome.src.display();
        match outcome.result {
            Ok(dest) => println!("{src} → {}", dest.display()),
            Err(e)   => eprintln!("{src}: {e}"),
        }
    }
    Ok(())
}
```

---

## `html_files_to_markdown_with`

```rust,fragment
pub fn html_files_to_markdown_with<P>(
    paths: &[P],
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Vec<FileOutcome>
where
    P: AsRef<Path> + Sync,
```

Same as `html_files_to_markdown`, but applies the given `ConversionOptions` to every file.

---

## `version`

```rust,fragment
pub fn version() -> &'static str
```

The crate's version, as declared in `Cargo.toml`.

**Errors:** None.

```rust
assert!(!mdka::version().is_empty());
```

---

## `FileOutcome`

```rust,fragment
pub struct FileOutcome {
    pub src:    PathBuf,
    pub result: Result<PathBuf, MdkaError>,
}
```

What happened to one file in a bulk conversion. `src` is the input path as it was
passed in; `result` is the path that was written, or the error that stopped that
file — exactly one of the two, which the type expresses.

The distinction behind it is **can it fail**, not one versus many. A string
conversion cannot fail, so it returns a string. A single file can, and you asked
about exactly one thing, so it fails the call: `html_file_to_markdown` returns the
destination path or an `Err`. Many files can each fail, and one failure must not
abort the batch, so each gets its own outcome.

The Node.js and Python bindings cannot export a Rust `Result`, so they flatten it
to `src`, `dest`, `error` and `ok`; see the [Types table](./index.md#types).
