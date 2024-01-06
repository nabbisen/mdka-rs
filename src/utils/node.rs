use html5ever::{parse_document, ParseOpts};
use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{RcDom, NodeData, Handle, SerializableHandle};

use crate::INDENT_DEFAULT_SIZE;
use crate::utils::element::*;

/// parse html str
pub fn parse_html(html: &str) -> RcDom {
    let optimized_html = optimze_html_to_be_well_parsed(html);
    parse_document(RcDom::default(), ParseOpts::default()).from_utf8().read_from(&mut optimized_html.as_bytes()).unwrap()
}

/// fix dirtily parsed with: `>\n<`, `>  <`
fn optimze_html_to_be_well_parsed(html: &str) -> String {
    let mut ret = String::new();

    let chars: Vec<char> = html.chars().collect();

    let mut start = 0;
    // pre, blockquote
    while let Some(pos) = chars[start..].iter().position(|&c| c == '>') {
        let end = match chars[(start + pos)..].iter().position(|&c| c == '<') {
            Some(end_pos) => start + pos + end_pos,
            None => {
                break
            }
        };

        let start_to_bracket_end = &chars[start..(start + pos)].iter().collect::<String>();
        ret.push_str(start_to_bracket_end);
        ret.push('>');
        let between_brackets_end_start = &chars[(start + pos + 1)..end].iter().collect::<String>();
        ret.push_str(between_brackets_end_start.trim());
        ret.push('<');

        start = end + 1;
    }
    ret.push_str(&chars[start..].iter().collect::<String>());

    ret
}

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
        serialized.split("\n").into_iter().fold(String::new(), |mut acc, x| {
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
        ret = inner_text_scan(child, ret);
    }
    ret
}

/// scan inner nodes recursively to generate inner text
fn inner_text_scan(node: &Handle, s: String) -> String {
    match node.data {
        NodeData::Text { ref contents } => {
            let escaped = contents.borrow().escape_default().to_string();
            let replaced = escaped.replace("\\n", "\n").replace("\\r", "\r").trim_end().to_owned();
            if s.is_empty() {
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
        _ => { String::new() }
    }
}
