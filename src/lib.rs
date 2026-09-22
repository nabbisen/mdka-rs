//! HTML to Markdown converter - Lightweight and optimized library.
//!
//! Usage as CLI tool is in [`mdka-cli`](../mdka_cli/index.html).
//!
//! Full documentation: <https://nabbisen.github.io/mdka-rs/>
//!
//! # Quick start
//!
//! ```rust
//! use mdka::{html_to_markdown, html_to_markdown_with};
//! use mdka::options::{ConversionMode, ConversionOptions};
//!
//! // default mode (balanced)
//! let md = html_to_markdown("<h1>Hello</h1>");
//! assert!(md.contains("# Hello"));
//!
//! // convert by specifying the mode
//! let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
//! let md = html_to_markdown_with("<h1>Hello</h1>", &opts);
//! assert!(md.contains("# Hello"));
//! ```

pub mod options;

mod renderer;
mod table;
mod traversal;
mod utils;

#[doc(hidden)]
#[deprecated(
    since = "2.2.2",
    note = "benchmark-only utility, never part of the conversion API; \
            scheduled for removal in 2.4.0. See \
            https://github.com/nabbisen/mdka-rs/blob/main/CHANGELOG.md"
)]
pub mod alloc_counter;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use options::{ConversionMode, ConversionOptions};

// ── Error type ─────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum MdkaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Conversion result type ─────────────────────────────────────────────────

/// Result of a file conversion: the input path and the output path.
#[derive(Debug, Clone)]
pub struct ConvertResult {
    /// Path of the input file that was converted.
    pub src: PathBuf,
    /// Path of the output file that was written.
    pub dest: PathBuf,
}

// ── String conversion API ──────────────────────────────────────────────────

/// Converts an HTML string to a Markdown string (default mode: `balanced`).
///
/// # Example
///
/// ```rust
/// let md = mdka::html_to_markdown("<h1>Hello</h1>");
/// assert!(md.contains("# Hello"));
/// ```
pub fn html_to_markdown(html: &str) -> String {
    html_to_markdown_with(html, &ConversionOptions::default())
}

/// Converts an HTML string to Markdown with the given [`ConversionOptions`].
///
/// The conversion completes in a single parse and a single traversal.
/// Preprocessing (tag exclusion and unwrapping) runs inline during that
/// traversal rather than as a separate pass.
///
/// # Example
///
/// ```rust
/// use mdka::options::{ConversionMode, ConversionOptions};
///
/// let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
/// let md = mdka::html_to_markdown_with(
///     "<nav><a href='/'>Home</a></nav><main><p>Content</p></main>",
///     &opts,
/// );
/// assert!(md.contains("Content"));
/// ```
///
/// Note:
/// This library builds a full DOM tree in memory using the `html5ever` parser before conversion.
/// While the traversal itself is stack-safe and non-recursive, memory consumption scales linearly with the input size.
/// For extremely large HTML files (e.g., 5MB+),
/// please be aware of the memory overhead compared to stream-based parsers like `lol_html``.
pub fn html_to_markdown_with(html: &str, opts: &ConversionOptions) -> String {
    let document = scraper::Html::parse_document(html);
    traversal::traverse(&document, opts)
}

/// Converts multiple HTML strings to Markdown (default mode: `balanced`),
/// each independently.
///
/// # Example
///
/// ```rust
/// let mds = mdka::html_to_markdown_many(&["<h1>A</h1>", "<h1>B</h1>"]);
/// assert!(mds[0].contains("# A") && mds[1].contains("# B"));
/// ```
pub fn html_to_markdown_many<S>(htmls: &[S]) -> Vec<String>
where
    S: AsRef<str> + Sync,
{
    html_to_markdown_many_with(htmls, &ConversionOptions::default())
}

/// Converts multiple HTML strings to Markdown with the given
/// [`ConversionOptions`], each independently. It cannot fail, so it returns
/// plain strings, not a result type (RFC 039 §3 A2). Parallel across CPU
/// cores when the `parallel` feature is on (the default); sequential
/// otherwise -- this function exists either way, only *how* it runs changes
/// (RFC 039 §2.4, §3 A2).
pub fn html_to_markdown_many_with<S>(htmls: &[S], opts: &ConversionOptions) -> Vec<String>
where
    S: AsRef<str> + Sync,
{
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        htmls
            .par_iter()
            .map(|h| html_to_markdown_with(h.as_ref(), opts))
            .collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        htmls
            .iter()
            .map(|h| html_to_markdown_with(h.as_ref(), opts))
            .collect()
    }
}

/// The crate's version, as declared in `Cargo.toml`. Node and Python have
/// long had their own; Rust callers had to reach for `CARGO_PKG_VERSION`
/// themselves (RFC 039 §2.5, §3 A5).
///
/// # Example
///
/// ```rust
/// assert!(!mdka::version().is_empty());
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ── Single-file conversion API ─────────────────────────────────────────────

