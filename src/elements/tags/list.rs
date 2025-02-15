use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::nodes::node::children_md;

use super::super::{
    consts::{INDENT_DEFAULT_SIZE, INDENT_UNIT_SIZE},
    utils::{block_trailing_new_line, element_name_attrs_map, enclose, indent, is_emtpy_element},
};

/// ul, ol, li
pub fn list_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_ordered: bool,
    parents: &Vec<String>,
) -> String {
    let prefix = if is_ordered { "1." } else { "-" };

    let current_indent_size = indent_size.unwrap_or(INDENT_DEFAULT_SIZE);
    let indent_str = indent(indent_size);
    let next_indent_size = Some(current_indent_size + INDENT_UNIT_SIZE);
    let is_nested = INDENT_DEFAULT_SIZE < current_indent_size;

    let mut content = (if is_nested { "\n" } else { "" }).to_string();
    for (i, child) in node.children.borrow().iter().enumerate() {
        let (child_name, child_attrs_map) = element_name_attrs_map(child);
        let child_content = match child_name.as_str() {
            "li" => {
                let child_children_content = children_md(child, next_indent_size, parents);
                let is_last = i == node.children.borrow().len() - 1;
                let new_line = if is_last { "" } else { "\n" };
                let s = format!(
                    "{}{} {}{}",
                    indent_str, prefix, child_children_content, new_line
                );
                enclose(s.as_str(), indent_size, &child_attrs_map, false)
            }
            _ => String::new(),
        };
        if is_emtpy_element(child_content.as_str(), &child_attrs_map) {
            return String::new();
        }

        content = format!("{}{}", content, child_content);
    }

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.replace("\n", "").is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let (_, attrs_map) = element_name_attrs_map(node);
    let trailing = if is_nested {
        String::new()
    } else {
        block_trailing_new_line(indent_size)
    };
    let enclosed = format!("{}{}{}", content, trailing, trailing);
    enclose(enclosed.as_str(), indent_size, &attrs_map, true)
}
