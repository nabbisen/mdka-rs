//! The parsed structure of a Markdown string, as a compact tree.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

/// The parser reads plain CommonMark: no GFM extensions. mdka does not emit
/// tables, strikethrough or task lists today, and a harness that enabled them
/// would accept syntax CommonMark readers render as text.
pub(super) fn parser(md: &str) -> Parser<'_> {
    Parser::new_ext(md, Options::empty())
}

struct Frame {
    name: String,
    block: bool,
    children: Vec<Child>,
}

enum Child {
    Text(String),
    Node(String),
}

/// The node name in the tree notation, and whether the tag is a block.
pub(super) fn tag_name(tag: &Tag<'_>) -> (String, bool) {
    match tag {
        Tag::Paragraph => ("para".into(), true),
        Tag::Heading { level, .. } => (format!("{level:?}").to_lowercase(), true),
        Tag::BlockQuote(_) => ("quote".into(), true),
        Tag::CodeBlock(CodeBlockKind::Fenced(info)) if !info.is_empty() => {
            (format!("codeblock[{info}]"), true)
        }
        Tag::CodeBlock(_) => ("codeblock".into(), true),
        Tag::HtmlBlock => ("htmlblock".into(), true),
        Tag::List(None) => ("ul".into(), true),
        Tag::List(Some(n)) => (format!("ol[{n}]"), true),
        Tag::Item => ("li".into(), true),
        Tag::Emphasis => ("em".into(), false),
        Tag::Strong => ("strong".into(), false),
        Tag::Link {
            dest_url, title, ..
        } if title.is_empty() => (format!("link[{dest_url}]"), false),
        Tag::Link {
            dest_url, title, ..
        } => (format!("link[{dest_url} {title:?}]"), false),
        Tag::Image {
            dest_url, title, ..
        } if title.is_empty() => (format!("image[{dest_url}]"), false),
        Tag::Image {
            dest_url, title, ..
        } => (format!("image[{dest_url} {title:?}]"), false),
        other => (format!("{other:?}"), true),
    }
}

fn push_text(children: &mut Vec<Child>, s: &str) {
    if let Some(Child::Text(t)) = children.last_mut() {
        t.push_str(s);
    } else {
        children.push(Child::Text(s.to_string()));
    }
}

fn render(frame: Frame) -> String {
    let in_code_block = frame.name.starts_with("codeblock");
    let n = frame.children.len();
    let parts: Vec<String> = frame
        .children
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            Child::Node(s) => Some(s),
            Child::Text(t) if in_code_block => {
                Some(format!("{:?}", t.strip_suffix('\n').unwrap_or(&t)))
            }
            Child::Text(t) => {
                let mut t = collapse(&t);
                if frame.block && i == 0 {
                    t = t.trim_start().to_string();
                }
                if frame.block && i + 1 == n {
                    t = t.trim_end().to_string();
                }
                (!t.is_empty()).then(|| format!("{t:?}"))
            }
        })
        .collect();
    if frame.name.is_empty() {
        parts.join(", ")
    } else {
        format!("{}({})", frame.name, parts.join(", "))
    }
}

fn collapse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !space {
                out.push(' ');
            }
            space = true;
        } else {
            out.push(ch);
            space = false;
        }
    }
    out
}

/// The parsed structure of `md`, as a compact string:
/// `para(link[/out](image[i.png]("pic")))`.
///
/// Text is whitespace-collapsed and trimmed at block edges; code block and
/// code span text is kept exactly (minus a code block's final newline).
pub fn structure(md: &str) -> String {
    let mut stack = vec![Frame {
        name: String::new(),
        block: true,
        children: Vec::new(),
    }];
    for event in parser(md) {
        let top = stack.last_mut().expect("root frame");
        match event {
            Event::Start(tag) => {
                let (name, block) = tag_name(&tag);
                stack.push(Frame {
                    name,
                    block,
                    children: Vec::new(),
                });
            }
            Event::End(_) => {
                let done = stack.pop().expect("balanced events");
                let s = render(done);
                stack
                    .last_mut()
                    .expect("root frame")
                    .children
                    .push(Child::Node(s));
            }
            Event::Text(t) => push_text(&mut top.children, &t),
            Event::SoftBreak => push_text(&mut top.children, " "),
            Event::HardBreak => top.children.push(Child::Node("br".into())),
            Event::Code(c) => top.children.push(Child::Node(format!("code({:?})", &*c))),
            Event::Html(h) | Event::InlineHtml(h) => top
                .children
                .push(Child::Node(format!("html({:?})", h.trim_end()))),
            Event::Rule => top.children.push(Child::Node("rule".into())),
            other => top.children.push(Child::Node(format!("{other:?}"))),
        }
    }
    render(stack.pop().expect("root frame"))
}
