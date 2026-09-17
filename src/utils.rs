//! テキスト正規化・属性抽出・タグ分類ヘルパー
//!
//! すべての処理は正規表現を使わず、`char` イテレータによる
//! シングルパスのステートマシンとして実装されている。

#[cfg(test)]
mod tests;

// ─── タグ分類（traversal 用） ──────────────────────────────────────────────

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

/// A block the renderer emits for an element (RFC 028 criterion 11).
///
/// This is the single classification of block elements: the renderer's block
/// arms match on it, and the traversal's "wraps block content" pre-pass asks
/// it, so the two cannot disagree about what a block is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Block {
    /// `h1`–`h6`, with the level.
    Heading(usize),
    /// Elements rendered as a paragraph-like block.
    Paragraph,
    UnorderedList,
    OrderedList,
    ListItem,
    Quote,
    Pre,
    Rule,
}

/// The block the renderer emits for `tag`, if any.
#[inline]
pub(crate) fn block_kind(tag: &str) -> Option<Block> {
    Some(match tag {
        "h1" => Block::Heading(1),
        "h2" => Block::Heading(2),
        "h3" => Block::Heading(3),
        "h4" => Block::Heading(4),
        "h5" => Block::Heading(5),
        "h6" => Block::Heading(6),
        "p" | "div" | "article" | "section" | "main" | "header" | "footer" | "nav" | "aside"
        | "figure" | "figcaption" => Block::Paragraph,
        "ul" => Block::UnorderedList,
        "ol" => Block::OrderedList,
        "li" => Block::ListItem,
        "blockquote" => Block::Quote,
        "pre" => Block::Pre,
        "hr" => Block::Rule,
        _ => return None,
    })
}

/// Inline elements whose rendering changes when they wrap block content
/// (RFC 028): emphasis, code spans and links.
#[inline]
pub(crate) fn is_inline_wrapper(tag: &str) -> bool {
    matches!(tag, "strong" | "b" | "em" | "i" | "code" | "a")
}

/// Whether an emphasis element's **own** inline `style` negates its emphasis
/// (RFC 028 Amendment 1).
///
/// - `<b>`/`<strong>`: `font-weight` is `normal` or a number ≤ 500. Relative
///   values (`lighter`, `bolder`) and anything else do not negate.
/// - `<i>`/`<em>`: `font-style` is `normal`.
///
/// Declarations are split on `;`; the name before `:` and the value are
/// trimmed and case-folded, `!important` is stripped, and the last declaration
/// of the property wins. No other property is read, nothing is inherited, and
/// emphasis is never added. This is the only place mdka reads `style`.
pub(crate) fn emphasis_negated_by_style(tag: &str, style: Option<&str>) -> bool {
    let property = match tag {
        "b" | "strong" => "font-weight",
        "i" | "em" => "font-style",
        _ => return false,
    };
    let Some(style) = style else {
        return false;
    };
    let mut value: Option<String> = None;
    for declaration in style.split(';') {
        let Some((name, raw)) = declaration.split_once(':') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case(property) {
            continue;
        }
        let mut v = raw.trim().to_ascii_lowercase();
        if let Some(stripped) = v.strip_suffix("!important") {
            v = stripped.trim_end().to_string();
        }
        value = Some(v);
    }
    let Some(value) = value else {
        return false;
    };
    if property == "font-style" {
        return value == "normal";
    }
    value == "normal" || value.parse::<f64>().is_ok_and(|weight| weight <= 500.0)
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

/// `<code class="language-xxx">` からコード言語を抽出する。
pub fn extract_code_lang(class: Option<&str>) -> Option<&str> {
    class?
        .split_whitespace()
        .find(|cls| cls.starts_with("language-"))
        .map(|cls| &cls["language-".len()..])
}
