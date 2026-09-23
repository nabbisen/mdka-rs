# RFC 009 — Element coverage extension

**Status.** Implemented (2.4.0) — closed 2026-09-23 at `d2c5448`
**Author.** Architect
**Created.** 2026-09-23
**Milestone.** M4 · Coverage and durability → `2.4.0`
**Source.** The roadmap's P2 companion to RFC 008 (moved from M3, 2026-09-16); bekoedit's items 5 and 6; the `2.3.0` consumer pass §6.5.
**Depends on.** RFC 008 — **done** (`7e7cec2`).
**Touches.** `src/renderer.rs`, `src/utils.rs`, `src/renderer/escape.rs`, `docs/src/api/elements.md`, `tests/output_validity/`.

---

## 0. Closed, 2026-09-23 at `d2c5448`

All four pieces shipped; seven CI workflows green; **566 Rust / 42 Node / 90 Python**; 570 non-affected
comparisons against `7e7cec2` byte-identical.

| | |
|---|---|
| `<dl>` | **no longer welds** — the last element in the codebase that did. `<dt>`/`<dd>` are blocks, inheriting container prefixes, `<pre>` suppression and RFC 008's cell flattening for free |
| `<del>`/`<s>` | `~~struck~~`; `<ins>`/`<u>` stay plain text |
| checkbox list items | `- [x]` / `- [ ]` |
| `<sup>`/`<sub>` | Unicode where every character maps — `2⁷`, `H₂O`, `xⁿ`, `⁻¹`, `⁽²⁾`; everything else byte-identical |

**This closed bekoedit's last two outstanding items** (5, strikethrough; 6, task-list checkboxes).

### One fix found in review

Strikethrough was built as a dedicated mechanism rather than reusing `emphasis_open` — correctly, since the
`*`/`_` interchange logic has no `~~` equivalent — but **RFC 037's collapse rule did not come with it**.
Nested `<del>` emitted `~~~~x~~~~`, which at the start of a line is a **tilde code fence**: the content was
destroyed under both readings. `<del><s>` did the same, confirming RFC 037's other finding that the rule is
about the delimiter, not the tag.

**RFC 037's remedy did not transfer.** It solved the adjacent case by alternating `**`/`__`; `~~` has no
alternation — `~single~` parses, but `~~a~~~b~` does not, since both forms use the same character. Adjacent
spans therefore **merge**: `<del>a</del><del>b</del>` → `~~ab~~`, which renders identically to the source and
loses only a boundary between two spans that were visually one.

### Recorded for later: the `<dl>` cost is real

§4.1 chose plain blocks over an invented bold term. The implementer, asked for a judgement after building it,
endorsed the decision and showed what it looks like at scale — a six-entry glossary becomes six unrelated
paragraphs, with the pairing genuinely gone rather than re-expressed. Markdown has no definition-list syntax
and both alternatives invent structure, so the decision stands. **Noted in case a consumer reports it the way
bekoedit reported `<del>`.**

## 1. Summary

Four separate problems, only one of which is a defect. Current behaviour, measured:

```
<dl><dt>Term</dt><dd>Desc</dd><dt>T2</dt><dd>D2</dd></dl>  ->  "TermDescT2D2"
<del>gone</del> and <s>old</s>                             ->  "gone and old"
<li><input type="checkbox" checked>done                    ->  "- done"
2<sup>7</sup> and H<sub>2</sub>O                           ->  "27 and H2O"
```

**`<dl>` welds.** It is the last element in the codebase that concatenates content with no separator, now
that RFC 008 has fixed tables — the same defect, in the element the consumer pass called out as describing
"the worst case as if it were the only case".

## 2. Measurement — six real pages

| Element | Occurrences | Note |
|---|---|---|
| `<sup>` | **201** | but see §4.3 — **99% are citation markers, already correct** |
| `<cite>` | 154 | no Markdown syntax |
| `<dl>` | **62** | **welds today** |
| `<abbr>` | 36 | no Markdown syntax |
| checkbox `<input>` | 20 | GFM expressible |
| `<q>` 13, `<kbd>` 11, `<small>` 9, `<time>` 2 | | no Markdown syntax |
| `<del>`, `<s>`, `<ins>`, `<u>`, `<mark>` | **0** | GFM expressible; zero in this sample, but a **reported** field issue |

Frequency is not the only criterion: `<del>` scores zero here and is bekoedit's item 5, reported from real
use. But it does reorder the work — `<dl>` is both the most common of these and the only one that is broken.

## 3. What Markdown can express — verified, not assumed

**Can** (GFM): `~~struck~~` → `Strikethrough`; `- [x]` / `- [ ]` → `TaskListMarker`.

**Cannot:** definition lists, `<sup>`/`<sub>`, and every inline in the "no syntax" rows above. Checked and
rejected: `2^7^` and `H~2~O` are Pandoc, not CommonMark — both come out as literal text; `Term\n: Desc` is
kramdown, likewise literal.

**Emitting `~~` is safe.** Literal tildes in prose are *already* escaped — `a ~~b~~ c` → `a \~\~b\~\~ c` —
so turning on strikethrough cannot collide with text that merely contains tildes.

