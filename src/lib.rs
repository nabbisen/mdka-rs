//! HTML to Markdown converter - Lightweight library written in Rust.

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html;
/// 
/// let input = "<h1>heading 1</h1>\n<p>Hello, world.</p>";
/// let expect = "# heading 1\n\nHello, world.\n\n";
/// let ret = from_html(input);
/// assert_eq!(ret, expect);
/// ```

const INDENT_DEFAULT_SIZE: usize = 0;
const INDENT_UNIT_SIZE: usize = 4;

mod components;
mod utils;

use crate::utils::node::parse_html;

pub fn from_html(html: &str) -> String {
    let dom = parse_html(html);
    components::node::manipulate_node(&dom.document, None::<usize>)
}
