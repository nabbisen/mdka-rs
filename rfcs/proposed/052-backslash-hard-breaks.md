# RFC 052 — A hard break that survives the editor

**Status.** Proposed — architect, 2026-10-06
**Author.** Architect
**Created.** 2026-10-06
**Milestone.** Unassigned. Opt-in and off by default; a minor.
**Source.** bekoedit's item 9, 2026-09-16, their last open request: *"`<br>` becomes two trailing
spaces and a newline. Many editors strip trailing whitespace, which silently removes the break. An
option for the backslash form (`\` + newline) would be more robust."* Reaffirmed 2026-09-24 as
*"still minor, unchanged."*
**Touches.** `src/renderer/sink.rs` (`hard_break`), `src/options.rs`, the bindings,
`docs/src/api/options.md`, tests.

---

## 1. What mdka emits today

`src/renderer/sink.rs:1262` — `dest.put("  \n")`. Two spaces and a newline, everywhere a `<br>`
becomes a real break. Measured on the published `3.2.0` binary, trailing spaces shown:

```
<p>one<br>two</p>                      →  one··⏎two
<blockquote><p>one<br>two</p></...>    →  > one··⏎> two
<ul><li>one<br>two</li></ul>           →  - one··⏎··two
<table>…<td>a<br>b</td>…               →  | a<br>b |     ← literal, unaffected
```

Their complaint is exact: **those two spaces are invisible, and stripping them is the default
behaviour of a great deal of software** — editors on save, `markdownlint`'s MD009, `git diff
--check`. The break disappears and nothing says so.

## 2. I set out to argue this should be the default. The measurement stopped me.

Both forms parse **identically** wherever the break is real. `pulldown-cmark`, GFM options on:

| Context | `··⏎` | `\⏎` |
|---|---|---|
| paragraph | `para("one" HB "two")` | **identical** |
| blockquote | `quote(para("one" HB "two"))` | **identical** |
| list item | `ul(li("one" HB "two"))` | **identical** |

On that evidence the backslash is strictly better — same meaning, robust against stripping — and a
*new option* would be a knob with no trade-off behind it, which is precisely the inert-option surface
the `3.0` programme spent itself removing.

**Then I checked a second renderer, and the argument collapsed.** Python-Markdown 3.11:

```
'one  \ntwo'   →  '<p>one<br />\ntwo</p>'     break: yes
'one\\\ntwo'   →  '<p>one\\\ntwo</p>'          break: NO, and a literal backslash is shown
```

**Python-Markdown does not implement backslash hard breaks at all.** It is the engine behind MkDocs
and a large share of static-site tooling. So the two forms have genuinely different constituencies:

| | survives whitespace stripping | works in Python-Markdown |
|---|---|---|
| two trailing spaces | **no** | **yes** |
| backslash | **yes** | **no** |

**Neither dominates.** There is a real trade-off, only the consumer knows which renderer their
Markdown is going to meet, and that is what an option is *for*. **bekoedit asked for an option and
they were right; my instinct to make it the default would have silently broken every MkDocs user.**

## 3. Proposal

A fourth option, `backslash_hard_breaks: bool`, **default `false`** — so output is unchanged for
everyone who does not ask. Same shape and naming as `emphasis_from_style`: mechanism-named, boolean,
off, identical on all four surfaces (`backslashHardBreaks` in Node, `--backslash-hard-breaks` on the
CLI).

With it on, `hard_break()` emits `\` + newline instead of two spaces + newline.

## 4. The trap this must not fall into 🛑

**A one-line change at `sink.rs:1262` is wrong.** A heading is not a context where either form makes
a hard break, and the two forms are **not** equivalent there:

```
## one··⏎two    →  h2("one")      para("two")       ← two spaces vanish
## one\⏎two     →  h2("one\")     para("two")       ← the backslash is VISIBLE in the heading
```

Markdown has no hard break inside an ATX heading, so mdka already splits the heading — but today it
splits it cleanly, and a naive swap would leave a stray `\` in the heading text.

**The option must emit the backslash only where the break survives as a break** — paragraph,
blockquote, list item — and leave the heading path exactly as it is.

Table cells already emit a literal `<br>` and are untouched; `in_pre`, `cell_pre` and code spans
have their own arms above the general case and are untouched.

## 5. What this RFC does not do

**It does not fix the heading itself.** `<h2>one<br>two</h2>` splitting into a heading plus a
paragraph is arguably wrong — flattening to `## one two` may be the better answer — but that is a
separate question about `<br>` in headings, it predates this RFC, and no one has reported it.
**Recorded here so it is not mistaken for something this RFC introduced.**

## 6. Criteria

1. Option off → **byte-identical to `3.2.0`**. `mode_identity`'s goldens untouched.
2. On: paragraph, blockquote and list item emit `\` + newline, and each **parses to the same event
   stream as the two-space form** — assert the parse, not just the bytes.
3. **The heading case is unchanged with the option on and off**, with a test named for it. This is
   the one way the change goes wrong.
4. Table cell, `<pre>`, and code span all unchanged, option on or off.
5. All four surfaces, one name, each with its own can-affect-output test.
6. `docs/src/api/options.md` states the trade-off **in both directions** — which form survives
   whitespace stripping, which form Python-Markdown understands — so a reader can choose. Naming the
   benefit without the cost would be the same failure this project corrected in RFC 050's
   documentation.
