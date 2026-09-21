# Addendum — RFC 028 · slice `028b`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-17
**RFC.** [RFC 028](../../done/028-emphasis-around-block-content.md)
**Base.** `b91aafb`, approved: `.git-exclude/reviewed/028-inline-around-blocks/README.md`
**Size.** Small. **Tests only** — no `src/` change.

---

## 0. Why

Two guards on RFC 028's harness work:

1. The ten `<a>`/`<code>` × block cells still read `undecided("Q4")`/`("Q5")`. Amendment 2 decided their structure before you
   started; they should assert it. That omission is mine, in the handoff.
2. `starts_markdown_block` is a second block list that must agree with the renderer's. Agreement should be **checked**,
   without the harness importing `src/`.

**This addendum is an instruction to start.**

## 1. The ten cells — structures written by the architect

Written from Amendment 2 and criterion 7 **before looking at output**, then checked against a release build of `b91aafb`: all ten
match. Replace each `undecided(...)` in `tests/output_validity/block_in_inline.rs` with exactly:

| Cell | Expectation |
|---|---|
| `p_in_a` | `tree(r#"para(link[/out]("x")), para(link[/out]("y"))"#)` |
| `ul_in_a` | `tree(r#"ul(li(link[/out]("x")), li(link[/out]("y")))"#)` |
| `blockquote_in_a` | `tree(r#"quote(para(link[/out]("x")))"#)` |
| `pre_in_a` | `tree(r#"codeblock("x")"#)` |
| `heading_in_a` | `tree(r#"h2(link[/out]("x"))"#)` |
| `p_in_code` | `tree(r#"para("x"), para("y")"#)` |
| `ul_in_code` | `tree(r#"ul(li("x"), li("y"))"#)` |
| `blockquote_in_code` | `tree(r#"quote(para("x"))"#)` |
| `pre_in_code` | `tree(r#"codeblock("x")"#)` |
| `heading_in_code` | `tree(r#"h2("x")"#)` |

**Only these ten lines change**, plus the module comment that says these cells assert only properties. If any fails, **stop and
report** — do not adjust an expectation or the renderer.

## 2. A collapse cell for the property

`joined_run` accepts a single link for several blocks. Add a proof in `proofs.rs`, using a stub converter, showing that for
`<a href="/o"><p>x</p><p>y</p></a>` the output `[x y](/o)` **is flagged by the tree assertion** of a cell with §1's `p_in_a`
expectation, even though the link properties pass it. That records, in code, which check guards the collapse case.

## 3. Block-list agreement — behavioural, no import

Add a proof that the harness's `starts_markdown_block` agrees with what mdka actually renders — **by converting HTML, not by
calling `src/`**:

- For **every tag** in the harness list, in a mode where it is not unwrapped or dropped, `<p>a</p><TAG>b</TAG>` must parse as two
  separate blocks (list items and `hr` need suitable wrappers — choose, and say how).
- For a set of **inline tags** — at least `span`, `b`, `i`, `em`, `strong`, `code`, `a`, `abbr`, `small`, `sub`, `sup`, `mark` —
  `<p>a<TAG>b</TAG></p>` must parse as one paragraph.
- For `div`, `article`, `section`, `main`: agreement in **both** settings of `unwrap_unknown_wrappers`.

If a tag disagrees today, **stop and report** which side is wrong — do not change either list in this slice.

## 4. Required verification

1. The ten cells: before (they pass properties) and after (they pass their trees), 5 modes × 2 readings.
2. §2 proof red where stated, green otherwise.
3. §3 proof: the tags it covered, per mode setting.
4. **No other cell changes state**; no `src/` change (`git diff --stat -- src` empty).
5. `cargo test --workspace --all-features --locked --no-fail-fast` — reconcile against **333**; count CI results by test name.
6. fmt; clippy per CI.

## 5. Acceptance checklist

- [ ] Ten `undecided` lines replaced exactly; module comment updated; nothing else in cell files
- [ ] Collapse proof: tree flags `[x y](/o)`
- [ ] Behavioural block-list agreement proof, both wrapper settings
- [ ] No other cell changed state; no `src/` change
- [ ] Count reconciled; fmt, clippy

## 6. Report back

`.git-exclude/review-request/028b-harness-guards/README.md`, evidence under `evidence/`, exit codes inside the files.
