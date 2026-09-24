# Supported HTML Elements

The table below shows every HTML element that mdka recognises and what
Markdown it produces. Elements not listed are either silently removed
(script, style, etc.) or their children are kept as plain text.

## Block Elements

| HTML | Markdown output | Notes |
|---|---|---|
| `<h1>` – `<h6>` | `# ` – `###### ` | ATX-style headings |
| `<p>` | Paragraph (blank lines around) | |
| `<blockquote>` | `> ` prefix | On every line inside the quote, including blank lines (`>`) and code block lines. Nesting produces `> > `, `> > > `, … |
| `<pre><code>` | Fenced code block ` ``` ` | Preserves whitespace and newlines |
| `<ul>` | `- ` list | Tight or loose, see [Lists](#lists-tight-and-loose) |
| `<ol>` | `1. ` list | Respects `start` attribute. Tight or loose, see [Lists](#lists-tight-and-loose) |
| `<li>` | List item | Everything inside it stays in the item: later lines are indented to the item's content column (`- ` → 2 spaces, `1. ` → 3, `10. ` → 4), including nested lists and code block lines |
| `<li><input type="checkbox">…` | `- [ ] …` / `- [x] …` (checked) | Only when the checkbox is the item's own first content; any other `<input>`, or one anywhere else in the item, is not a task marker |
| `<hr>` | `---` | |
| `<dl>` | — | Not itself rendered; a transparent container. `<dt>`/`<dd>` do the work, below |
| `<dt>`, `<dd>` | Block separator | Each its own paragraph-like block, in source order. A `<dl>` therefore reads as a run of paragraphs — Markdown has no definition-list syntax, and inventing one (bolding the term, a `- ` prefix) would assert a structure the source never had |
| `<div>`, `<article>`, `<section>`, `<main>` | Block separator | Act as paragraph breaks; unwrapped (tag removed, children kept) when [`unwrap_unknown_wrappers`](./options.md) is on — Minimal and Semantic by default |
| `<figure>`, `<figcaption>` | Block separator | **Never unwrapped, in any mode.** These carry structural meaning `unwrap_unknown_wrappers` is not meant to discard — they're excluded from the wrapper-candidate set entirely, not merely blocked by a secondary check |
| `<table>` | GFM table, or a non-welding fallback | See [Tables](#tables) |

## Inline Elements

| HTML | Markdown output | Notes |
|---|---|---|
| `<strong>`, `<b>` | `**text**` | **No delimiters** when the element's own inline `style` sets `font-weight` to `normal` or a number ≤ 500: `<b style="font-weight:normal">x</b>` converts to `x`. Google Docs, for one, wraps pasted content in such an element. `bolder`, `lighter` and numbers above 500 do not suppress anything. **Only this one property, on this element, is read.** Nothing is inherited, and bold is never *added*: text made bold only by `<span style="font-weight:700">`, or by a `font-weight` on a parent element, converts as plain text |
| `<em>`, `<i>` | `*text*` | **No delimiters** when the element's own inline `style` sets `font-style: normal`. The same rule: only that property, on that element, and italic carried only by a `<span style="font-style:italic">` is lost |
| `<del>`, `<s>` | `~~text~~` | No delimiters at all if the element wraps a block, or is inside code — the same rule `<strong>`/`<em>` already follow |
| `<code>` (inline) | `` `text` `` | Only when not inside `<pre>` |
| `<a href="…">` | `[text](url)` | `title` attribute → `[text](url "title")` |
| `<img src="…" alt="…">` | `![alt](src)` | `title` attribute → `![alt](src "title")` |
| `<sup>` | A Unicode superscript (`²`, `ⁿ`, `⁻⁹`), or a marker: `^(…)` | See [Superscript and subscript](#superscript-and-subscript) below |
| `<sub>` | A Unicode subscript (`₂`, `ₙ`, `ᵢ`), or a marker: `_(…)` | Same rules; see below |
| `<br>` | `  \n` (trailing two spaces + newline) | |

### Superscript and subscript

`<sup>` and `<sub>` are converted by three rules, tried in this order. The reason
there is a rule three at all: `2<sup>n − 1</sup>` used to convert to `2n − 1`,
which reads as "two times n minus one" — a different statement, not a lossy one.

1. **A real Unicode superscript or subscript, when every character has one.**
   `10<sup>−9</sup>` → `10⁻⁹`, `x<sup><i>n</i></sup>` → `xⁿ`, `H<sub>2</sub>O` → `H₂O`,
   `x<sub>max</sub>` → `xₘₐₓ`. The characters that map are digits, `+ - = ( )`,
   U+2212 MINUS SIGN (treated as `-`), and lowercase letters:
   - `<sup>`: `a b c d e f g h i j k l m n o p r s t u v w x y z` — 25 of 26, there is no
     superscript `q`;
   - `<sub>`: `a e h i j k l m n o p r s t u v x` — only 17, Unicode has no subscript
     `b c d f g q w y z`, so `x<sub>y</sub>` can never map.

   Content that is only emphasis (`<i>`, `<em>`, `<b>`, `<strong>`, `<var>`) is seen
   through: a superscript character cannot carry emphasis anyway. The rule is all or
   nothing — a run with one character that has no form (an uppercase letter, `/`, a
   space) is **not** half-converted, which would be a different number.
2. **Text that already delimits itself is left exactly as it was.** If the text inside is
   bracketed — `[1]` or `(a b)` — it is a marker already (a citation, a note, a
   parenthetical) and nothing is added. This is why the citation markers on real pages
   (`<sup><a href="#c1">[1]</a></sup>`) convert as they always did.
3. **Everything else gets a visible marker, always parenthesised:** `2<sup>n − 1</sup>` →
   `2^(n − 1)`, `x<sub>y</sub>` → `x_(y)`, `x<sup>N</sup>` → `x^(N)`. Emphasis inside
   survives: `x<sup><i>n</i> + N</sup>` → `x^(*n* + N)`. The subscript marker's `_` is
   escaped (`\_(y)`) wherever it could otherwise pair with another underscore and
   italicise the text between them — after a space or punctuation, inside emphasis, or
   when an unescaped `_` is already open in the paragraph.

Inside code (`<pre>`, `<code>`) nothing is mapped and no marker is written; the text
stays as it is. `<sup>`/`<sub>` are converted the same way in every mode.

`<span>` is **not** in either table, and that is deliberate: it produces no
output of its own and no break. `<span>A</span><span>B</span>` converts to
`AB`, with the children passed straight through.

`<ins>` and `<u>` are also not in either table: their children are kept as
plain text, with no markup at all. GFM has no insertion or underline syntax,
and `~~` (the closest available delimiter) would say the opposite of what
`<ins>` means, so nothing is emitted rather than something misleading.

Also kept as plain text, permanently rather than pending: `<cite>`, `<abbr>`,
`<q>`, `<kbd>`, `<small>`, `<time>`, `<mark>`. Markdown has no syntax for any
of them, and inventing one — the way a bolded `<cite>` or a blockquoted
`<q>` would — is the mistake this project does not make.

## Tables

Almost every `<table>` becomes a GFM table. Only four shapes are inexpressible:
more than one whole-row header, the row-header pattern (`<th>` used as each
row's own first cell rather than as a whole header row), a nested `<table>`
anywhere in a cell, and `<caption>` — GFM has no syntax for any of these.

```html
<table><thead><tr><th>Name</th><th>Age</th></tr></thead>
<tbody><tr><td>Alice</td><td>30</td></tr></tbody></table>
```

converts to:

```markdown
| Name | Age |
| --- | --- |
| Alice | 30 |
```

A table missing a `<th>` entirely gets an empty header row synthesized over
the data, rather than promoting row 1 to a heading the source never wrote.
`colspan`/`rowspan` are expanded by repeating the spanned content across
every cell it covers, header included. A cell holding block content —
multiple paragraphs, a list, a code block, a heading, a blockquote — flattens
to inline: paragraphs join with `<br>`; a list becomes `<br>`-separated
`- `/`N. ` marker text (two spaces of indent per nesting level); a code block
becomes one code span per source line, `<br>`-joined; a heading or blockquote
contributes its text plain, without inventing emphasis or a quote marker the
cell can't hold:

```html
<table><tr><th colspan="2">Name</th><th>Age</th></tr>
<tr><td>Alice</td><td>Smith</td><td>30</td></tr>
<tr><td colspan="2"><p>Bob Jones</p><p>(pending)</p></td><td>27</td></tr></table>
```

converts to:

```markdown
| Name | Name | Age |
| --- | --- | --- |
| Alice | Smith | 30 |
| Bob Jones<br>(pending) | Bob Jones<br>(pending) | 27 |
```

Alignment is carried from `align=` or `text-align:` into `:--`/`--:`/`:-:`. A
literal `|` in a cell is escaped (`\|`), including one produced by a nested
code span, link, or image. A ragged row — one with a different cell count
from the header — is not a blocker: GFM pads a short row, and a body row
longer than the header widens the whole table's grid instead (an empty
header cell is synthesized over the extra column, so nothing is lost).
`<br>` inside a cell survives as literal inline HTML rather than a Markdown
hard break, which would end the cell early.

**A table that is inexpressible never welds its cells together** — the
`H1H2ab` defect below is fixed regardless of which case applies. Every
`<tr>`/`<td>`/`<th>`/`<caption>` renders as its own paragraph-like block, so
row and column structure is not preserved, but every cell's text is clearly
separated:

```html
<table><tr><th>R1</th><td>1</td></tr><tr><th>R2</th><td>2</td></tr></table>
```

converts to `R1`, `1`, `R2`, `2`, each its own paragraph (blank-line
separated) — not `R11R22`. A nested table is analyzed on its own terms: it
may become its own expressible table even while the table containing it
falls back. A `<caption>` is never dropped silently; it appears as its own
paragraph, in its document position.

Turning the row/column structure of an inexpressible table into a real GFM
table (flattening a cell's block content, synthesising a header where none
exists, expanding a span) is a separate, larger piece of work, tracked as its
own follow-up. See
[`ROADMAP.md`](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) for
scheduling.

## Lists: tight and loose

Markdown has two kinds of list: **tight**, with no blank line between items,
and **loose**, where items are separated by blank lines and each item's text is
a paragraph. HTML has no such distinction — whether an item's text is wrapped
in `<p>` is usually a matter of how the page was produced — so mdka decides by
the item's content:

> A list is **loose** if and only if some item contains **two or more
> blocks**, where a run of inline content (text, links, bold, images, line
> breaks) counts as one block, and **nested lists are not counted**.
> Otherwise it is **tight**.

A block is a paragraph, heading, blockquote, code block or rule, and anything
else the chosen mode renders as a block: `<div>` counts in every mode,
including Minimal and Semantic, which unwrap it — unwrapping removes the tag,
not the paragraph break it stood for; an element the mode drops counts as
nothing.

A CMS list with one `<p>` per item stays tight:

```html
<ul><li><p>a</p></li><li><p>b</p></li></ul>
```

```
- a
- b
```

A nested list is not counted, so text followed by a sublist is still one block:

```html
<ol><li>one<ol><li>inner</li></ol></li></ol>
```

```
1. one
   1. inner
