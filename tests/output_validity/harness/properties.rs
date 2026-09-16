//! The intent-free properties: checks that need no per-input expectation, so
//! a directory of real-world HTML can be run through them as data.

use ego_tree::iter::Edge;
use mdka::options::ConversionOptions;
use pulldown_cmark::{Event, Tag, TagEnd};
use scraper::{Html, Node};

use super::structure::{Reading, parser, tag_name};

/// Elements whose content a browser never renders as text.
const INVISIBLE: &[&str] = &[
    "script", "style", "template", "head", "title", "meta", "link", "iframe", "noscript",
];

/// Elements whose start and end separate words.
const HTML_BLOCKS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "br",
    "dd",
    "details",
    "dialog",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "html",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

/// HTML elements that have a Markdown block counterpart, and its name in the
/// `structure` notation. `p` is deliberately absent: `div`, bare text and
/// tight list items all become paragraphs (or none), so paragraph count is
/// not something the HTML determines.
fn skeleton_kind(html_name: &str) -> Option<&'static str> {
    Some(match html_name {
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "h4" => "h4",
        "h5" => "h5",
        "h6" => "h6",
        "blockquote" => "quote",
        "ul" => "ul",
        "ol" => "ol",
        "li" => "li",
        "pre" => "codeblock",
        _ => return None,
    })
}

