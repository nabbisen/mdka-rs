use std::collections::HashMap;

use markup5ever_rcdom::{Handle, NodeData};

use crate::components::node::*;
use crate::utils::element::*;
use crate::utils::node::*;

use crate::INDENT_DEFAULT_SIZE;
use crate::INDENT_UNIT_SIZE;

/// h1, h2, h3, h4, h5, h6
pub fn heading_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    name: &str,
    parents: &Vec<String>,
) -> String {
    let content = children_md(node, indent_size, parents);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        return enclose(content.as_str(), indent_size, attrs_map, false);
    }

    let level = name
        .chars()
        .last()
        .unwrap()
        .to_digit(10)
        .unwrap()
        .try_into()
        .unwrap();
    let prefix = "#".repeat(level);
    let trailing = block_trailing_new_line(indent_size);
    let enclosed = format!("{} {}{}{}", prefix, content, trailing, trailing);
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}

/// span, b/strong, i/em
pub enum InlineStyle {
    Regular,
    Bold,
    Italic,
}
/// span
pub fn inline_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    inline_style: InlineStyle,
    parents: &Vec<String>,
) -> String {
    let mut content = children_md(node, indent_size, parents);
    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }

    match inline_style {
        InlineStyle::Bold => content = bold(content.as_str(), parents),
        InlineStyle::Italic => content = italic(content.as_str(), parents),
        _ => {}
    }
    enclose(content.as_str(), indent_size, attrs_map, false)
}
/// b, strong
fn bold(s: &str, parents: &Vec<String>) -> String {
    let parent_element = parents.last().unwrap();
    match parent_element.as_str() {
        "i" | "em" => format!("**{}**", s),
        _ => format!(" **{}** ", s),
    }
}
/// i, em
fn italic(s: &str, parents: &Vec<String>) -> String {
    let parent_element = parents.last().unwrap();
    match parent_element.as_str() {
        "b" | "strong" => format!("_{}_", s),
        _ => format!(" _{}_ ", s),
    }
}

/// div, p
pub fn block_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_paragraph: bool,
    parents: &Vec<String>,
) -> String {
    let content = children_md(node, indent_size, parents);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        return enclose(content.as_str(), indent_size, attrs_map, false);
    }

    let indent_str = indent(indent_size);
    let new_line = if is_paragraph {
        format!("{}{}", "\n", indent_str)
    } else {
        String::new()
    };
    let trailing = block_trailing_new_line(indent_size);
    let enclosed = format!("{}{}{}", content, new_line, trailing);
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}

/// ul, ol, li
pub fn list_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_ordered: bool,
    parents: &Vec<String>,
) -> String {
    let prefix = if is_ordered { "1." } else { "-" };

    let current_indent_size = indent_size.unwrap_or(INDENT_DEFAULT_SIZE);
    let indent_str = indent(indent_size);
    let next_indent_size = Some(current_indent_size + INDENT_UNIT_SIZE);
    let is_nested = INDENT_DEFAULT_SIZE < current_indent_size;

    let mut content = (if is_nested { "\n" } else { "" }).to_string();
    for (i, child) in node.children.borrow().iter().enumerate() {
        let (child_name, child_attrs_map) = element_name_attrs_map(child);
        let child_content = match child_name.as_str() {
            "li" => {
                let child_children_content = children_md(child, next_indent_size, parents);
                let is_last = i == node.children.borrow().len() - 1;
                let new_line = if is_last { "" } else { "\n" };
                let s = format!(
                    "{}{} {}{}",
                    indent_str, prefix, child_children_content, new_line
                );
                enclose(s.as_str(), indent_size, &child_attrs_map, false)
            }
            _ => String::new(),
        };
        if is_emtpy_element(child_content.as_str(), &child_attrs_map) {
            return String::new();
        }

        content = format!("{}{}", content, child_content);
    }

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.replace("\n", "").is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let (_, attrs_map) = element_name_attrs_map(node);
    let trailing = if is_nested {
        String::new()
    } else {
        block_trailing_new_line(indent_size)
    };
    let enclosed = format!("{}{}{}", content, trailing, trailing);
    enclose(enclosed.as_str(), indent_size, &attrs_map, true)
}

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

/// pre, code
pub fn preformatted_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_inline: bool,
) -> String {
    if is_inline {
        let content = inner_html(node, indent_size);

        if is_emtpy_element(content.as_str(), attrs_map) {
            return content;
        }
        if content.is_empty() {
            let empty_str = String::new();
            return enclose(empty_str.as_str(), indent_size, attrs_map, false);
        }

        let enclosed = format!(" `{}` ", content);
        return enclose(enclosed.as_str(), indent_size, attrs_map, false);
    }

    let node_children = &node.children.borrow();
    let code_node = node_children.iter().find(|child| {
        let name = element_name(child);
        name == "code"
    });
    let next_node = if code_node.is_some() {
        code_node.unwrap()
    } else {
        node
    };
    let content = inner_html(next_node, indent_size);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let prefix = if code_node.is_some() {
        let code_lang = match code_node.unwrap().data {
            NodeData::Element { ref attrs, .. } => attrs
                .borrow()
                .iter()
                .find(|attr| attr.name.local.to_string().as_str() == "lang")
                .map(|attr| attr.value.to_string())
                .unwrap_or(String::new()),
            _ => String::new(),
        };
        format!("```{}", code_lang)
    } else {
        "```".to_owned()
    };
    let is_nested = INDENT_DEFAULT_SIZE < indent_size.unwrap();
    let leading = if is_nested {
        block_trailing_new_line(indent_size)
    } else {
        String::new()
    };
    let trailing = block_trailing_new_line(indent_size);
    let indent_str = indent(indent_size);
    let enclosed = format!(
        "{}{}\n{}\n{}```\n{}{}",
        leading, prefix, content, indent_str, indent_str, trailing
    );
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}

/// blockquote
pub fn blockquote_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    parents: &Vec<String>,
) -> String {
    let md_str = children_md(node, indent_size, parents);

    if is_emtpy_element(md_str.as_str(), attrs_map) {
        return md_str;
    }
    if md_str.is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let indent_str = indent(indent_size);
    let lines = md_str
        .split('\n')
        .map(|line| format!("{}> {}", indent_str, line.to_string()))
        .collect::<Vec<String>>();
    let rejoined = lines.join("\n");

    let is_nested = INDENT_DEFAULT_SIZE < indent_size.unwrap();
    let content = if is_nested {
        rejoined
    } else {
        let trailing = block_trailing_new_line(indent_size);
        format!("{}{}{}", rejoined, trailing, trailing)
    };

    enclose(content.as_str(), indent_size, attrs_map, true)
}

/// a
pub fn link_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
) -> String {
    let content = inner_text(node);
    let empty_str = String::new();
    let href = attrs_map.get("href").unwrap_or(&empty_str);

    if is_emtpy_element(content.as_str(), attrs_map) && href.is_empty() {
        return empty_str;
    }
    if content.is_empty() && href.is_empty() {
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let enclosed = format!(" [{}]({}) ", content, href);
    enclose(enclosed.as_str(), indent_size, attrs_map, false)
}

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
