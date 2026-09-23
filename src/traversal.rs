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

/// Whether `li`'s own first rendered child is an `<input type="checkbox">`
/// (RFC 009 §4.2), and if so, whether it is checked. A plain lookahead at
/// the `<li>` itself, not a hint the bottom-up pre-pass needs to compute:
/// unlike `wraps_blocks`/`loose_lists`/`needs_disambiguation`, this never
/// depends on anything below the checkbox itself, so there is nothing to
/// aggregate. Whitespace-only text before it does not count against it
/// (`<li>\n  <input ...>`); anything else does.
fn task_checkbox(li: ego_tree::NodeRef<'_, scraper::Node>) -> Option<bool> {
    for child in li.children() {
        match child.value() {
            scraper::Node::Text(text) if text.trim().is_empty() => continue,
            scraper::Node::Element(elem) if elem.name() == "input" => {
                return elem
                    .attr("type")
                    .is_some_and(|t| t.eq_ignore_ascii_case("checkbox"))
                    .then(|| elem.attr("checked").is_some());
            }
            _ => return None,
        }
    }
    None
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
pub(crate) struct Hints {
    /// Inline wrapper elements (`strong`/`b`, `em`/`i`, `code`, `a`) with a
    /// rendered block among their descendants (RFC 028).
    wrappers: HashSet<NodeId>,
    /// Lists (`ul`/`ol`) that are loose (RFC 035 §3.1).
    loose_lists: HashSet<NodeId>,
    /// Lists (`ul`/`ol`) whose own first item is empty (RFC 038): nested one
    /// level in after real content, such a list's marker line would
    /// otherwise be read as a setext-heading underline (unordered) or
    /// absorbed as lazy-continuation text (ordered), so entering it needs a
    /// disambiguating blank line first.
    needs_disambiguation: HashSet<NodeId>,
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
    /// For a list frame: whether its first `li` child was empty, recorded
    /// once, the first time a child closes as a `ListItem` (RFC 038).
    /// `None` until that first item closes.
    first_item_empty: Option<bool>,
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
        needs_disambiguation: HashSet::new(),
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
                        first_item_empty: None,
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
                // RFC 038: a nested list's own first item being empty is what
                // risks a marker-line collision with the parent's text -- a
                // setext underline for an unordered child, lazy-continuation
                // text for an ordered one. Recorded against the list itself.
                if matches!(
                    kind,
                    Some(utils::Block::UnorderedList | utils::Block::OrderedList)
                ) && frame.first_item_empty == Some(true)
                {
                    hints.needs_disambiguation.insert(node.id());
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
                    // RFC 038: recorded once, the first time a child closes as
                    // a `ListItem` -- later siblings don't change whether the
                    // list's *first* item was empty.
                    if kind == Some(utils::Block::ListItem)
                        && parent.is_list
                        && parent.first_item_empty.is_none()
                    {
                        parent.first_item_empty = Some(frame.units.empty);
                    }
                    parent.has_block |= frame.has_block || is_block;
                    parent.units.append(contribution);
                }
            }
        }
    }
    hints
}

/// Runs the Enter/Leave stack machine over `roots`, dispatching each element
/// through `renderer`. Shared by the top-level document walk and, for an
/// expressible GFM table's own cells and an inexpressible table's fallback
/// content (RFC 008), a subtree rooted anywhere else: same disposition rules,
/// same hints, same renderer methods, so a `<strong>`/`<a>`/`<code>` inside a
/// table cell behaves exactly as it would anywhere else in the document.
///
/// `tables` is consulted for every `<table>` this reaches, including one
/// found while driving a fallback table's own cell content -- a nested
/// table is analyzed and rendered exactly like a top-level one.
pub(crate) fn drive<'a>(
    renderer: &mut MarkdownRenderer,
    roots: impl DoubleEndedIterator<Item = ego_tree::NodeRef<'a, scraper::Node>>,
    hints: &Hints,
    opts: &ConversionOptions,
    tables: &crate::table::Tables<'a>,
) {
    let mut stack: Vec<Event> = Vec::with_capacity(16);
    for child in roots.rev() {
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
                            // ...and not the anchor it carried either (2.4.1):
                            // `preserve_ids` asks for one for every element with
                            // an `id`, and a wrapper Semantic/Minimal unwrap is
                            // still such an element. Placed after the separator,
                            // as a rendered wrapper's is after its `begin_block`.
                            renderer.emit_id_anchor(elem, opts.preserve_ids);
                            for child in node.children().rev() {
                                stack.push(Event::Enter(child));
                            }
                            continue;
                        }
                        Disposition::Render => {}
                    }

                    // RFC 008: an expressible table (never inside `<pre>` --
                    // there its `tr`/`td`/`th` fall through below, to the
                    // ordinary `Block::Paragraph` dispatch that `enter_block`'s
                    // `in_pre` guard already empties of markup) is rendered as
                    // GFM here, in one call, rather than through the per-tag
                    // dispatch every other element uses -- a header row, its
                    // delimiter row and every data row must be written
                    // together, not discovered one `<tr>` at a time. No Leave
                    // is pushed and its children are not queued: `table::render`
                    // does the whole subtree itself.
                    if tag == "table"
                        && !renderer.in_pre()
                        && let Some(info) = tables.get(&node.id())
                        && info.grid.is_some()
                    {
                        crate::table::render(renderer, info, hints, opts, tables);
                        continue;
                    }

                    renderer.enter_element(
                        elem,
                        opts.preserve_ids,
                        hints.wrappers.contains(&node.id()),
                        hints.loose_lists.contains(&node.id()),
                        hints.needs_disambiguation.contains(&node.id()),
                        (tag == "li").then(|| task_checkbox(node)).flatten(),
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
    let tables = crate::table::analyze(document);

    // root() は Document ノードなので子ノードだけを渡す
    drive(
        &mut renderer,
        document.tree.root().children(),
        &hints,
        opts,
        &tables,
    );

    renderer.finish()
}
