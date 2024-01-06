use std::collections::HashMap;

use markup5ever_rcdom::{NodeData, Handle};

use crate::utils::node::*;
use crate::utils::element::*;
use crate::components::node::*;

use crate::INDENT_DEFAULT_SIZE;
use crate::INDENT_UNIT_SIZE;

pub fn manipulate_heading(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, name: String) -> String {
    let level = name.chars().last().unwrap().to_digit(10).unwrap().try_into().unwrap();
    let prefix = "#".repeat(level);
    let trailing = block_trailing_new_line(indent_size);
    let ret = format!("{} {}{}{}", prefix, manipulate_children(node, indent_size), trailing, trailing);
    enclose(ret, indent_size, attrs_map, true)
}

pub enum InlineStyle {
    Regular,
    Bold,
    Italic,
}
pub fn manipulate_inline(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, inline_style: InlineStyle) -> String{
    let mut ret = manipulate_children(node, indent_size);
    match inline_style {
        InlineStyle::Bold => ret = bold(ret.as_str()),
        InlineStyle::Italic => ret = italic(ret.as_str()),
        _ => {}
    }
    enclose(ret, indent_size, attrs_map, false)
}
fn bold(s: &str) -> String {
    format!(" **{}** ", s)
}
fn italic(s: &str) -> String {
    format!(" *{}* ", s)
}

pub fn manipulate_block(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, is_paragraph: bool) -> String{
    let indent_str = indent(indent_size);
    let new_line = if is_paragraph { format!("{}{}", "\n", indent_str) } else { String::new() };
    let trailing = block_trailing_new_line(indent_size);
    let ret = format!("{}{}{}", manipulate_children(node, indent_size), new_line, trailing);
    enclose(ret, indent_size, attrs_map, true)
}

pub fn manipulate_list(node: &Handle, indent_size: Option<usize>, is_ordered: bool) -> String {
    let prefix = if is_ordered { "1." } else { "-"};

    let current_indent_size = indent_size.unwrap_or(INDENT_DEFAULT_SIZE);
    let indent_str = indent(indent_size);
    let next_indent_size = Some(current_indent_size + INDENT_UNIT_SIZE);
    let is_nested = INDENT_DEFAULT_SIZE < current_indent_size;
    let mut ret = (if is_nested { "\n" } else { "" }).to_string();
    for (i, child) in node.children.borrow().iter().enumerate() {
        let (name, attrs_map) = element_name_attrs_map(child);
        let child_ret = match name.as_str() {
            "li" => {
                let child_children_ret = manipulate_children(child, next_indent_size);
                let is_last = i == node.children.borrow().len() - 1;
                let new_line = if is_last { "" } else { "\n" };
                let s = format!("{}{} {}{}", indent_str, prefix, child_children_ret, new_line);
                enclose(s, indent_size, &attrs_map, false)
            },
            _ => String::new()
        };
        ret = format!("{}{}", ret, child_ret);
    }

    let (_, attrs_map) = element_name_attrs_map(node);
    let trailing = if is_nested { String::new() } else { block_trailing_new_line(indent_size) };
    let ret = format!("{}{}", ret, trailing);
    enclose(ret, indent_size, &attrs_map, true)
}

pub fn manipulate_table(node: &Handle, indent_size: Option<usize>, _attrs_map: &HashMap<String, String>) -> String {
    let trs = find_trs(node);
    let mut ret = String::new();
    let indent_str = indent(indent_size);
    for (i, tr) in trs.iter().enumerate() {
        if tr.children.borrow().len() == 0 { break }

        let mut row = format!("{}|", indent_str);
        for td in tr.children.borrow().iter() {
            let name = element_name(td);
            let _ = match name.as_str() {
                "th" | "td" => {
                    row = format!("{} {} |", row, manipulate_node(td, Some(INDENT_DEFAULT_SIZE)));
                },
                _ => {}
            };
        }
        row = format!("{}\n", row);
        if i == 0 {
            row = format!("{}{}|", row, indent_str);
            for td in tr.children.borrow().iter() {
                let (name, attrs_map) = element_name_attrs_map(td);
                let _ = match name.as_str() {
                    "th" | "td" => {
                        let align = match attrs_map.get("style") {
                            Some(style) => style_text_align(style),
                            _ => {
                                match attrs_map.get("class") {
                                    Some(class) => class_text_align(class),
                                    _ => None,
                                }
                            }
                        };
                        let devider = match align {
                            Some("left") => ":--- ",
                            Some("center") => " --- ",
                            Some("right") => " ---:",
                            _ => " --- "
                        };
                        row = format!("{}{}|", row, devider);
                    },
                    _ => {}
                };
            }
            row = format!("{}\n", row);
        }
        ret = format!("{}{}", ret, row);
    }

    let (_, attrs_map) = element_name_attrs_map(node);
    let trailing = block_trailing_new_line(indent_size);
    let ret = format!("{}{}", ret, trailing);
    enclose(ret, indent_size, &attrs_map, true)
}

pub fn manipulate_preformatted(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, is_inline: bool) -> String {
    if is_inline {
        let ret = format!("`{}`", inner_html(node, indent_size));
        return enclose(ret, indent_size, attrs_map, false);
    }
    
    let node_children = &node.children.borrow();
    let code_node = node_children
        .iter()
        .find(|child| {
            let name = element_name(child);
            name == "code"
        });
    let prefix = if code_node.is_some() {
        let code_lang = 
            match code_node.unwrap().data {
                NodeData::Element {
                    ref attrs,
                    ..
                } => {
                    attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.to_string().as_str() == "lang")
                        .map(|attr| attr.value.to_string()).unwrap_or(String::new())
                },
                _ => String::new()
            }
        ;
        format!("```{}", code_lang)
    } else {
        "```".to_owned()
    };
    let is_nested = INDENT_DEFAULT_SIZE < indent_size.unwrap();
    let leading = if is_nested { block_trailing_new_line(indent_size) } else { String::new() };
    let trailing = block_trailing_new_line(indent_size);
    let indent_str = indent(indent_size);
    let next_node = if code_node.is_some() { code_node.unwrap() } else { node };
    let ret = format!("{}{}\n{}\n{}```\n{}{}", leading, prefix, inner_html(next_node, indent_size), indent_str, indent_str, trailing);
    enclose(ret, indent_size, attrs_map, true)
}

pub fn manipulate_blockquote(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let md_str = manipulate_children(node, indent_size);
    let indent_str = indent(indent_size);
    let lines = md_str
        .split('\n')
        .map(|line| format!("{}> {}", indent_str, line.to_string()))
        .collect::<Vec<String>>();
    let trailing = block_trailing_new_line(indent_size);
    let ret = lines.join(&trailing);
    enclose(ret, indent_size, attrs_map, true)
}

pub fn manipulate_link(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let href = attrs_map.get("href");
    let ret = format!("[{}]({})", inner_text(node), href.unwrap_or(&String::new()));
    enclose(ret, indent_size, attrs_map, false)
}

pub fn manipulate_media(_node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let src = attrs_map.get("src");
    let alt = attrs_map.get("alt");
    let trailing = block_trailing_new_line(indent_size);
    let ret = format!("![{}]({}){}", alt.unwrap_or(&String::new()), src.unwrap_or(&String::new()), trailing);
    enclose(ret, indent_size, attrs_map, true)
}
