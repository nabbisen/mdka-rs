/// HTML を既定モードで Markdown に変換するテスト用ヘルパー。
pub fn conv(html: &str) -> String {
    mdka::html_to_markdown(html)
}
