# RFC 035 — Block structure inside containers: list items and blockquotes

**Status.** Accepted (2026-09-17, owner)
**Author.** Architect
**Created.** 2026-09-17
**Milestone.** M3 · Output validity → `2.3.0` (owner, 2026-09-17)
**Sequencing.** After RFC 028, before RFC 010 — RFC 010's block-start escaping needs correct list and quote structure.
**Source.** Audit A-06, A-07, A-08 — parked in RFC 009's scope, which moved to `2.4.0`; found still present at `1de7f2c`.
**Touches.** `src/renderer.rs`, `src/renderer/sink.rs`, `tests/output_validity/`.

---

## 1. Summary

Block content inside a list item or a blockquote does not stay inside it:

| Audit | HTML | Output at `1de7f2c` | Reader gets |
|---|---|---|---|
| A-06 | `<ul><li><p>para</p></li></ul>` | `- \n\npara` | an empty bullet, content outside the list |
| A-06 | `<ul><li><p>a</p><p>b</p></li></ul>` | `- \n\na\n\nb` | same |
| A-07 | `<ol><li>one<ol><li>inner</li></ol></li></ol>` | `1. one\n  1. inner` | not a sublist — a lazy continuation |
| A-07 | `<ol><li>one<ul><li>inner</li></ul></li></ol>` | `1. one\n  - inner` | not nested either — found at handoff, 2026-09-17: any sublist under an ordered parent, not only an ordered one |
| A-08 | `<blockquote><p>one</p><p>two</p></blockquote>` | `> one\n\n> two` | two quotes |
| A-08 | `<blockquote><pre><code>l1\nl2</code></pre></blockquote>` | ``> ```\nl1\nl2\n``` `` | the code escapes the quote after line 1 |

A-06's shape — list content wrapped in `<p>` — is what WordPress, most CMSs and Markdown-to-HTML
round-trips produce. These are **validity** defects on common input.

## 2. Why — one prefix, applied to some lines

RFC 024's sink emits a pending **blockquote** prefix before content writes. Two things are missing:

1. **Lists have no prefix at all.** An item's marker is written once; any block inside the item starts a
   new block at column 0, which is outside the list (A-06). Nested list indent is a fixed two spaces,
   not the parent's content column (A-07).
2. **The quote prefix is not applied to every line.** Blank separator lines get none, and CommonMark ends
   a quote at an unprefixed blank line; code-block content lines get none, by RFC 024's deliberate choice
   (A-08).

## 3. Design — a container prefix stack

Generalise the sink's single `blockquote_depth` into a **stack of containers**, innermost last:

| Container | Continuation prefix |
|---|---|
| list item | spaces equal to that item's **content column** — marker width plus one space: `- ` → 2, `1. ` → 3, `10. ` → 4 |
| blockquote | `> ` (on a blank line, `>` alone) |

**Every line written inside a container gets the whole stack's prefix**, outermost first — content lines,
blank separator lines, and **code-block content lines**. The first line of an item gets the marker in
place of the item's own indent.

### 3.1 Loose and tight lists — the rule

Markdown distinguishes tight lists (`- a\n- b`) from loose (`- a\n\n- b`). HTML's `<p>` inside `<li>` is a
presentation detail more often than a meaning. **Rule:**

> A list is **loose** if and only if some item contains **two or more blocks**, where a maximal run of inline
> content counts as one block (the paragraph Markdown will make of it) and **nested lists are not counted**.
> Otherwise it is **tight**.

So a CMS list whose every item is one `<p>` stays tight and clean (`p_in_each_li`); an item with two
paragraphs (`two_p_in_li`), or text followed by a code block (`text_then_pre_in_li`), makes the list loose;
ordinary nested lists stay tight (`ol_in_ol` — text plus a sublist is one counted block).

### 3.2 Consequences to preserve

- **Tight lists of plain items are byte-identical to 2.2.3**: `<ul><li>a</li><li>b</li></ul>` → `- a\n- b`.
- Unordered nesting under `- ` is already 2 spaces and must not move.
- The blockquote prefix RFC 024 fixed (a quote beginning with an inline element) must not regress.

## 4. Harness first — the block-inside-container direction

RFC 025's matrix has inline-inside-container and block-inside-inline. **This RFC adds the third direction.**
Expectations are written here, from the HTML's meaning and §3.1's rule. **Add them first, run them against
the current tree, mark every failure `known_defect(Rfc035, …)`, then implement** so each fix is observed.

