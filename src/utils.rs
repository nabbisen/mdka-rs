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
        // RFC 008: a table row or cell, on its own, is a paragraph-like
        // block -- the floor that keeps a table's cells from welding
        // together (`H1H2ab`) wherever the table isn't rendered specially
        // (an inexpressible table's fallback, or any table content inside
        // `<pre>`, where `enter_block`'s own `in_pre` guard already
        // suppresses every block's markup, this one included). An
        // expressible table intercepts `<table>` itself before its children
        // ever reach this dispatch, so this arm never fires for one.
        // `<caption>` is content to keep, not lost silently (criterion 7);
        // as a paragraph it lands in the flow at its own document position.
        "tr" | "td" | "th" | "caption" => Block::Paragraph,
        // RFC 009 §4.1: the last welding element in the codebase, fixed the
        // same way -- `<dt>`/`<dd>` each become their own paragraph-like
        // block, so a `<dl>` inherits container prefixes, `<pre>`
        // suppression and RFC 008 F1's cell-flattening for free, the same
        // as `tr`/`td`/`th` did. `<dl>` itself is not classified: it is a
        // transparent container, not content of its own -- its `<dt>`/`<dd>`
        // children already provide every separator needed.
        "dt" | "dd" => Block::Paragraph,
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
/// (RFC 028): emphasis, code spans and links. `<del>`/`<s>` (RFC 009 §4.2)
/// join this set for the same reason -- GFM strikethrough is inline syntax,
/// and `~~` around a block would be no more valid than `**` around one.
#[inline]
pub(crate) fn is_inline_wrapper(tag: &str) -> bool {
    matches!(
        tag,
        "strong" | "b" | "em" | "i" | "code" | "a" | "del" | "s"
    )
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
/// emphasis is never added.
pub(crate) fn emphasis_negated_by_style(tag: &str, style: Option<&str>) -> bool {
    let property = match tag {
        "b" | "strong" => "font-weight",
        "i" | "em" => "font-style",
        _ => return false,
    };
    let Some(style) = style else {
        return false;
    };
    let Some(value) = style_property(style, property) else {
        return false;
    };
    if property == "font-style" {
        return value == "normal";
    }
    value == "normal" || value.parse::<f64>().is_ok_and(|weight| weight <= 500.0)
}

/// Reads one property out of an inline `style` attribute: declarations split
/// on `;`, the name before `:` and the value trimmed and case-folded,
/// `!important` stripped, the last declaration of the property wins. Shared
/// by [`emphasis_negated_by_style`] and the table alignment reader (RFC 008
/// §3, `align=`/`text-align:`) -- the only two places mdka reads `style`.
pub(crate) fn style_property(style: &str, property: &str) -> Option<String> {
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
    value
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

/// A `<sup>`'s character mapped to Unicode superscript, if it has one (RFC
/// 009 §4.3): digits, `+ - = ( )`, and `n`/`i` -- the set with a superscript
/// form in Unicode at all.
#[inline]
fn superscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        _ => return None,
    })
}

/// A `<sub>`'s character mapped to Unicode subscript, if it has one (RFC 009
/// §4.3): digits, `+ - = ( )`, and the Latin letters Unicode gives a
/// subscript form.
#[inline]
fn subscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'o' => 'ₒ',
        'x' => 'ₓ',
        'h' => 'ₕ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'p' => 'ₚ',
        's' => 'ₛ',
        't' => 'ₜ',
        _ => return None,
    })
}

/// `<sup>`/`<sub>` content mapped to Unicode, only if **every** character
/// maps (RFC 009 §4.3): `2<sup>7</sup>` is a different number if only some
/// of it becomes superscript, so a partial map is not an improvement, it is
/// a new defect. Empty content maps to `Some(String::new())` -- vacuously,
/// every character (there are none) maps -- which the caller treats as
/// "nothing to write", the same as an empty `<strong>` (RFC 037).
pub(crate) fn map_script(content: &str, superscript: bool) -> Option<String> {
    let mut out = String::with_capacity(content.len());
    for c in content.chars() {
        let mapped = if superscript {
            superscript_char(c)
        } else {
            subscript_char(c)
        };
        out.push(mapped?);
    }
    Some(out)
}
