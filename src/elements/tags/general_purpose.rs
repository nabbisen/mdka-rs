use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::nodes::node::children_md;

use super::super::types::InlineStyle;
use super::super::utils::{block_trailing_new_line, enclose, indent, is_emtpy_element};
use super::text_content::{bold, italic};

/// span
pub fn inline_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    inline_style: InlineStyle,
    parents: &Vec<String>,
) -> String {
    let mut content = children_md(node, indent_size, parents);
    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }

    match inline_style {
        InlineStyle::Bold => content = bold(content.as_str(), parents),
        InlineStyle::Italic => content = italic(content.as_str(), parents),
        _ => {}
    }
    enclose(content.as_str(), indent_size, attrs_map, false)
}

/// div, p
pub fn block_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_paragraph: bool,
    parents: &Vec<String>,
) -> String {
    let content = children_md(node, indent_size, parents);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        return enclose(content.as_str(), indent_size, attrs_map, false);
    }

    let indent_str = indent(indent_size);
    let new_line = if is_paragraph {
        format!("{}{}", "\n", indent_str)
    } else {
        String::new()
    };
    let trailing = block_trailing_new_line(indent_size);
    let enclosed = format!("{}{}{}", content, new_line, trailing);
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}
