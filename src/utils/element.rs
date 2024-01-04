use std::collections::HashMap;
use std::cell::RefCell;

use html5ever::Attribute;
use markup5ever_rcdom::{NodeData, Handle};

pub fn element_name_attrs_map(node: &Handle) -> (String, HashMap<String, String>) {
    match node.data {
        NodeData::Element {
            ref name,
            attrs: ref node_attrs,
            ..
        } => {
            (name.local.to_string(), attrs_map(node_attrs))
        },
        _ => { ("".to_string(), HashMap::<String, String>::new()) }
    }
}

pub fn element_name(node: &Handle) -> String {
    let (ret, _) = element_name_attrs_map(node);
    ret
}

// todo: fix key-value pairs
pub fn attrs_map(node_attrs: &RefCell<Vec<Attribute>>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in node_attrs.borrow().iter() {
        map.insert(attr.name.local.to_string(), attr.value.escape_default().to_string());
    }
    map
}

pub fn find_trs(node: &Handle) -> Vec<Handle> {
    let mut trs = Vec::<Handle>::new();
    for child in node.children.borrow().iter() {
        let name = element_name(child);
        let _ = match name.as_str() {
            "tr" => trs.push(child.clone()),
            _ => trs.append(&mut find_trs(&child))
        };
    }
    trs
}

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

pub fn class_text_align(class: &String) -> Option<&str> {
    for s in class.split(' ') {
        match s {
            "text-left" => return Some("left"),
            "text-center" => return Some("center"),
            "text-right" => return Some("right"),
            _ => {}
        }
    }
    None
}

pub fn indent(indent_size: Option<usize>) -> String {
    " ".repeat(indent_size.unwrap())
}

pub fn enclose(s: String, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, requires_new_line: bool) -> String {
    if requires_enclosure(attrs_map) {
        let new_line = if requires_new_line { "\n" } else { "" };
        let indent_str = indent(indent_size);
        let enclosure_attrs = enclosure_attrs(attrs_map);
        format!("{}{}<span{}>{}{}</span>{}", new_line, indent_str, enclosure_attrs, new_line, s, new_line)
    } else {
        s
    }
}

fn requires_enclosure(attrs_map: &HashMap<String, String>) -> bool {
    attrs_map.contains_key("id") || attrs_map.contains_key("style")
}

fn enclosure_attrs(attrs_map: &HashMap<String, String>) -> String {
    let style = attrs_map.get("style");
    let id = attrs_map.get("id");
    format!("{}{}{}{}", 
        if style.is_some() || id.is_some() { " " } else { "" },
        if id.is_some() { format!("id=\"{}\"", id.clone().unwrap()) } else { "".to_string() },
        if style.is_some() && id.is_some() { " " } else { "" },
        if style.is_some() { format!("style=\"{}\"", style.clone().unwrap()) } else { "".to_string() }
    )
}
