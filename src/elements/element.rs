use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use crate::nodes::node::children_md;

use super::{
    consts::INDENT_DEFAULT_SIZE,
    tags::general_purpose::{block_md, inline_md},
    tags::heading::heading_md,
    tags::link::link_md,
    tags::list::list_md,
    tags::media::media_md,
    tags::preformatted::preformatted_md,
    tags::table::table_md,
    tags::text_content::blockquote_md,
    types::InlineStyle,
    utils::element_name,
};

/// process by element type
pub fn element_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    parents: &Vec<String>,
) -> String {
    let name = element_name(node);
    let ret = match name.as_str() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            heading_md(node, indent_size, attrs_map, name.as_str(), parents)
        }
        "div" => block_md(node, indent_size, attrs_map, false, parents),
        "p" => block_md(node, indent_size, attrs_map, true, parents),
        "span" => inline_md(node, indent_size, attrs_map, InlineStyle::Regular, parents),
        "b" | "strong" => inline_md(node, indent_size, attrs_map, InlineStyle::Bold, parents),
        "i" | "em" => inline_md(node, indent_size, attrs_map, InlineStyle::Italic, parents),
        "ul" => list_md(node, indent_size, attrs_map, false, parents),
        "ol" => list_md(node, indent_size, attrs_map, true, parents),
        "table" => table_md(node, indent_size, attrs_map, parents),
        "th" | "td" => children_md(node, Some(INDENT_DEFAULT_SIZE), parents),
        "pre" => preformatted_md(node, indent_size, attrs_map, false),
        "code" => preformatted_md(node, indent_size, attrs_map, true),
        "blockquote" => blockquote_md(node, indent_size, attrs_map, parents),
        "a" => link_md(node, indent_size, attrs_map),
        "img" | "audio" | "video" => media_md(node, indent_size, attrs_map),
        "br" => "    \n".to_owned(),
        "hr" => "\n---\n".to_owned(),
        "html" | "body" | "main" | "header" | "footer" | "nav" | "section" | "article"
        | "aside" | "time" | "address" | "figure" | "figcaption" => {
            children_md(node, None, parents)
        }
        // skip: script, style, svg
        _ => String::new(),
    };
    ret
}
