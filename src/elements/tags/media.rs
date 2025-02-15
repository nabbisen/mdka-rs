use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use super::super::utils::{block_trailing_new_line, enclose, is_emtpy_element};

/// img, audio, video
pub fn media_md(
    _node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
) -> String {
    let empty_str = String::new();
    let src = attrs_map.get("src").unwrap_or(&empty_str);
    let alt = attrs_map.get("alt").unwrap_or(&empty_str);

    if is_emtpy_element(empty_str.as_str(), attrs_map) && src.is_empty() && alt.is_empty() {
        return empty_str;
    }
    if src.is_empty() && alt.is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let trailing = block_trailing_new_line(indent_size);
    let content = format!("![{}]({}){}", alt, src, trailing);
    enclose(content.as_str(), indent_size, attrs_map, true)
}
