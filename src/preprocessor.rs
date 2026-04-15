//! 前処理パイプライン
//!
//! モードに応じて DOM ノードをフィルタリングし、変換用 HTML を再構築する。
//! 非再帰 DFS（`Vec` スタック）を使用するためスタックオーバーフローが発生しない。

use scraper::{Html, Node};

use crate::options::{ConversionMode, ConversionOptions};

// ─── 定数 ─────────────────────────────────────────────────────────────────

const ALWAYS_DROP: &[&str] = &[
    "script", "style", "meta", "link", "template", "iframe", "object", "embed", "noscript",
];

const SHELL_TAGS: &[&str] = &["nav", "header", "footer", "aside"];

/// アンラップ候補のラッパー系タグ（構造タグ以外）。
const WRAPPER_TAGS: &[&str] = &["span", "div", "section", "article", "main"];

const SEMANTIC_ATTRS: &[&str] = &[
    "href", "src", "alt", "title", "lang", "dir", "type", "start", "colspan", "rowspan",
];

// ─── エントリポイント ──────────────────────────────────────────────────────

/// DOM を前処理して変換用 HTML 文字列を返す。
/// 非再帰 DFS で実装されているためスタックオーバーフローが発生しない。
pub fn preprocess(document: &Html, opts: &ConversionOptions) -> String {
    let mut out = String::with_capacity(document.html().len());

    // スタックの各エントリは (ノード, 開きタグ出力済みか)
    // Enter: 開きタグを出力して子を積む
    // Leave: 閉じタグを出力する
    enum Frame<'a> {
        Enter(ego_tree::NodeRef<'a, Node>),
        Leave { tag: String },
    }

    let mut stack: Vec<Frame> = vec![Frame::Enter(document.tree.root())];

    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Enter(node) => match node.value() {
                Node::Document | Node::Fragment => {
                    // 子をすべて逆順でプッシュ
                    for child in node.children().rev() {
                        stack.push(Frame::Enter(child));
                    }
                }
                Node::Element(elem) => {
                    let tag = elem.name().to_ascii_lowercase();

                    // 全モード共通除外
                    if ALWAYS_DROP.contains(&tag.as_str()) {
                        continue;
                    }
                    // シェル除外
                    if opts.drop_interactive_shell && SHELL_TAGS.contains(&tag.as_str()) {
                        continue;
                    }
                    // ラッパーアンラップ
                    if opts.unwrap_unknown_wrappers
                        && WRAPPER_TAGS.contains(&tag.as_str())
                        && !is_structural(&tag)
                    {
                        for child in node.children().rev() {
                            stack.push(Frame::Enter(child));
                        }
                        continue;
                    }

                    // 開きタグを出力
                    out.push('<');
                    out.push_str(&tag);
                    emit_attrs(elem, opts, &mut out);

                    if is_void_element(&tag) {
                        out.push('>');
                        continue;
                    }
                    out.push('>');

                    // 閉じタグを後で出力するためにプッシュ
                    stack.push(Frame::Leave { tag: tag.clone() });

                    // 子を逆順でプッシュ
                    for child in node.children().rev() {
                        stack.push(Frame::Enter(child));
                    }
                }
                Node::Text(text) => {
                    for ch in text.text.chars() {
                        match ch {
                            '&' => out.push_str("&amp;"),
                            '<' => out.push_str("&lt;"),
                            '>' => out.push_str("&gt;"),
                            '"' => out.push_str("&quot;"),
                            c => out.push(c),
                        }
                    }
                }
                Node::Comment(c) => {
                    if opts.mode == ConversionMode::Preserve {
                        out.push_str("<!--");
                        out.push_str(&c.comment);
                        out.push_str("-->");
                    }
                }
                _ => {}
            },
            Frame::Leave { tag } => {
                out.push_str("</");
                out.push_str(&tag);
                out.push('>');
            }
        }
    }

    out
}

// ─── 属性フィルタリング ───────────────────────────────────────────────────

fn emit_attrs(elem: &scraper::node::Element, opts: &ConversionOptions, out: &mut String) {
    for (key, val) in &elem.attrs {
        let k = key.local.as_ref();

        // 意味属性は常に保持
        if SEMANTIC_ATTRS.contains(&k) {
            push_attr(out, k, val);
            continue;
        }

        // aria-* 属性
        if k.starts_with("aria-") {
            if opts.preserve_aria_attrs {
                push_attr(out, k, val);
            }
            continue;
        }

        // data-* 属性
        if k.starts_with("data-") {
            if opts.preserve_data_attrs {
                push_attr(out, k, val);
            }
            continue;
        }

        // id 属性
        if k == "id" {
            if opts.preserve_ids {
                push_attr(out, k, val);
            }
            continue;
        }

        // class 属性（"language-*" は常に保持）
        if k == "class" {
            let has_lang = val.split_whitespace().any(|c| c.starts_with("language-"));
            if has_lang || opts.preserve_classes {
                push_attr(out, k, val);
            }
            continue;
        }

        // style 属性（常に装飾属性）
        if k == "style" {
            if !opts.drop_presentation_attrs {
                push_attr(out, k, val);
            }
            continue;
        }

        // preserve / strict では追加属性を保持
        if matches!(opts.mode, ConversionMode::Preserve | ConversionMode::Strict) {
            push_attr(out, k, val);
            continue;
        }

        // 未知の属性
        if opts.preserve_unknown_attrs {
            push_attr(out, k, val);
        }
    }
}

#[inline]
fn push_attr(out: &mut String, key: &str, val: &str) {
    out.push(' ');
    out.push_str(key);
    out.push_str("=\"");
    for ch in val.chars() {
        if ch == '"' {
            out.push_str("&quot;");
        } else {
            out.push(ch);
        }
    }
    out.push('"');
}

// ─── ヘルパー ──────────────────────────────────────────────────────────────

fn is_structural(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "ul"
            | "ol"
            | "li"
            | "blockquote"
            | "pre"
            | "code"
            | "table"
            | "thead"
            | "tbody"
            | "tr"
            | "th"
            | "td"
            | "a"
            | "img"
            | "strong"
            | "b"
            | "em"
            | "i"
            | "hr"
            | "br"
            | "figure"
            | "figcaption"
    )
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

// ─── テスト ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
