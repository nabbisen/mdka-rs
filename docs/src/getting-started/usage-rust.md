# Usage — Rust

## Basic Conversion

```rust
use mdka::html_to_markdown;

fn main() {
    let html = r#"
        <h1>Getting Started</h1>
        <p>mdka converts <strong>HTML</strong> to <em>Markdown</em>.</p>
        <ul>
            <li>Fast</li>
            <li>Configurable</li>
            <li>Crash-resistant</li>
        </ul>
    "#;

    let md = html_to_markdown(html);
    println!("{md}");
}
```

Output:

```
# Getting Started

mdka converts **HTML** to *Markdown*.

- Fast
- Configurable
- Crash-resistant
```

## Conversion with Options

Use `html_to_markdown_with` to control the conversion pipeline via
[`ConversionOptions`](../api/options.md).

```rust
use mdka::{html_to_markdown_with};
use mdka::options::{ConversionMode, ConversionOptions};

// Strip navigation and extract body text — good for LLM input
let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
opts.drop_interactive_shell = true;

let html = r#"
    <header><nav><a href="/">Home</a></nav></header>
    <main>
        <article>
            <h1>Article Title</h1>
            <p>The main content of the page.</p>
        </article>
    </main>
    <footer>Copyright 2025</footer>
"#;

let md = html_to_markdown_with(html, &opts);
assert!(md.contains("# Article Title"));
assert!(!md.contains("Home"));       // nav removed
assert!(!md.contains("Copyright"));  // footer removed
```

## Converting Multiple Strings

```rust
use mdka::{html_to_markdown_many, html_to_markdown_many_with};
use mdka::options::{ConversionMode, ConversionOptions};

let pages = vec!["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"];
let results = html_to_markdown_many(&pages);
assert_eq!(results, vec!["# A\n", "B\n", "- C\n"]);

let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let results = html_to_markdown_many_with(&pages, &opts);
```

Each input is converted independently and the results keep the input order.
When the `parallel` feature is on (the default), the conversions run across
CPU cores via [rayon](https://crates.io/crates/rayon); with the feature off,
the same functions run sequentially instead.

## Converting a Single File

```rust,no_run
use mdka::html_file_to_markdown;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Output goes to the same directory as the input: page.html → page.md
    let result = html_file_to_markdown("page.html", None::<&str>)?;
    println!("{} → {}", result.src.display(), result.dest.display());

    // Output goes to a specific directory
    let result = html_file_to_markdown("page.html", Some("out/"))?;
    Ok(())
}
```

## Bulk Parallel Conversion

```rust,no_run
use mdka::html_files_to_markdown;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec!["a.html", "b.html", "c.html"];
    let out_dir = Path::new("out/");
    std::fs::create_dir_all(out_dir)?;

    for (src, result) in html_files_to_markdown(&files, out_dir) {
        match result {
            Ok(dest) => println!("{} → {}", src, dest.display()),
            Err(e)   => eprintln!("Error: {src}: {e}"),
        }
    }
    Ok(())
}
```

Conversion runs in parallel using [rayon](https://crates.io/crates/rayon).
The number of threads defaults to the number of logical CPU cores.

## Bulk Conversion with Options

```rust
use mdka::{html_files_to_markdown_with};
use mdka::options::{ConversionMode, ConversionOptions};
use std::path::Path;

let opts = ConversionOptions::for_mode(ConversionMode::Semantic);
let files = vec!["a.html", "b.html"];
let results = html_files_to_markdown_with(&files, Path::new("out/"), &opts);
```

## Conversion Modes at a Glance

| Mode | Best for |
|---|---|
| `Balanced` | General use; default |
| `Minimal` | LLM pre-processing, compression — the only mode that converts differently |
| `Strict`, `Semantic`, `Preserve` | Aliases of `Balanced`; kept for compatibility |

`Balanced`, `Strict`, `Semantic` and `Preserve` produce identical output and
cannot differ. See [Conversion Modes](../api/modes.md) for why.

## Error Handling

```rust
use mdka::{html_file_to_markdown, MdkaError};

match html_file_to_markdown("missing.html", None::<&str>) {
    Ok(result) => println!("→ {}", result.dest.display()),
    Err(MdkaError::Io(e)) => eprintln!("IO error: {e}"),
}
```

`MdkaError` currently has one variant: `Io`, wrapping `std::io::Error`.
`html_to_markdown` and `html_to_markdown_with` are infallible — they always
return a `String` and never panic on any input, no matter how malformed.

## Crate Version

```rust
println!("{}", mdka::version()); // e.g. "2.3.0"
```

Returns the crate's version as declared in `Cargo.toml`.
