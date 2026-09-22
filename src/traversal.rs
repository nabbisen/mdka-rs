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
/// [`structure_hints`], so the pre-pass sees exactly the elements
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

/// What one pass over the document knows before rendering.
struct Hints {
    /// Inline wrapper elements (`strong`/`b`, `em`/`i`, `code`, `a`) with a
    /// rendered block among their descendants (RFC 028).
    wrappers: HashSet<NodeId>,
    /// Lists (`ul`/`ol`) that are loose (RFC 035 §3.1).
    loose_lists: HashSet<NodeId>,
}

/// What an element's rendered content amounts to, for counting the blocks of
/// a list item (RFC 035 §3.1): how many blocks, where a maximal run of inline
/// content is one block, and whether it starts or ends with inline content
/// (so that runs meeting across element boundaries merge). A nested list is
/// a barrier: it separates runs and is not counted.
#[derive(Clone, Copy, Default)]
struct Units {
    count: usize,
    leading_inline: bool,
    trailing_inline: bool,
    empty: bool,
}

impl Units {
    const NONE: Units = Units {
        count: 0,
        leading_inline: false,
        trailing_inline: false,
        empty: true,
    };
    const INLINE: Units = Units {
        count: 1,
        leading_inline: true,
        trailing_inline: true,
        empty: false,
    };
    const BLOCK: Units = Units {
        count: 1,
        leading_inline: false,
        trailing_inline: false,
        empty: false,
    };
    const BARRIER: Units = Units {
        count: 0,
        leading_inline: false,
        trailing_inline: false,
        empty: false,
    };

    fn append(&mut self, next: Units) {
        if next.empty {
            return;
        }
        if self.empty {
            *self = next;
            return;
        }
        let merged = usize::from(self.trailing_inline && next.leading_inline);
        self.count = self.count + next.count - merged;
        self.trailing_inline = next.trailing_inline;
    }
}

/// One element open during the pass.
struct Frame {
    disposition: Disposition,
    /// A rendered block has been seen among its descendants.
    has_block: bool,
    /// Its rendered content so far, as counted units.
    units: Units,
    /// A rendered `ul`/`ol`, the list its `li` descendants belong to.
    is_list: bool,
    id: NodeId,
}

/// One bottom-up pass over the document computing [`Hints`] -- O(n) in total,
/// not O(subtree) per element, so one `<b>` wrapping a whole Google Docs
/// payload costs nothing extra -- and non-recursive (`Traverse` is an
/// iterator), so deep nesting cannot overflow the stack. It follows the
/// traversal's own decisions: skipped subtrees contribute nothing, and
/// unwrapped wrappers are not blocks themselves but pass their content up.
fn structure_hints(document: &Html, opts: &ConversionOptions) -> Hints {
    let mut hints = Hints {
        wrappers: HashSet::new(),
        loose_lists: HashSet::new(),
    };
    let mut open: Vec<Frame> = Vec::with_capacity(64);
    for edge in document.tree.root().traverse() {
        match edge {
            Edge::Open(node) => match node.value() {
                scraper::Node::Element(elem) => {
                    let tag = elem.name();
                    open.push(Frame {
                        disposition: disposition(tag, opts),
                        has_block: false,
                        units: Units::NONE,
                        is_list: disposition(tag, opts) == Disposition::Render
                            && matches!(
                                utils::block_kind(tag),
                                Some(utils::Block::UnorderedList | utils::Block::OrderedList)
                            ),
                        id: node.id(),
                    });
                }
                scraper::Node::Text(text) if !text.trim().is_empty() => {
                    if let Some(frame) = open.last_mut() {
                        frame.units.append(Units::INLINE);
                    }
                }
                _ => {}
            },
            Edge::Close(node) => {
                let scraper::Node::Element(elem) = node.value() else {
                    continue;
                };
                let Some(frame) = open.pop() else {
                    continue;
                };
                if frame.disposition == Disposition::Skip {
                    continue;
                }
                let tag = elem.name();
                let render = frame.disposition == Disposition::Render;
                // An unwrapped wrapper that still separates its content
                // (`div`/`section`/`article`/`main`; RFC 036 §5.2, `036d`)
                // is, for every purpose downstream of this pass, the
                // paragraph-like block it stands in for -- not rendering it
                // itself no longer means "no block here". Anything that
                // depends on that (RFC 028's "does an inline wrapper enclose
                // a block" detection, just below) must agree with what the
                // traversal now actually does.
                let kind = if render {
                    utils::block_kind(tag)
                } else if frame.disposition == Disposition::Unwrap {
                    utils::block_kind(tag).filter(|k| *k == utils::Block::Paragraph)
                } else {
                    None
                };
                if frame.has_block && render && utils::is_inline_wrapper(tag) {
                    hints.wrappers.insert(node.id());
                }
                // RFC 035 §3.1: two or more blocks in an item make its list loose.
                if kind == Some(utils::Block::ListItem)
                    && frame.units.count >= 2
                    && let Some(list) = open.iter().rev().find(|f| f.is_list)
                {
                    hints.loose_lists.insert(list.id);
                }
                let contribution = match kind {
                    Some(utils::Block::UnorderedList | utils::Block::OrderedList) => Units::BARRIER,
                    Some(utils::Block::ListItem) => Units::BARRIER,
                    // A paragraph-like block holding blocks renders as those
                    // blocks, so its content counts, not itself.
                    Some(utils::Block::Paragraph) if frame.has_block => frame.units,
                    Some(_) => Units::BLOCK,
                    None if !render => frame.units,
                    // An inline element around blocks renders as its content.
                    None if frame.has_block => frame.units,
                    None if matches!(tag, "img" | "br") => Units::INLINE,
                    None if frame.units.empty => Units::NONE,
                    None => Units::INLINE,
                };
                let is_block = kind.is_some();
                if let Some(parent) = open.last_mut() {
                    parent.has_block |= frame.has_block || is_block;
                    parent.units.append(contribution);
                }
            }
        }
    }
    hints
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
    let hints = structure_hints(document, opts);

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
                        // A wrapper classified as a paragraph-like block when
                        // rendered (`div`, `section`, `article`, `main`; not
                        // `span`, which is inline and never was one) still
                        // separates its content from what surrounds it even
                        // unwrapped -- unwrapping removes the tag, not the
                        // paragraph break it stood for (RFC 036 §5.2, `036d`).
                        Disposition::Unwrap => {
                            let separates = utils::block_kind(tag) == Some(utils::Block::Paragraph);
                            if separates {
                                renderer.begin_unwrapped_separator();
                                stack.push(Event::Leave(node));
                            }
                            for child in node.children().rev() {
                                stack.push(Event::Enter(child));
                            }
                            continue;
                        }
                        Disposition::Render => {}
                    }

                    renderer.enter_element(
                        elem,
                        opts.preserve_ids,
                        hints.wrappers.contains(&node.id()),
                        hints.loose_lists.contains(&node.id()),
                    );

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
                    // Only ever pushed for a rendered element, or for an
                    // unwrapped separating wrapper (see the `Unwrap` arm
                    // above) -- never for `Skip`, and never for `Unwrap`
                    // without a separator.
                    if disposition(elem.name(), opts) == Disposition::Unwrap {
                        renderer.end_unwrapped_separator();
                    } else {
                        renderer.leave_element(elem);
                    }
                }
            }
        }
    }

    renderer.finish()
}
