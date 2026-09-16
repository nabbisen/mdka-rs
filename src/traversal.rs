//! DOM トラバーサル + Markdown 生成
//!
//! 1回のパース済み DOM を直接トラバースして Markdown を生成する。
//! 前処理（タグ除外・アンラップ）は Enter 時にインラインで判定するため、
//! HTML 文字列の再構築と再パースが不要になっている。
//!
//! 非再帰 DFS（`Vec` スタック）を使用するためスタックオーバーフローが発生しない。

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use crate::options::ConversionOptions;
use crate::renderer::MarkdownRenderer;
use crate::utils;
use ego_tree::NodeId;
use ego_tree::iter::Edge;
use scraper::Html;

/// トラバーサルイベント：Enter（開きタグ相当）と Leave（閉じタグ相当）
enum Event<'a> {
    Enter(ego_tree::NodeRef<'a, scraper::Node>),
    Leave(ego_tree::NodeRef<'a, scraper::Node>),
}

/// What the traversal does with an element. Shared by the traversal and by
/// [`inline_wrappers_of_blocks`], so the pre-pass sees exactly the elements
/// the renderer will see.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Disposition {
    /// Skipped with its whole subtree.
    Skip,
    /// Not rendered itself; its children are.
    Unwrap,
    Render,
}

fn disposition(tag: &str, opts: &ConversionOptions) -> Disposition {
    if utils::is_skip_tag(tag) || (opts.drop_interactive_shell && utils::is_shell_tag(tag)) {
        Disposition::Skip
    } else if opts.unwrap_unknown_wrappers
        && utils::is_wrapper_tag(tag)
        && !utils::is_structural_tag(tag)
    {
        Disposition::Unwrap
    } else {
        Disposition::Render
    }
}

/// The inline wrapper elements (`strong`/`b`, `em`/`i`, `code`, `a`) that have
/// a rendered block among their descendants (RFC 028).
///
/// One bottom-up pass over the document -- O(n) in total, not O(subtree) per
/// element, so one `<b>` wrapping a whole Google Docs payload costs nothing
/// extra -- and non-recursive (`Traverse` is an iterator), so deep nesting
/// cannot overflow the stack. A block counts if the traversal will render it:
/// skipped subtrees contribute nothing, unwrapped wrappers are not blocks
/// themselves but pass their children's blocks up.
fn inline_wrappers_of_blocks(document: &Html, opts: &ConversionOptions) -> HashSet<NodeId> {
    let mut wrappers = HashSet::new();
    // Per open element: its disposition, and whether a rendered block has been
    // seen among its descendants so far.
    let mut open: Vec<(Disposition, bool)> = Vec::with_capacity(64);
    for edge in document.tree.root().traverse() {
        match edge {
            Edge::Open(node) => {
                if let scraper::Node::Element(elem) = node.value() {
                    open.push((disposition(elem.name(), opts), false));
                }
            }
            Edge::Close(node) => {
                let scraper::Node::Element(elem) = node.value() else {
                    continue;
                };
                let Some((disp, has_block)) = open.pop() else {
                    continue;
                };
                if disp == Disposition::Skip {
                    continue;
                }
                let tag = elem.name();
                let is_block = disp == Disposition::Render && utils::block_kind(tag).is_some();
                if has_block && disp == Disposition::Render && utils::is_inline_wrapper(tag) {
                    wrappers.insert(node.id());
                }
                if let Some(parent) = open.last_mut() {
                    parent.1 |= has_block || is_block;
                }
            }
        }
    }
    wrappers
}

/// HTML ドキュメントをトラバースして Markdown 文字列を生成する。
///
/// 再帰を使わず `Vec` ベースのスタックで深さ優先探索を行うため、
/// 10,000段以上のネストでもスタックオーバーフローが発生しない。
///
/// 前処理（タグ除外・ラッパーアンラップ）もこの関数内でインライン実行する。
pub fn traverse(document: &Html, opts: &ConversionOptions) -> String {
    // 元の HTML サイズの半分を初期容量として確保
    let capacity = document.html().len() / 2;
    let mut renderer = MarkdownRenderer::new(capacity.max(256));
    let wrappers = inline_wrappers_of_blocks(document, opts);

    // root() は Document ノードなので子ノードだけを逆順で積む
    let mut stack: Vec<Event> = Vec::with_capacity(64);
    for child in document.tree.root().children().rev() {
        stack.push(Event::Enter(child));
    }

    while let Some(event) = stack.pop() {
        match event {
            Event::Enter(node) => match node.value() {
                scraper::Node::Element(elem) => {
                    // scraper (html5ever) はタグ名を小文字正規化済みで保持する
                    let tag = elem.name();

                    match disposition(tag, opts) {
                        // Skipped with its content.
                        Disposition::Skip => continue,
                        // Not rendered itself; only its children are traversed.
                        Disposition::Unwrap => {
                            for child in node.children().rev() {
                                stack.push(Event::Enter(child));
                            }
                            continue;
                        }
                        Disposition::Render => {}
                    }

                    renderer.enter_element(elem, opts.preserve_ids, wrappers.contains(&node.id()));

                    // Leave イベントを先にスタックへ（子より後に処理される）
                    stack.push(Event::Leave(node));

                    // 子を逆順でスタックへ
                    for child in node.children().rev() {
                        stack.push(Event::Enter(child));
                    }
                }
                scraper::Node::Text(text) => {
                    renderer.process_text(&text.text);
                }
                // Document / Comment / ProcessingInstruction などは
                // 子ノードを辿るだけで Enter/Leave は発行しない
                _ => {
                    for child in node.children().rev() {
                        stack.push(Event::Enter(child));
                    }
                }
            },
            Event::Leave(node) => {
                if let scraper::Node::Element(elem) = node.value() {
                    renderer.leave_element(elem);
                }
            }
        }
    }

    renderer.finish()
}
