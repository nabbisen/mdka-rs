use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};

use crate::elements::util::*;
use crate::INDENT_DEFAULT_SIZE;

/// parse html str
pub fn parse_html(html: &str) -> RcDom {
    let optimized_html = optimize_html_to_be_well_parsed(html);
    parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut optimized_html.as_bytes())
        .unwrap()
}

/// trim spaces and new lines between end of tag and start of next tag
/// to prevent dirtily parsed with: either `</a>\n<a ...` or `</a> <a ...`
fn optimize_html_to_be_well_parsed(html: &str) -> String {
    let mut ret = String::new();

    let chars: Vec<char> = html.chars().collect();

    let mut start = 0;
    // trim between end of tag and start of next tag
    while let Some(pos) = chars[start..].iter().position(|&c| c == '>') {
        let end = match chars[(start + pos)..].iter().position(|&c| c == '<') {
            Some(end_pos) => start + pos + end_pos,
            None => break,
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
