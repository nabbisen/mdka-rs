use std::collections::HashMap;

use markup5ever_rcdom::{Handle, NodeData};

use crate::components::element::*;
use crate::utils::element::*;

use crate::INDENT_DEFAULT_SIZE;

/// entry point
pub fn root_node_md(node: &Handle, indent_size: Option<usize>) -> String {
    node_md(node, indent_size, &vec![])
}

/// main process on node
pub fn node_md(node: &Handle, indent_size: Option<usize>, parents: &Vec<String>) -> String {
    let ret = match node.data {
        NodeData::Text { ref contents } => {
            let contents_str = contents.borrow().to_string();
            contents_str
        }
        NodeData::Element {
            attrs: ref node_attrs,
            ..
        } => {
            let attrs_map = attrs_map(node_attrs);
            element_md(node, indent_size, &attrs_map, parents)
        }
        NodeData::Document | NodeData::Doctype { .. } => children_md(node, None, parents),
        // skip: comments
        NodeData::Comment { .. } => String::new(),
        NodeData::ProcessingInstruction { .. } => unreachable!(),
    };
    ret
}

/// process on children of node
pub fn children_md(node: &Handle, indent_size: Option<usize>, parents: &Vec<String>) -> String {
    let mut ret = String::new();
    let next_indent_size = if indent_size.is_some() {
        indent_size.unwrap()
    } else {
        INDENT_DEFAULT_SIZE
    };
    let mut parents = parents.clone();
    parents.push(element_name(node));
    for child in node.children.borrow().iter() {
        ret = format!(
            "{}{}",
            ret,
            node_md(child, Some(next_indent_size), &parents)
        );
    }
    ret
}

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
        },
        "div" => block_md(node, indent_size, attrs_map, false, parents),
        "p" => block_md(node, indent_size, attrs_map, true, parents),
        "span" => inline_md(
            node,
            indent_size,
            attrs_map,
            InlineStyle::Regular,
            parents,
        ),
        "b" | "strong" => inline_md(
            node,
            indent_size,
            attrs_map,
            InlineStyle::Bold,
            parents,
        ),
        "i" | "em" => inline_md(
            node,
            indent_size,
            attrs_map,
            InlineStyle::Italic,
            parents,
        ),
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
        | "aside" | "time" | "address" | "figure" | "figcaption" => children_md(node, None, parents),
        // skip: script, style, svg
        _ => String::new(),
    };
    ret
}
