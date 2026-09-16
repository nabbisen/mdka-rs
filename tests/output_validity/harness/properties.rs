//! The intent-free properties: checks that need no per-input expectation, so
//! a directory of real-world HTML can be run through them as data.

use ego_tree::iter::Edge;
use mdka::options::ConversionOptions;
use pulldown_cmark::{Event, Tag, TagEnd};
use scraper::{Html, Node};

use super::structure::{parser, tag_name};

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

/// What the HTML says, read from the same parse mdka performs.
pub struct HtmlFacts {
    /// Visible text, and alt text of images outside code, with a space at
    /// every block edge.
    pub text: String,
    /// `href` of every `<a>` outside `<pre>`/`<code>`.
    pub links: Vec<String>,
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
        images: Vec::new(),
        pres: Vec::new(),
        skeleton: Vec::new(),
    };
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
                        match name {
                            "pre" if pre.is_none() => pre = Some((depth, String::new())),
                            "code" | "pre" => code += 1,
                            "a" if code == 0 && pre.is_none() => {
                                if let Some(h) = e.attr("href").filter(|h| !h.is_empty()) {
                                    facts.links.push(h.to_string());
                                }
                            }
                            // Alt text stands in for the image only where an
                            // image can exist; inside code it has no place.
                            "img" if code == 0 && pre.is_none() => {
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

pub fn markdown_facts(md: &str) -> MarkdownFacts {
    let mut facts = MarkdownFacts {
        text: String::new(),
        links: Vec::new(),
        images: Vec::new(),
        code_blocks: Vec::new(),
        skeleton: Vec::new(),
    };
    let mut block: Option<String> = None;
    // (skeleton index, text offset) per open tag; None for tags outside it.
    let mut open: Vec<Option<(usize, usize)>> = Vec::new();
    for event in parser(md) {
        match event {
            Event::Start(tag) => {
                match &tag {
                    Tag::Link { dest_url, .. } => facts.links.push(dest_url.to_string()),
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
                    TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link | TagEnd::Image
                );
                if !inline {
                    facts.text.push(' ');
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
            Event::Code(c) => facts.text.push_str(&c),
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
fn terminated(md: &str) -> bool {
    let probe = format!("{md}\n\n{SENTINEL}\n");
    let mut depth = 0usize;
    let mut last_top: Option<(bool, String)> = None;
    for event in parser(&probe) {
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

/// The intent-free properties (handoff §6, plus `blocks` and `unterminated`).
/// Each violation names its property.
pub fn properties(html: &str, md: &str, opts: &ConversionOptions) -> Vec<String> {
    let h = html_facts(html, opts);
    let m = markdown_facts(md);
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
    if !terminated(md) {
        out.push(
            "[unterminated] a paragraph appended after the output is swallowed (open fence or HTML block)"
                .to_string(),
        );
    }

    out
}
