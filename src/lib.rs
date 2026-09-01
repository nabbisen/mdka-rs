//! HTML to Markdown converter - Lightweight and optimized library.
//!
//! Usage as CLI tool is in [`mdka-cli`](../mdka_cli/index.html).
//!
//! Full documentation: https://nabbisen.github.io/mdka-rs/
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
mod traversal;
mod utils;

use std::fs;
use std::io;
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
/// 1回のパース + 1回のトラバースで変換を完了する。
/// 前処理（タグ除外・アンラップ）はトラバース時にインライン実行される。
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

// ── バルクファイル変換 API（parallel フィーチャー） ─────────────────────────

/// 複数の HTML ファイルを rayon で並列変換し、`out_dir` へ書き出す（既定モード）。
#[cfg(feature = "parallel")]
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
///
/// Important: Unlike single-file conversion,
/// `out_dir` is **required** for bulk processing
/// to ensure a consistent and predictable output location for all generated files.
///
/// 出力先が衝突する入力は、入力順で最初のものだけが変換される。以降の衝突は
/// 変換を行わずエラーを返す（RFC 021: 衝突を検知せず最後に書いた者が勝つ挙動は、
/// 他の入力のデータを無言で失わせる競合状態だった）。
#[cfg(feature = "parallel")]
pub fn html_files_to_markdown_with<'a, P>(
    paths: &'a [P],
    out_dir: &Path,
    opts: &ConversionOptions,
) -> Vec<(&'a P, Result<PathBuf, MdkaError>)>
where
    P: AsRef<Path> + Sync,
{
    use rayon::prelude::*;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    // 変換を始める前に、入力順で出力先の衝突を検知する。最初の出現が勝ち、
    // 以降の同一出力先はここでエラー確定し、ファイル読み込みも書き込みも行わない。
    // rayon のワーカーがどちらを先に書き終えるかに結果が左右される競合状態を、
    // 検知を並列変換より前に置くことで断つ。
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

// ── 共通コア ───────────────────────────────────────────────────────────────

/// 入力パスと出力先ディレクトリから、書き出し先パスを一意に決定する。
/// 衝突検知（バルク変換）と実際の書き出し（`do_convert_file`）の両方から
/// 呼ばれる共通ロジック。ここが二箇所に分かれると、検知と書き込みが食い違い
/// うる（RFC 021）。
fn dest_path(src: &Path, out_dir: &Path) -> PathBuf {
    let stem = src.file_stem().unwrap_or_default();
    out_dir.join(stem).with_extension("md")
}

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
    let dest = dest_path(src, out_dir);
    fs::write(&dest, md)?;
    Ok(dest)
}
