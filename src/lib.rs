//! HTML to Markdown converter - Lightweight library written in Rust.

mod elements;
mod nodes;

use nodes::{node::root_node_md, utils::parse_html};

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html;
///
/// let input = r#"
/// <h1>heading 1</h1>
/// <p>Hello, world.</p>"#;
/// let expect = "# heading 1\n\nHello, world.\n\n";
/// let ret = from_html(input);
/// assert_eq!(ret, expect);
/// ```
///
pub fn from_html(html: &str) -> String {
    let dom = parse_html(html);
    root_node_md(&dom.document, None::<usize>)
}

// python integration
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyfunction]
fn md_from_html(html: &str) -> String {
    from_html(html)
}

#[cfg(feature = "pyo3")]
#[pymodule]
fn mdka(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(md_from_html, m)?)?;

    Ok(())
}
