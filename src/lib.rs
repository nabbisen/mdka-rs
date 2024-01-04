//! HTML to Markdown converter - Lightweight library written in Rust

use std::cell::RefCell;

use html5ever::{parse_document, Attribute};
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use markup5ever_rcdom::{RcDom, NodeData, Handle, SerializableHandle};

const INDENT_INIT_VALUE: usize = 0;
const INDENT_SIZE: usize = 4;

/// Convert HTML to Markdown
///
/// ```
/// use mdka::from_html;
/// 
/// let input = "<h1>heading 1</h1>\n<p>Hello, world.</p>";
/// let expect = "# heading 1\nHello, world.\n\n";
/// let ret = from_html(input);
/// assert_eq!(ret, expect);
/// ```
pub fn from_html(html: &str) -> String {
    let dom = parse_document(RcDom::default(), ParseOpts::default()).from_utf8().read_from(&mut html.as_bytes()).unwrap();
    // let mut result = StructuredPrinter::default();
    // walk(&dom.document, &mut result, custom);
    // println!("{:?}", &dom.document);
    manipulate_node(&dom.document, None)
}

fn manipulate_node(node: &Handle, indent: Option<usize>) -> String {
    let ret = match node.data {
        NodeData::Text { ref contents } => {
            let escaped = contents.borrow().escape_default().to_string();
            escaped.replace("\\n", "\n").replace("\\r", "\r").trim().to_string()
        },
        NodeData::Element {
            attrs: ref node_attrs,
            ..
        } => {
            let attrs = attrs(node_attrs);
            manipulate_element(node, attrs, indent)
        },
        NodeData::Document | NodeData::Doctype { .. } => manipulate_children(node, None), // todo
        NodeData::Comment { .. } => "".to_string(),
        NodeData::ProcessingInstruction { .. } => unreachable!(),
    };
    ret
}

fn element_name_attrs(node: &Handle) -> (String, String) {
    match node.data {
        NodeData::Element {
            ref name,
            attrs: ref node_attrs,
            ..
        } => {
            (name.local.to_string(), attrs(node_attrs))
        },
        _ => { ("".to_string(), "".to_string()) }
    }
}
fn element_name(node: &Handle) -> String {
    let (ret, _) = element_name_attrs(node);
    ret
}
fn attrs(node_attrs: &RefCell<Vec<Attribute>>) -> String {
    let style = node_attrs
        .borrow()
        .iter()
        .find(|attr| attr.name.local.to_string().as_str() == "style")
        .and_then(|attr| Some(attr.value.escape_default().to_string()));
    let id = node_attrs
        .borrow()
        .iter()
        .find(|attr| attr.name.local.to_string().as_str() == "id")
        .and_then(|attr| Some(attr.value.escape_default().to_string()));
    format!("{}{}{}{}", 
        if style.is_some() || id.is_some() { " " } else { "" },
        if id.is_some() { format!("id=\"{}\"", id.clone().unwrap()) } else { "".to_string() },
        if style.is_some() && id.is_some() { " " } else { "" },
        if style.is_some() { format!("style=\"{}\"", style.clone().unwrap()) } else { "".to_string() }
    )
}

fn manipulate_element(node: &Handle, attrs: String, indent: Option<usize>) -> String {
    let element_name = element_name(node);
    let ret = match element_name.as_str() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => manipulate_heading(node, indent, attrs, element_name),
        "span" => manipulate_block(node, indent, attrs, 0),
        "div" => manipulate_block(node, indent, attrs, 1),
        "p" => manipulate_block(node, indent, attrs, 2),
        "ul" => manipulate_list(node, indent, false),
        "ol" => manipulate_list(node, indent, true),
        "table" => manipulate_table(node, indent, attrs),
        "th" | "td" => manipulate_children(node, Some(INDENT_INIT_VALUE)),
        "pre" => manipulate_preformatted(node, indent, attrs, false),
        "code" => manipulate_preformatted(node, indent, attrs, true),
        "blockquote" => manipulate_blockquote(node, indent, attrs),
        "b" | "strong" => manipulate_bold(node, indent, attrs),
        "i" | "em" => manipulate_italic(node, indent, attrs),
        "a" => manipulate_link(node, indent, attrs),
        "img" | "video" => manipulate_media(node, indent, attrs),
        "br" => "    \n".to_string(),
        "hr" => "\n---\n".to_string(),
        "style" => "".to_string(),
        "html" | "head" | "body" => manipulate_children(node, None),
        _ => "".to_string()
    };
    ret
}
fn manipulate_children(node: &Handle, indent: Option<usize>) -> String {
    let mut ret = "".to_string();
    let next_indent = if indent.is_some() { indent.unwrap() } else { INDENT_INIT_VALUE };
    for child in node.children.borrow().iter() {
        ret = format!("{}{}", ret, manipulate_node(child, Some(next_indent)));
    }
    ret
}
fn manipulate_attrs(s: String, attrs: String, indent: Option<usize>, new_line: bool) -> String {
    let indent_str = " ".repeat(indent.unwrap());
    if 0 < attrs.len() {
        format!("{}<span{}>\n{}{}\n{}</span>\n", indent_str, attrs, indent_str, s, indent_str)
    } else {
        format!("{}{}{}", indent_str, s, if new_line { "\n" } else { "" })
    }
}

