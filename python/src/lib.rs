//! Python bindings for mdka (PyO3 0.28)

use pyo3::prelude::*;
use pyo3::types::PyString;
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

// ─── FileOutcome ──────────────────────────────────────────────────────────

/// What happened to one file in a bulk conversion, successful or not.
///
/// The distinction that shapes this type is *can it fail*, not *one or many*.
/// A single-file call can fail and the caller asked about exactly one thing, so
/// `html_file_to_markdown` returns the destination path or raises `MdkaError`.
/// A bulk call must not let one bad file hide the others, so each file gets its
/// own outcome -- and the outcome says which file it belongs to.
///
/// Exactly one of `dest` and `error` is set, and `ok` says which.
///
/// Attributes:
///     src (str): input file path, as it was passed in
///     dest (str | None): output file path, when `ok`
///     error (str | None): error message, when not `ok`
///     ok (bool): whether the conversion succeeded
#[pyclass(get_all, frozen)]
pub struct FileOutcome {
    pub src: String,
    pub dest: Option<String>,
    pub error: Option<String>,
}

#[pymethods]
impl FileOutcome {
    /// Python's own `repr`, with all four fields: `FileOutcome(src='a.html',
    /// dest='out/a.md', error=None, ok=True)`. (It used to print a Rust
    /// `Option` -- `error=Some("...")` -- and omit `dest` and `ok`.)
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let py_str = |s: &str| -> PyResult<String> { Ok(PyString::new(py, s).repr()?.to_string()) };
        let opt = |o: &Option<String>| -> PyResult<String> {
            match o {
                Some(s) => py_str(s),
                None => Ok("None".to_string()),
            }
        };
        Ok(format!(
            "FileOutcome(src={}, dest={}, error={}, ok={})",
            py_str(&self.src)?,
            opt(&self.dest)?,
            opt(&self.error)?,
            if self.dest.is_some() { "True" } else { "False" },
        ))
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
///     str: the path of the file that was written
///
/// Raises:
///     MdkaError: if reading or writing fails. A single file fails the call,
///         through Python's own channel; there is no result object to inspect.
///         (`html_files_to_markdown` reports per file instead.)
///
/// Example:
///     >>> import mdka
///     >>> dest = mdka.html_file_to_markdown("index.html")          # same directory
///     >>> dest = mdka.html_file_to_markdown("index.html", "out/")  # another directory
fn html_file_to_markdown_impl(
    py: Python<'_>,
    path: String,
    out_dir: Option<String>,
    mode: ConversionMode,
    preserve_ids: Option<bool>,
    drop_interactive_shell: Option<bool>,
) -> PyResult<String> {
    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
    let out_dir_ref: Option<&str> = out_dir.as_deref();

    let result = py.detach(|| ::mdka::html_file_to_markdown_with(&path, out_dir_ref, &opts));

    result
        .map(|dest| dest.to_string_lossy().into_owned())
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
) -> PyResult<String> {
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
) -> PyResult<String> {
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
) -> PyResult<Vec<FileOutcome>> {
    use std::path::Path;
    let out = Path::new(&out_dir);
    std::fs::create_dir_all(out)
        .map_err(|e| MdkaError::new_err(format!("cannot create out_dir: {e}")))?;

    let opts = build_opts(mode, preserve_ids, drop_interactive_shell);
    let path_bufs: Vec<std::path::PathBuf> = paths.iter().map(std::path::PathBuf::from).collect();

    let results = py.detach(|| ::mdka::html_files_to_markdown_with(&path_bufs, out, &opts));

    Ok(results
        .into_iter()
        .map(|o| {
            let src = o.src.to_string_lossy().into_owned();
            match o.result {
                Ok(dest) => FileOutcome {
                    src,
                    dest: Some(dest.to_string_lossy().into_owned()),
                    error: None,
                },
                Err(e) => FileOutcome {
                    src,
                    dest: None,
                    error: Some(e.to_string()),
                },
            }
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
) -> PyResult<Vec<FileOutcome>> {
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
) -> PyResult<Vec<FileOutcome>> {
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
    m.add_class::<FileOutcome>()?;
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
