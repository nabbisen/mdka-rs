// python integration
use pyo3::prelude::*;

#[pymodule]
fn mdka(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(from_html, m)?)?;
    m.add_function(wrap_pyfunction!(from_file, m)?)?;
    m.add_function(wrap_pyfunction!(from_html_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(from_file_to_file, m)?)?;

    m.add_function(wrap_pyfunction!(md_from_html, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_html_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file_to_file, m)?)?;

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

#[pyfunction]
fn md_from_html(html_text: &str) -> String {
    println!("*** deprecated ***\nThis function is renamed to `from_html()` and will be removed.\nPlease use the renamed function by removing leading `md_`. The function is equivalent.");
    from_html(html_text)
}

#[pyfunction]
fn md_from_file(html_filepath: &str) -> PyResult<String> {
    println!("*** deprecated ***\nThis function is renamed to `from_file()` and will be removed.\nPlease use the renamed function by removing leading `md_`. The function is equivalent.");
    from_file(html_filepath)
}

#[pyfunction]
fn md_from_html_to_file(
    html_text: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> PyResult<()> {
    println!("*** deprecated ***\nThis function is renamed to `from_html_to_file()` and will be removed.\nPlease use the renamed function by removing leading `md_`. The function is equivalent.");
    from_html_to_file(html_text, markdown_filepath, overwrites)
}

#[pyfunction]
fn md_from_file_to_file(
    html_filepath: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> PyResult<()> {
    println!("*** deprecated ***\nThis function is renamed to `from_file_to_file()` and will be removed.\nPlease use the renamed function by removing leading `md_`. The function is equivalent.");
    from_file_to_file(html_filepath, markdown_filepath, overwrites)
}
