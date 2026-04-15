//! HTML to Markdown converter - Lightweight and optimized library.
//!
//! CLI ツールとしての使い方は [`mdka-cli`](../mdka_cli/index.html) を参照。
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

mod preprocessor;
mod renderer;
mod traversal;
mod utils;

#[doc(hidden)]
pub mod alloc_counter;

use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use options::{ConversionMode, ConversionOptions};

// ── エラー型 ───────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum MdkaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── 変換結果型 ─────────────────────────────────────────────────────────────

/// ファイル変換の結果。入力パスと出力パスを保持する。
#[derive(Debug, Clone)]
pub struct ConvertResult {
    /// 変換した入力ファイルのパス。
    pub src: PathBuf,
    /// 書き出した出力ファイルのパス。
    pub dest: PathBuf,
}

// ── 文字列変換 API ─────────────────────────────────────────────────────────

/// HTML 文字列を Markdown 文字列に変換する（既定モード: `balanced`）。
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

/// HTML 文字列を指定した [`ConversionOptions`] で Markdown に変換する。
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
pub fn html_to_markdown_with(html: &str, opts: &ConversionOptions) -> String {
    let document = scraper::Html::parse_document(html);
    let cleaned = preprocessor::preprocess(&document, opts);
    let cleaned_doc = scraper::Html::parse_document(&cleaned);
    traversal::traverse(&cleaned_doc)
}

// ── 単体ファイル変換 API ───────────────────────────────────────────────────

/// 単一の HTML ファイルを Markdown に変換する（既定モード: `balanced`）。
///
/// `out_dir` が `None` の場合は入力ファイルと同じディレクトリに
/// 拡張子を `.md` に変えて出力する。
///
/// # Example
///
/// ```rust,no_run
/// // 同じディレクトリに index.md を生成
/// let result = mdka::html_file_to_markdown("index.html", None::<&str>).unwrap();
///
/// // 別ディレクトリに出力
/// let result = mdka::html_file_to_markdown("index.html", Some("out/")).unwrap();
/// println!("{} -> {}", result.src.display(), result.dest.display());
/// ```
pub fn html_file_to_markdown(
    path: impl AsRef<Path>,
    out_dir: Option<impl AsRef<Path>>,
) -> Result<ConvertResult, MdkaError> {
    html_file_to_markdown_with(path, out_dir, &ConversionOptions::default())
}

/// 単一の HTML ファイルを指定した [`ConversionOptions`] で Markdown に変換する。
///
/// `out_dir` が `None` の場合は入力ファイルと同じディレクトリに出力する。
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

// ── バルクファイル変換 API ─────────────────────────────────────────────────

/// 複数の HTML ファイルを rayon で並列変換し、`out_dir` へ書き出す（既定モード）。
pub fn html_files_to_markdown<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
{
    html_files_to_markdown_with(paths, out_dir, &ConversionOptions::default())
}

/// 複数の HTML ファイルを指定した [`ConversionOptions`] で並列変換し `out_dir` へ書き出す。
pub fn html_files_to_markdown_with<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
{
    paths
        .par_iter()
        .map(|path| (path, do_convert_file(path.as_ref(), out_dir, opts)))
        .collect()
}

// ── 共通コア ───────────────────────────────────────────────────────────────

/// HTML ファイルを読み込み → 変換 → 書き出しする共通処理。
/// 単体変換・バルク変換の両方から呼ばれる。
fn do_convert_file(
    src: &Path,
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Result<PathBuf, MdkaError> {
    // out_dir が存在しない場合は自動作成する
    fs::create_dir_all(out_dir)?;
    let html = fs::read_to_string(src)?;
    let md = html_to_markdown_with(&html, opts);
    let stem = src.file_stem().unwrap_or_default();
    let dest = out_dir.join(stem).with_extension("md");
    fs::write(&dest, md)?;
    Ok(dest)
}
