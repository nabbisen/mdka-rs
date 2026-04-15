//! DOM トラバーサル + Markdown 生成
//!
//! 1回のパース済み DOM を直接トラバースして Markdown を生成する。
//! 前処理（タグ除外・アンラップ）は Enter 時にインラインで判定するため、
//! HTML 文字列の再構築と再パースが不要になっている。
//!
//! 非再帰 DFS（`Vec` スタック）を使用するためスタックオーバーフローが発生しない。

#[cfg(test)]
mod tests;

use crate::options::ConversionOptions;
use crate::renderer::MarkdownRenderer;
use crate::utils;
use scraper::Html;

/// トラバーサルイベント：Enter（開きタグ相当）と Leave（閉じタグ相当）
enum Event<'a> {
    Enter(ego_tree::NodeRef<'a, scraper::Node>),
    Leave(ego_tree::NodeRef<'a, scraper::Node>),
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

                    // ── 前処理: スキップ判定 ──────────────────────────
                    // 常にスキップ（コンテンツごと無視）
                    if utils::is_skip_tag(tag) {
                        continue;
                    }
                    // シェル要素の除外（オプション）
                    if opts.drop_interactive_shell && utils::is_shell_tag(tag) {
                        continue;
                    }

                    // ── 前処理: ラッパーアンラップ ─────────────────────
                    // タグ自体は出力せず、子だけをトラバースする
                    if opts.unwrap_unknown_wrappers
                        && utils::is_wrapper_tag(tag)
                        && !utils::is_structural_tag(tag)
                    {
                        for child in node.children().rev() {
                            stack.push(Event::Enter(child));
                        }
                        continue;
                    }

                    // ── 通常処理: 要素を出力 ──────────────────────────
                    renderer.enter_element(elem);

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
