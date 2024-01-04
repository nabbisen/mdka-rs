//! HTML to Markdown converter - Lightweight library written in Rust.

use std::collections::HashMap;
use std::cell::RefCell;

use html5ever::{parse_document, Attribute};
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use markup5ever_rcdom::{RcDom, NodeData, Handle, SerializableHandle};

const INDENT_DEFAULT_SIZE: usize = 0;
const INDENT_UNIT_SIZE: usize = 4;

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html;
/// 
/// let input = "<h1>heading 1</h1>\n<p>Hello, world.</p>";
/// let expect = "# heading 1\n\n\nHello, world.\n\n";
/// let ret = from_html(input);
/// assert_eq!(ret, expect);
/// ```
pub fn from_html(html: &str) -> String {
    let dom = parse_document(RcDom::default(), ParseOpts::default()).from_utf8().read_from(&mut html.as_bytes()).unwrap();
    // let mut result = StructuredPrinter::default();
    // walk(&dom.document, &mut result, custom);
    // println!("{:?}", &dom.document);
    manipulate_node(&dom.document, None::<usize>)
}

fn manipulate_node(node: &Handle, indent_size: Option<usize>) -> String {
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
        NodeData::Comment { .. } => "".to_string(),
        NodeData::ProcessingInstruction { .. } => unreachable!(),
    };
    ret
}

