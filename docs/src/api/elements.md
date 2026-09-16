# Supported HTML Elements

The table below shows every HTML element that mdka recognises and what
Markdown it produces. Elements not listed are either silently removed
(script, style, etc.) or their children are kept as plain text.

## Block Elements

| HTML | Markdown output | Notes |
|---|---|---|
| `<h1>` – `<h6>` | `# ` – `###### ` | ATX-style headings |
| `<p>` | Paragraph (blank lines around) | |
| `<blockquote>` | `> ` prefix | Nesting produces `> > `, `> > > `, … |
| `<pre><code>` | Fenced code block ` ``` ` | Preserves whitespace and newlines |
| `<ul>` | `- ` list | Nested lists indented by 2 spaces |
| `<ol>` | `1. ` list | Respects `start` attribute |
| `<li>` | List item | |
| `<hr>` | `---` | |
| `<div>`, `<article>`, `<section>`, `<main>` | Block separator | Act as paragraph breaks; unwrapped (tag removed, children kept) when [`unwrap_unknown_wrappers`](./options.md) is on — Minimal and Semantic by default |
| `<figure>`, `<figcaption>` | Block separator | **Never unwrapped, in any mode.** These carry structural meaning `unwrap_unknown_wrappers` is not meant to discard — they're excluded from the wrapper-candidate set entirely, not merely blocked by a secondary check |

## Inline Elements

| HTML | Markdown output | Notes |
|---|---|---|
| `<strong>`, `<b>` | `**text**` | |
| `<em>`, `<i>` | `*text*` | |
| `<code>` (inline) | `` `text` `` | Only when not inside `<pre>` |
| `<a href="…">` | `[text](url)` | `title` attribute → `[text](url "title")` |
| `<img src="…" alt="…">` | `![alt](src)` | `title` attribute → `![alt](src "title")` |
| `<br>` | `  \n` (trailing two spaces + newline) | |

`<span>` is **not** in either table, and that is deliberate: it produces no
output of its own and no break. `<span>A</span><span>B</span>` converts to
`AB`, with the children passed straight through.

## Not Yet Supported

These elements are **not** converted to their Markdown equivalent. Their text
content still appears — children are kept as plain text — so the output is not
empty, but the structure or emphasis they carry is lost.

| HTML | Current behaviour | Status |
|---|---|---|
| `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>` | Cell text is emitted as plain text; no GFM table is produced and the row/column structure is lost | [Planned](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) |
| `<dl>`, `<dt>`, `<dd>` | Term and description text run together as plain text | [Planned](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) |
| `<del>`, `<s>` | Text kept, strike-through (`~~text~~`) not emitted | [Planned](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) |
| `<sup>`, `<sub>` | Text kept inline, with no indication it was raised or lowered | [Planned](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) |
| `<video>`, `<audio>` | No output for the media element itself | Not yet scheduled |

**Tables are the largest gap**, and the cell text is not merely unstructured —
it is run together without separators:

```html
<table><thead><tr><th>H1</th><th>H2</th></tr></thead><tbody><tr><td>a</td><td>b</td></tr></tbody></table>
```

converts to `H1H2ab`. mdka adds no separator between cells; the only spacing
that survives is whitespace already present in the source, so the same table
written across two lines comes out as `H1H2 ab`. If your input is table-heavy,
the converted Markdown will read as runs of joined text where the table was.
See
[`ROADMAP.md`](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) for
scheduling.

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
