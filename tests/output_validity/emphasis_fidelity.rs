//! RFC 037 — emphasis emission fidelity: empty and nested.
//!
//! An emphasis element writes its delimiters before it is known whether
//! there will be anything inside them; if there is not, `**`/`**` becomes
//! the literal text `****` -- or, when that is the paragraph's *entire*
//! content, four asterisks on their own line, which CommonMark reads as a
//! thematic break (the same family RFC 038 named, in emphasis clothing).
//! And two single-delimiter emphases of the same class immediately nested
//! concatenate into a double: `*` + `*x*` + `*` is `**x**`, which is
//! `<strong>`, not two nested italics.
//!
//! The fix mirrors the settled rule for a link with no text and no image
//! (RFC 024 criterion 7): an emphasis with no rendered content emits no
//! delimiters at all, whitespace-only content included. Nested emphasis of
//! the same class collapses to one level. Mixed classes keep both, and the
//! one order that used to invert (`<strong><em>`) is corrected by swapping
//! the inner element's delimiter to `_` -- the other order (`<em><strong>`)
//! already reparsed correctly and is a control here, not a fix.

use crate::harness::tree;

// ── empty emphasis: no delimiters for nothing ──────────────────────────────

cells! {
    empty_bold_between_text: "<p>a<b></b>b</p>"
        => tree(r#"para("ab")"#);
    empty_italic_between_text: "<p>a<em></em>b</p>"
        => tree(r#"para("ab")"#);
    empty_bold_after_a_word_space: "<p>Hello <b></b>world</p>"
        => tree(r#"para("Hello world")"#);
    // RFC 037 finding C: the space collapses to exactly one, not zero and
    // not two -- it must not survive twice (once flushed before the empty
    // span opened, once still pending after it closed) nor disappear.
    whitespace_only_bold_keeps_one_space: "<p>a<b> </b>b</p>"
        => tree(r#"para("a b")"#);
    whitespace_on_both_sides_of_empty_bold: "<p>a <strong></strong> b</p>"
        => tree(r#"para("a b")"#);
    // A paragraph whose only content is empty emphasis emits nothing at
    // all -- not an `hr`, not an empty paragraph. This is RFC 038's family
    // in emphasis clothing: a line of only delimiters collides with a
    // thematic break.
    only_empty_bold_in_a_paragraph: "<p><b></b></p>"
        => tree(r#""#);
    only_empty_nested_italic_in_a_paragraph: "<p><em><em></em></em></p>"
        => tree(r#""#);
    empty_bold_in_a_list_item: "<ul><li><b></b>x</li></ul>"
        => tree(r#"ul(li("x"))"#);
}

// ── nested emphasis: same class collapses, mixed classes keep both ────────

cells! {
    nested_em_collapses: "<p><em><em>x</em></em></p>"
        => tree(r#"para(em("x"))"#);
    nested_i_collapses: "<p><i><i>x</i></i></p>"
        => tree(r#"para(em("x"))"#);
    // Mixed tags, same delimiter class: the rule is about the delimiter,
    // not the tag name (RFC 037 §1.1 B).
    mixed_italic_tags_collapse: "<p><em><i>x</i></em></p>"
        => tree(r#"para(em("x"))"#);
    nested_bold_collapses: "<p><b><b>x</b></b></p>"
        => tree(r#"para(strong("x"))"#);
    // Control: already correct at the baseline, must stay byte-identical --
    // `*` then `**` concatenate into `***`, and this is the one order
    // CommonMark's own resolution of that run happens to read back
    // correctly as written.
    em_around_strong_keeps_order: "<p><em><strong>x</strong></em></p>"
        => tree(r#"para(em(strong("x")))"#);
    // The one that was broken: the opposite order produces the identical
    // `***x***` bytes, which CommonMark reads the same way regardless --
    // matching the *other* order instead. Fixed by swapping the inner
    // `<em>`'s delimiter to `_`.
    strong_around_em_keeps_order: "<p><strong><em>x</em></strong></p>"
        => tree(r#"para(strong(em("x")))"#);
    // Three levels, not a named requirement (RFC 037 §1.1 called it out of
    // scope) but not garbage either: the general mechanism collapses the
    // repeated outer/inner bold to one level and still gets the mixed
    // class's order right.
    three_level_mixed_collapses_reasonably: "<p><b><i><b>x</b></i></b></p>"
        => tree(r#"para(strong(em("x")))"#);
}

// ── intraword adjacency: the `_`-swap must never lose an emphasis level ───
//
// Addendum, 2026-09-22 (review of this RFC's first implementation): the
// `_`-swap above is only safe when `**` can still open where it is.
// CommonMark requires a delimiter run followed by punctuation -- here, the
// `_` the swap is about to write -- to be preceded by whitespace,
// punctuation, or nothing. Preceded by a letter or digit instead, `**`
// cannot open at all: the bold is lost outright and literal asterisks
// appear in the prose, which is worse than the order-inverted bug this RFC
// exists to fix. Order is preserved only where the delimiters can flank;
// otherwise both spans stay, unswapped, exactly as the baseline always
// wrote them.

cells! {
    // Alphanumeric on both sides: the swap is never attempted (caught at
    // open time, against the character before the Bold ancestor's own
    // delimiter -- already-written history).
    intraword_both_sides_keeps_both_emphases: "<p>a<strong><em>x</em></strong>b</p>"
        => tree(r#"para("a", em(strong("x")), "b")"#);
    intraword_both_sides_multichar: "<p>foo<strong><em>bar</em></strong>baz</p>"
        => tree(r#"para("foo", em(strong("bar")), "baz")"#);
    // Alphanumeric only after: nothing precedes the Bold ancestor, so the
    // swap is attempted at open time, then undone once "tail" turns out to
    // follow -- only knowable once the whole nested span, not just the
    // inner `<em>`, has closed.
    intraword_after_only_undoes_the_swap: "<p><strong><em>x</em></strong>tail</p>"
        => tree(r#"para(em(strong("x")), "tail")"#);
    // Alphanumeric only before: caught at open time, the same as the
    // both-sides case.
    intraword_before_only_skips_the_swap: "<p>lead<strong><em>x</em></strong></p>"
        => tree(r#"para("lead", em(strong("x")))"#);
    // Controls: whitespace or punctuation on the affected side is safe, and
    // the swap fires (or stays) as it did before this addendum.
    intraword_guard_safe_with_trailing_space: "<p><strong><em>x</em></strong> word</p>"
        => tree(r#"para(strong(em("x")), " word")"#);
    intraword_guard_safe_with_trailing_punctuation: "<p><strong><em>x</em></strong>.</p>"
        => tree(r#"para(strong(em("x")), ".")"#);
}

// ── control: sibling adjacency, must stay byte-identical ──────────────────

cells! {
    // Not this RFC's concern, but its own mechanism must survive this one
    // untouched: two adjacent same-delimiter spans still alternate to `_`
    // so they do not merge into one.
    adjacent_bold_spans_still_alternate: "<p><b>a</b><b>b</b></p>"
        => tree(r#"para(strong("a"), strong("b"))"#);
}
