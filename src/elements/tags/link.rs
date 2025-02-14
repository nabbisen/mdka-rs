use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::nodes::util::inner_text;

use super::super::util::{enclose, is_emtpy_element};

/// a
pub fn link_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
) -> String {
    let content = inner_text(node);
    let empty_str = String::new();
    let href = attrs_map.get("href").unwrap_or(&empty_str);

    if is_emtpy_element(content.as_str(), attrs_map) && href.is_empty() {
        return empty_str;
    }
    if content.is_empty() && href.is_empty() {
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let enclosed = format!(" [{}]({}) ", content, href);
    enclose(enclosed.as_str(), indent_size, attrs_map, false)
}
