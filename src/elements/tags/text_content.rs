use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::{elements::utils::block_trailing_new_line, nodes::node::children_md};

use super::super::{
    consts::INDENT_DEFAULT_SIZE,
    utils::{enclose, indent, is_emtpy_element},
};

/// b, strong
pub fn bold(s: &str, parents: &Vec<String>) -> String {
    let parent_element = parents.last().unwrap();
    match parent_element.as_str() {
        "i" | "em" => format!("**{}**", s),
        _ => format!(" **{}** ", s),
    }
}

/// i, em
pub fn italic(s: &str, parents: &Vec<String>) -> String {
    let parent_element = parents.last().unwrap();
    match parent_element.as_str() {
        "b" | "strong" => format!("_{}_", s),
        _ => format!(" _{}_ ", s),
    }
}

/// blockquote
pub fn blockquote_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    parents: &Vec<String>,
) -> String {
    let md_str = children_md(node, indent_size, parents);

    if is_emtpy_element(md_str.as_str(), attrs_map) {
        return md_str;
    }
    if md_str.is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let indent_str = indent(indent_size);
    let lines = md_str
        .split('\n')
        .map(|line| format!("{}> {}", indent_str, line.to_string()))
        .collect::<Vec<String>>();
    let rejoined = lines.join("\n");

    let is_nested = INDENT_DEFAULT_SIZE < indent_size.unwrap();
    let content = if is_nested {
        rejoined
    } else {
        let trailing = block_trailing_new_line(indent_size);
        format!("{}{}{}", rejoined, trailing, trailing)
    };

    enclose(content.as_str(), indent_size, attrs_map, true)
}
