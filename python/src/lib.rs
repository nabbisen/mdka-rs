//! Python bindings for mdka (PyO3 0.28)

use pyo3::prelude::*;
use rayon::prelude::*;

pyo3::create_exception!(mdka, MdkaError, pyo3::exceptions::PyException);

// ─── ConversionMode ────────────────────────────────────────────────────────

#[pyclass(from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConversionMode {
    Balanced = 0,
    Minimal = 2,
}

#[pymethods]
impl ConversionMode {
    fn __repr__(&self) -> &'static str {
        match self {
            Self::Balanced => "ConversionMode.BALANCED",
            Self::Minimal => "ConversionMode.MINIMAL",
        }
    }
}

fn to_rust_mode(m: ConversionMode) -> ::mdka::ConversionMode {
    match m {
        ConversionMode::Balanced => ::mdka::ConversionMode::Balanced,
        ConversionMode::Minimal => ::mdka::ConversionMode::Minimal,
    }
}

fn build_opts(
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> ::mdka::ConversionOptions {
    let mut opts = ::mdka::ConversionOptions::for_mode(to_rust_mode(mode));
    if let Some(v) = preserve_ids {
        opts.preserve_ids = v;
    }
    if let Some(v) = drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }
    opts
}

// ─── ConvertResult ────────────────────────────────────────────────────────

/// Result of a file conversion.
///
/// Attributes:
///     src (str): path of the input file that was converted
///     dest (str): path of the output file that was written
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

/// Result for one file in a bulk conversion, successful or not.
///
/// Attributes:
///     src (str): input file path
///     dest (str | None): output file path, on success
///     error (str | None): error message, on failure
///     ok (bool): whether the conversion succeeded
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

// ─── String conversion API ───────────────────────────────────────────────

#[pyfunction]
fn html_to_markdown(html: &str) -> String {
    ::mdka::html_to_markdown(html)
}

#[pyfunction]
#[pyo3(signature = (html, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_to_markdown_with(
    html: &str,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<String> {
    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
    Ok(::mdka::html_to_markdown_with(html, &opts))
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

/// RFC 039 Half A: `html_to_markdown_many` had no way to pass options at all
/// -- the only conversion function in the project that could not be
/// configured. Same parallel-batch behaviour, with `_with`'s established
/// keyword-argument shape.
#[pyfunction]
#[pyo3(signature = (html_list, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_to_markdown_many_with(
    py: Python<'_>,
    html_list: Vec<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<Vec<String>> {
    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
    Ok(py.detach(|| {
        html_list
            .par_iter()
            .map(|h| ::mdka::html_to_markdown_with(h, &opts))
            .collect()
    }))
}

// ─── Single-file conversion API ──────────────────────────────────────────

/// Converts a single HTML file to Markdown (default mode: balanced).
///
/// Args:
///     path (str): path of the input HTML file
///     out_dir (str | None): output directory; when None, the output is
///         written next to the input file
///
/// Returns:
///     ConvertResult: the conversion result (src, dest)
///
/// Raises:
///     MdkaError: if reading or writing fails
///
/// Example:
///     >>> import mdka
///     >>> r = mdka.html_file_to_markdown("index.html")          # same directory
///     >>> r = mdka.html_file_to_markdown("index.html", "out/")  # another directory
///     >>> print(r.src, "->", r.dest)
fn html_file_to_markdown_impl(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<ConvertResult> {
    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
    let out_dir_ref: Option<&str> = out_dir.as_deref();

    let result = py.detach(|| ::mdka::html_file_to_markdown_with(&path, out_dir_ref, &opts));

    result
        .map(|r| ConvertResult {
            src: r.src.to_string_lossy().into_owned(),
            dest: r.dest.to_string_lossy().into_owned(),
        })
        .map_err(|e| MdkaError::new_err(e.to_string()))
}

#[pyfunction]
#[pyo3(signature = (path, out_dir=None, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_file_to_markdown(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<ConvertResult> {
    html_file_to_markdown_impl(
        py,
        path,
        out_dir,
        mode,
        preserve_ids,
        drop_interactive_shell,
    )
}

/// RFC 039 Half A: `html_to_markdown_with` exists, but the file-conversion
/// equivalent was only ever `html_file_to_markdown` -- itself already
/// configurable, just under a name that does not say so. A user who learns
/// the `_with` convention from the string API and looks for it here finds
/// nothing and concludes file conversion cannot be configured (RFC 039 §2.2).
/// A thin wrapper over the same implementation; the plain name keeps
/// accepting the same keyword arguments too, unchanged, for compatibility.
#[pyfunction]
#[pyo3(signature = (path, out_dir=None, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_file_to_markdown_with(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<ConvertResult> {
    html_file_to_markdown_impl(
        py,
        path,
        out_dir,
        mode,
        preserve_ids,
        drop_interactive_shell,
    )
}

// ─── Bulk file conversion API ────────────────────────────────────────────

fn html_files_to_markdown_impl(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    use std::path::Path;
    let out = Path::new(&out_dir);
    std::fs::create_dir_all(out)
        .map_err(|e| MdkaError::new_err(format!("cannot create out_dir: {e}")))?;

    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
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
#[pyo3(signature = (paths, out_dir, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_files_to_markdown(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    html_files_to_markdown_impl(
        py,
        paths,
        out_dir,
        mode,
        preserve_ids,
        drop_interactive_shell,
    )
}

/// RFC 039 Half A: same gap as `html_file_to_markdown_with`, for the bulk
/// path. A thin wrapper over the same implementation; the plain name keeps
/// accepting the same keyword arguments too, unchanged, for compatibility.
#[pyfunction]
#[pyo3(signature = (paths, out_dir, mode=ConversionMode::Balanced, preserve_ids=None,
    drop_interactive_shell=None))]
fn html_files_to_markdown_with(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    html_files_to_markdown_impl(
        py,
        paths,
        out_dir,
        mode,
        preserve_ids,
        drop_interactive_shell,
    )
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ─── Module registration ─────────────────────────────────────────────────

// Free-threaded CPython is not supported (RFC 034, owner 2026-09-16), and this
// module has not been reviewed for running without the GIL. PyO3 0.28 treats
// an unannotated module as GIL-free, so the requirement is stated here rather
// than left to a default that can change between PyO3 versions. On a
// free-threaded interpreter Python re-enables the GIL when this is imported.
// Changing it to `false` requires a thread-safety review in its own RFC.
#[pymodule(gil_used = true)]
fn mdka_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("MdkaError", py.get_type::<MdkaError>())?;
    m.add_class::<ConversionMode>()?;
    m.add_class::<ConvertResult>()?;
    m.add_class::<BulkConvertResult>()?;
    m.add_function(wrap_pyfunction!(html_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(html_to_markdown_with, m)?)?;
    m.add_function(wrap_pyfunction!(html_to_markdown_many, m)?)?;
    m.add_function(wrap_pyfunction!(html_to_markdown_many_with, m)?)?;
    m.add_function(wrap_pyfunction!(html_file_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(html_file_to_markdown_with, m)?)?;
    m.add_function(wrap_pyfunction!(html_files_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(html_files_to_markdown_with, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