/// A block of the skeleton: its kind and its whitespace-normalised text.
type Skeleton = Vec<(&'static str, String)>;

fn words(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether an emphasis element's **own** `style` says it is not emphasis, by
/// RFC 028 Amendment 1's written rule -- implemented here from that text, not
/// by calling mdka, so the harness cannot share a defect with the code it
/// checks.
///
/// `<b>`/`<strong>`: `font-weight` is `normal` or a number <= 500.
/// `<i>`/`<em>`: `font-style` is `normal`. Declarations split on `;`; name and
/// value trimmed and case-folded; `!important` stripped; the last declaration
/// of the property wins; nothing else is read.
pub fn emphasis_negated_by_own_style(name: &str, style: Option<&str>) -> bool {
    let property = match name {
        "b" | "strong" => "font-weight",
        "i" | "em" => "font-style",
        _ => return false,
    };
    let Some(style) = style else { return false };
    let mut last: Option<String> = None;
    for declaration in style.split(';') {
        let Some((key, value)) = declaration.split_once(':') else {
            continue;
        };
        if key.trim().to_ascii_lowercase() != property {
            continue;
        }
        let mut value = value.trim().to_ascii_lowercase();
        if let Some(stripped) = value.strip_suffix("!important") {
            value = stripped.trim().to_string();
        }
        last = Some(value);
    }
    let Some(value) = last else { return false };
    match property {
        "font-weight" => value == "normal" || value.parse::<f64>().is_ok_and(|n| n <= 500.0),
        _ => value == "normal",
    }
}

/// What one link holds: its destination, its words, and the inline elements
/// inside it in order (`strong`, `em`, `code`, `image[src]`).
#[derive(Debug, PartialEq, Eq)]
pub struct LinkContent {
    pub dest: String,
    pub text: String,
    pub inline: Vec<String>,
}

impl LinkContent {
    fn show(&self) -> String {
        format!("text {:?} inline [{}]", self.text, self.inline.join(", "))
    }
}

/// What the HTML says, read from the same parse mdka performs.
pub struct HtmlFacts {
    /// Visible text, and alt text of images outside code, with a space at
    /// every block edge.
    pub text: String,
    /// `href` of every `<a>` outside `<pre>`/`<code>` that holds text or an
    /// image. An empty link is exempt: RFC 025 review Q1 directs that a link
    /// with no text and no image emits nothing.
    pub links: Vec<String>,
    /// The content of each of those links.
    pub link_contents: Vec<LinkContent>,
    /// `src` of every `<img>` outside `<pre>`/`<code>`.
    pub images: Vec<String>,
    /// Text content of every outermost `<pre>`.
    pub pres: Vec<String>,
    /// Headings, quotes, lists, items and preformatted blocks, in document order.
    pub skeleton: Skeleton,
}

pub fn html_facts(html: &str, opts: &ConversionOptions) -> HtmlFacts {
    let doc = Html::parse_document(html);
    let mut facts = HtmlFacts {
        text: String::new(),
        links: Vec::new(),
        link_contents: Vec::new(),
        images: Vec::new(),
        pres: Vec::new(),
        skeleton: Vec::new(),
    };
    // (depth, href, text offset, inline elements) for each open link.
    let mut anchors: Vec<(usize, String, usize, Vec<String>)> = Vec::new();
    let mut hidden = 0usize;
    let mut code = 0usize;
    let mut pre: Option<(usize, String)> = None;
    let mut depth = 0usize;
    // (depth, skeleton index, text offset) for each open skeleton element.
    let mut open: Vec<(usize, usize, usize)> = Vec::new();
    for edge in doc.tree.root().traverse() {
        match edge {
            Edge::Open(node) => {
                depth += 1;
                match node.value() {
                    Node::Element(e) => {
                        let name = e.name();
                        // The documented option drops these elements with
                        // their content; a mode that asks for that is not
                        // losing text.
                        let shell = opts.drop_interactive_shell
                            && matches!(name, "nav" | "header" | "footer" | "aside");
                        if hidden > 0 || INVISIBLE.contains(&name) || shell {
                            hidden += 1;
                            continue;
                        }
                        if HTML_BLOCKS.contains(&name) {
                            facts.text.push(' ');
                        }
                        let nested_pre = name == "pre" && pre.is_some();
                        if let Some(kind) = skeleton_kind(name).filter(|_| !nested_pre) {
                            open.push((depth, facts.skeleton.len(), facts.text.len()));
                            facts.skeleton.push((kind, String::new()));
                        }
                        let in_code = code > 0 || pre.is_some();
                        let negated = emphasis_negated_by_own_style(name, e.attr("style"));
                        let inline_kind = match name {
                            "strong" | "b" if !negated => Some("strong".to_string()),
                            "em" | "i" if !negated => Some("em".to_string()),
                            "code" => Some("code".to_string()),
                            "img" => e
                                .attr("src")
                                .filter(|s| !s.is_empty())
                                .map(|s| format!("image[{s}]")),
                            _ => None,
                        };
                        if let (Some(kind), Some(anchor), false) =
                            (inline_kind, anchors.last_mut(), in_code)
                        {
                            anchor.3.push(kind);
                        }
                        match name {
                            "pre" if pre.is_none() => pre = Some((depth, String::new())),
                            "code" | "pre" => code += 1,
                            "a" if !in_code => {
                                if let Some(h) = e.attr("href").filter(|h| !h.is_empty()) {
                                    anchors.push((
                                        depth,
                                        h.to_string(),
                                        facts.text.len(),
                                        Vec::new(),
                                    ));
                                }
                            }
                            // Alt text stands in for the image only where an
                            // image can exist; inside code it has no place.
                            "img" if !in_code => {
                                if let Some(alt) = e.attr("alt") {
                                    facts.text.push_str(alt);
                                }
                                if let Some(s) = e.attr("src").filter(|s| !s.is_empty()) {
                                    facts.images.push(s.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                    Node::Text(t) if hidden == 0 => {
                        facts.text.push_str(t);
                        if let Some((_, buf)) = pre.as_mut() {
                            buf.push_str(t);
                        }
                    }
                    _ => {}
                }
            }
            Edge::Close(node) => {
                if let Node::Element(e) = node.value() {
                    if hidden > 0 {
                        hidden -= 1;
                    } else {
                        let name = e.name();
                        if HTML_BLOCKS.contains(&name) {
                            facts.text.push(' ');
                        }
                        if open.last().is_some_and(|(d, _, _)| *d == depth) {
                            let (_, i, from) = open.pop().expect("open skeleton element");
                            facts.skeleton[i].1 = words(&facts.text[from..]);
                        }
                        if name == "a" && anchors.last().is_some_and(|(d, ..)| *d == depth) {
                            let (_, dest, from, inline) = anchors.pop().expect("open link");
                            let text = words(&facts.text[from..]);
                            if !text.is_empty() || inline.iter().any(|k| k.starts_with("image[")) {
                                facts.links.push(dest.clone());
                                facts.link_contents.push(LinkContent { dest, text, inline });
                            }
                        }
                        match name {
                            "pre" if pre.as_ref().is_some_and(|(d, _)| *d == depth) => {
                                let (_, buf) = pre.take().expect("open pre");
                                facts.pres.push(buf);
                            }
                            "code" | "pre" => code = code.saturating_sub(1),
                            _ => {}
                        }
                    }
                }
                depth -= 1;
            }
        }
    }
    facts
}

/// What the Markdown says, read from the parsed events.
pub struct MarkdownFacts {
    pub text: String,
    pub links: Vec<String>,
    pub link_contents: Vec<LinkContent>,
    pub images: Vec<String>,
    pub code_blocks: Vec<String>,
    pub skeleton: Skeleton,
}

fn markdown_skeleton_kind(tag: &Tag<'_>) -> Option<&'static str> {
    Some(match tag {
        Tag::Heading { level, .. } => ["h1", "h2", "h3", "h4", "h5", "h6"][*level as usize - 1],
        Tag::BlockQuote(_) => "quote",
        Tag::List(None) => "ul",
        Tag::List(Some(_)) => "ol",
        Tag::Item => "li",
        Tag::CodeBlock(_) => "codeblock",
        _ => return None,
    })
}

pub fn markdown_facts(md: &str, reading: Reading) -> MarkdownFacts {
    let mut facts = MarkdownFacts {
        text: String::new(),
        links: Vec::new(),
        link_contents: Vec::new(),
        images: Vec::new(),
        code_blocks: Vec::new(),
        skeleton: Vec::new(),
    };
    let mut block: Option<String> = None;
    // (skeleton index, text offset) per open tag; None for tags outside it.
    let mut open: Vec<Option<(usize, usize)>> = Vec::new();
    // (destination, text offset, inline elements) for each open link.
    let mut links: Vec<(String, usize, Vec<String>)> = Vec::new();
    for event in parser(md, reading) {
        match event {
            Event::Start(tag) => {
                let kind = match &tag {
                    Tag::Strong => Some("strong".to_string()),
                    Tag::Emphasis => Some("em".to_string()),
                    Tag::Strikethrough => Some("del".to_string()),
                    Tag::Image { dest_url, .. } => Some(format!("image[{dest_url}]")),
                    _ => None,
                };
                if let (Some(kind), Some(link)) = (kind, links.last_mut()) {
                    link.2.push(kind);
                }
                match &tag {
                    Tag::Link { dest_url, .. } => {
                        facts.links.push(dest_url.to_string());
                        links.push((dest_url.to_string(), facts.text.len(), Vec::new()));
                    }
                    Tag::Image { dest_url, .. } => facts.images.push(dest_url.to_string()),
                    Tag::CodeBlock(_) => block = Some(String::new()),
                    _ => {}
                }
                if tag_name(&tag).1 {
                    facts.text.push(' ');
                }
                open.push(markdown_skeleton_kind(&tag).map(|kind| {
                    facts.skeleton.push((kind, String::new()));
                    (facts.skeleton.len() - 1, facts.text.len())
                }));
            }
            Event::End(end) => {
                if matches!(end, TagEnd::CodeBlock) {
                    facts.code_blocks.push(block.take().unwrap_or_default());
                }
                let inline = matches!(
                    end,
                    TagEnd::Emphasis
                        | TagEnd::Strong
                        | TagEnd::Strikethrough
                        | TagEnd::Link
                        | TagEnd::Image
                );
                if !inline {
                    facts.text.push(' ');
                }
                if matches!(end, TagEnd::Link)
                    && let Some((dest, from, inline)) = links.pop()
                {
                    let text = words(&facts.text[from..]);
                    facts.link_contents.push(LinkContent { dest, text, inline });
                }
                if let Some((i, from)) = open.pop().flatten() {
                    facts.skeleton[i].1 = words(&facts.text[from..]);
                }
            }
            Event::Text(t) => {
                facts.text.push_str(&t);
                if let Some(b) = block.as_mut() {
                    b.push_str(&t);
                }
            }
            Event::Code(c) => {
                facts.text.push_str(&c);
                if let Some(link) = links.last_mut() {
                    link.2.push("code".to_string());
                }
            }
            Event::SoftBreak | Event::HardBreak | Event::Rule => facts.text.push(' '),
            _ => {}
        }
    }
    facts
}

fn missing(want: &[String], have: &[String]) -> Vec<String> {
    let mut have: Vec<&String> = have.iter().collect();
    let mut lost = Vec::new();
    for w in want {
        match have.iter().position(|h| *h == w) {
            Some(i) => {
                have.remove(i);
            }
            None => lost.push(w.clone()),
        }
    }
    lost
}

fn strip_one_newline(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}

const SENTINEL: &str = "MDKA-HARNESS-SENTINEL";

/// Whether a paragraph written after the output is read as a paragraph. A
/// code fence or HTML block left open swallows it.
fn terminated(md: &str, reading: Reading) -> bool {
    let probe = format!("{md}\n\n{SENTINEL}\n");
    let mut depth = 0usize;
    let mut last_top: Option<(bool, String)> = None;
    for event in parser(&probe, reading) {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    last_top = Some((matches!(tag, Tag::Paragraph), String::new()));
                }
                depth += 1;
            }
            Event::End(_) => depth -= 1,
            Event::Text(t) if depth == 1 => {
                if let Some((_, s)) = last_top.as_mut() {
                    s.push_str(&t);
                }
            }
            _ => {}
        }
    }
    matches!(last_top, Some((true, s)) if s == SENTINEL)
}

fn show(skeleton: &Skeleton) -> String {
    let parts: Vec<String> = skeleton.iter().map(|(k, t)| format!("{k} {t:?}")).collect();
    format!("[{}]", parts.join(", "))
}

/// The intent-free properties (handoff §6, plus `blocks`, `unterminated` and
/// `link-content`), read under `reading`. Each violation names its property.
pub fn properties(html: &str, md: &str, opts: &ConversionOptions, reading: Reading) -> Vec<String> {
    let h = html_facts(html, opts);
    let m = markdown_facts(md, reading);
    let mut out = Vec::new();

    // 6.3 No stray delimiter text: more of a delimiter character in the parsed
    // text than the HTML's text contains came from mdka's own markup.
    for ch in ['*', '_', '`'] {
        let (hn, mn) = (h.text.matches(ch).count(), m.text.matches(ch).count());
        if mn > hn {
            out.push(format!(
                "[stray-delimiter] {} extra {ch:?} in the parsed text (HTML text has {hn}, parsed text has {mn})",
                mn - hn
            ));
        }
    }

    // 6.3 / 6.1 Text survives, and whatever was escaped parses back to the
    // original characters.
    let ht: Vec<&str> = h.text.split_whitespace().collect();
    let mt: Vec<&str> = m.text.split_whitespace().collect();
    if ht != mt {
        let i = ht
            .iter()
            .zip(&mt)
            .position(|(a, b)| a != b)
            .unwrap_or(ht.len().min(mt.len()));
        let from = i.saturating_sub(3);
        out.push(format!(
            "[text] parsed text differs from the HTML text at word {i}: HTML {:?} vs parsed {:?}",
            ht[from..(i + 4).min(ht.len())].join(" "),
            mt[from..(i + 4).min(mt.len())].join(" "),
        ));
    }

    // 6.3 Links and images survive, with their destinations (6.2).
    for lost in missing(&h.links, &m.links) {
        out.push(format!(
            "[link-lost] no parsed link with destination {lost:?}"
        ));
    }
    for lost in missing(&h.images, &m.images) {
        out.push(format!("[image-lost] no parsed image with source {lost:?}"));
    }

    // 025c (review Q10): each link still holds what the HTML put inside it --
    // its words and its inline elements, images included. Only links whose
    // destination was found; a lost destination is reported above.
    let mut have: Vec<&LinkContent> = m.link_contents.iter().collect();
    for want in &h.link_contents {
        if let Some(i) = have.iter().position(|c| *c == want) {
            have.remove(i);
        } else if let Some(got) = have.iter().find(|c| c.dest == want.dest) {
            out.push(format!(
                "[link-content] link {:?}: HTML holds {} vs parsed {}",
                want.dest,
                want.show(),
                got.show()
            ));
        }
    }

    // 6.2 Every <pre> is one code block holding exactly its text: a fence too
    // short for its content fails here.
    let want: Vec<String> = h
        .pres
        .iter()
        .map(|p| strip_one_newline(p).to_string())
        .collect();
    let have: Vec<String> = m
        .code_blocks
        .iter()
        .map(|c| strip_one_newline(c).to_string())
        .collect();
    for lost in missing(&want, &have) {
        out.push(format!(
            "[code-block] no parsed code block with content {lost:?}"
        ));
    }

    // Beyond §6: headings, quotes, lists, items and code blocks survive as
    // those blocks, holding their text. Without this, `<a><h2>x</h2></a>` →
    // an empty heading followed by the text passes every property above.
    if h.skeleton != m.skeleton {
        out.push(format!(
            "[blocks] HTML {} vs parsed {}",
            show(&h.skeleton),
            show(&m.skeleton)
        ));
    }

    // Beyond §6: nothing is left open. An unmatched fence turns the rest of
    // any document the output is placed in into code.
    if !terminated(md, reading) {
        out.push(
            "[unterminated] a paragraph appended after the output is swallowed (open fence or HTML block)"
                .to_string(),
        );
    }

    out
}
