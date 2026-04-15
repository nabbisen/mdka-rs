use std::fmt::Write;

use crate::utils;

#[derive(Debug, Clone)]
pub enum ListKind {
    Unordered,
    Ordered { counter: usize },
}

#[derive(Debug, Clone)]
pub struct ListContext {
    pub kind: ListKind,
}

#[derive(Debug)]
enum InlineCapture {
    Link {
        href: String,
        title: Option<String>,
        buf: String,
    },
    None,
}

pub struct MarkdownRenderer {
    pub output: String,
    pub list_stack: Vec<ListContext>,
    pub blockquote_depth: usize,
    pub newlines_emitted: usize,
    pub in_pre: bool,
    pub pre_lang: Option<String>,
    last_was_space: bool,
    at_line_start: bool,
    inline_capture: InlineCapture,
    capture_depth: usize,
    link_depth: usize,
}

impl MarkdownRenderer {
    pub fn new(capacity: usize) -> Self {
        Self {
            output: String::with_capacity(capacity),
            list_stack: Vec::with_capacity(8),
            blockquote_depth: 0,
            newlines_emitted: 0,
            in_pre: false,
            pre_lang: None,
            last_was_space: false,
            at_line_start: true,
            inline_capture: InlineCapture::None,
            capture_depth: 0,
            link_depth: 0,
        }
    }

    // ─── 改行制御 ──────────────────────────────────────────────────────────

    pub fn ensure_newlines(&mut self, count: usize) {
        // 出力が空のときは先頭に改行を入れない
        // （scraper が補完する <html><body> の begin_block 対策）
        if self.output.is_empty() {
            self.at_line_start = true;
            return;
        }
        while self.newlines_emitted < count {
            self.output.push('\n');
            self.newlines_emitted += 1;
        }
        self.last_was_space = false;
        self.at_line_start = true;
    }

    /// 遅延プレフィックス: コンテンツ書き込み直前に呼ぶ。
    /// 行頭かつ blockquote 内なら "> "×depth を出力してフラグをリセット。
    fn emit_pending_prefix(&mut self) {
        if self.at_line_start && self.blockquote_depth > 0 {
            for _ in 0..self.blockquote_depth {
                self.output.push_str("> ");
            }
            self.at_line_start = false;
        }
    }

    fn begin_block(&mut self) {
        // プレフィックスはここでは出力しない。
        // コンテンツ書き込み時に emit_pending_prefix() が担う。
        self.ensure_newlines(2);
    }

    fn end_block(&mut self) {
        self.ensure_newlines(2);
    }

    // ─── 生文字列プッシュ ──────────────────────────────────────────────────

    pub fn push_raw(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.output.push_str(s);
        let trailing = s
            .as_bytes()
            .iter()
            .rev()
            .take_while(|&&b| b == b'\n')
            .count();
        if trailing > 0 {
            self.newlines_emitted = trailing;
            self.at_line_start = true;
            self.last_was_space = false;
        } else {
            self.newlines_emitted = 0;
            self.at_line_start = false;
        }
    }

    // ─── テキスト処理 ──────────────────────────────────────────────────────

    pub fn process_text(&mut self, text: &str) {
        if self.in_pre {
            self.push_raw(text);
            return;
        }
        if self.capture_depth > 0 {
            if let InlineCapture::Link { buf, .. } = &mut self.inline_capture {
                let mut _ls = false;
                let mut _al = false;
                utils::write_normalised(text, buf, &mut _ls, false, &mut _al);
            }
            return;
        }
        // 実際のテキストを書く前にプレフィックスを確定させる
        if !text.trim().is_empty() {
            self.emit_pending_prefix();
        }
        let at_block = self.at_line_start;
        utils::write_normalised(
            text,
            &mut self.output,
            &mut self.last_was_space,
            at_block,
            &mut self.at_line_start,
        );
        if !text.trim().is_empty() {
            self.newlines_emitted = 0;
        }
    }

    // ─── 要素 Enter ────────────────────────────────────────────────────────

