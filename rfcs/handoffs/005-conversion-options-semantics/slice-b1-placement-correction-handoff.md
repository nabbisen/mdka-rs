# Developer Handoff — RFC 005 Slice B1, anchor placement correction

**Governing RFC.** [RFC 005](../../proposed/005-conversion-options-semantics.md), Option 3
**Corrects.** [`slices-bc-handoff.md`](./slices-bc-handoff.md) §5 — its required-cases table was incomplete
**Milestone.** M2 · Truth in the API surface
**Prepared.** 2026-08-12
**Follows.** [`.git-exclude/reviewed/005-slices-bc/README.md`](../../../.git-exclude/reviewed/005-slices-bc/README.md)

---

## 1. Purpose

Slice B1 landed correctly against the contract it was given. That contract was
incomplete: it specified a heading, a paragraph, an inline span and two negative
cases, and **no list, blockquote or nesting**. In those contexts the anchor is
misplaced.

This slice corrects placement. **Nothing from Slices B/C is being reverted.**

## 2. The two defects

Both observed, not inferred.

### 2.1 Lists — the anchor lands in the wrong item

```
<ul><li>one</li><li id="b">two</li><li>three</li></ul>
→ "- one\n<a id=\"b\"></a>\n- two\n- three\n"
```

In CommonMark the anchor line is a **lazy continuation** of item *one*'s
paragraph. The anchor for `id="b"` therefore renders inside item **one**. A
reader following `#b` arrives at the wrong item.

### 2.2 Blockquotes — the anchor escapes the quote

```
<blockquote><p id="p">Q</p></blockquote>
→ "<a id=\"p\"></a>\n\n> Q\n"
```

The `<p>` is inside the quote; its anchor is emitted outside it, with no `> `
prefix, as a separate paragraph above.

## 3. The cause — two mechanism findings

I read these in the code; you do not need to rediscover them.

**1 · The call site precedes all block machinery.**

```rust
pub fn enter_element(&mut self, elem: &Element, preserve_ids: bool) {
    self.emit_id_anchor(elem, preserve_ids);   // ← before the match
    let tag = elem.name();
    match tag { … }
}
```

Every arm's `begin_block()` / `emit_pending_prefix()` / marker work happens
*after* the anchor is already written. So the anchor can never sit inside the
structure it names.

**2 · `push_raw` does not emit the pending prefix.**

`emit_pending_prefix()` is what writes `"> " × blockquote_depth`, and it is
called at each content write. `push_raw` — which `emit_id_anchor` uses — writes
straight to `self.output` and sets `at_line_start = false`. An anchor emitted via
`push_raw` inside a blockquote therefore both misses its own prefix *and*
suppresses it for the content that follows.

## 4. The required contract

**Emit the anchor as leading *content* of the element — after any prefix or
marker — rather than before the element.**

| Input | Expected |
|---|---|
| `<h2 id="install">Install</h2>` | `"## <a id=\"install\"></a>Install\n"` |
| `<p id="intro">Text</p>` | `"<a id=\"intro\"></a>Text\n"` |
| `<ul><li id="b">two</li></ul>` | `"- <a id=\"b\"></a>two\n"` |
| `<ul><li>one</li><li id="b">two</li><li>three</li></ul>` | `"- one\n- <a id=\"b\"></a>two\n- three\n"` |
| `<blockquote><p id="p">Q</p></blockquote>` | `"> <a id=\"p\"></a>Q\n"` |
| `<ol><li id="x">a</li></ol>` | `"1. <a id=\"x\"></a>a\n"` |
| `<p>a <span id="s">b</span> c</p>` | **unchanged from today** |
| `<h2>No id</h2>` | `"## No id\n"` — unchanged |
| `<h2 id="">Empty</h2>` | `"## Empty\n"` — unchanged |
| `preserve_ids = false` | no anchor, whatever the input |
| `<a href="/"><span id="s">Home</span></a>` | `"[Home](/)\n"` — guard |
| `<pre><code id="c">x</code></pre>` | ` "```\nx\n```\n" ` — guard |

### This changes two previously-approved outputs

The heading and paragraph cases previously put the anchor *above* the element
with a blank line. They now carry it inside. **That is the correction, not a
regression** — an anchor inside the heading it names is a more accurate link
target than one floating above it, and it is the only placement that also works
in lists and blockquotes.

Update the Slice C tests that assert the old shape. Same treatment as before:
**update with a comment, do not delete.** The comment should record that the
earlier placement was emitted before the element and why it changed.

## 5. Direction, not prescription

Determine the mechanics yourself, as before. What I verified against the current
code, offered so you do not repeat the work:

- Moving the `emit_id_anchor` call from **before** the match to **after** it
  satisfies the heading and list cases — by then `## ` and `- ` are written and
  `at_line_start` is already false.
- The blockquote case additionally needs `emit_pending_prefix()` **inside**
  `emit_id_anchor`, before the `push_raw`.
- The inline-span case must stay byte-identical; it has no block arm.

If you find a cleaner construction, take it. The §4 table is the contract.

## 6. Non-change scope

- **The escaping.** It is correct and better than specified — a single pass over
  `chars()`, so ordering cannot matter. Do not touch it.
- **The `capture_depth` / `in_pre` guards.** Reasoning confirmed; I observed both
  behaving correctly. Keep them exactly as they are. They now need *tests* (§7),
  not changes.
- **Slice B2** — the five deprecations, the builder, every `#[allow(deprecated)]`
  site, the corrected doc comments. All approved.
- **Binding crates.** `cli/`, `node/`, `python/` should not be touched at all.
- **`docs/`** — RFC 006.
- The file-level `#![allow(deprecated)]` on `characterisation_attributes.rs` —
  approved, right scope.

### ⚠ `.github/workflows/create-release.yaml`

Still untracked, still not gitignored. Stage by explicit path. Never `git add -A`.

## 7. Required new tests

Beyond updating the existing Slice C assertions:

1. `id` on a mid-list `<li>` — the §2.1 case, three items, anchor on the second
2. `id` on an `<ol>` item
3. `id` on a paragraph inside a blockquote — the §2.2 case
4. `id` on a nested element inside another `id`-bearing element
5. `id` inside `<a href>` — the `capture_depth` guard
6. `id` inside `<pre>` — the `in_pre` guard

5 and 6 close the gap you named in your own §9. You were right that they needed
coverage.

## 8. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Baseline 122.** Expect 122 plus your additions, with the Slice C placement
assertions changed per §4.

Assert only on `html_to_markdown` / `html_to_markdown_with` output.

## 9. Prohibited shortcuts

- Do not revert any part of Slices B or C.
- Do not modify the escaping.
- Do not remove the guards.
- Do not delete tests that change — update them with a comment.
- Do not touch the binding crates or `docs/`.

## 10. Known risks

| Risk | If it happens |
|---|---|
| A §4 expected output looks wrong to you | **Report before adjusting it.** The table was wrong once already; if it is wrong again I want to hear it from you, not discover it later. |
| The inline-span case shifts | It must not. If it does, the anchor is being treated as block content — report. |
| Moving the call breaks an arm I did not enumerate | Report the arm and its output. My table covers headings, paragraphs, lists, blockquotes and nesting; it is not proof there is no sixth case. |
| The fix needs `traversal.rs` | Report it. Slices B/C needed only a call-site update there; more than that is worth a look. |

## 11. Required evidence

1. All §4 contract cases, run by you.
2. The §7 tests, including both guard cases.
3. Which Slice C assertions changed, and the new expected values.
4. `git diff src/renderer.rs`.
5. Confirmation that `cli/`, `node/`, `python/` and `docs/` are untouched.
6. Test count reconciled against 122 + additions.
7. fmt and clippy clean.

## 12. Acceptance checklist

- [ ] All §4 contract cases pass
- [ ] Mid-list anchor sits inside its own item
- [ ] Blockquote anchor sits inside the quote, after `> `
- [ ] Inline-span case byte-identical to today
- [ ] Both guards covered by tests
- [ ] Escaping, guards and Slice B2 untouched
- [ ] Changed assertions carry explanatory comments
- [ ] Count reconciles; fmt and clippy clean
- [ ] Bindings and `docs/` untouched; `create-release.yaml` untracked

## 13. Required review-request format

Standard eleven parts. The substance:

4. **The §4 contract cases, run by you**
5. **Which Slice C assertions changed, and why**
6. **Any placement case you found that §4 does not cover** — this is the second
   time an incomplete table has been the defect. If you think of a structure I
   have missed, test it and tell me even if it passes.

## 14. Escalate rather than decide

Stop and raise it if: a §4 expected output seems wrong; the inline-span case
cannot be held stable; the correction requires touching the bindings; or you find
a context where no placement is correct.

## 15. After this lands

RFC 005 closes and moves to `rfcs/done/`. RFC 006 — option documentation and
binding parity, carrying the `figure`/`figcaption` finding — becomes unblocked.

## 16. A note on how this came back

Your §9 said the guards were reasoned rather than observed, and offered tests.
That sentence is why this review went where it did: I went looking at placement
because you told me that was the soft ground, and the defect was one step past
where you had looked.

Flagging your own uncertainty worked. Keep doing it.
