# Addendum — RFC 024 · slice `024c`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-17
**RFC.** [RFC 024](../../done/024-inline-composition-output-sink.md) — amendment of 2026-09-17 (rule 6)
**Base.** `467ebf1`, approved: `.git-exclude/reviewed/024b-code-holds-text-only/README.md`
**Size.** Small.

---

## 0. Why

The review found that whitespace held before `<code>` is written **in front of** the fence. At four or more spaces the
fence is not a fence, and the closing ```` ``` ```` swallows the rest of the document:

```
<pre>\n    <code class="language-js">x</code>\n</pre><p>after</p><h2>later</h2>
  →  "    ```js\nx\n```\n\nafter\n\n## later"
```

2.2.3 did the same. You kept it because criterion 4 and my `024b` addendum said to — that was my error.
**This addendum is an instruction to start.**

## 1. Rules

6. **Whitespace inside `<pre>` and outside `<code>` is part of the code block's text**, in order (rule 3). The fence is
   written at the **start of its line** — never after held whitespace. Whitespace before the first `<code>` is still held,
   so that `<code>` can supply the language, and is then written **inside** the block after the fence line.
4. (restated) The language comes from the first `<code>` child **when only whitespace precedes it** in the `<pre>`.

Unchanged: the newline directly after `<pre>` (dropped by the HTML parser) and a line break directly before `</pre>` are not
content.

## 2. Cells — written by the architect; enter exactly

Add to `tests/output_validity/code_context.rs`, owner **RFC 024**. **Run them against `467ebf1` before changing `src/`**; mark
each failing cell `known_defect(Rfc024, …)` and remove the marker with the fix. `\n` below means a newline character in the
HTML — use a non-raw string with `\n` escapes, or real newlines.

| Cell | HTML | Expectation |
|---|---|---|
| `four_spaces_before_code_in_pre` | `<pre>    <code class="language-js">x</code></pre>` | `codeblock[js]("    x")` |
| `two_spaces_before_code_in_pre` | `<pre>  <code class="language-js">x</code></pre>` | `codeblock[js]("  x")` |
| `pretty_printed_pre_code_then_content` | `<pre>\n    <code class="language-js">x</code>\n</pre><p>after</p><h2>later</h2>` | `codeblock[js]("    x"), para("after"), h2("later")` |
| `newline_before_code_in_pre` | `<pre>\n<code>x</code></pre>` | `codeblock("x")` |
| `newline_after_code_in_pre` | `<pre><code>x</code>\n</pre>` | `codeblock("x")` |
| `language_not_taken_after_text` | `<pre>text<code class="language-js">x</code></pre>` | `codeblock("textx")` |

Expected at `467ebf1`: the first three fail; the last three pass (controls). **Confirm with the harness.** If an expectation
looks wrong, stop and report.

## 3. Tests to update

`tests/inline_composition.rs` pins the pretty-printed whitespace bytes (`  ```js`). **Remove or rewrite that assertion** to the
rule-6 output; say which. Keep every other `<pre><code>` pin.

## 4. Required verification

1. Cells before (which fail) and after (all pass), 5 modes × 2 readings.
2. **No other harness cell changes state.**
3. Re-run the 34-shape battery: list every moved shape; each must involve whitespace between `<pre>` and `<code>`.
4. `cargo test --workspace --all-features --locked --no-fail-fast` — reconcile against **298**.
5. fmt; clippy per CI.
6. CHANGELOG `[Unreleased] → Fixed`: indented `<code>` inside `<pre>` no longer breaks the fence, with before/after, and the
   2-space shape's change stated.

## 5. Acceptance checklist

- [ ] Six cells added first, exactly as written; failing ones marked at `467ebf1`
- [ ] Rule 6 implemented; fence always at line start; language still from the first `<code>` after whitespace
- [ ] All six pass; no other cell changed state
- [ ] Battery moves listed, all whitespace-before-code shapes
- [ ] Pinned whitespace test updated; count reconciled; fmt, clippy; CHANGELOG

## 6. Report back

`.git-exclude/review-request/024c-fence-at-line-start/README.md`, evidence under `evidence/`, exit codes inside the files.