```

**Exception: a nested list whose own first item is empty gets a blank line
first, regardless.** Without one, its bare marker line is not read as a
nested list at all — an unordered marker alone is a setext-heading underline
for the text above it, and an ordered one is absorbed as that text's own
lazy-continuation, taking any non-empty siblings after it along (RFC 038).
mdka inserts the blank line to keep the output unambiguous. This is a
disambiguation, not a looseness call: the rule above is unchanged, and the
list itself still counts as tight by it — but CommonMark reads any blank
line as loose once it sees one, so a reparse of this shape shows every
item's text wrapped in a paragraph regardless of what counted this list as
tight in the first place. A nested list whose first item has real content,
like `inner` above, is never affected.

Two paragraphs in one item make the whole list loose:

```html
<ul><li><p>a</p><p>b</p></li><li>c</li></ul>
```

```
- a

  b

- c
```

Each list is decided on its own: a loose list nested inside a tight one leaves
the outer list tight.

## Not Yet Supported

| HTML | Current behaviour | Status |
|---|---|---|
| `<video>`, `<audio>` | No output for the media element itself | Not yet scheduled |

Tables were the largest gap here until they gained the treatment described in
[Tables](#tables), above: an expressible table is real GFM; anything else at
least never welds its cells together. `<dl>`, `<del>`/`<s>`, checkbox list
items and `<sup>`/`<sub>` closed the remaining ones — see their own rows in
[Block Elements](#block-elements) and [Inline Elements](#inline-elements),
above.

## Code Blocks and Language Hints

When a `<code>` element has a `class` containing `language-<name>`, the
language name is included in the fenced block:

```html
<pre><code class="language-rust">fn main() {}</code></pre>
```

Produces:

````markdown
```rust
fn main() {}
```
````

The `language-*` class is preserved in **all** conversion modes, including
`Balanced` which otherwise strips `class` attributes.

## Always-Removed Elements

These elements and all their descendants are removed unconditionally,
regardless of conversion mode:

`<script>` · `<style>` · `<meta>` · `<link>` · `<template>` ·
`<iframe>` · `<object>` · `<embed>` · `<noscript>` · `<head>` · `<svg>`

HTML comments are removed in **all** conversion modes, including
`Preserve`. No mode retains comment content.

## Shell Elements

`<nav>`, `<header>`, `<footer>`, `<aside>` are kept by default but can
be removed by setting [`drop_interactive_shell = true`](./options.md)
or using `ConversionMode::Minimal`.