fn element_name_attrs_map(node: &Handle) -> (String, HashMap<String, String>) {
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
fn element_name(node: &Handle) -> String {
    let (ret, _) = element_name_attrs_map(node);
    ret
}
fn attrs_map(node_attrs: &RefCell<Vec<Attribute>>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in node_attrs.borrow().iter() {
        map.insert(attr.name.local.to_string(), attr.value.escape_default().to_string());
    }
    map
}

fn manipulate_element(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
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
        // head, script, style, comments
        _ => "".to_string()
    };
    ret
}
fn manipulate_children(node: &Handle, indent_size: Option<usize>) -> String {
    let mut ret = "".to_string();
    let next_indent_size = if indent_size.is_some() { indent_size.unwrap() } else { INDENT_DEFAULT_SIZE };
    for child in node.children.borrow().iter() {
        ret = format!("{}{}", ret, manipulate_node(child, Some(next_indent_size)));
    }
    ret
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
fn indent(indent_size: Option<usize>) -> String {
    " ".repeat(indent_size.unwrap())
}
fn requires_enclosure(attrs_map: &HashMap<String, String>) -> bool {
    attrs_map.contains_key("id") || attrs_map.contains_key("style")
}
fn enclose(s: String, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, requires_new_line: bool) -> String {
    if requires_enclosure(attrs_map) {
        let new_line = if requires_new_line { "\n" } else { "" };
        let indent_str = indent(indent_size);
        let enclosure_attrs = enclosure_attrs(attrs_map);
        format!("{}{}<span{}>{}{}</span>{}", new_line, indent_str, enclosure_attrs, new_line, s, new_line)
    } else {
        s
    }
}

fn manipulate_heading(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, name: String) -> String {
    let level = name.chars().last().unwrap().to_digit(10).unwrap().try_into().unwrap();
    let prefix = "#".repeat(level);
    let ret = format!("{} {}\n\n", prefix, manipulate_children(node, indent_size));
    enclose(ret, indent_size, attrs_map, true)
}
fn manipulate_block(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, new_lines_size: usize) -> String{
    let start = "\n".repeat(if 1 < new_lines_size { new_lines_size - 1 } else { 0 });
    let end = "\n".repeat(new_lines_size);
    let ret = format!("{}{}{}", start, manipulate_children(node, indent_size), end);
    enclose(ret, indent_size, attrs_map, false)
}
fn manipulate_list(node: &Handle, indent_size: Option<usize>, is_ordered: bool) -> String {
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
            _ => "".to_string()
        };
        ret = format!("{}{}", ret, child_ret);
    }
    format!("{}{}", ret, if is_nested { "" } else { "\n" })
}
fn find_trs(node: &Handle) -> Vec<Handle> {
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
fn manipulate_table(node: &Handle, indent_size: Option<usize>, _attrs_map: &HashMap<String, String>) -> String {
    let trs = find_trs(node);
    let mut ret = "".to_string();
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
        // todo: text-align
        if i == 0 {
            row = format!("{}{}|", row, indent_str);
            for td in tr.children.borrow().iter() {
                let name = element_name(td);
                let _ = match name.as_str() {
                    "th" | "td" => {
                        row = format!("{} --- |", row);
                    },
                    _ => {}
                };
            }
            row = format!("{}\n", row);
        }
        ret = format!("{}{}", ret, row);
    }
    format!("\n{}\n{}", ret, indent_str)
}
// todo: indent
fn inner_html(node: &Handle) -> String {
    let h: SerializableHandle = (*node).clone().into();
    let opts = SerializeOpts {
        scripting_enabled: false,
        traversal_scope: TraversalScope::ChildrenOnly(None),
        create_missing_parent: false,
    };
    let mut buf = Vec::new();
    serialize(&mut buf, &h, opts).unwrap();
    String::from_utf8(buf).unwrap()
}
fn inner_text_scan(node: &Handle, s: String) -> String {
    match node.data {
        NodeData::Text { ref contents } => {
            let escaped = contents.borrow().escape_default().to_string();
            let replaced = escaped.replace("\\n", "\n").replace("\\r", "\r").trim().to_string();
            if s.len() == 0 {
                replaced
            } else {
                format!("{} {}", s, replaced)
            }
        },
        NodeData::Element {
            ..
        } => {
            let mut ret = s;
            for child in node.children.borrow().iter() {
                ret = inner_text_scan(child, ret)
            }
            ret
        },
        _ => { "".to_string() }
    }
}
fn inner_text(node: &Handle) -> String {
    let mut ret = "".to_string();
    for child in node.children.borrow().iter() {
        ret = inner_text_scan(child, ret);
    }
    ret
}
fn manipulate_preformatted(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>, is_inline: bool) -> String {
    if is_inline {
        let ret = format!("`{}`", inner_html(node));
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
                        .map(|attr| attr.value.to_string()).unwrap_or("".to_string())
                },
                _ => "".to_string()
            }
        ;
        format!("```{}", code_lang)
    } else {
        "```".to_string()
    };
    let next_node = if code_node.is_some() { code_node.unwrap() } else { node };
    let indent_str = indent(indent_size);
    let ret = format!("\n{}\n{}\n```\n{}\n", prefix, inner_html(next_node), indent_str);
    enclose(ret, indent_size, attrs_map, true)
}
// todo
fn manipulate_blockquote(_node: &Handle, _indent_size: Option<usize>, _attrs_map: &HashMap<String, String>) -> String {
    "".to_string()
}
fn manipulate_link(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let href = attrs_map.get("href");
    let ret = format!("[{}]({})", inner_text(node), href.unwrap_or(&String::new()));
    enclose(ret, indent_size, attrs_map, false)
}
fn manipulate_media(_node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let src = attrs_map.get("src");
    let alt = attrs_map.get("alt");
    let indent_str = indent(indent_size);
    let ret = format!("\n{}![{}]({})\n", indent_str, alt.unwrap_or(&String::new()), src.unwrap_or(&String::new()));
    enclose(ret, indent_size, attrs_map, true)
}
fn manipulate_bold(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let ret = format!(" **{}** ", manipulate_children(node, indent_size));
    enclose(ret, indent_size, attrs_map, false)
}
fn manipulate_italic(node: &Handle, indent_size: Option<usize>, attrs_map: &HashMap<String, String>) -> String {
    let ret = format!(" *{}* ", manipulate_children(node, indent_size));
    enclose(ret, indent_size, attrs_map, false)
}
