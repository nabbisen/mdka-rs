//! Python bindings for mdka (PyO3 0.28)

use std::ffi::CString;

use pyo3::prelude::*;
use rayon::prelude::*;

pyo3::create_exception!(mdka, MdkaError, pyo3::exceptions::PyException);

/// `warnings.warn(..., DeprecationWarning)` for a field that is
/// `#[deprecated]` on the Rust side. `#[deprecated]` does not cross FFI, so
/// Python callers see nothing unless this is emitted explicitly. Only called
/// when the field was **explicitly passed** (`Some(_)`), never for a default
/// -- warning on every call regardless of intent would just get the warning
/// suppressed wholesale.
fn warn_deprecated_field(py: Python<'_>, field: &str) -> PyResult<()> {
    let message = CString::new(format!(
        "mdka: `{field}` has no effect and is deprecated (see https://nabbisen.github.io/mdka-rs/api/options.html). \
         Markdown has no attribute syntax, so this option was never \
         expressible in the output."
    ))
    .expect("warning message contains no NUL bytes");
    let category = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    PyErr::warn(py, &category, &message, 1)
}

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

#[allow(clippy::too_many_arguments)]
fn build_opts(
    py: Python<'_>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<::mdka::ConversionOptions> {
    let mut opts = ::mdka::ConversionOptions::for_mode(to_rust_mode(mode));
    if let Some(v) = preserve_ids {
        opts.preserve_ids = v;
    }
    // preserve_classes/preserve_data_attrs/preserve_aria_attrs are deprecated
    // no-ops (RFC 005 Slice B2); the Python-facing params are kept as no-op
    // passthroughs for API compatibility rather than removed.
    #[allow(deprecated)]
    {
        if let Some(v) = preserve_classes {
            warn_deprecated_field(py, "preserve_classes")?;
            opts.preserve_classes = v;
        }
        if let Some(v) = preserve_data_attrs {
            warn_deprecated_field(py, "preserve_data_attrs")?;
            opts.preserve_data_attrs = v;
        }
        if let Some(v) = preserve_aria_attrs {
            warn_deprecated_field(py, "preserve_aria_attrs")?;
            opts.preserve_aria_attrs = v;
        }
    }
    if let Some(v) = drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }
    if let Some(v) = unwrap_unknown_wrappers {
        opts.unwrap_unknown_wrappers = v;
    }
    Ok(opts)
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
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
#[allow(clippy::too_many_arguments)]
fn html_to_markdown_with(
    py: Python<'_>,
    html: &str,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<String> {
    let opts = build_opts(
        py,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )?;
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
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
#[allow(clippy::too_many_arguments)]
fn html_to_markdown_many_with(
    py: Python<'_>,
    html_list: Vec<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<Vec<String>> {
    let opts = build_opts(
        py,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )?;
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
#[allow(clippy::too_many_arguments)]
fn html_file_to_markdown_impl(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<ConvertResult> {
    let opts = build_opts(
        py,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )?;
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
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
// This argument list is the published Python keyword-argument API; restructuring
// it to satisfy clippy would break the published surface.
#[allow(clippy::too_many_arguments)]
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
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<ConvertResult> {
    html_file_to_markdown_impl(
        py,
        path,
        out_dir,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
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
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
#[allow(clippy::too_many_arguments)]
fn html_file_to_markdown_with(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<ConvertResult> {
    html_file_to_markdown_impl(
        py,
        path,
        out_dir,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )
}

// ─── Bulk file conversion API ────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn html_files_to_markdown_impl(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    use std::path::Path;
    let out = Path::new(&out_dir);
    std::fs::create_dir_all(out)
        .map_err(|e| MdkaError::new_err(format!("cannot create out_dir: {e}")))?;

    let opts = build_opts(
        py,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )?;
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
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
// This argument list is the published Python keyword-argument API; restructuring
// it to satisfy clippy would break the published surface.
#[allow(clippy::too_many_arguments)]
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
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    html_files_to_markdown_impl(
        py,
        paths,
        out_dir,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
    )
}

/// RFC 039 Half A: same gap as `html_file_to_markdown_with`, for the bulk
/// path. A thin wrapper over the same implementation; the plain name keeps
/// accepting the same keyword arguments too, unchanged, for compatibility.
#[pyfunction]
#[pyo3(signature = (paths, out_dir, mode=ConversionMode::Balanced, preserve_ids=None,
    preserve_classes=None, preserve_data_attrs=None, preserve_aria_attrs=None,
    drop_interactive_shell=None, unwrap_unknown_wrappers=None))]
#[allow(clippy::too_many_arguments)]
fn html_files_to_markdown_with(
    py: Python<'_>,
    paths: Vec<String>,
    out_dir: String,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    preserve_classes: Option<bool>,
    preserve_data_attrs: Option<bool>,
    preserve_aria_attrs: Option<bool>,
    drop_interactive_shell: Option<bool>,
    unwrap_unknown_wrappers: Option<bool>,
) -> PyResult<Vec<BulkConvertResult>> {
    html_files_to_markdown_impl(
        py,
        paths,
        out_dir,
        mode,
        preserve_ids,
        preserve_classes,
        preserve_data_attrs,
        preserve_aria_attrs,
        drop_interactive_shell,
        unwrap_unknown_wrappers,
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
