//! Python バインディング for mdka (PyO3 0.28)

use pyo3::prelude::*;
use rayon::prelude::*;

pyo3::create_exception!(mdka, MdkaError, pyo3::exceptions::PyException);

// ─── ConversionMode ────────────────────────────────────────────────────────

#[pyclass(from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConversionMode {
    Balanced = 0,
    Strict = 1,
    Minimal = 2,
    Semantic = 3,
    Preserve = 4,
}

#[pymethods]
impl ConversionMode {
    fn __repr__(&self) -> &'static str {
        match self {
            Self::Balanced => "ConversionMode.BALANCED",
            Self::Strict => "ConversionMode.STRICT",
            Self::Minimal => "ConversionMode.MINIMAL",
            Self::Semantic => "ConversionMode.SEMANTIC",
            Self::Preserve => "ConversionMode.PRESERVE",
        }
    }
}

fn to_rust_mode(m: ConversionMode) -> ::mdka::ConversionMode {
    match m {
        ConversionMode::Balanced => ::mdka::ConversionMode::Balanced,
        ConversionMode::Strict => ::mdka::ConversionMode::Strict,
        ConversionMode::Minimal => ::mdka::ConversionMode::Minimal,
        ConversionMode::Semantic => ::mdka::ConversionMode::Semantic,
        ConversionMode::Preserve => ::mdka::ConversionMode::Preserve,
    }
}

fn build_opts(
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> ::mdka::ConversionOptions {
    let mut opts = ::mdka::ConversionOptions::for_mode(to_rust_mode(mode));
    if let Some(v) = preserve_ids {
        opts.preserve_ids = v;
    }
    if let Some(v) = preserve_classes {
        opts.preserve_classes = v;
    }
    if let Some(v) = preserve_data_attrs {
        opts.preserve_data_attrs = v;
    }
    if let Some(v) = preserve_aria_attrs {
        opts.preserve_aria_attrs = v;
    }
    if let Some(v) = drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }
    opts
}

// ─── ConvertResult ────────────────────────────────────────────────────────

/// ファイル変換の結果。
///
/// Attributes:
///     src (str): 変換した入力ファイルのパス
///     dest (str): 書き出した出力ファイルのパス
#[pyclass(get_all)]
pub struct ConvertResult {
    pub src: String,
    pub dest: String,
}

#[pymethods]
impl ConvertResult {
    fn __repr__(&self) -> String {
        format!("ConvertResult(src={:?}, dest={:?})", self.src, self.dest)
    }
}

/// バルク変換の個別結果（成功・失敗を含む）。
///
/// Attributes:
///     src (str): 入力ファイルパス
///     dest (str | None): 出力ファイルパス（成功時）
///     error (str | None): エラーメッセージ（失敗時）
///     ok (bool): 変換成功か否か
#[pyclass(get_all)]
pub struct BulkConvertResult {
    pub src: String,
    pub dest: Option<String>,
    pub error: Option<String>,
}

#[pymethods]
impl BulkConvertResult {
    fn __repr__(&self) -> String {
        match &self.dest {
            Some(d) => format!("BulkConvertResult(src={:?}, dest={:?})", self.src, d),
            None => format!(
                "BulkConvertResult(src={:?}, error={:?})",
                self.src, self.error
            ),
        }
    }

    #[getter]
    fn ok(&self) -> bool {
        self.dest.is_some()
    }
}

// ─── 文字列変換 API ───────────────────────────────────────────────────────

#[pyfunction]
fn html_to_markdown(html: &str) -> String {
    ::mdka::html_to_markdown(html)
}

#[pyfunction]
#[pyo3(signature = (html, mode=ConversionMode::Balanced, preserve_ids=None,
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None))]
fn html_to_markdown_with(
    html: &str,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> String {
    let opts = build_opts(
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
    );
    ::mdka::html_to_markdown_with(html, &opts)
}

/// # The `html_to_markdown_many` function releases the Python GIL and utilizes `rayon`
/// to perform conversions in parallel across multiple CPU cores for maximum throughput.
#[pyfunction]
fn html_to_markdown_many(py: Python<'_>, html_list: Vec<String>) -> Vec<String> {
    py.detach(|| {
        html_list
            .par_iter()
            .map(|h| ::mdka::html_to_markdown(h))
            .collect()
    })
}

// ─── 単体ファイル変換 API ─────────────────────────────────────────────────

/// 単一の HTML ファイルを Markdown に変換する（既定モード: balanced）。
///
/// Args:
///     path (str): 入力 HTML ファイルのパス
///     out_dir (str | None): 出力ディレクトリ。None の場合は入力と同じディレクトリに出力
///
/// Returns:
///     ConvertResult: 変換結果（src, dest）
///
/// Raises:
///     MdkaError: 読み込み・書き出しに失敗した場合
///
/// Example:
///     >>> import mdka
///     >>> r = mdka.html_file_to_markdown("index.html")          # 同じディレクトリに出力
///     >>> r = mdka.html_file_to_markdown("index.html", "out/")  # 別ディレクトリに出力
///     >>> print(r.src, "->", r.dest)
#[pyfunction]
#[pyo3(signature = (path, out_dir=None, mode=ConversionMode::Balanced, preserve_ids=None,
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None))]
fn html_file_to_markdown(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<ConvertResult> {
    let opts = build_opts(
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
    );
    let out_dir_ref: Option<&str> = out_dir.as_deref();

    let result = py.detach(|| ::mdka::html_file_to_markdown_with(&path, out_dir_ref, &opts));

    result
        .map(|r| ConvertResult {
            src: r.src.to_string_lossy().into_owned(),
            dest: r.dest.to_string_lossy().into_owned(),
        })
        .map_err(|e| MdkaError::new_err(e.to_string()))
}

// ─── バルクファイル変換 API ───────────────────────────────────────────────

#[pyfunction]
#[pyo3(signature = (paths, out_dir, mode=ConversionMode::Balanced, preserve_ids=None,
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None))]
fn html_files_to_markdown(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    use std::path::Path;
    let out = Path::new(&out_dir);
    std::fs::create_dir_all(out)
        .map_err(|e| MdkaError::new_err(format!("cannot create out_dir: {e}")))?;

    let opts = build_opts(
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
    );
    let path_bufs: Vec<std::path::PathBuf> = paths.iter().map(std::path::PathBuf::from).collect();

    let results = py.detach(|| ::mdka::html_files_to_markdown_with(&path_bufs, out, &opts));

    Ok(results
        .into_iter()
        .map(|(p, res)| match res {
            Ok(dest) => BulkConvertResult {
                src: p.to_string_lossy().into_owned(),
                dest: Some(dest.to_string_lossy().into_owned()),
                error: None,
            },
            Err(e) => BulkConvertResult {
                src: p.to_string_lossy().into_owned(),
                dest: None,
                error: Some(e.to_string()),
            },
        })
        .collect())
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ─── モジュール登録 ────────────────────────────────────────────────────────

#[pymodule]
fn mdka_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("MdkaError", py.get_type::<MdkaError>())?;
    m.add_class::<ConversionMode>()?;
    m.add_class::<ConvertResult>()?;
    m.add_class::<BulkConvertResult>()?;
    m.add_function(wrap_pyfunction!(html_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(html_to_markdown_with, m)?)?;
    m.add_function(wrap_pyfunction!(html_to_markdown_many, m)?)?;
    m.add_function(wrap_pyfunction!(html_file_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(html_files_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