/// Converts a single HTML file to Markdown (default mode: `balanced`).
///
/// When `out_dir` is `None`, the output is written next to the input file
/// with the extension changed to `.md`.
///
/// # Example
///
/// ```rust,no_run
/// // writes index.md into the same directory
/// let result = mdka::html_file_to_markdown("index.html", None::<&str>).unwrap();
///
/// // writes into a different directory
/// let result = mdka::html_file_to_markdown("index.html", Some("out/")).unwrap();
/// println!("{} -> {}", result.src.display(), result.dest.display());
/// ```
pub fn html_file_to_markdown(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
) -> Result<ConvertResult, MdkaError> {
    html_file_to_markdown_with(path, out_dir, &ConversionOptions::default())
}

/// Converts a single HTML file to Markdown with the given [`ConversionOptions`].
///
/// When `out_dir` is `None`, the output is written next to the input file.
pub fn html_file_to_markdown_with(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
    opts: &ConversionOptions,
) -> Result<ConvertResult, MdkaError> {
    let path = path.as_ref();
    let resolved_out_dir = match out_dir {
        Some(d) => d.as_ref().to_path_buf(),
        None => path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };
    let dest = do_convert_file(path, &resolved_out_dir, opts)?;
    Ok(ConvertResult {
        src: path.to_path_buf(),
        dest,
    })
}

// ── Bulk file conversion API ────────────────────────────────────────────────

/// Converts multiple HTML files, writing them to `out_dir` (default mode).
/// Parallel across CPU cores when the `parallel` feature is on (the
/// default); sequential otherwise -- this function exists either way (RFC
/// 039 §2.4, §3 A4): opting out of parallelism changes only how the work is
/// done, never which functions are available.
pub fn html_files_to_markdown<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
{
    html_files_to_markdown_with(paths, out_dir, &ConversionOptions::default())
}

/// Converts multiple HTML files with the given [`ConversionOptions`],
/// writing them to `out_dir`. Parallel when the `parallel` feature is on;
/// sequential otherwise -- see [`html_files_to_markdown`].
///
/// Important: Unlike single-file conversion,
/// `out_dir` is **required** for bulk processing
/// to ensure a consistent and predictable output location for all generated files.
///
/// When two inputs resolve to the same output path, only the first in input
/// order is converted; every later collision returns an error without
/// converting anything. Without this check, whichever worker wrote last would
/// win, silently destroying the other inputs' content -- a rule the
/// sequential path honours identically, not merely as a side effect of
/// running in order.
pub fn html_files_to_markdown_with<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
{
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    // Detect output-path collisions in input order, before any conversion
    // starts. The first occurrence wins; every later one resolves to an error
    // here and is never read from or written to. Deciding this ahead of the
    // parallel map is what removes the race -- otherwise the result depends on
    // which rayon worker happens to finish writing last.
    let mut claimed_by: HashMap<PathBuf, usize> = HashMap::with_capacity(paths.len());
    let rejections: Vec<Option<MdkaError>> = paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let dest = dest_path(path.as_ref(), out_dir);
            match claimed_by.entry(dest.clone()) {
                Entry::Vacant(e) => {
                    e.insert(i);
                    None
                }
                Entry::Occupied(e) => {
                    let first = paths[*e.get()].as_ref();
                    Some(
                        io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            format!(
                                "output path collision: '{}' and '{}' both resolve to '{}'; \
                                 the first occurrence in input order wins",
                                first.display(),
                                path.as_ref().display(),
                                dest.display(),
                            ),
                        )
                        .into(),
                    )
                }
            }
        })
        .collect();

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        paths
            .par_iter()
            .zip(rejections.into_par_iter())
            .map(|(path, rejection)| {
                let result = match rejection {
                    Some(err) => Err(err),
                    None => do_convert_file(path.as_ref(), out_dir, opts),
                };
                (path, result)
            })
            .collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        paths
            .iter()
            .zip(rejections)
            .map(|(path, rejection)| {
                let result = match rejection {
                    Some(err) => Err(err),
                    None => do_convert_file(path.as_ref(), out_dir, opts),
                };
                (path, result)
            })
            .collect()
    }
}

// ── Shared core ────────────────────────────────────────────────────────────

/// Determines the single output path for an input path and output directory.
/// Shared by collision detection (bulk conversion) and the actual write
/// (`do_convert_file`). Splitting this into two implementations would let
/// detection and writing disagree about where a file lands (RFC 021).
fn dest_path(src: &Path, out_dir: &Path) -> PathBuf {
    let stem = src.file_stem().unwrap_or_default();
    out_dir.join(stem).with_extension("md")
}

/// Reads an HTML file, converts it, and writes the result.
/// Shared by both single-file and bulk conversion.
fn do_convert_file(
    src: &Path,
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Result<PathBuf, MdkaError> {
    // create out_dir if it does not exist
    fs::create_dir_all(out_dir)?;
    let html = fs::read_to_string(src)?;
    let md = html_to_markdown_with(&html, opts);
    let dest = dest_path(src, out_dir);
    fs::write(&dest, md)?;
    Ok(dest)
}