| Cell | HTML | Expectation |
|---|---|---|
| `p_in_li` | `<ul><li><p>para</p></li></ul>` | `ul(li("para"))` |
| `p_in_each_li` | `<ul><li><p>a</p></li><li><p>b</p></li></ul>` | `ul(li("a"), li("b"))` |
| `two_p_in_li` | `<ul><li><p>a</p><p>b</p></li></ul>` | `ul(li(para("a"), para("b")))` |
| `pre_in_li` | `<ul><li><pre><code>x</code></pre></li></ul>` | `ul(li(codeblock("x")))` |
| `text_then_pre_in_li` | `<ul><li>see<pre><code>x</code></pre></li></ul>` | `ul(li(para("see"), codeblock("x")))` |
| `blockquote_in_li` | `<ul><li><blockquote><p>q</p></blockquote></li></ul>` | `ul(li(quote(para("q"))))` |
| `ol_in_ol` | `<ol><li>one<ol><li>inner</li></ol></li></ol>` | `ol[1](li("one", ol[1](li("inner"))))` |
| `ol_in_ol_two_digit` | `<ol start="10"><li>ten<ol><li>inner</li></ol></li></ol>` | `ol[10](li("ten", ol[1](li("inner"))))` |
| `ul_in_ol` | `<ol><li>one<ul><li>inner</li></ul></li></ol>` | `ol[1](li("one", ul(li("inner"))))` |
| `two_p_in_blockquote` | `<blockquote><p>one</p><p>two</p></blockquote>` | `quote(para("one"), para("two"))` |
| `multiline_pre_in_blockquote` | `<blockquote><pre><code>l1\nl2</code></pre></blockquote>` | `quote(codeblock("l1\nl2"))` |
| `ul_in_blockquote` | `<blockquote><ul><li>a</li><li>b</li></ul></blockquote>` | `quote(ul(li("a"), li("b")))` |
| `two_p_in_blockquote_in_li` | `<ul><li><blockquote><p>a</p><p>b</p></blockquote></li></ul>` | `ul(li(quote(para("a"), para("b"))))` |
| `two_p_in_nested_blockquote` | `<blockquote><blockquote><p>a</p><p>b</p></blockquote></blockquote>` | `quote(quote(para("a"), para("b")))` |
| `multiline_pre_in_li_in_blockquote` | `<blockquote><ul><li><pre><code>x\ny</code></pre></li></ul></blockquote>` | `quote(ul(li(codeblock("x\ny"))))` |
| `tight_list_control` | `<ul><li>a</li><li>b</li></ul>` | `ul(li("a"), li("b"))` |
| `indented_code_in_pre_in_blockquote` | `<blockquote><pre>    <code>x</code></pre></blockquote><p>after</p>` | `quote(codeblock("    x")), para("after")` — *added 2026-09-17, RFC 024c review* |

In the harness's raw-string notation the `\n` inside `codeblock("…")` is the two characters `\` `n`, as the
tree renders code text with `{:?}`.

## 5. Not in scope

- Escaping at block start — RFC 010, which follows and depends on this.
- Inline elements around blocks — RFC 028.
- Tables, definition lists — RFC 008/009, `2.4.0`.

## 6. Compatibility

Output changes for lists whose items contain blocks, nested ordered lists, and quotes containing more than
one block or multi-line code. Every change is from output that does not parse as meant to output that does.
Minor release, CHANGELOG with before/after.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Prefix stack desyncs from the element stack | One push per container enter, one pop per leave, asserted balanced at document end in debug builds |
| Tight/loose decision needs look-ahead | Item block-child counts are known before the item's content is emitted only with a tree query or buffering — the same choice RFC 028 makes; reuse its mechanism rather than add a second |
| Byte churn in plain lists | §3.2 — tight plain lists must not move; the harness control cell plus existing tests pin it |
| RFC 024's quote-prefix fix regresses | Its five harness cells stay green |

## 8. Acceptance criteria

1. The 17 §4 cells added first; every one that fails at the base commit marked, and every marker removed by
   this RFC — passing under CommonMark and GFM, all five modes.
2. No other harness cell changes state.
3. Tight lists of plain items byte-identical to 2.2.3.
4. A container prefix stack in the sink, applied to every line including blank lines and code content.
5. §3.1's loose/tight rule implemented and stated in `docs/src/api/elements.md`.
6. CHANGELOG with before/after.