## 4. Decisions

### 4.1 `<dl>`/`<dt>`/`<dd>` — each becomes its own block. **The defect fix.**

`<dt>` and `<dd>` each emit as their own block, exactly as a `<dd>` containing a `<p>` already does today.
Welding stops; nothing is invented.

**Rejected: bolding the term**, or joining as `- **Term** — Desc`. Both invent emphasis or a list the source
never had — the mistake RFC 037 exists to correct, and the same reasoning that chose an empty header row
over a promoted one in RFC 008 F2. Markdown has no definition-list syntax; the honest output is the content
in the right order, not a decoration that implies structure we cannot express.

The cost is that a `<dl>` becomes indistinguishable from a run of paragraphs. That is a real loss, it is
Markdown's, and **`api/elements.md` will then describe it accurately** — which it currently does not.

### 4.2 `<del>`, `<s>` → `~~…~~`. `<input type="checkbox">` in a list item → `- [x]` / `- [ ]`.

Both are plain GFM, both verified. bekoedit items 5 and 6.

`<ins>` and `<u>` stay plain text: GFM has no insertion or underline syntax, and `~~` would say the opposite
of what `<ins>` means.

### 4.3 `<sup>`/`<sub>` — Unicode where every character maps, plain text otherwise

**The measurement reframes this one.** `<sup>` is the most common element in scope at 201 occurrences, and
**99% of them are Wikipedia citation markers** — `<sup><a href="#c1">[1]</a></sup>`. Those are **already
correct**: RFC 010's escaping produces `text[\[1\]](#c1) more`, which parses as a link whose text is `[1]`.

Only **1%** are the mathematical case the consumer pass flagged, and those are the ones that are silently
wrong: `2<sup>7</sup>` → `27`, which is not a typographic loss but a different number.

**Rule:** if **every** character of the content maps to a Unicode superscript or subscript, emit those
characters; otherwise leave the content as plain text, exactly as today.

```
2<sup>7</sup>   -> 2⁷        H<sub>2</sub>O -> H₂O        x<sup>n</sup> -> xⁿ
<sup><a …>[1]</a></sup>      -> unchanged: `[` and `]` do not map
```

Mappable: `0-9 + - = ( ) n i` for superscript; `0-9 + - = ( ) a e o x h k l m n p s t` for subscript.

This fixes precisely the cases that are currently *wrong* and touches nothing that is currently right.

**Rejected: inline `<sup>` HTML.** It survives — verified — and it is the only way to express arbitrary
content. But RFC 008 §4 declined raw HTML passthrough on the grounds that a paste-as-Markdown consumer
cannot put HTML in a user's document, and that reasoning holds identically here. If a consumer ever asks for
fidelity over portability, it belongs with F4 as an opt-in, not as a default.

### 4.4 The unlisted inlines stay plain text — and get documented

`<cite>` (154), `<abbr>` (36), `<q>`, `<kbd>`, `<small>`, `<time>`, `<mark>`, `<ins>`, `<u>`: Markdown has no
syntax for any of them, and inventing emphasis is the thing we do not do. They remain their own text.

**But `api/elements.md` lists none of them**, which the consumer pass raised: the page's blanket "elements
not listed keep their children as plain text" covers them technically while leaving a reader to guess. Add
them explicitly, with what is lost.

## 5. Acceptance criteria

1. `<dl>` **cannot weld**, for any input. `TermDescT2D2` must be impossible.
2. `<del>`/`<s>` round-trip through `~~`, verified by parsing with `ENABLE_STRIKETHROUGH`; literal tildes in
   prose stay escaped and unchanged.
3. Checkbox list items parse with `ENABLE_TASKLISTS` as `TaskListMarker(true/false)`; a list item **without**
   a checkbox is byte-identical to today.
4. `<sup>`/`<sub>` with fully-mappable content emit Unicode; **anything else is byte-identical to `7e7cec2`**,
   the citation shape especially.
5. `api/elements.md` gains rows for §4.4's elements and an accurate `<dl>` description.
6. Cells for every rule, all five modes.
7. Interaction with RFC 008: all of this inside a **table cell** behaves — `~~` and task markers in a cell,
   a `<dl>` in a cell flattening per RFC 008 §4.1. Prove it; do not assume it.
8. All three suites reported separately; `fmt`/`clippy`; counts against 537 / 42 / 90.

## 6. Size

**M holds.** Four local mechanisms, no new machinery: `<dl>` is block classification, the RFC 008 fallback
pattern exactly; `~~` and task markers are emission; the Unicode map is a lookup table. Nothing here needs a
new renderer context, which is what made RFC 008 an L.

The one place it could grow is criterion 7 — these are all inline or block constructs that can appear inside
a table cell, and RFC 008's flattening rules will have to accommodate them.

## 7. Not in scope

Raw-HTML passthrough in any form (RFC 008 §4's F4). `<dl>` inside `<dl>`. Any new public option — RFC 039
Half B owns the option surface.
