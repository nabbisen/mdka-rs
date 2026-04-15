//! テキスト正規化・属性抽出・タグ分類ヘルパー
//!
//! すべての処理は正規表現を使わず、`char` イテレータによる
//! シングルパスのステートマシンとして実装されている。

#[cfg(test)]
mod tests;

// ─── タグ分類（traversal / preprocessor 共通） ────────────────────────────

/// コンテンツごと常にスキップするタグ。
#[inline]
pub(crate) fn is_skip_tag(tag: &str) -> bool {
    matches!(
        tag,
        "script"
            | "style"
            | "meta"
            | "link"
            | "template"
            | "iframe"
            | "object"
            | "embed"
            | "noscript"
            | "head"
            | "svg"
    )
}

/// シェル要素（minimal モードで除外対象）。
#[inline]
pub(crate) fn is_shell_tag(tag: &str) -> bool {
    matches!(tag, "nav" | "header" | "footer" | "aside")
}

/// アンラップ候補のラッパータグ。
#[inline]
pub(crate) fn is_wrapper_tag(tag: &str) -> bool {
    matches!(tag, "span" | "div" | "section" | "article" | "main")
}

/// 構造タグ（アンラップ対象外）。
#[inline]
pub(crate) fn is_structural_tag(tag: &str) -> bool {
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

// moved to /tests/utils/preprocessor.rs
// /// Void 要素（自己閉じ・子なし）。
// #[inline]
// pub(crate) fn is_void_element(tag: &str) -> bool {
//     matches!(
//         tag,
//         "area"
//             | "base"
//             | "br"
//             | "col"
//             | "embed"
//             | "hr"
//             | "img"
//             | "input"
//             | "link"
//             | "meta"
//             | "param"
//             | "source"
//             | "track"
//             | "wbr"
//     )
// }

// ─── テキスト正規化 ───────────────────────────────────────────────────────

/// シングルパス テキスト正規化 + Markdownエスケープ。
///
/// * HTML Whitespace Collapsing（連続する空白を1スペースに集約）
/// * Markdown 予約文字のコンテキスト依存エスケープ
/// * 直接 `out` へ書き込み、中間アロケーションをゼロに抑える
///
/// `at_block_start`: 呼び出し時点でブロック先頭（先頭空白を無視する）か否か。
/// ただし最初の実文字を出力した時点でこのフラグはリセットされる。
pub fn write_normalised(
    text: &str,
    out: &mut String,
    last_was_space: &mut bool,
    at_block_start: bool,
    at_line_start: &mut bool,
) {
    // ローカルで管理: 最初の実文字を出力したらブロック先頭フラグを解除
    let mut block_start = at_block_start;

    for c in text.chars() {
        // 空白類（NBSP 含む）はコラプス対象
        if c.is_ascii_whitespace() || c == '\u{00a0}' {
            // ブロック先頭の空白は捨てる（先頭の実文字より前）
            if !*last_was_space && !block_start {
                *last_was_space = true;
            }
            continue;
        }

        // 実文字の直前に保留スペースをフラッシュ
        if *last_was_space && !block_start {
            out.push(' ');
        }
        *last_was_space = false;

        // 実文字を1つでも出力したらブロック先頭フラグを解除
        block_start = false;

        let line_start = *at_line_start;
        *at_line_start = false;

        match c {
            // 常にエスケープ
            '\\' | '*' | '_' | '`' | '[' | ']' | '!' => {
                out.push('\\');
                out.push(c);
            }
            // 行頭のみエスケープ
            '#' | '>' | '+' | '-' if line_start => {
                out.push('\\');
                out.push(c);
            }
            // 行頭の数字（順序リストとの混同防止）
            '0'..='9' if line_start => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
}

/// `<code class="language-xxx">` からコード言語を抽出する。
pub fn extract_code_lang(class: Option<&str>) -> Option<&str> {
    class?
        .split_whitespace()
        .find(|cls| cls.starts_with("language-"))
        .map(|cls| &cls["language-".len()..])
}
