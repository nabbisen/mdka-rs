# Addendum — RFC 024 · slice `024e`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-17
**RFC.** [RFC 024](../../done/024-inline-composition-output-sink.md) — amendment of 2026-09-17 (rule 8)
**Base.** `e317a72`, approved: `.git-exclude/reviewed/024d-blocks-in-code-are-text/README.md`
**Size.** Small.

---

## 0. Why

Your §3.3.3: `<br>` inside code is written as a Markdown hard break, which inside a code block is literal text —
`<pre>line1<br>line2</pre>` → `code("line1  \nline2")`, two trailing spaces on every line. **This addendum is an instruction to start.**

## 1. Rule 8

Inside a code context, **`<br>` is text**:

- in a `<pre>`, including `<pre><code>`: **one line break**, with no trailing spaces;
- in an inline code span: **one space**.

## 2. Harness model

`html_facts` currently gives `<br>` no text (`tests/output_validity/harness/properties.rs:23`). Model rule 8 there — `"\n"` inside a `<pre>`,
`" "` inside an inline `<code>` — **from this text, not from the renderer**. Prove it: `[code-block]` fires on `line1  \nline2` and holds on
`line1\nline2`.

## 3. Cells — written by the architect; enter exactly

In `tests/output_validity/code_context.rs`, owner **RFC 024**. Run against `e317a72` first; mark failing cells; remove markers with the fix.

| Cell | HTML | Expectation |
|---|---|---|
| `br_in_pre` | `<pre>line1<br>line2<br>line3</pre>` | `codeblock("line1\nline2\nline3")` |
| `br_in_pre_code` | `<pre><code>x<br>y</code></pre>` | `codeblock("x\ny")` |
| `br_in_code_span` | `<p><code>a<br>b</code></p>` | `para(code("a b"))` |
| `pretty_printed_blocks_in_pre` | `<pre>\n  <p>a</p>\n  <p>b</p>\n</pre><p>after</p>` | `codeblock("  \na\n  \nb"), para("after")` |

The last one is **not** a defect today. It pins the reading accepted in the 024d review — a text node's own newline serves as the block
boundary's line break — with a structure the architect derived by walking rules 6 and 7 over the HTML's text nodes, independently of both
the renderer and the harness. If the base does not produce it, **stop and report**.

Expected at `e317a72`: the three `br` cells fail; `pretty_printed_blocks_in_pre` passes.

## 4. Required verification

1. Cells before and after, 5 modes × 2 readings.
2. No other cell changes state.
3. Battery: every existing input containing `<br>` — list outputs that moved; each must have the `<br>` inside code.
4. `cargo test --workspace --all-features --locked --no-fail-fast` — reconcile against **365**; CI by test name.
5. fmt; clippy; CHANGELOG `[Unreleased] → Fixed` with before/after.

## 5. Acceptance checklist

- [ ] Four cells added first, exactly as written
- [ ] Harness models rule 8 from its text, proven both ways
- [ ] Rule 8 implemented; `<br>` outside code unchanged
- [ ] No other cell changed state; `<br>` battery moves listed
- [ ] Count reconciled; fmt, clippy; CHANGELOG

## 6. Report back

`.git-exclude/review-request/024e-br-in-code-is-text/README.md`, evidence under `evidence/`, exit codes inside the files.
