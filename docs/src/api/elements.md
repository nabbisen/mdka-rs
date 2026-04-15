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
| `<div>`, `<article>`, `<section>`, `<main>`, `<figure>`, `<figcaption>` | Block separator | Act as paragraph breaks; unwrapped in Minimal/Semantic |

## Inline Elements

| HTML | Markdown output | Notes |
|---|---|---|
| `<strong>`, `<b>` | `**text**` | |
| `<em>`, `<i>` | `*text*` | |
| `<code>` (inline) | `` `text` `` | Only when not inside `<pre>` |
| `<a href="…">` | `[text](url)` | `title` attribute → `[text](url "title")` |
| `<img src="…" alt="…">` | `![alt](src)` | `title` attribute → `![alt](src "title")` |
| `<br>` | `  \n` (trailing two spaces + newline) | |

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
`<iframe>` · `<object>` · `<embed>` · `<noscript>`

HTML comments are removed in all modes **except `Preserve`**, where they
are retained as `<!-- … -->` in the pre-processed DOM (though they do not
appear in Markdown output).

## Shell Elements

`<nav>`, `<header>`, `<footer>`, `<aside>` are kept by default but can
be removed by setting [`drop_interactive_shell = true`](./options.md)
or using `ConversionMode::Minimal`.
