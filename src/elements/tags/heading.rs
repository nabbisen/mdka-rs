use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::nodes::node::children_md;

use super::super::utils::{block_trailing_new_line, enclose, is_emtpy_element};

/// h1, h2, h3, h4, h5, h6
pub fn heading_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    name: &str,
    parents: &Vec<String>,
) -> String {
    let content = children_md(node, indent_size, parents);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        return enclose(content.as_str(), indent_size, attrs_map, false);
    }

    let level = name
        .chars()
        .last()
        .unwrap()
        .to_digit(10)
        .unwrap()
        .try_into()
        .unwrap();
    let prefix = "#".repeat(level);
    let trailing = block_trailing_new_line(indent_size);
    let enclosed = format!("{} {}{}{}", prefix, content, trailing, trailing);
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}
