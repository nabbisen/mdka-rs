// tests/common.rs is compiled fresh into each integration-test binary that
// declares `mod common;`. Not every binary uses both helpers below, so each
// is individually dead code in whichever binaries only call the other one.
#![allow(dead_code)]

use mdka::options::ConversionOptions;

/// HTML を既定モードで Markdown に変換するテスト用ヘルパー。
pub fn conv(html: &str) -> String {
    mdka::html_to_markdown(html)
}

/// HTML を指定したオプションで Markdown に変換するテスト用ヘルパー。
pub fn conv_with(html: &str, opts: &ConversionOptions) -> String {
    mdka::html_to_markdown_with(html, opts)
}
