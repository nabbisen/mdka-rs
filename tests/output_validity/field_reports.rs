//! Minimal reproductions from field reports and public issue reports. These
//! are reproductions, not captures: real-world input belongs in `corpus/`.
//! Only the HTML is taken; no report is copied into the repository.

use crate::harness::tree;

cells! {
    // bekoedit, letter of 2026-09-16, item 2. Hand-written by bekoedit and
    // vendored with their permission.
    bekoedit_digit_period_escape: "<p>1. not a list</p>"
        => tree(r#"para("1. not a list")"#),
        defect(Rfc010Planned, "the backslash goes before the digit: literal `\\1.` (bekoedit item 2, audit A-09)");
    // bekoedit, letter of 2026-09-16, item 3. Hand-written by bekoedit and
    // vendored with their permission. The multi-paragraph shape of the Google
    // Docs clipboard wrapper; RFC 028 behaviour A.
    bekoedit_google_docs_bold_wrapper: r#"<b style="font-weight:normal;"><p>para one</p><p>para two</p></b>"#
        => tree(r#"para("para one"), para("para two")"#),
        defect(Rfc028, "`**` emitted around the paragraphs as stray `**` lines (bekoedit item 3)");
    // Google Docs wraps copied content in a non-bold
    // `<b style="font-weight:normal" id="docs-internal-guid-…">`, per public
    // issue reports quoting clipboard captures: ProseMirror #459 (2016),
    // MarkText #4688 (2026). The single-paragraph shape: the wrapper around
    // inline content. Written for this harness in that form; no third-party
    // HTML is copied. UNOWNED pending the owner's decision on RFC 028's
    // proposed style amendment -- relabelled by that decision, not here.
    google_docs_bold_wrapper_inline: r#"<b style="font-weight:normal;" id="docs-internal-guid-x"><span style="font-weight:400">Hello world</span></b>"#
        => tree(r#"para("Hello world")"#),
        defect(Unowned, "the non-bold wrapper is emitted as `**`: the whole paragraph becomes bold; pending the owner's decision on RFC 028's style amendment");
    // 2.2.2 consumer pass (RFC 024 addendum): a space lost inside a link only.
    space_around_inline_in_link: r#"<a href="/x">Read <strong>more</strong> now</a>"#
        => tree(r#"para(link[/x]("Read ", strong("more"), " now"))"#),
        defect(Rfc024, "`**` escapes the link and the space before `more` is lost: `****[Readmore now](/x)`");
    // Its control case, outside a link.
    space_around_inline_in_paragraph: "<p>Read <strong>more</strong> now</p>"
        => tree(r#"para("Read ", strong("more"), " now")"#);
}
