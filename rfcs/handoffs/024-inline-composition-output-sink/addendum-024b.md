# Addendum — RFC 024 · slice `024b`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-17
**RFC.** [RFC 024](../../done/024-inline-composition-output-sink.md) — amendment of 2026-09-17
**Base.** `1de7f2c`, approved: `.git-exclude/reviewed/024-inline-composition-output-sink/README.md`
**Size.** Small. `src/renderer.rs`, `src/renderer/sink.rs` if needed, `tests/`.

---

## 0. Why this exists

Decisions on your §2.1 and §2.3 escalations, and a gap I left: the Q2/Q3 cells were decided in the
RFC 025 review and never turned into assertions. **This addendum is an instruction to start.**

**The expected structures below are written by the architect, from rules decided before RFC 024 was
implemented.** Enter them exactly. If one looks wrong to you, **stop and report** — do not adjust it
to match output.

## 1. Rules

1. **Code holds text only — `<pre><code>` included.** Inline markup elements inside any code context
   (bare `<pre>`, `<pre><code>`, inline `<code>`) contribute their text: no `**`, `*`, `[`, `](`, `![`.
2. **An image inside code contributes nothing** — not its alt text. (Consistent with the accepted harness
   model Q8b: alt counts as text outside code only.)
3. **One `<pre>`, one code block.** Its text is the text of everything inside it, in document order,
   across any number of `<code>` children and any text outside them.
4. **Language hint from the first `<code>` child only**, if it carries one.
5. An empty `<pre>` or `<pre><code>` still produces an empty code block (unchanged — criterion 4 protects it).

## 2. Criterion 4, amended

`<pre><code>` output stays **byte-identical to 2.2.3 for everything except** content containing inline
markup elements, a second `<code>`, or text outside the `<code>` — which now follow §1.

**Re-run your 34-shape battery.** Shapes that do not contain markup, a second `<code>`, or text outside
`<code>` must be byte-identical. List every shape that moved and confirm each is covered by §1.

## 3. Harness — convert the Q2/Q3 cells

In `tests/output_validity/inline_in_container.rs`, replace `undecided(...)` with these trees. **Only
these eight lines change.**

| Cell | Expectation |
|---|---|
| `img_in_pre` | `tree(r#"codeblock()"#)` |
| `strong_in_pre` | `tree(r#"codeblock("b")"#)` |
| `em_in_pre` | `tree(r#"codeblock("e")"#)` |
| `a_in_pre` | `tree(r#"codeblock("t")"#)` |
| `img_in_code` | `tree(r#""#)` — an empty document |
| `strong_in_code` | `tree(r#"para(code("b"))"#)` |
| `em_in_code` | `tree(r#"para(code("e"))"#)` |
| `a_in_code` | `tree(r#"para(code("t"))"#)` |

## 4. Harness — new cells, owner RFC 024

Add to `inline_in_container.rs` (or a new `code_context.rs` — say which). **Write the cells first and
run them against `1de7f2c` before changing `src/`**: every one that fails today is marked
`known_defect(Rfc024, …)` with the harness's reason, and the marker is removed when this slice fixes it —
so each fix is observed.

| Cell | HTML | Expectation |
|---|---|---|
| `strong_in_pre_code` | `<pre><code><b>kw</b> fn</code></pre>` | `codeblock("kw fn")` |
| `em_in_pre_code_with_language` | `<pre><code class="language-rs"><em>let</em> x</code></pre>` | `codeblock[rs]("let x")` |
| `a_in_pre_code` | `<pre><code><a href="/x">t</a></code></pre>` | `codeblock("t")` |
| `img_in_pre_code` | `<pre><code><img src="i.png" alt="pic"></code></pre>` | `codeblock()` |
| `two_codes_in_pre` | `<pre><code>a</code><code>b</code></pre>` | `codeblock("ab")` |
| `language_from_first_code_only` | `<pre><code>a</code><code class="language-js">b</code></pre>` | `codeblock("ab")` |
| `text_before_code_in_pre` | `<pre>text<code>x</code></pre>` | `codeblock("textx")` |
| `text_after_code_in_pre` | `<pre><code>x</code> tail</pre>` | `codeblock("x tail")` |

## 5. Remove the pinned test

Delete `markup_inside_pre_code_is_byte_identical_to_2_2_3` from `tests/inline_composition.rs`, and any
other assertion there that pins the §2.1 or §2.3 bytes. The harness cells replace them. Keep every
`inline_composition.rs` test that pins **text-only** `<pre><code>` bytes.

## 6. Out of scope

- Blockquote prefix on code continuation lines (A-08) — pending the owner's decision on RFC 035.
- `<a>`/`<code>` around blocks — RFC 028. Their current bytes (§2.2 of your request) must not move.
- Escaping inside code — RFC 010.

## 7. Required verification

1. §3 and §4 cells: state before the `src/` change (which fail) and after (all pass), 5 modes × 2 readings.
2. **No other harness cell changes state.**
3. §2 battery: every moved shape listed and covered by §1; every other shape byte-identical.
4. `cargo test --workspace --all-features --locked --no-fail-fast` — count reconciled against **291**.
5. fmt; clippy per CI.
6. CHANGELOG `[Unreleased]`: the `<pre><code>` markup and multi-`<code>` changes, before/after.

## 8. Acceptance checklist

- [ ] §3 eight Q2/Q3 lines replaced exactly; no other expectation touched
- [ ] §4 eight cells added first, marked where failing at `1de7f2c`, markers removed by the fix
- [ ] §1 rules implemented; §2 battery re-run
- [ ] §5 pinned-bytes tests removed
- [ ] No other cell changed state
- [ ] Count reconciled; fmt, clippy; CHANGELOG

## 9. Report back

`.git-exclude/review-request/024b-code-holds-text-only/README.md`, evidence under `evidence/`, exit codes
inside the files.
