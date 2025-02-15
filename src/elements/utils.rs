use std::cell::RefCell;
use std::collections::HashMap;

use html5ever::{
    serialize::{serialize, SerializeOpts, TraversalScope},
    Attribute,
};
use markup5ever_rcdom::{Handle, NodeData, SerializableHandle};

use crate::elements::consts::INDENT_DEFAULT_SIZE;

/// generate inner_html from serialized node
pub fn inner_html(node: &Handle, indent_size: Option<usize>) -> String {
    let h: SerializableHandle = (*node).clone().into();
    let opts = SerializeOpts {
        scripting_enabled: false,
        traversal_scope: TraversalScope::ChildrenOnly(None),
        create_missing_parent: false,
    };
    let mut buf = Vec::new();
    serialize(&mut buf, &h, opts).unwrap();
    let serialized = String::from_utf8(buf).unwrap();
    if INDENT_DEFAULT_SIZE < indent_size.unwrap_or(INDENT_DEFAULT_SIZE) {
        let indent_str = indent(indent_size);
        serialized
            .split("\n")
            .into_iter()
            .fold(String::new(), |mut acc, x| {
                let s = format!("{}{}", indent_str, &x);
                acc.push_str(s.as_str());
                acc
            })
    } else {
        serialized
    }
}

/// generate inner text from node data
pub fn inner_text(node: &Handle) -> String {
    let mut ret = String::new();
    for child in node.children.borrow().iter() {
        ret = inner_text_scan(child, ret.as_str());
    }
    ret
}

/// scan inner nodes recursively to generate inner text
fn inner_text_scan(node: &Handle, s: &str) -> String {
    match node.data {
        NodeData::Text { ref contents } => {
            let contents_str = contents.borrow().to_string().trim_end().to_owned();
            if s.is_empty() {
                contents_str
            } else {
                format!("{} {}", s, contents_str)
            }
        }
        NodeData::Element { .. } => {
            let mut ret = s.to_string();
            for child in node.children.borrow().iter() {
                ret = inner_text_scan(child, ret.as_str())
            }
            ret
        }
        _ => String::new(),
    }
}

/// get name and attrs of element from node data
pub fn element_name_attrs_map(node: &Handle) -> (String, HashMap<String, String>) {
    match node.data {
        NodeData::Element {
            ref name,
            attrs: ref node_attrs,
            ..
        } => (name.local.to_string(), attrs_map(node_attrs)),
        _ => (String::new(), HashMap::<String, String>::new()),
    }
}

/// get name of element from node data
pub fn element_name(node: &Handle) -> String {
    let (ret, _) = element_name_attrs_map(node);
    ret
}

/// get attrs of element from node data as key-value map
pub fn attrs_map(node_attrs: &RefCell<Vec<Attribute>>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut attr_key = String::new();
    let mut attr_value = String::new();
    for attr in node_attrs.borrow().iter() {
        let k = attr.name.local.to_string();
        let v = attr.value.to_string();

        let is_quote_inside = !attr_key.is_empty();
        if is_quote_inside {
            attr_value = format!("{} {}{}", attr_value, k, v);
            if k.trim_end().ends_with("\\\"") {
                map.insert(attr_key, attr_value);

                attr_key = String::new();
                attr_value = String::new();
            };
            continue;
        };
        let is_quote_start = v.trim_start().starts_with("\\\"") && !v.trim_end().ends_with("\\\"");
        if is_quote_start {
            attr_key = k;
            attr_value = v;
            continue;
        };

        map.insert(k, v);
    }
    map
}

/// get tr vec in table
pub fn find_trs(node: &Handle) -> Vec<Handle> {
    let mut trs = Vec::<Handle>::new();
    for child in node.children.borrow().iter() {
        let name = element_name(child);
        let _ = match name.as_str() {
            "tr" => trs.push(child.clone()),
            _ => trs.append(&mut find_trs(&child)),
        };
    }
    trs
}

/// parse style str to find align attr
pub fn style_text_align(style: &String) -> Option<&str> {
    if let Some(start) = style.find("text-align") {
        for pos in (start + "text-align".len())..style.len() {
            if style.as_bytes()[pos] == b':' {
                if let Some(end) = style[pos..].find(';') {
                    return Some((style[(pos + 1)..(pos + end)]).trim());
                }
            }
        }
    }
    None
}

/// parse class str to find align attr
pub fn class_text_align(class_list: &String) -> Option<&str> {
    for class in class_list.split(' ') {
        match class.replace("\\\"", "").replace(";", "").as_str() {
            "text-left" => return Some("left"),
            "text-center" => return Some("center"),
            "text-right" => return Some("right"),
            _ => {}
        }
    }
    None
}

/// generate indent str from size
pub fn indent(indent_size: Option<usize>) -> String {
    " ".repeat(indent_size.unwrap())
}

/// generate trailing new line with or without indent
pub fn block_trailing_new_line(indent_size: Option<usize>) -> String {
    let indent = indent_size.unwrap();
    if indent == INDENT_DEFAULT_SIZE {
        "\n".to_owned()
    } else {
        format!("\n{}", " ".repeat(indent))
    }
}

/// check if empty content
pub fn is_emtpy_element(content: &str, attrs_map: &HashMap<String, String>) -> bool {
    content.is_empty() && !requires_enclosure(attrs_map)
}

/// generate span enclosure to apply block style
pub fn enclose(
    s: &str,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    requires_new_line: bool,
) -> String {
    if !requires_enclosure(attrs_map) {
        return s.to_string();
    }

    let new_line = if requires_new_line {
        "\n".to_owned()
    } else {
        String::new()
    };
    let indent_str = indent(indent_size);

    let id = attrs_map.get("id");
    let style = attrs_map.get("style");

    let id_tag = if let Some(id) = id {
        // id attr
        format!(
            "{}{}<span id=\"{}\"></span>",
            new_line,
            indent_str,
            id.clone()
        )
    } else {
        String::new()
    };

    if let Some(style) = style {
        // style attr
        let style_attr = format!(" style=\"{}\"", style.clone());
        format!(
            "{}{}{}<span{}>{}{}</span>{}",
            id_tag, new_line, indent_str, style_attr, new_line, s, new_line
        )
    } else {
        format!("{}{}{}{}{}", id_tag, new_line, indent_str, s, new_line)
    }
}

const ENCLOSURE_ATTRS: [&str; 2] = ["id", "style"];

/// check if enclosure is necessary
fn requires_enclosure(attrs_map: &HashMap<String, String>) -> bool {
    ENCLOSURE_ATTRS.iter().any(|attr_key| {
        let k = attr_key.to_string();
        attrs_map.contains_key(k.as_str())
    })
}
