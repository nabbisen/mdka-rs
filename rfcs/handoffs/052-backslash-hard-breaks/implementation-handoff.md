# Developer Handoff — RFC 052, backslash hard breaks

**Authorised.** Owner, 2026-10-06 — RFC 052 accepted. **Milestone `3.3.0`**; *when* to cut is the
owner's, and prep is a separate slice.
**RFC.** `rfcs/accepted/052-backslash-hard-breaks.md` — **read §2 and §4 before any code.** §2 is why
this is an option and not a default; §4 is the one way it goes wrong.
**Baseline.** `origin/main` after `3ca5940`; eight workflows green; 660 Rust / 60 Node / 25 loader /
93 Python.
**Scope.** `src/renderer/sink.rs`, `src/options.rs`, the three bindings, `docs/src/api/options.md`,
tests. **No release work.**

---

## 1. The change

`backslash_hard_breaks: bool`, **default `false`**. With it on, a real hard break is `\` + newline
instead of two spaces + newline. Same shape as `emphasis_from_style`: mechanism-named boolean, off,
present on all four surfaces (`backslashHardBreaks` in Node, `--backslash-hard-breaks` on the CLI).

Today's emission is one line: **`src/renderer/sink.rs:1262`, `dest.put("  \n")`**, inside
`hard_break()`.

## 2. The trap 🛑

**Changing that line alone is wrong.** In a heading the two forms are not equivalent — measured:

```
## one··⏎two    →  h2("one")   para("two")      ← two spaces vanish
## one\⏎two     →  h2("one\")  para("two")      ← the backslash is left VISIBLE
```

Markdown has no hard break inside an ATX heading; mdka already splits the heading, and it currently
splits it **cleanly**. The option must not change that.

**So the backslash is emitted only where the break survives as a break** — paragraph, blockquote,
list item — and the heading path keeps `"  \n"`.

**A discriminator already exists**: `line.heading` (`src/renderer/escape.rs:88`), set by
`heading_marker()` (`sink.rs:865`), and `hard_break()` already reads `dest.line.pipe_here` from the
same struct two lines above. **Verify it is actually still `true` at that point before relying on
it** — I read the code, I did not run it, and `escape.rs:227` is the only other reader. If it is not
live there, say so and thread the context explicitly rather than guessing.

Untouched, each by its own earlier match arm in `renderer.rs:1156–1162`: `in_pre`, `cell_pre`, code
spans, and table cells (which emit a literal `<br>`).

## 3. Criteria

1. **Option off → byte-identical to `3.2.0`.** `mode_identity`'s goldens untouched, and say so.
2. On: paragraph, blockquote and list item emit `\` + newline, and **each parses to the same event
   stream as the two-space form** — assert the parse, not only the bytes. `.git-exclude/tmp/mdprobe`
   is the existing pulldown-cmark probe if you want it.
3. 🛑 **The heading case is unchanged, option on and off**, with a test named for it.
4. Table cell, `<pre>` and code span unchanged, option on and off.
5. All four surfaces, one name, each with its own can-affect-output test.
6. `docs/src/api/options.md` states the trade-off **in both directions**: two trailing spaces are
   stripped by many editors (and by `markdownlint` MD009), the backslash form is **not understood by
   Python-Markdown** — measured, 3.11: `'one\\⏎two'` → `<p>one\⏎two</p>`, no `<br>`, literal
   backslash shown. A reader must be able to choose. Naming the benefit without the cost is the
   failure RFC 050's documentation round already corrected once.
7. Counts, each with its command; `cargo fmt --check`, `clippy -D warnings`, docs-example gate clean.

## 4. Not in scope

`<h2>one<br>two</h2>` splitting into a heading plus a paragraph is arguably wrong — flattening to
`## one two` may be better — but it predates this RFC and nobody has reported it. RFC 052 §5 records
it. **Do not fix it here**, and do not let criterion 3 drift into changing it.

## 5. Report, then stop

`.git-exclude/review-request/052-backslash-hard-breaks/README.md`, leading with criteria 1 and 3.
Commit and push **only this slice** — never a release, never unrelated commits.