    pub fn enter_element(&mut self, elem: &scraper::node::Element) {
        let tag = elem.name();
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.begin_block();
                self.emit_pending_prefix();
                let level = (tag.as_bytes()[1] - b'0') as usize;
                for _ in 0..level {
                    self.output.push('#');
                }
                self.output.push(' ');
                self.newlines_emitted = 0;
                self.at_line_start = false;
                self.last_was_space = false;
            }
            "p" | "div" | "article" | "section" | "main" | "header" | "footer" | "nav"
            | "aside" | "figure" | "figcaption" => {
                self.begin_block();
            }
            "ul" => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                self.list_stack.push(ListContext {
                    kind: ListKind::Unordered,
                });
            }
            "ol" => {
                if self.list_stack.is_empty() {
                    self.begin_block();
                }
                let start = elem
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.list_stack.push(ListContext {
                    kind: ListKind::Ordered { counter: start },
                });
            }
            "li" => {
                self.ensure_newlines(1);
                self.emit_pending_prefix();
                let depth = self.list_stack.len().saturating_sub(1);
                for _ in 0..depth {
                    self.output.push_str("  ");
                }
                if let Some(ctx) = self.list_stack.last_mut() {
                    match &mut ctx.kind {
                        ListKind::Unordered => self.output.push_str("- "),
                        ListKind::Ordered { counter } => {
                            let n = *counter;
                            *counter += 1;
                            push_usize(&mut self.output, n);
                            self.output.push_str(". ");
                        }
                    }
                }
                self.newlines_emitted = 0;
                self.at_line_start = false;
                self.last_was_space = false;
            }
            "blockquote" => {
                self.begin_block();
                self.blockquote_depth += 1;
                // プレフィックスは次のコンテンツ書き込み時に emit_pending_prefix() が出す
            }
            "pre" => {
                self.begin_block();
                self.emit_pending_prefix();
                self.in_pre = true;
                self.pre_lang = None;
            }
            "code" if self.in_pre => {
                let lang = elem
                    .attr("class")
                    .and_then(|cls| utils::extract_code_lang(Some(cls)))
                    .unwrap_or("");
                self.output.push_str("```");
                self.output.push_str(lang);
                self.output.push('\n');
                self.newlines_emitted = 0;
                self.at_line_start = true;
                self.pre_lang = if lang.is_empty() {
                    None
                } else {
                    Some(lang.to_string())
                };
            }
            "code" => {
                self.flush_space();
                self.output.push('`');
                self.newlines_emitted = 0;
                self.at_line_start = false;
            }
            "strong" | "b" => {
                self.flush_space();
                self.output.push_str("**");
                self.newlines_emitted = 0;
                self.at_line_start = false;
            }
            "em" | "i" => {
                self.flush_space();
                self.output.push('*');
                self.newlines_emitted = 0;
                self.at_line_start = false;
            }
            "a" => {
                self.link_depth += 1;
                if self.link_depth == 1 {
                    let href = elem.attr("href").unwrap_or("").to_string();
                    let title = elem.attr("title").map(|t| t.to_string());
                    self.flush_space();
                    self.inline_capture = InlineCapture::Link {
                        href,
                        title,
                        buf: String::new(),
                    };
                    self.capture_depth += 1;
                }
            }
            "img" => {
                let src = elem.attr("src").unwrap_or("");
                let alt = elem.attr("alt").unwrap_or("");
                let title = elem.attr("title");
                self.flush_space();
                self.output.push_str("![");
                self.output.push_str(alt);
                self.output.push_str("](");
                self.output.push_str(src);
                if let Some(t) = title {
                    self.output.push_str(" \"");
                    self.output.push_str(t);
                    self.output.push('"');
                }
                self.output.push(')');
                self.newlines_emitted = 0;
                self.at_line_start = false;
                self.last_was_space = false;
            }
            "hr" => {
                self.begin_block();
                self.emit_pending_prefix();
                self.output.push_str("---");
                self.end_block();
            }
            "br" => {
                self.output.push_str("  \n");
                self.newlines_emitted = 1;
                self.at_line_start = true;
                self.last_was_space = false;
                // br 後の行頭プレフィックスは次のコンテンツ書き込み時に出る
            }
            _ => {}
        }
    }

    // ─── 要素 Leave ────────────────────────────────────────────────────────

    pub fn leave_element(&mut self, elem: &scraper::node::Element) {
        let tag = elem.name();
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => self.end_block(),
            "p" | "div" | "article" | "section" | "main" | "header" | "footer" | "nav"
            | "aside" | "figure" | "figcaption" => self.end_block(),
            "ul" | "ol" => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.end_block();
                }
            }
            "li" => self.ensure_newlines(1),
            "blockquote" => {
                self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
                self.end_block();
            }
            "pre" => {
                if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
                self.output.push_str("```");
                self.in_pre = false;
                self.pre_lang = None;
                self.end_block();
            }
            "code" if !self.in_pre => {
                self.output.push('`');
                self.newlines_emitted = 0;
            }
            "strong" | "b" => {
                self.output.push_str("**");
                self.newlines_emitted = 0;
            }
            "em" | "i" => {
                self.output.push('*');
                self.newlines_emitted = 0;
            }
            "a" => {
                if self.link_depth == 1 {
                    self.capture_depth = self.capture_depth.saturating_sub(1);
                    let captured = std::mem::replace(&mut self.inline_capture, InlineCapture::None);
                    if let InlineCapture::Link { href, title, buf } = captured {
                        self.output.push('[');
                        self.output.push_str(&buf);
                        self.output.push_str("](");
                        self.output.push_str(&href);
                        if let Some(t) = &title {
                            self.output.push_str(" \"");
                            self.output.push_str(t);
                            self.output.push('"');
                        }
                        self.output.push(')');
                        self.newlines_emitted = 0;
                        self.at_line_start = false;
                        self.last_was_space = false;
                    }
                }
                self.link_depth = self.link_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn flush_space(&mut self) {
        if self.last_was_space && !self.at_line_start {
            self.output.push(' ');
            self.last_was_space = false;
        }
    }

    pub fn finish(mut self) -> String {
        // 末尾の空白・改行を除去
        let end = self.output.trim_end().len();
        self.output.truncate(end);
        if !self.output.is_empty() {
            self.output.push('\n');
        }
        self.output
    }
}

/// usize を String へ直接書き込む（`format!` によるアロケーション回避）。
#[inline]
fn push_usize(s: &mut String, n: usize) {
    let _ = write!(s, "{n}");
}
