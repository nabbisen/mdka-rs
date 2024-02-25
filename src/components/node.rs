use std::collections::HashMap;

use markup5ever_rcdom::{NodeData, Handle};

use crate::components::element::*;
use crate::utils::element::*;

use crate::INDENT_DEFAULT_SIZE;

/// main process on node
pub fn manipulate_node(node: &Handle, indent_size: Option<usize>) -> String {
    let ret = match node.data {
        NodeData::Text { ref contents } => {
            let contents_str = contents.borrow().to_string();
            contents_str
        },
        NodeData::Element {
            attrs: ref node_attrs,
            ..
        } => {
            let attrs_map = attrs_map(node_attrs);
            manipulate_element(node, indent_size, &attrs_map)
        },
        NodeData::Document | NodeData::Doctype { .. } => manipulate_children(node, None),
        // skip: comments
        NodeData::Comment { .. } => String::new(),
        NodeData::ProcessingInstruction { .. } => unreachable!(),
    };
    ret
}

/// process on children of node
pub fn manipulate_children(node: &Handle, indent_size: Option<usize>) -> String {
    let mut ret = String::new();
    let next_indent_size = if indent_size.is_some() { indent_size.unwrap() } else { INDENT_DEFAULT_SIZE };
    for child in node.children.borrow().iter() {
        ret = format!("{}{}", ret, manipulate_node(child, Some(next_indent_size)));
    }
    ret
}

/// process by element type
pub fn manipulate_element(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let name = element_name(node);
    let ret = match name.as_str() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => manipulate_heading(node, indent_size, attrs_map, name.as_str()),
        "div" => manipulate_block(node, indent_size, attrs_map, false),
        "p" => manipulate_block(node, indent_size, attrs_map, true),
        "span" => manipulate_inline(node, indent_size, attrs_map, InlineStyle::Regular),
        "b" | "strong" => manipulate_inline(node, indent_size, attrs_map, InlineStyle::Bold),
        "i" | "em" => manipulate_inline(node, indent_size, attrs_map, InlineStyle::Italic),
        "ul" => manipulate_list(node, indent_size, attrs_map, false),
        "ol" => manipulate_list(node, indent_size, attrs_map, true),
        "table" => manipulate_table(node, indent_size, attrs_map),
        "th" | "td" => manipulate_children(node, Some(INDENT_DEFAULT_SIZE)),
        "pre" => manipulate_preformatted(node, indent_size, attrs_map, false),
        "code" => manipulate_preformatted(node, indent_size, attrs_map, true),
        "blockquote" => manipulate_blockquote(node, indent_size, attrs_map),
        "a" => manipulate_link(node, indent_size, attrs_map),
        "img" | "video" => manipulate_media(node, indent_size, attrs_map),
        "br" => "    \n".to_owned(),
        "hr" => "\n---\n".to_owned(),
        "html" | "body" |
            "main" | "header" | "footer" | "nav" |
            "section" | "article" | "aside" |
            "time" | "address" | "figure" | "figcaption" =>
            manipulate_children(node, None),
        // skip: script, style, svg
        _ => String::new()
    };
    ret
}