fn manipulate_heading(node: &Handle, indent: Option<usize>, attrs: String, element_name: String) -> String {
    let level = element_name.chars().last().unwrap().to_digit(10).unwrap().try_into().unwrap();
    let prefix = "#".repeat(level);
    let ret = format!("{} {}", prefix, manipulate_children(node, indent));
    manipulate_attrs(ret, attrs, indent, true)
}
fn manipulate_block(node: &Handle, indent: Option<usize>, attrs: String, new_lines: usize) -> String{
    let devider = "\n".repeat(new_lines);
    let ret = format!("{}{}", manipulate_children(node, indent), devider);
    manipulate_attrs(ret, attrs, indent, false)
}
fn manipulate_list(node: &Handle, indent: Option<usize>, is_ordered: bool) -> String {
    let prefix = if is_ordered { "1." } else { "-"};

    let current_indent = indent.unwrap_or(INDENT_INIT_VALUE);
    let next_indent = Some(current_indent + INDENT_SIZE);
    let is_nested = INDENT_INIT_VALUE < current_indent;
    let mut ret = (if is_nested { "\n" } else { "" }).to_string();
    for (i, child) in node.children.borrow().iter().enumerate() {
        let (element_name, attrs) = element_name_attrs(child);
        let child_ret = match element_name.as_str() {
            "li" => {
                let child_children_ret = manipulate_children(child, next_indent);
                let is_last = i == node.children.borrow().len() - 1;
                manipulate_attrs(format!("{} {}", prefix, child_children_ret), attrs, indent, !is_last)
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
        // todo: local name
        let element_name = element_name(child);
        let _ = match element_name.as_str() {
            "tr" => trs.push(child.clone()),
            _ => trs.append(&mut find_trs(&child))
        };
    }
    trs
}
fn manipulate_table(node: &Handle, _indent: Option<usize>, _attrs: String) -> String {
    let trs = find_trs(node);
    let mut ret = "".to_string();
    for (i, tr) in trs.iter().enumerate() {
        if tr.children.borrow().len() == 0 { break }

        let mut row = "|".to_string();
        for td in tr.children.borrow().iter() {
            let element_name = element_name(td);
            let _ = match element_name.as_str() {
                "th" | "td" => {
                    row = format!("{} {} |", row, manipulate_node(td, Some(INDENT_INIT_VALUE)));
                },
                _ => {}
            };
        }
        row = format!("{}\n", row);
        // todo: text-align
        if i == 0 {
            row = format!("{}|", row);
            for td in tr.children.borrow().iter() {
                let element_name = element_name(td);
                let _ = match element_name.as_str() {
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
    ret
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
fn manipulate_preformatted(node: &Handle, indent: Option<usize>, attrs: String, is_inline: bool) -> String {
    let ret = if is_inline {
        format!("`{}`", inner_html(node))
    } else {
        let node_children = &node.children.borrow();
        let code_node = node_children
            .iter()
            .find(|child| {
                let element_name = element_name(child);
                element_name == "code"
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
        format!("{}\n{}\n```\n", prefix, inner_html(next_node))
    };
    manipulate_attrs(ret, attrs, indent, false)
}
// todo
fn manipulate_blockquote(_node: &Handle, _indent: Option<usize>, _attrs: String) -> String {
    "".to_string()
}
// todo
fn manipulate_link(_node: &Handle, _indent: Option<usize>, _attrs: String) -> String {
    "link".to_string()
}
// todo
fn manipulate_media(_node: &Handle, _indent: Option<usize>, _attrs: String) -> String {
    "media".to_string()
}
fn manipulate_bold(node: &Handle, indent: Option<usize>, attrs: String) -> String {
    let ret = format!(" **{}** ", manipulate_children(node, indent));
    manipulate_attrs(ret, attrs, indent, false)
}
fn manipulate_italic(node: &Handle, indent: Option<usize>, attrs: String) -> String {
    let ret = format!(" *{}* ", manipulate_children(node, indent));
    manipulate_attrs(ret, attrs, indent, false)
}
