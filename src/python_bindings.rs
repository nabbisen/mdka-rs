// python integration
use pyo3::prelude::*;

#[pymodule]
fn mdka(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(from_html, m)?)?;
    m.add_function(wrap_pyfunction!(from_file, m)?)?;
    m.add_function(wrap_pyfunction!(from_html_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(from_file_to_file, m)?)?;

    Ok(())
}

#[pyfunction]
fn from_html(html_text: &str) -> String {
    crate::from_html(html_text)
}

#[pyfunction]
fn from_file(html_filepath: &str) -> PyResult<String> {
    match crate::from_file(html_filepath) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}

#[pyfunction]
fn from_html_to_file(html_text: &str, markdown_filepath: &str, overwrites: bool) -> PyResult<()> {
    match crate::from_html_to_file(html_text, markdown_filepath, overwrites) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}

#[pyfunction]
fn from_file_to_file(
    html_filepath: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> PyResult<()> {
    match crate::from_file_to_file(html_filepath, markdown_filepath, overwrites) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}
