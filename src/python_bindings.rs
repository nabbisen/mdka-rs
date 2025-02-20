use crate::{from_file, from_file_to_file, from_html, from_html_to_file};
// python integration
use pyo3::prelude::*;

#[pymodule]
fn mdka(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(md_from_html, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_html_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file_to_file, m)?)?;

    Ok(())
}

#[pyfunction]
fn md_from_html(html_text: &str) -> String {
    from_html(html_text)
}

#[pyfunction]
fn md_from_file(html_filepath: &str) -> PyResult<String> {
    match from_file(html_filepath) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}

#[pyfunction]
fn md_from_html_to_file(
    html_text: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> PyResult<()> {
    match from_html_to_file(html_text, markdown_filepath, overwrites) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}

#[pyfunction]
fn md_from_file_to_file(
    html_filepath: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> PyResult<()> {
    match from_file_to_file(html_filepath, markdown_filepath, overwrites) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}
