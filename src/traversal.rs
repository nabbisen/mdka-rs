use crate::renderer::MarkdownRenderer;
use scraper::Html;

/// スキップすべきタグ（コンテンツごと無視）
const SKIP_TAGS: &[&str] = &["script", "style", "head", "noscript", "template", "svg"];

/// トラバーサルイベント：Enter（開きタグ相当）と Leave（閉じタグ相当）
enum Event<'a> {
    Enter(ego_tree::NodeRef<'a, scraper::Node>),
    Leave(ego_tree::NodeRef<'a, scraper::Node>),
}

/// HTMLドキュメントをトラバースしてMarkdown文字列を生成する。
///
/// 再帰を使わず `Vec` ベースのスタックで深さ優先探索を行うため、
/// 10,000段以上のネストでもスタックオーバーフローが発生しない。
pub fn traverse(document: &Html) -> String {
    // 元のHTMLサイズの半分を初期容量として確保
    let capacity = document.html().len() / 2;
    let mut renderer = MarkdownRenderer::new(capacity.max(256));

    // スタックに積む。root() は Document ノード（Element でも Text でもない）
    // なので直接 Enter/Leave は発行せず、子ノードだけを逆順で積む。
    let mut stack: Vec<Event> = Vec::with_capacity(64);
    for child in document.tree.root().children().rev() {
        stack.push(Event::Enter(child));
    }

    while let Some(event) = stack.pop() {
        match event {
            Event::Enter(node) => {
                match node.value() {
                    scraper::Node::Element(elem) => {
                        let tag = elem.name().to_lowercase();

                        // スキップタグはコンテンツごと無視
                        if SKIP_TAGS.contains(&tag.as_str()) {
                            continue;
                        }

                        renderer.enter_element(elem);

                        // Leave イベントを先にスタックへ（子より後に処理される）
                        stack.push(Event::Leave(node));

                        // 子を逆順でスタックへ（先頭の子が最後にポップされるため正順になる）
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
                }
            }
            Event::Leave(node) => {
                if let scraper::Node::Element(elem) = node.value() {
                    let tag = elem.name().to_lowercase();
                    if !SKIP_TAGS.contains(&tag.as_str()) {
                        renderer.leave_element(elem);
                    }
                }
            }
        }
    }

    renderer.finish()
}

#[cfg(test)]
mod tests;
