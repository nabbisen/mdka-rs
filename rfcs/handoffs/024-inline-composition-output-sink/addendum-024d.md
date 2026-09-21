# Addendum — RFC 024 · slice `024d`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-17
**RFC.** [RFC 024](../../done/024-inline-composition-output-sink.md) — amendment of 2026-09-17 (rule 7)
**Base.** `a90307d`, approved: `.git-exclude/reviewed/035-block-structure-inside-containers/README.md`
**Size.** Small.

---

## 0. Why

Blocks inside `<pre>` push RFC 035's containers **inside the code block**:

```
<pre><blockquote><p>q</p></blockquote><ul><li>a</li></ul></pre><p>after</p>
  →  "> ```\n> q\n\n- a\n\n\n```\n\nafter"      parses: quote(code("q")) ul(li("a")) code("\nafter")
```

`after` is swallowed. This breaks RFC 024's accepted rule 3 (one `<pre>`, one code block holding the text of everything inside). No harness
cell covered it. **This addendum is an instruction to start.**

## 1. Rule 7

**Inside a code context** — a `<pre>`, including `<pre><code>` — **block elements contribute their text only.** Each block boundary is
**one line break**. No container is pushed, no prefix, no list marker, no heading marker, no blank line.

A block that ends at the end of the `<pre>` adds no trailing line break beyond what rule 6 already allows.

## 2. Cells — written by the architect; enter exactly

In `tests/output_validity/code_context.rs`, owner **RFC 024**. **Run against `a90307d` first**; mark failing cells
`known_defect(Rfc024, …)`; remove markers with the fix. If an expectation looks wrong, **stop and report**.

| Cell | HTML | Expectation |
|---|---|---|
| `blockquote_and_ul_in_pre` | `<pre><blockquote><p>q</p></blockquote><ul><li>a</li></ul></pre><p>after</p>` | `codeblock("q\na"), para("after")` |
| `p_in_pre` | `<pre><p>one</p><p>two</p></pre><p>after</p>` | `codeblock("one\ntwo"), para("after")` |
| `div_in_pre` | `<pre><div>x</div></pre><p>after</p>` | `codeblock("x"), para("after")` |
| `div_in_pre_code` | `<pre><code><div>x</div></code></pre><p>after</p>` | `codeblock("x"), para("after")` |

In the raw-string tree notation `\n` inside `codeblock("…")` is the two characters `\` `n`.

Expected at `a90307d`: **all four fail.** Confirm with the harness.

## 3. Also check, and report

- A block inside `<pre>` **inside a list item and a quote** — `<ul><li><pre><p>a</p><p>b</p></pre></li></ul>` and the quote equivalent — must
  produce one code block with the item's or quote's prefix on every line (RFC 035) and `a\nb` as its text. Add them as cells with those
  structures if they fail today; report either way.
- **No other cell changes state**; every RFC 035 cell stays green.

## 4. Required verification

1. Cells before (fail) and after (pass), 5 modes × 2 readings.
2. No other cell changes state.
3. Battery: every existing input containing `<pre>` — list any output that moved; each must contain a block element inside `<pre>`.
4. `cargo test --workspace --all-features --locked --no-fail-fast` — reconcile against **358**; CI counted by test name.
5. fmt; clippy per CI; CHANGELOG `[Unreleased] → Fixed` with before/after.

## 5. Acceptance checklist

- [ ] Four cells added first, exactly as written; failing ones marked at `a90307d`
- [ ] Rule 7 implemented; no container, prefix or marker inside code
- [ ] §3 list-item and quote cases checked and reported
- [ ] All pass; no other cell changed state; battery moves listed
- [ ] Count reconciled; fmt, clippy; CHANGELOG

## 6. Report back

`.git-exclude/review-request/024d-blocks-in-code-are-text/README.md`, evidence under `evidence/`, exit codes inside the files.
