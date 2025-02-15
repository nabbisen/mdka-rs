use std::collections::HashMap;

use markup5ever_rcdom::Handle;

use super::super::{
    consts::INDENT_DEFAULT_SIZE,
    utils::{
        block_trailing_new_line, class_text_align, element_name, element_name_attrs_map, enclose,
        find_trs, indent, is_emtpy_element, style_text_align,
    },
};

use crate::nodes::node::node_md;

/// table, thead, tbody, tr, th, td
pub fn table_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    parents: &Vec<String>,
) -> String {
    let trs = find_trs(node);
    let mut content = String::new();
    let indent_str = indent(indent_size);
    for (i, tr) in trs.iter().enumerate() {
        if tr.children.borrow().len() == 0 {
            break;
        }

        let mut row = format!("{}|", indent_str);
        for td in tr.children.borrow().iter() {
            let name = element_name(td);
            let _ = match name.as_str() {
                "th" | "td" => {
                    let name = name.clone();
                    let mut parents = parents.clone();
                    parents.push(name);
                    let md = node_md(td, Some(INDENT_DEFAULT_SIZE), &parents)
                        .trim_end()
                        .to_string();
                    row = format!("{} {} |", row, md);
                }
                _ => {}
            };
        }
        row = format!("{}\n", row);
        if i == 0 {
            row = format!("{}{}|", row, indent_str);
            for td in tr.children.borrow().iter() {
                let (child_name, child_attrs_map) = element_name_attrs_map(td);
                match child_name.as_str() {
                    "th" | "td" => {
                        let align = match child_attrs_map.get("style") {
                            Some(style) => style_text_align(style),
                            _ => match child_attrs_map.get("class") {
                                Some(class) => class_text_align(class),
                                _ => None,
                            },
                        };
                        let divider = match align {
                            Some("left") => ":--- ",
                            Some("center") => " --- ",
                            Some("right") => " ---:",
                            _ => " --- ",
                        };
                        row = format!("{}{}|", row, divider);
                    }
                    _ => {}
                };
            }
            row = format!("{}\n", row);
        }
        content = format!("{}{}", content, row);
    }

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.replace("\n", "").is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let (_, attrs_map) = element_name_attrs_map(node);
    let trailing = block_trailing_new_line(indent_size);
    let enclosed = format!("{}{}", content, trailing);
    enclose(enclosed.as_str(), indent_size, &attrs_map, true)
}
