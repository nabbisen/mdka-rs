use super::*;
use crate::options::ConversionOptions;

fn conv(html: &str) -> String {
    let doc = Html::parse_document(html);
    traverse(&doc, &ConversionOptions::default())
}

#[test]
fn test_heading() {
    assert_eq!(conv("<h1>Hello</h1>").trim(), "# Hello");
}

#[test]
fn test_paragraph() {
    assert_eq!(conv("<p>Hello world</p>").trim(), "Hello world");
}

#[test]
fn test_deep_nest_no_overflow() {
    // 10,000段のネストでスタックオーバーフローしないことを確認
    let open: String = "<div>".repeat(10_000);
    let close: String = "</div>".repeat(10_000);
    let html = format!("{}<p>deep</p>{}", open, close);
    let doc = Html::parse_document(&html);
    let md = traverse(&doc, &ConversionOptions::default());
    assert!(
        md.contains("deep"),
        "output: {:?}",
        &md[..md.len().min(200)]
    );
}
