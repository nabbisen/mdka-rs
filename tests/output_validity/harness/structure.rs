//! The parsed structure of a Markdown string, as a compact tree, under each
//! reading a consumer may apply.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

/// How a consumer reads the Markdown. Every tree assertion and every property
/// runs under both; a cell passes only if both readings are right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// Plain CommonMark: no extensions.
    CommonMark,
    /// GitHub Flavored Markdown as pulldown-cmark 0.13.4 offers it: tables,
    /// footnotes, strikethrough, task lists, and `ENABLE_GFM` (blockquote
    /// alerts such as `> [!NOTE]`). Not enabled, because GFM does not have
    /// them: smart punctuation, heading attributes, metadata blocks, math,
    /// definition lists, super/subscript, wikilinks. Not available in the
    /// parser at all: GFM's extended autolinks (`www.example.com`) and its
    /// disallowed-raw-HTML filter.
    Gfm,
}

pub const READINGS: [Reading; 2] = [Reading::CommonMark, Reading::Gfm];

impl Reading {
    fn options(self) -> Options {
        match self {
            Reading::CommonMark => Options::empty(),
            Reading::Gfm => {
                Options::ENABLE_TABLES
                    | Options::ENABLE_FOOTNOTES
                    | Options::ENABLE_STRIKETHROUGH
                    | Options::ENABLE_TASKLISTS
                    | Options::ENABLE_GFM
            }
        }
    }
}

impl std::fmt::Display for Reading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Reading::CommonMark => "commonmark",
            Reading::Gfm => "gfm",
        })
    }
}

pub(super) fn parser(md: &str, reading: Reading) -> Parser<'_> {
    Parser::new_ext(md, reading.options())
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
        Tag::BlockQuote(None) => ("quote".into(), true),
        Tag::BlockQuote(Some(kind)) => (format!("quote[{kind:?}]").to_lowercase(), true),
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
        Tag::Strikethrough => ("del".into(), false),
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
        Tag::Table(_) => ("table".into(), true),
        Tag::TableHead => ("thead".into(), true),
        Tag::TableRow => ("tr".into(), true),
        Tag::TableCell => ("td".into(), true),
        other => (format!("{other:?}"), true),
    }
}

/// Whether an inline HTML event is the id anchor mdka emits for an element
/// with an `id` when `preserve_ids` is on (RFC 005 Slice B1): `<a id="…">`.
pub(super) fn is_id_anchor_open(html: &str) -> bool {
    html.strip_prefix("<a id=\"")
        .and_then(|rest| rest.strip_suffix("\">"))
        .is_some_and(|id| !id.contains('"'))
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

/// The parsed structure of `md` under `reading`, as a compact string:
/// `para(link[/out](image[i.png]("pic")))`.
///
/// Text is whitespace-collapsed and trimmed at block edges; code block and
/// code span text is kept exactly (minus a code block's final newline).
///
/// `skip_id_anchors`: omit the `<a id="…"></a>` pairs `preserve_ids` emits.
/// They are a documented option's output, not part of what the HTML's content
/// means, so a mode with the option on is held to the same tree.
pub fn structure(md: &str, reading: Reading, skip_id_anchors: bool) -> String {
    let mut stack = vec![Frame {
        name: String::new(),
        block: true,
        children: Vec::new(),
    }];
    let mut skip_close = false;
    for event in parser(md, reading) {
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
            Event::InlineHtml(h) if skip_id_anchors && is_id_anchor_open(&h) => skip_close = true,
            Event::InlineHtml(h) if skip_close && &*h == "</a>" => skip_close = false,
            Event::Html(h) | Event::InlineHtml(h) => top
                .children
                .push(Child::Node(format!("html({:?})", h.trim_end()))),
            Event::Rule => top.children.push(Child::Node("rule".into())),
            Event::TaskListMarker(done) => top
                .children
                .push(Child::Node(if done { "task[x]" } else { "task[ ]" }.into())),
            Event::FootnoteReference(label) => top
                .children
                .push(Child::Node(format!("footnote_ref[{label}]"))),
            other => top.children.push(Child::Node(format!("{other:?}"))),
        }
    }
    render(stack.pop().expect("root frame"))
}
