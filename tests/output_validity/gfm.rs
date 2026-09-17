//! Text that must stay text when the Markdown is read as GitHub Flavored
//! Markdown (RFC 025 review Q8d). Every cell in the harness already runs under
//! both readings; these cells aim at the syntax only GFM has.

use crate::harness::tree;

cells! {
    strikethrough_like_text: "<p>~~not struck~~</p>"
        => tree(r#"para("~~not struck~~")"#),
        defect(Rfc010, "GFM reading: `~~` is not escaped, so the text becomes strikethrough and the tildes are lost (CommonMark reading correct)");
    single_tilde_strikethrough_like_text: "<p>~not struck~</p>"
        => tree(r#"para("~not struck~")"#),
        defect(Rfc010, "GFM reading: `~` is not escaped, so the text becomes strikethrough and the tildes are lost (CommonMark reading correct)");
    table_like_paragraphs: "<p>a | b | c</p><p>--- | --- | ---</p>"
        => tree(r#"para("a | b | c"), para("--- | --- | ---")"#);
    table_like_lines: "<p>a | b<br>--- | ---</p>"
        => tree(r#"para("a | b", br, "--- | ---")"#);
    table_like_lines_with_pipes: "<p>| a | b |<br>| --- | --- |</p>"
        => tree(r#"para("| a | b |", br, "| --- | --- |")"#),
        defect(Rfc010, "GFM reading: two text lines with `|` become a table; the delimiter row is consumed and the pipes lost (CommonMark reading correct)");
    table_like_lines_with_alignment: "<p>a | b<br>:-- | --:</p>"
        => tree(r#"para("a | b", br, ":-- | --:")"#),
        defect(Rfc010, "GFM reading: a `:--` line is not escaped, so the lines become a table and the delimiter row is lost (CommonMark reading correct)");
    task_like_list_item: "<ul><li>[ ] not a task</li></ul>"
        => tree(r#"ul(li("[ ] not a task"))"#);
    checked_task_like_list_item: "<ul><li>[x] not done</li></ul>"
        => tree(r#"ul(li("[x] not done"))"#);
    footnote_like_text: "<p>see [^1]</p><p>[^1]: not a footnote</p>"
        => tree(r#"para("see [^1]"), para("[^1]: not a footnote")"#);
    alert_like_text: "<blockquote><p>[!NOTE] not an alert</p></blockquote>"
        => tree(r#"quote(para("[!NOTE] not an alert"))"#);
}
