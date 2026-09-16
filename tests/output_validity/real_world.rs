//! Minimal reproductions from field reports. Only the HTML is taken; the
//! reports themselves are not copied into the repository.

use crate::harness::tree;

cells! {
    // bekoedit, 2026-09-16, item 2.
    bekoedit_digit_period_escape: "<p>1. not a list</p>"
        => tree(r#"para("1. not a list")"#),
        defect(Rfc010Planned, "the backslash goes before the digit: literal `\\1.` (bekoedit item 2, audit A-09)");
    // bekoedit, 2026-09-16, item 3: Google Docs wraps its payload in a
    // non-bold <b>. RFC 028 behaviour A.
    bekoedit_google_docs_bold_wrapper: r#"<b style="font-weight:normal;"><p>para one</p><p>para two</p></b>"#
        => tree(r#"para("para one"), para("para two")"#),
        defect(Rfc028, "`**` emitted around the paragraphs as stray `**` lines (bekoedit item 3)");
    // 2.2.2 consumer pass (RFC 024 addendum): a space lost inside a link only.
    space_around_inline_in_link: r#"<a href="/x">Read <strong>more</strong> now</a>"#
        => tree(r#"para(link[/x]("Read ", strong("more"), " now"))"#),
        defect(Rfc024, "`**` escapes the link and the space before `more` is lost: `****[Readmore now](/x)`");
    // Its control case, outside a link.
    space_around_inline_in_paragraph: "<p>Read <strong>more</strong> now</p>"
        => tree(r#"para("Read ", strong("more"), " now")"#);
}
