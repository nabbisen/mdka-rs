//! HTML to Markdown converter - Lightweight library written in Rust.

mod elements;
mod nodes;
mod utils;

use nodes::{node::root_node_md, utils::parse_html};
use utils::file::{read_from_filepath, write_to_filepath};

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html;
///
/// let html_text = r#"
/// <h1>heading 1</h1>
/// <p>Hello, world.</p>"#;
/// let expect = "# heading 1\n\nHello, world.\n\n";
///
/// let ret = from_html(html_text);
/// assert_eq!(ret, expect);
/// ```
///
pub fn from_html(html_text: &str) -> String {
    let dom = parse_html(html_text);
    root_node_md(&dom.document, None::<usize>)
}

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_file;
///
/// let html_filepath = "tests/fixtures/simple-01.html";
/// let expect = "# heading 1\n\nHello, world.\n\n";
///
/// let ret = from_file(html_filepath).expect("Failed to read");
/// assert_eq!(ret.as_str(), expect);
/// ```
///
pub fn from_file(html_filepath: &str) -> Result<String, String> {
    let html_text = read_from_filepath(html_filepath)
        .expect(format!("Failed to read: {}", html_filepath).as_str());
    Ok(from_html(html_text.as_str()))
}

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html_to_file;
///
/// let html_text = r#"
/// <h1>heading 1</h1>
/// <p>Hello, world.</p>"#;
/// let markdown_filepath = "tests/tmp/from_html_file_doc_test_result.html";
/// let expect = "# heading 1\n\nHello, world.\n\n";
///
/// let ret = from_html_to_file(html_text, markdown_filepath, true).expect("Failed to write");
/// let markdown_file_content = std::fs::read_to_string(markdown_filepath).expect("Failed to read from markdown filepath");
/// assert_eq!(expect, markdown_file_content);
/// ```
///
pub fn from_html_to_file(
    html_text: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> Result<(), String> {
    let md = from_html(html_text);
    write_to_filepath(md.as_str(), markdown_filepath, overwrites)
}

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_file_to_file;
///
/// let html_filepath = "tests/fixtures/simple-01.html";
/// let markdown_filepath = "tests/tmp/from_html_file_doc_test_result.html";
/// let expect = "# heading 1\n\nHello, world.\n\n";
///
/// let ret = from_file_to_file(html_filepath, markdown_filepath, true).expect("Failed to write");
/// let markdown_file_content = std::fs::read_to_string(markdown_filepath).expect("Failed to read from markdown filepath");
/// assert_eq!(expect, markdown_file_content);
/// ```
///
pub fn from_file_to_file(
    html_filepath: &str,
    markdown_filepath: &str,
    overwrites: bool,
) -> Result<(), String> {
    let html = read_from_filepath(html_filepath)
        .expect(format!("Failed to read: {}", html_filepath).as_str());
    from_html_to_file(html.as_str(), markdown_filepath, overwrites)
}

// python integration
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn mdka(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(md_from_html, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_html_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(md_from_file_to_file, m)?)?;

    Ok(())
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn md_from_html(html_text: &str) -> String {
    from_html(html_text)
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn md_from_file(html_filepath: &str) -> PyResult<String> {
    match from_file(html_filepath) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
    }
}

#[cfg(feature = "pyo3")]
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

#[cfg(feature = "pyo3")]
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
