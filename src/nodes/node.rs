use markup5ever_rcdom::{Handle, NodeData};

use crate::elements::element::element_md;

use crate::elements::{
    consts::INDENT_DEFAULT_SIZE,
    utils::{attrs_map, element_name},
};

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
