use std::collections::HashMap;

use markup5ever_rcdom::{NodeData, Handle};

use crate::components::element::*;
use crate::utils::element::*;

use crate::INDENT_DEFAULT_SIZE;

pub fn manipulate_node(node: &Handle, indent_size: Option<usize>) -> String {
    let ret = match node.data {
        NodeData::Text { ref contents } => {
            let escaped = contents.borrow().escape_default().to_string();
            escaped.replace("\\n", "\n").replace("\\r", "\r").trim().to_string()
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
        NodeData::Comment { .. } => "".to_string(),
        NodeData::ProcessingInstruction { .. } => unreachable!(),
    };
    ret
}

pub fn manipulate_children(node: &Handle, indent_size: Option<usize>) -> String {
    let mut ret = "".to_string();
    let next_indent_size = if indent_size.is_some() { indent_size.unwrap() } else { INDENT_DEFAULT_SIZE };
    for child in node.children.borrow().iter() {
        ret = format!("{}{}", ret, manipulate_node(child, Some(next_indent_size)));
    }
    ret
}

pub fn manipulate_element(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let name = element_name(node);
    let ret = match name.as_str() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => manipulate_heading(node, indent_size, attrs_map, name),
        "span" => manipulate_block(node, indent_size, attrs_map, 0),
        "div" => manipulate_block(node, indent_size, attrs_map, 1),
        "p" => manipulate_block(node, indent_size, attrs_map, 2),
        "ul" => manipulate_list(node, indent_size, false),
        "ol" => manipulate_list(node, indent_size, true),
        "table" => manipulate_table(node, indent_size, attrs_map),
        "th" | "td" => manipulate_children(node, Some(INDENT_DEFAULT_SIZE)),
        "pre" => manipulate_preformatted(node, indent_size, attrs_map, false),
        "code" => manipulate_preformatted(node, indent_size, attrs_map, true),
        "blockquote" => manipulate_blockquote(node, indent_size, attrs_map),
        "b" | "strong" => manipulate_bold(node, indent_size, attrs_map),
        "i" | "em" => manipulate_italic(node, indent_size, attrs_map),
        "a" => manipulate_link(node, indent_size, attrs_map),
        "img" | "video" => manipulate_media(node, indent_size, attrs_map),
        "br" => "    \n".to_string(),
        "hr" => "\n---\n".to_string(),
        "html" | "body" => manipulate_children(node, None),
        // skip: script, style
        _ => "".to_string()
    };
    ret
}
